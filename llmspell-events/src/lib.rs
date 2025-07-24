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
//! - **CrossLanguageEventBridge**: Event propagation between languages
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_events::{EventBus, UniversalEvent, Language};
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut bus = EventBus::new();
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

pub mod bridge;
pub mod bus;
pub mod flow_controller;
pub mod handler;
pub mod language_adapters;
pub mod metrics;
pub mod overflow;
pub mod pattern;
pub mod serialization;
pub mod universal_event;

// Re-export main types
pub use bridge::{CrossLanguageEventBridge, EventBridge};
pub use bus::{EventBus, EventBusBuilder};
pub use flow_controller::{BackpressureNotification, FlowController};
pub use handler::{AsyncEventHandler, EventHandler};
pub use metrics::{EventMetrics, MetricsCollector};
pub use overflow::{OverflowHandler, OverflowStrategy};
pub use pattern::{EventPattern, PatternMatcher};
pub use universal_event::{EventMetadata, Language, UniversalEvent};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        AsyncEventHandler, BackpressureNotification, EventBus, EventBusBuilder, EventHandler,
        EventMetadata, EventPattern, FlowController, Language, OverflowStrategy, UniversalEvent,
    };
}
