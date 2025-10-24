//! Background consolidation daemon with adaptive scheduling
//!
//! Processes unprocessed episodic entries in background using LLM consolidation engine.
//! Aligns with llmspell-kernel daemon architecture (graceful shutdown, operation tracking).
//!
//! # Architecture Alignment
//!
//! Follows llmspell-kernel/daemon patterns:
//! - `tokio::sync::watch` for shutdown coordination (like `ShutdownCoordinator`)
//! - `Arc<AtomicBool>` for running flag
//! - RAII guard pattern for tracking in-flight operations
//! - `tokio::select!` for graceful shutdown
//! - Phase-based shutdown (Running → Stopping → Stopped)
//!
//! # Features
//!
//! - **Adaptive Intervals**: 30s (fast), 5m (normal), 30m (slow) based on queue depth
//! - **Session Prioritization**: Active sessions processed first (fairness via round-robin)
//! - **Health Monitoring**: LLM provider checks, circuit breaker integration
//! - **Graceful Shutdown**: Completes in-flight consolidations before stopping
//!
//! # Example
//!
//! ```rust,ignore
//! use llmspell_memory::consolidation::{ConsolidationDaemon, DaemonConfig};
//!
//! let config = DaemonConfig {
//!     fast_interval_secs: 30,
//!     normal_interval_secs: 300,
//!     slow_interval_secs: 1800,
//!     batch_size: 10,
//!     max_concurrent: 1,
//! };
//!
//! let daemon = ConsolidationDaemon::new(engine, episodic_memory, config);
//! let handle = daemon.start().await?;
//!
//! // ... daemon runs in background ...
//!
//! daemon.stop().await?;
//! handle.await?;
//! ```

use crate::error::{MemoryError, Result};
use crate::traits::EpisodicMemory;

use super::llm_engine::LLMConsolidationEngine;
use super::ConsolidationEngine;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Fast interval when >100 unprocessed entries (default: 30s)
    pub fast_interval_secs: u64,
    /// Normal interval when 10-100 entries (default: 5min)
    pub normal_interval_secs: u64,
    /// Slow interval when <10 entries (default: 30min)
    pub slow_interval_secs: u64,
    /// Batch size per consolidation (default: 10)
    pub batch_size: usize,
    /// Max concurrent consolidations (default: 1)
    pub max_concurrent: usize,
    /// Active session threshold in seconds (default: 300s = 5min)
    pub active_session_threshold_secs: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            fast_interval_secs: 30,
            normal_interval_secs: 300,
            slow_interval_secs: 1800,
            batch_size: 10,
            max_concurrent: 1,
            active_session_threshold_secs: 300,
        }
    }
}

/// Daemon metrics
#[derive(Debug, Clone, Default)]
pub struct DaemonMetrics {
    /// Total consolidations performed
    pub consolidations: u64,
    /// Total entries processed
    pub entries_processed: u64,
    /// Total decisions made
    pub decisions_made: u64,
    /// Consecutive failures (for circuit breaker)
    pub consecutive_failures: u64,
    /// Current queue depth
    pub queue_depth: u64,
}

/// Background consolidation daemon
///
/// Runs in background tokio task, processes unprocessed episodic entries using
/// LLM consolidation engine with adaptive scheduling and session prioritization.
pub struct ConsolidationDaemon {
    engine: Arc<LLMConsolidationEngine>,
    episodic_memory: Arc<dyn EpisodicMemory>,
    config: DaemonConfig,
    running: Arc<AtomicBool>,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    metrics: Arc<Mutex<DaemonMetrics>>,
    in_flight_operations: Arc<AtomicU64>,
}

impl ConsolidationDaemon {
    /// Create new consolidation daemon
    pub fn new(
        engine: Arc<LLMConsolidationEngine>,
        episodic_memory: Arc<dyn EpisodicMemory>,
        config: DaemonConfig,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        Self {
            engine,
            episodic_memory,
            config,
            running: Arc::new(AtomicBool::new(false)),
            shutdown_tx,
            shutdown_rx,
            metrics: Arc::new(Mutex::new(DaemonMetrics::default())),
            in_flight_operations: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Start daemon in background tokio task
    ///
    /// Returns join handle for awaiting daemon completion.
    ///
    /// # Errors
    ///
    /// Returns error if daemon is already running.
    pub fn start(self: Arc<Self>) -> Result<JoinHandle<()>> {
        if self.running.load(Ordering::SeqCst) {
            return Err(MemoryError::InvalidInput(
                "Daemon already running".to_string(),
            ));
        }

        self.running.store(true, Ordering::SeqCst);
        info!("Starting consolidation daemon");

        let daemon = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            daemon.run().await;
        });

        Ok(handle)
    }

    /// Stop daemon gracefully
    ///
    /// Signals shutdown and waits for in-flight operations to complete.
    ///
    /// # Errors
    ///
    /// Returns error if shutdown signal fails to send.
    pub async fn stop(&self) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            debug!("Daemon not running, nothing to stop");
            return Ok(());
        }

        info!("Stopping consolidation daemon gracefully");
        self.shutdown_tx.send(true).map_err(|e| {
            MemoryError::InvalidInput(format!("Failed to send shutdown signal: {e}"))
        })?;

        // Wait for in-flight operations to complete (max 30s)
        let start = std::time::Instant::now();
        let max_wait = Duration::from_secs(30);

        while self.in_flight_operations.load(Ordering::SeqCst) > 0 {
            if start.elapsed() > max_wait {
                warn!(
                    "Timeout waiting for in-flight operations: {} remaining",
                    self.in_flight_operations.load(Ordering::SeqCst)
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        self.running.store(false, Ordering::SeqCst);
        info!("Consolidation daemon stopped");
        Ok(())
    }

    /// Check if daemon is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get current metrics snapshot
    pub async fn metrics(&self) -> DaemonMetrics {
        self.metrics.lock().await.clone()
    }

    /// Main daemon loop (runs in background task)
    async fn run(self: Arc<Self>) {
        let mut shutdown_rx = self.shutdown_rx.clone();
        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.fast_interval_secs));

        info!("Daemon loop started");

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!("Daemon tick");

                    if !self.engine.is_ready() {
                        warn!("LLM engine not ready, skipping consolidation");
                        continue;
                    }

                    interval = self.handle_batch_processing(interval).await;
                }
                _ = shutdown_rx.changed() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        info!("Daemon loop exited");
    }

    /// Handle batch processing and interval adjustment
    async fn handle_batch_processing(
        &self,
        _current_interval: tokio::time::Interval,
    ) -> tokio::time::Interval {
        match self.process_batch().await {
            Ok(queue_depth) => {
                self.handle_batch_success(queue_depth).await;
                let new_interval = self.select_interval(queue_depth);
                debug!(
                    "Queue depth: {}, next interval: {:?}",
                    queue_depth, new_interval
                );
                tokio::time::interval(new_interval)
            }
            Err(e) => {
                self.handle_batch_failure(e).await;
                tokio::time::interval(Duration::from_secs(self.config.fast_interval_secs))
            }
        }
    }

    /// Handle successful batch processing
    async fn handle_batch_success(&self, queue_depth: usize) {
        let mut metrics = self.metrics.lock().await;
        metrics.queue_depth = queue_depth as u64;
        metrics.consecutive_failures = 0;
    }

    /// Handle batch processing failure
    async fn handle_batch_failure(&self, error: MemoryError) {
        error!("Consolidation batch failed: {}", error);

        let mut metrics = self.metrics.lock().await;
        metrics.consecutive_failures += 1;
        let failures = metrics.consecutive_failures;
        drop(metrics);

        if failures >= 10 {
            warn!("10+ consecutive failures, pausing daemon for 5 minutes");
            tokio::time::sleep(Duration::from_secs(300)).await;
        }
    }

    /// Process a batch of unprocessed entries
    ///
    /// Returns remaining queue depth.
    async fn process_batch(&self) -> Result<usize> {
        // Track in-flight operation
        let _guard = OperationGuard::new(Arc::clone(&self.in_flight_operations));

        // Get all sessions with unprocessed entries
        let sessions = self.get_sessions_with_unprocessed().await?;

        if sessions.is_empty() {
            debug!("No unprocessed entries, skipping batch");
            return Ok(0);
        }

        // Prioritize active sessions
        let prioritized_sessions = Self::prioritize_sessions(sessions);

        // Process up to batch_size entries across sessions (round-robin)
        let mut total_processed = 0;
        for session_id in prioritized_sessions.iter().take(self.config.batch_size) {
            match self.consolidate_session(session_id).await {
                Ok(count) => {
                    total_processed += count;
                    debug!("Consolidated {} entries for session {}", count, session_id);
                }
                Err(e) => {
                    error!("Failed to consolidate session {}: {}", session_id, e);
                }
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.consolidations += 1;
            metrics.entries_processed += total_processed as u64;
        }

        // Calculate remaining queue depth
        let remaining = self.count_unprocessed_total().await?;
        Ok(remaining)
    }

    /// Consolidate unprocessed entries for a session
    async fn consolidate_session(&self, session_id: &str) -> Result<usize> {
        // Get unprocessed entries for this session
        let mut entries = self
            .episodic_memory
            .list_unprocessed(session_id)
            .await
            .map_err(|e| MemoryError::Storage(format!("Failed to list unprocessed: {e}")))?;

        if entries.is_empty() {
            return Ok(0);
        }

        // Consolidate (engine marks entries as processed)
        let result = self.engine.consolidate(&[session_id], &mut entries).await?;

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.decisions_made += (result.entities_added
                + result.entities_updated
                + result.entities_deleted
                + result.entries_skipped) as u64;
        }

        Ok(result.entries_processed)
    }

    /// Get sessions with unprocessed entries
    ///
    /// Sessions are returned ordered by last activity (most recent first).
    async fn get_sessions_with_unprocessed(&self) -> Result<Vec<String>> {
        self.episodic_memory
            .list_sessions_with_unprocessed()
            .await
            .map_err(|e| MemoryError::Storage(format!("Failed to list sessions: {e}")))
    }

    /// Prioritize sessions for consolidation
    ///
    /// Sessions are already ordered by last activity from `list_sessions_with_unprocessed()`.
    /// This method can apply additional prioritization logic (e.g., session metadata).
    ///
    /// Current implementation: pass-through (sessions already prioritized by last activity).
    #[inline]
    const fn prioritize_sessions(sessions: Vec<String>) -> Vec<String> {
        // Sessions are already ordered by last activity (descending) from list_sessions_with_unprocessed()
        // Additional prioritization logic can be added here (e.g., session importance, user tier)
        sessions
    }

    /// Count total unprocessed entries across all sessions
    async fn count_unprocessed_total(&self) -> Result<usize> {
        let sessions = self.get_sessions_with_unprocessed().await?;

        let mut total = 0;
        for session_id in sessions {
            let entries = self
                .episodic_memory
                .list_unprocessed(&session_id)
                .await
                .map_err(|e| MemoryError::Storage(format!("Failed to list unprocessed: {e}")))?;
            total += entries.len();
        }

        Ok(total)
    }

    /// Select interval based on queue depth
    const fn select_interval(&self, queue_depth: usize) -> Duration {
        if queue_depth > 100 {
            Duration::from_secs(self.config.fast_interval_secs)
        } else if queue_depth >= 10 {
            Duration::from_secs(self.config.normal_interval_secs)
        } else {
            Duration::from_secs(self.config.slow_interval_secs)
        }
    }
}

/// RAII guard for tracking in-flight operations
///
/// Increments counter on creation, decrements on drop (aligned with llmspell-kernel `OperationGuard`).
struct OperationGuard {
    counter: Arc<AtomicU64>,
}

impl OperationGuard {
    fn new(counter: Arc<AtomicU64>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        Self { counter }
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        let prev = self.counter.fetch_sub(1, Ordering::SeqCst);
        if prev == 0 {
            error!("Operation counter underflow!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consolidation::prompts::PromptVersion;
    use crate::consolidation::LLMConsolidationConfig;
    use crate::types::EpisodicEntry;
    use async_trait::async_trait;
    use chrono::Utc;
    use llmspell_graph::traits::KnowledgeGraph;
    use llmspell_graph::types::{Entity, Relationship, TemporalQuery};
    use llmspell_providers::{ProviderCapabilities, ProviderInstance};
    use std::collections::HashMap;
    use std::pin::Pin;

    // Mock episodic memory
    struct MockEpisodicMemory {
        entries: Mutex<Vec<EpisodicEntry>>,
    }

    impl MockEpisodicMemory {
        fn new() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EpisodicMemory for MockEpisodicMemory {
        async fn add(&self, entry: EpisodicEntry) -> Result<String> {
            let id = entry.id.clone();
            self.entries.lock().await.push(entry);
            Ok(id)
        }

        async fn get(&self, id: &str) -> Result<EpisodicEntry> {
            self.entries
                .lock()
                .await
                .iter()
                .find(|e| e.id == id)
                .cloned()
                .ok_or_else(|| MemoryError::NotFound(format!("Entry not found: {id}")))
        }

        async fn search(&self, _query: &str, _top_k: usize) -> Result<Vec<EpisodicEntry>> {
            Ok(Vec::new())
        }

        async fn list_unprocessed(&self, _session_id: &str) -> Result<Vec<EpisodicEntry>> {
            Ok(Vec::new())
        }

        async fn get_session(&self, session_id: &str) -> Result<Vec<EpisodicEntry>> {
            Ok(self
                .entries
                .lock()
                .await
                .iter()
                .filter(|e| e.session_id == session_id)
                .cloned()
                .collect())
        }

        async fn mark_processed(&self, _entry_ids: &[String]) -> Result<()> {
            Ok(())
        }

        async fn delete_before(&self, _timestamp: chrono::DateTime<Utc>) -> Result<usize> {
            Ok(0)
        }

        async fn list_sessions_with_unprocessed(&self) -> Result<Vec<String>> {
            use std::collections::HashSet;

            let sessions: HashSet<String> = self
                .entries
                .lock()
                .await
                .iter()
                .filter(|e| !e.processed)
                .map(|e| e.session_id.clone())
                .collect();

            Ok(sessions.into_iter().collect())
        }
    }

    // Mock knowledge graph
    struct MockKnowledgeGraph;

    #[async_trait]
    impl KnowledgeGraph for MockKnowledgeGraph {
        async fn add_entity(&self, _entity: Entity) -> llmspell_graph::error::Result<String> {
            Ok("mock-id".to_string())
        }

        async fn update_entity(
            &self,
            _id: &str,
            _changes: HashMap<String, serde_json::Value>,
        ) -> llmspell_graph::error::Result<()> {
            Ok(())
        }

        async fn get_entity(&self, id: &str) -> llmspell_graph::error::Result<Entity> {
            Err(llmspell_graph::error::GraphError::EntityNotFound(
                id.to_string(),
            ))
        }

        async fn get_entity_at(
            &self,
            _id: &str,
            _event_time: chrono::DateTime<Utc>,
        ) -> llmspell_graph::error::Result<Entity> {
            Err(llmspell_graph::error::GraphError::EntityNotFound(
                "mock".to_string(),
            ))
        }

        async fn add_relationship(
            &self,
            _relationship: Relationship,
        ) -> llmspell_graph::error::Result<String> {
            Ok("mock-rel-id".to_string())
        }

        async fn get_related(
            &self,
            _entity_id: &str,
            _relationship_type: &str,
        ) -> llmspell_graph::error::Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn query_temporal(
            &self,
            _query: TemporalQuery,
        ) -> llmspell_graph::error::Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn delete_before(
            &self,
            _timestamp: chrono::DateTime<Utc>,
        ) -> llmspell_graph::error::Result<usize> {
            Ok(0)
        }
    }

    // Mock provider
    struct MockProvider;

    #[async_trait]
    impl ProviderInstance for MockProvider {
        fn complete<'life0, 'life1, 'async_trait>(
            &'life0 self,
            _input: &'life1 llmspell_core::types::AgentInput,
        ) -> Pin<
            Box<
                dyn std::future::Future<
                        Output = llmspell_core::error::Result<llmspell_core::types::AgentOutput>,
                    > + Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async { Ok(llmspell_core::types::AgentOutput::text("{}".to_string())) })
        }

        async fn validate(&self) -> llmspell_core::error::Result<()> {
            Ok(())
        }

        fn name(&self) -> &'static str {
            "mock"
        }

        fn model(&self) -> &'static str {
            "mock-model"
        }

        fn capabilities(&self) -> &ProviderCapabilities {
            use std::sync::LazyLock;

            static CAPS: LazyLock<ProviderCapabilities> = LazyLock::new(|| ProviderCapabilities {
                supports_streaming: false,
                supports_multimodal: false,
                max_context_tokens: Some(2048),
                max_output_tokens: Some(1024),
                available_models: vec!["mock-model".to_string()],
                custom_features: std::collections::HashMap::new(),
            });

            &CAPS
        }
    }

    #[tokio::test]
    async fn test_daemon_creation() {
        let provider = Arc::new(MockProvider) as Arc<dyn ProviderInstance>;
        let graph = Arc::new(MockKnowledgeGraph) as Arc<dyn KnowledgeGraph>;
        let memory = Arc::new(MockEpisodicMemory::new()) as Arc<dyn EpisodicMemory>;

        let config = LLMConsolidationConfig {
            model: "ollama/llama3.2:3b".to_string(),
            fallback_models: vec![],
            temperature: 0.0,
            max_tokens: 2000,
            timeout_secs: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            version: PromptVersion::V1,
        };

        let engine = Arc::new(LLMConsolidationEngine::new(provider, graph, config));
        let daemon_config = DaemonConfig::default();

        let daemon = ConsolidationDaemon::new(engine, memory, daemon_config);

        assert!(!daemon.is_running());
    }

    #[tokio::test]
    async fn test_daemon_start_stop() {
        let provider = Arc::new(MockProvider) as Arc<dyn ProviderInstance>;
        let graph = Arc::new(MockKnowledgeGraph) as Arc<dyn KnowledgeGraph>;
        let memory = Arc::new(MockEpisodicMemory::new()) as Arc<dyn EpisodicMemory>;

        let config = LLMConsolidationConfig {
            model: "ollama/llama3.2:3b".to_string(),
            fallback_models: vec![],
            temperature: 0.0,
            max_tokens: 2000,
            timeout_secs: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            version: PromptVersion::V1,
        };

        let engine = Arc::new(LLMConsolidationEngine::new(provider, graph, config));
        let daemon_config = DaemonConfig {
            fast_interval_secs: 1, // 1s for fast test
            normal_interval_secs: 5,
            slow_interval_secs: 10,
            ..Default::default()
        };

        let daemon = Arc::new(ConsolidationDaemon::new(engine, memory, daemon_config));

        // Start daemon
        let handle = Arc::clone(&daemon).start().unwrap();
        assert!(daemon.is_running());

        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop daemon
        daemon.stop().await.unwrap();
        assert!(!daemon.is_running());

        // Wait for handle
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_select_interval() {
        let provider = Arc::new(MockProvider) as Arc<dyn ProviderInstance>;
        let graph = Arc::new(MockKnowledgeGraph) as Arc<dyn KnowledgeGraph>;
        let memory = Arc::new(MockEpisodicMemory::new()) as Arc<dyn EpisodicMemory>;

        let config = LLMConsolidationConfig {
            model: "ollama/llama3.2:3b".to_string(),
            fallback_models: vec![],
            temperature: 0.0,
            max_tokens: 2000,
            timeout_secs: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            version: PromptVersion::V1,
        };

        let engine = Arc::new(LLMConsolidationEngine::new(provider, graph, config));
        let daemon_config = DaemonConfig::default();

        let daemon = ConsolidationDaemon::new(engine, memory, daemon_config);

        // Fast interval: >100 entries
        assert_eq!(daemon.select_interval(101), Duration::from_secs(30));

        // Normal interval: 10-100 entries
        assert_eq!(daemon.select_interval(50), Duration::from_secs(300));
        assert_eq!(daemon.select_interval(10), Duration::from_secs(300));

        // Slow interval: <10 entries
        assert_eq!(daemon.select_interval(9), Duration::from_secs(1800));
        assert_eq!(daemon.select_interval(0), Duration::from_secs(1800));
    }

    #[tokio::test]
    async fn test_operation_guard() {
        let counter = Arc::new(AtomicU64::new(0));

        assert_eq!(counter.load(Ordering::SeqCst), 0);

        {
            let _guard1 = OperationGuard::new(Arc::clone(&counter));
            assert_eq!(counter.load(Ordering::SeqCst), 1);

            {
                let _guard2 = OperationGuard::new(Arc::clone(&counter));
                assert_eq!(counter.load(Ordering::SeqCst), 2);
            }

            assert_eq!(counter.load(Ordering::SeqCst), 1);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
