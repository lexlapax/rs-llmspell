// ABOUTME: Async hook processing to remove hooks from critical state operation path
// ABOUTME: Uses lock-free queues and background processing for zero-overhead hook execution

use crossbeam::queue::SegQueue;
use llmspell_hooks::{Hook, HookContext, HookExecutor};
use llmspell_state_traits::{StateError, StateResult};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Hook event for async processing
#[derive(Clone)]
pub struct HookEvent {
    pub hook_type: HookEventType,
    pub context: HookContext,
    pub hooks: Vec<Arc<dyn Hook>>,
    pub correlation_id: Uuid,
    pub timestamp: Instant,
}

/// Type of hook event
#[derive(Clone, Debug)]
pub enum HookEventType {
    BeforeStateChange,
    AfterStateChange,
    BeforeAgentSave,
    AfterAgentSave,
    BeforeAgentDelete,
    AfterAgentDelete,
}

/// Async hook processor that runs hooks in background
pub struct AsyncHookProcessor {
    /// Lock-free queue for hook events
    event_queue: Arc<SegQueue<HookEvent>>,

    /// Background task handle
    processor_handle: Option<JoinHandle<()>>,

    /// Shutdown signal
    shutdown: Arc<AtomicBool>,

    /// Hook executor
    hook_executor: Arc<HookExecutor>,

    /// Statistics
    stats: Arc<HookProcessorStats>,

    /// Optional channel for completion notifications
    completion_tx: Option<mpsc::UnboundedSender<HookCompletionEvent>>,
}

/// Hook processing statistics
pub struct HookProcessorStats {
    pub events_queued: AtomicU64,
    pub events_processed: AtomicU64,
    pub events_failed: AtomicU64,
    pub total_processing_time_micros: AtomicU64,
    pub queue_depth: AtomicU64,
}

/// Hook completion event for monitoring
pub struct HookCompletionEvent {
    pub correlation_id: Uuid,
    pub hook_type: HookEventType,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

impl AsyncHookProcessor {
    /// Create new async hook processor
    pub fn new(hook_executor: Arc<HookExecutor>) -> Self {
        Self {
            event_queue: Arc::new(SegQueue::new()),
            processor_handle: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            hook_executor,
            stats: Arc::new(HookProcessorStats {
                events_queued: AtomicU64::new(0),
                events_processed: AtomicU64::new(0),
                events_failed: AtomicU64::new(0),
                total_processing_time_micros: AtomicU64::new(0),
                queue_depth: AtomicU64::new(0),
            }),
            completion_tx: None,
        }
    }

    /// Start background processing
    pub fn start(&mut self) -> StateResult<()> {
        if self.processor_handle.is_some() {
            return Err(StateError::already_exists(
                "Hook processor already running".to_string(),
            ));
        }

        let queue = self.event_queue.clone();
        let shutdown = self.shutdown.clone();
        let executor = self.hook_executor.clone();
        let stats = self.stats.clone();
        let completion_tx = self.completion_tx.clone();

        let handle = tokio::spawn(async move {
            loop {
                // Check shutdown
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                // Process events from queue
                if let Some(event) = queue.pop() {
                    stats.queue_depth.fetch_sub(1, Ordering::Relaxed);
                    let start = Instant::now();

                    // Process hooks
                    let mut context = event.context.clone();
                    let result = executor.execute_hooks(&event.hooks, &mut context).await;

                    let duration = start.elapsed();
                    let success = result.is_ok();

                    // Update statistics
                    stats.events_processed.fetch_add(1, Ordering::Relaxed);
                    if !success {
                        stats.events_failed.fetch_add(1, Ordering::Relaxed);
                    }
                    stats
                        .total_processing_time_micros
                        .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);

                    // Send completion notification if configured
                    if let Some(tx) = &completion_tx {
                        let _ = tx.send(HookCompletionEvent {
                            correlation_id: event.correlation_id,
                            hook_type: event.hook_type,
                            success,
                            duration,
                            error: result.err().map(|e| e.to_string()),
                        });
                    }
                } else {
                    // No events, yield to avoid busy spinning
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        });

        self.processor_handle = Some(handle);
        Ok(())
    }

    /// Stop background processing
    pub async fn stop(&mut self) -> StateResult<()> {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(handle) = self.processor_handle.take() {
            handle
                .await
                .map_err(|e| StateError::background_task_error(e.to_string()))?;
        }

        Ok(())
    }

    /// Queue hook event for async processing
    pub fn queue_hook_event(&self, event: HookEvent) -> StateResult<()> {
        self.event_queue.push(event);
        self.stats.events_queued.fetch_add(1, Ordering::Relaxed);
        self.stats.queue_depth.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Queue multiple hook events
    pub fn queue_hook_events(&self, events: Vec<HookEvent>) -> StateResult<()> {
        let count = events.len() as u64;
        for event in events {
            self.event_queue.push(event);
        }
        self.stats.events_queued.fetch_add(count, Ordering::Relaxed);
        self.stats.queue_depth.fetch_add(count, Ordering::Relaxed);
        Ok(())
    }

    /// Get current queue depth
    pub fn queue_depth(&self) -> u64 {
        self.stats.queue_depth.load(Ordering::Relaxed)
    }

    /// Get processing statistics
    pub fn stats(&self) -> HookProcessorStatsSnapshot {
        HookProcessorStatsSnapshot {
            events_queued: self.stats.events_queued.load(Ordering::Relaxed),
            events_processed: self.stats.events_processed.load(Ordering::Relaxed),
            events_failed: self.stats.events_failed.load(Ordering::Relaxed),
            total_processing_time_micros: self
                .stats
                .total_processing_time_micros
                .load(Ordering::Relaxed),
            queue_depth: self.stats.queue_depth.load(Ordering::Relaxed),
        }
    }

    /// Set completion notification channel
    pub fn set_completion_channel(&mut self, tx: mpsc::UnboundedSender<HookCompletionEvent>) {
        self.completion_tx = Some(tx);
    }

    /// Wait for queue to drain (for testing)
    pub async fn wait_for_drain(&self, timeout: Duration) -> StateResult<()> {
        let start = Instant::now();

        while self.queue_depth() > 0 {
            if start.elapsed() > timeout {
                return Err(StateError::timeout("Hook queue drain timeout".to_string()));
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }
}

/// Snapshot of hook processor statistics
#[derive(Debug, Clone)]
pub struct HookProcessorStatsSnapshot {
    pub events_queued: u64,
    pub events_processed: u64,
    pub events_failed: u64,
    pub total_processing_time_micros: u64,
    pub queue_depth: u64,
}

impl HookProcessorStatsSnapshot {
    /// Calculate average processing time in microseconds
    pub fn average_processing_time_micros(&self) -> f64 {
        if self.events_processed == 0 {
            0.0
        } else {
            self.total_processing_time_micros as f64 / self.events_processed as f64
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.events_processed == 0 {
            100.0
        } else {
            ((self.events_processed - self.events_failed) as f64 / self.events_processed as f64)
                * 100.0
        }
    }
}

/// Hook batcher for efficient batch processing
pub struct HookBatcher {
    batch_size: usize,
    batch_timeout: Duration,
    pending_events: RwLock<Vec<HookEvent>>,
    last_flush: RwLock<Instant>,
}

impl HookBatcher {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            batch_size,
            batch_timeout,
            pending_events: RwLock::new(Vec::with_capacity(batch_size)),
            last_flush: RwLock::new(Instant::now()),
        }
    }

    /// Add event to batch
    pub fn add_event(&self, event: HookEvent) -> Option<Vec<HookEvent>> {
        let mut events = self.pending_events.write();
        events.push(event);

        // Check if batch should be flushed
        if events.len() >= self.batch_size || self.should_flush_timeout() {
            let batch = std::mem::replace(&mut *events, Vec::with_capacity(self.batch_size));
            *self.last_flush.write() = Instant::now();
            Some(batch)
        } else {
            None
        }
    }

    /// Force flush current batch
    pub fn flush(&self) -> Vec<HookEvent> {
        let mut events = self.pending_events.write();
        let batch = std::mem::replace(&mut *events, Vec::with_capacity(self.batch_size));
        *self.last_flush.write() = Instant::now();
        batch
    }

    /// Check if timeout requires flush
    fn should_flush_timeout(&self) -> bool {
        self.last_flush.read().elapsed() >= self.batch_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{ComponentId, ComponentType, HookPoint};

    #[tokio::test]
    async fn test_async_hook_processor() {
        let executor = Arc::new(HookExecutor::new());
        let mut processor = AsyncHookProcessor::new(executor);

        // Start processor
        processor.start().unwrap();

        // Create test event
        let context = HookContext::new(
            HookPoint::Custom("test".to_string()),
            ComponentId::new(
                ComponentType::Custom("test".to_string()),
                "test".to_string(),
            ),
        );

        let event = HookEvent {
            hook_type: HookEventType::AfterStateChange,
            context,
            hooks: vec![],
            correlation_id: Uuid::new_v4(),
            timestamp: Instant::now(),
        };

        // Queue event
        processor.queue_hook_event(event).unwrap();
        assert_eq!(processor.queue_depth(), 1);

        // Wait for processing
        processor
            .wait_for_drain(Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(processor.queue_depth(), 0);

        // Check stats
        let stats = processor.stats();
        assert_eq!(stats.events_queued, 1);
        assert_eq!(stats.events_processed, 1);
        assert_eq!(stats.events_failed, 0);
        assert_eq!(stats.success_rate(), 100.0);

        // Stop processor
        processor.stop().await.unwrap();
    }

    #[test]
    fn test_hook_batcher() {
        let batcher = HookBatcher::new(3, Duration::from_millis(100));

        // Create test events
        let create_event = || HookEvent {
            hook_type: HookEventType::AfterStateChange,
            context: HookContext::new(
                HookPoint::Custom("test".to_string()),
                ComponentId::new(
                    ComponentType::Custom("test".to_string()),
                    "test".to_string(),
                ),
            ),
            hooks: vec![],
            correlation_id: Uuid::new_v4(),
            timestamp: Instant::now(),
        };

        // Add events below batch size
        assert!(batcher.add_event(create_event()).is_none());
        assert!(batcher.add_event(create_event()).is_none());

        // Third event triggers batch
        let batch = batcher.add_event(create_event()).unwrap();
        assert_eq!(batch.len(), 3);

        // Force flush empty batch
        let empty_batch = batcher.flush();
        assert_eq!(empty_batch.len(), 0);
    }
}
