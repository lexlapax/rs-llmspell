//! Memory provider for lazy initialization
//!
//! Allows deferring the heavy initialization of `MemoryManager` until it is actually needed.

use crate::globals::types::{LazyInitializationError, LazyMemoryInitializer};
use llmspell_memory::MemoryManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// State of the memory provider
enum ProviderState {
    /// Already initialized
    Initialized(Arc<dyn MemoryManager>),
    /// Waiting for initialization
    Uninitialized(LazyMemoryInitializer),
    /// Currently initializing (to handle re-entrancy/race)
    /// Note: We use RwLock so we can upgrade, but for simple lazy init we just need to ensure one init happens.
    /// Using a simple internal option/state is enough.
    #[allow(dead_code)]
    Initializing,
}

/// Thread-safe provider for MemoryManager that supports lazy initialization
#[derive(Clone)]
pub struct MemoryProvider {
    state: Arc<RwLock<ProviderState>>,
}

impl MemoryProvider {
    /// Create a new eager provider (already initialized)
    pub fn new_eager(manager: Arc<dyn MemoryManager>) -> Self {
        Self {
            state: Arc::new(RwLock::new(ProviderState::Initialized(manager))),
        }
    }

    /// Create a new lazy provider
    pub fn new_lazy(initializer: LazyMemoryInitializer) -> Self {
        Self {
            state: Arc::new(RwLock::new(ProviderState::Uninitialized(initializer))),
        }
    }

    /// Get the memory manager, initializing it if necessary
    pub async fn get(&self) -> Result<Arc<dyn MemoryManager>, LazyInitializationError> {
        // Fast path: check if initialized
        {
            let guard = self.state.read().await;
            if let ProviderState::Initialized(manager) = &*guard {
                return Ok(manager.clone());
            }
        }

        // Slow path: initialize
        let mut guard = self.state.write().await;

        // Double-check (someone might have initialized while we waited for write lock)
        if let ProviderState::Initialized(manager) = &*guard {
            return Ok(manager.clone());
        }

        // Extract initializer
        // We have to move out the initializer to call it, replacing state temporarily or effectively using Option
        // But `LazyMemoryInitializer` is a Box<dyn Fn>, so we can call it.
        // Wait, to call it we just need reference if it's Fn. `Box<dyn Fn>` is callable.

        let manager_opt = if let ProviderState::Uninitialized(init) = &*guard {
            // Call the initializer
            // It returns a BoxFuture
            init().await
        } else {
            // Should be impossible due to double check
            return Err(LazyInitializationError::InitializationFailed);
        };

        if let Some(manager) = manager_opt {
            *guard = ProviderState::Initialized(manager.clone());
            Ok(manager)
        } else {
            Err(LazyInitializationError::InitializationFailed)
        }
    }
}
