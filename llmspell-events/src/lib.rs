// ABOUTME: Event bus and cross-language event system for rs-llmspell
// ABOUTME: Provides UniversalEvent format, FlowController, and EventBus with backpressure

//! # LLMSpell Events
//!
//! This crate provides the event system for rs-llmspell with:
//! - Cross-language event support via UniversalEvent
//! - Backpressure handling with FlowController
//! - High-performance async event bus
//! - Pattern-based event routing
//!
//! ## Features
//!
//! - **UniversalEvent**: Language-agnostic event format
//! - **FlowController**: Rate limiting and backpressure
//! - **EventBus**: Async pub/sub with pattern matching
//! - **EventStorageAdapter**: Unified storage integration via llmspell-storage
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_events::{EventBus, UniversalEvent, Language, EventStorageAdapter};
//! use llmspell_storage::backends::MemoryBackend;
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create event bus (with optional persistence)
//!     let backend = MemoryBackend::new();
//!     let storage_adapter = EventStorageAdapter::new(backend);
//!     let bus = EventBus::with_persistence(
//!         Default::default(),
//!         storage_adapter,
//!         Default::default()
//!     );
//!     
//!     // Subscribe to events
//!     let mut receiver = bus.subscribe("system.*").await?;
//!     
//!     // Publish an event
//!     let event = UniversalEvent::new(
//!         "system.startup",
//!         serde_json::json!({"status": "ready"}),
//!         Language::Rust,
//!     );
//!     bus.publish(event).await?;
//!     
//!     // Receive events
//!     if let Some(event) = receiver.recv().await {
//!         println!("Received: {:?}", event);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod bus;
pub mod correlation;
pub mod flow_controller;
pub mod handler;
pub mod metrics;
pub mod overflow;
pub mod pattern;
pub mod serialization;
pub mod storage_adapter;
pub mod stream;
pub mod universal_event;

// Re-export main types
pub use bus::{EventBus, EventBusBuilder};
pub use correlation::{CorrelationContext, EventCorrelationTracker, EventLink, EventRelationship};
pub use flow_controller::{BackpressureNotification, FlowController};
pub use handler::{AsyncEventHandler, EventHandler};
pub use metrics::{EventMetrics, MetricsCollector};
pub use overflow::{OverflowHandler, OverflowStrategy};
pub use pattern::{EventPattern, PatternMatcher};
pub use serialization::EventSerializer;
pub use storage_adapter::{
    EventPersistenceManager, EventStorage, EventStorageAdapter, PersistenceConfig, StorageStats,
};
pub use stream::{EventStream, HighThroughputProcessor, StreamUtils, ThroughputMeasurement};
pub use universal_event::{EventMetadata, Language, UniversalEvent};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        AsyncEventHandler, BackpressureNotification, CorrelationContext, EventBus, EventBusBuilder,
        EventCorrelationTracker, EventHandler, EventLink, EventMetadata, EventPattern,
        EventRelationship, EventStream, FlowController, HighThroughputProcessor, Language,
        OverflowStrategy, StreamUtils, UniversalEvent,
    };
}
