// ABOUTME: Advanced tokio-stream integration for high-throughput event processing
// ABOUTME: Provides streaming event processing with backpressure and batching

use crate::bus::EventBus;
use crate::universal_event::UniversalEvent;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio_stream::wrappers::BroadcastStream;

/// High-throughput event stream processor
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = Result<UniversalEvent, EventStreamError>> + Send>>,
}

/// Event stream processing errors
#[derive(Debug, thiserror::Error)]
pub enum EventStreamError {
    #[error("Stream lagged behind")]
    Lagged(u64),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

impl EventStream {
    /// Create a new event stream from an EventBus
    pub fn from_bus(bus: &EventBus) -> Self {
        let receiver = bus.subscribe_all();
        let stream = BroadcastStream::new(receiver).map(|result| {
            result.map_err(|e| match e {
                tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n) => {
                    EventStreamError::Lagged(n)
                }
            })
        });

        Self {
            inner: Box::pin(stream),
        }
    }

    /// Create a batched stream that collects events into batches
    pub fn batched(self, batch_size: usize) -> BatchedEventStream {
        BatchedEventStream::new(self, batch_size)
    }

    /// Create a filtered stream that only processes certain event types
    pub fn filtered<F>(self, predicate: F) -> FilteredEventStream<F>
    where
        F: Fn(&UniversalEvent) -> bool + Send + 'static,
    {
        FilteredEventStream::new(self, predicate)
    }
}

impl Stream for EventStream {
    type Item = Result<UniversalEvent, EventStreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Batched event stream for bulk processing
pub struct BatchedEventStream {
    inner: EventStream,
    batch_size: usize,
    current_batch: Vec<UniversalEvent>,
}

impl BatchedEventStream {
    fn new(stream: EventStream, batch_size: usize) -> Self {
        Self {
            inner: stream,
            batch_size,
            current_batch: Vec::with_capacity(batch_size),
        }
    }
}

impl Stream for BatchedEventStream {
    type Item = Result<Vec<UniversalEvent>, EventStreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(event))) => {
                    self.current_batch.push(event);
                    if self.current_batch.len() >= self.batch_size {
                        let batch_size = self.batch_size;
                        let batch = std::mem::replace(
                            &mut self.current_batch,
                            Vec::with_capacity(batch_size),
                        );
                        return Poll::Ready(Some(Ok(batch)));
                    }
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    if !self.current_batch.is_empty() {
                        let batch = std::mem::take(&mut self.current_batch);
                        return Poll::Ready(Some(Ok(batch)));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    if !self.current_batch.is_empty() {
                        // Return partial batch after timeout
                        let batch_size = self.batch_size;
                        let batch = std::mem::replace(
                            &mut self.current_batch,
                            Vec::with_capacity(batch_size),
                        );
                        return Poll::Ready(Some(Ok(batch)));
                    }
                    return Poll::Pending;
                }
            }
        }
    }
}

/// Filtered event stream
pub struct FilteredEventStream<F> {
    inner: EventStream,
    predicate: F,
}

impl<F> FilteredEventStream<F>
where
    F: Fn(&UniversalEvent) -> bool,
{
    fn new(stream: EventStream, predicate: F) -> Self {
        Self {
            inner: stream,
            predicate,
        }
    }
}

impl<F> Stream for FilteredEventStream<F>
where
    F: Fn(&UniversalEvent) -> bool + Unpin,
{
    type Item = Result<UniversalEvent, EventStreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(event))) => {
                    if (self.predicate)(&event) {
                        return Poll::Ready(Some(Ok(event)));
                    }
                    // Continue polling for next event
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// High-throughput stream processor with advanced features
pub struct HighThroughputProcessor {
    buffer_size: usize,
    worker_count: usize,
}

impl HighThroughputProcessor {
    /// Create a new high-throughput processor
    pub fn new(buffer_size: usize, worker_count: usize) -> Self {
        Self {
            buffer_size,
            worker_count,
        }
    }

    /// Process events from a stream with parallel workers
    pub async fn process_stream<S, F, Fut>(
        &self,
        mut stream: S,
        processor: F,
    ) -> Result<(), EventStreamError>
    where
        S: Stream<Item = Result<UniversalEvent, EventStreamError>> + Unpin,
        F: Fn(UniversalEvent) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<UniversalEvent>(self.buffer_size);
        let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));

        // Spawn worker tasks
        let mut workers = Vec::new();
        for worker_id in 0..self.worker_count {
            let worker_rx = rx.clone();
            let worker_processor = processor.clone();

            let worker = tokio::spawn(async move {
                loop {
                    let event = {
                        let mut rx_guard = worker_rx.lock().await;
                        match rx_guard.recv().await {
                            Some(event) => event,
                            None => break, // Channel closed
                        }
                    };

                    if let Err(e) = worker_processor(event).await {
                        tracing::error!("Worker {} processing error: {}", worker_id, e);
                    }
                }
            });

            workers.push(worker);
        }

        // Feed events to workers
        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if tx.send(event).await.is_err() {
                        break; // All workers have finished
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    return Err(e);
                }
            }
        }

        // Close the sender to signal workers to finish
        drop(tx);

        // Wait for all workers to complete
        for worker in workers {
            let _ = worker.await;
        }

        Ok(())
    }
}

/// Stream utilities for high-frequency processing
pub struct StreamUtils;

impl StreamUtils {
    /// Create a high-frequency test stream
    pub fn high_frequency_test_stream(
        event_count: usize,
        events_per_second: u64,
    ) -> Pin<Box<dyn Stream<Item = Result<UniversalEvent, EventStreamError>> + Send>> {
        let interval_duration = std::time::Duration::from_nanos(1_000_000_000 / events_per_second);

        Box::pin(
            futures::stream::iter(0..event_count).then(move |i| async move {
                if i > 0 {
                    tokio::time::sleep(interval_duration).await;
                }
                Ok(UniversalEvent::new(
                    format!("test.high_freq.{}", i),
                    serde_json::json!({"sequence": i, "timestamp": std::time::SystemTime::now()}),
                    crate::universal_event::Language::Rust,
                ))
            }),
        )
    }

    /// Measure stream throughput
    pub async fn measure_throughput<S>(
        mut stream: S,
        duration: std::time::Duration,
    ) -> ThroughputMeasurement
    where
        S: Stream<Item = Result<UniversalEvent, EventStreamError>> + Unpin,
    {
        let start_time = std::time::Instant::now();
        let mut event_count = 0u64;
        let mut error_count = 0u64;

        let timeout = tokio::time::sleep(duration);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                result = stream.next() => {
                    match result {
                        Some(Ok(_)) => event_count += 1,
                        Some(Err(_)) => error_count += 1,
                        None => break,
                    }
                }
                _ = &mut timeout => break,
            }
        }

        let elapsed = start_time.elapsed();
        let events_per_second = event_count as f64 / elapsed.as_secs_f64();

        ThroughputMeasurement {
            event_count,
            error_count,
            elapsed,
            events_per_second,
        }
    }
}

/// Throughput measurement results
#[derive(Debug)]
pub struct ThroughputMeasurement {
    pub event_count: u64,
    pub error_count: u64,
    pub elapsed: std::time::Duration,
    pub events_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_event::Language;
    use serde_json::Value;
    use tokio_stream::StreamExt;

    fn create_test_event(event_type: &str) -> UniversalEvent {
        UniversalEvent::new(event_type, Value::Null, Language::Rust)
    }
    #[tokio::test]
    async fn test_batched_stream() {
        let events = vec![
            create_test_event("test1"),
            create_test_event("test2"),
            create_test_event("test3"),
        ];

        let stream = tokio_stream::iter(events.into_iter().map(Ok));
        let event_stream = EventStream {
            inner: Box::pin(stream),
        };

        let mut batched = event_stream.batched(2);

        let batch1 = batched.next().await.unwrap().unwrap();
        assert_eq!(batch1.len(), 2);

        let batch2 = batched.next().await.unwrap().unwrap();
        assert_eq!(batch2.len(), 1);
    }
    #[tokio::test]
    async fn test_filtered_stream() {
        let events = vec![
            create_test_event("system.start"),
            create_test_event("user.action"),
            create_test_event("system.stop"),
        ];

        let stream = tokio_stream::iter(events.into_iter().map(Ok));
        let event_stream = EventStream {
            inner: Box::pin(stream),
        };

        let mut filtered = event_stream.filtered(|event| event.event_type.starts_with("system"));

        let event1 = filtered.next().await.unwrap().unwrap();
        assert_eq!(event1.event_type, "system.start");

        let event2 = filtered.next().await.unwrap().unwrap();
        assert_eq!(event2.event_type, "system.stop");

        assert!(filtered.next().await.is_none());
    }
    #[tokio::test]
    async fn test_high_frequency_stream() {
        let stream = StreamUtils::high_frequency_test_stream(100, 1000);
        let measurement =
            StreamUtils::measure_throughput(stream, std::time::Duration::from_millis(200)).await;

        // Should process at least some events
        assert!(measurement.event_count > 0);
        assert!(measurement.events_per_second > 0.0);
    }
    #[tokio::test]
    async fn test_high_throughput_processor() {
        let processor = HighThroughputProcessor::new(1000, 4);

        let events = (0..1000).map(|i| Ok(create_test_event(&format!("test.{}", i))));
        let stream = tokio_stream::iter(events);

        let processed_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let counter = processed_count.clone();

        let process_fn = move |_event: UniversalEvent| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
        };

        processor.process_stream(stream, process_fn).await.unwrap();

        assert_eq!(
            processed_count.load(std::sync::atomic::Ordering::Relaxed),
            1000
        );
    }
}
