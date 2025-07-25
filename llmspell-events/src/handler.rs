// ABOUTME: Event handler traits for processing events
// ABOUTME: Provides both sync and async event handler interfaces

use crate::universal_event::UniversalEvent;
use anyhow::Result;
use async_trait::async_trait;

/// Synchronous event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an event synchronously
    fn handle_event(&self, event: UniversalEvent) -> Result<()>;

    /// Get handler name for debugging
    fn name(&self) -> &str {
        "anonymous_handler"
    }
}

/// Asynchronous event handler trait
#[async_trait]
pub trait AsyncEventHandler: Send + Sync {
    /// Handle an event asynchronously
    async fn handle_event(&self, event: UniversalEvent) -> Result<()>;

    /// Get handler name for debugging
    fn name(&self) -> &str {
        "anonymous_async_handler"
    }
}

/// Function-based event handler
pub struct FnEventHandler<F> {
    name: String,
    handler: F,
}

impl<F> FnEventHandler<F>
where
    F: Fn(UniversalEvent) -> Result<()> + Send + Sync,
{
    pub fn new(name: &str, handler: F) -> Self {
        Self {
            name: name.to_string(),
            handler,
        }
    }
}

impl<F> EventHandler for FnEventHandler<F>
where
    F: Fn(UniversalEvent) -> Result<()> + Send + Sync,
{
    fn handle_event(&self, event: UniversalEvent) -> Result<()> {
        (self.handler)(event)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
