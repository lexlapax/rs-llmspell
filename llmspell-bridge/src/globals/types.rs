//! ABOUTME: Core types and traits for the global object injection system
//! ABOUTME: Defines GlobalObject trait and supporting types

use crate::{ComponentRegistry, ProviderManager};
use llmspell_core::Result;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "lua")]
use mlua::Lua;

#[cfg(feature = "javascript")]
use boa_engine::Context;

/// Metadata about a global object
#[derive(Debug, Clone)]
pub struct GlobalMetadata {
    /// Name of the global (e.g., "Agent", "Tool")
    pub name: String,
    /// Description of the global's functionality
    pub description: String,
    /// Other globals this one depends on
    pub dependencies: Vec<String>,
    /// Whether this global is required or optional
    pub required: bool,
    /// Version of the global API
    pub version: String,
}

/// Shared context available to all globals during injection
#[derive(Clone)]
pub struct GlobalContext {
    /// Component registry for tools and agents
    pub registry: Arc<ComponentRegistry>,
    /// Provider manager for LLM access
    pub providers: Arc<ProviderManager>,
    /// Bridge references for cross-global communication
    pub bridge_refs: Arc<parking_lot::RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl GlobalContext {
    /// Create a new global context
    pub fn new(registry: Arc<ComponentRegistry>, providers: Arc<ProviderManager>) -> Self {
        Self {
            registry,
            providers,
            bridge_refs: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Store a bridge reference for cross-global access
    pub fn set_bridge<T: Any + Send + Sync + 'static>(&self, name: &str, bridge: Arc<T>) {
        let mut refs = self.bridge_refs.write();
        refs.insert(name.to_string(), bridge);
    }

    /// Get a bridge reference
    pub fn get_bridge<T: Any + Send + Sync + 'static>(&self, name: &str) -> Option<Arc<T>> {
        let refs = self.bridge_refs.read();
        refs.get(name)
            .and_then(|bridge| bridge.clone().downcast().ok())
    }
}

/// Trait for objects that can be injected as globals into script engines
pub trait GlobalObject: Send + Sync {
    /// Get metadata about this global
    fn metadata(&self) -> GlobalMetadata;

    /// Initialize any resources needed by this global
    fn initialize(&self, _context: &GlobalContext) -> Result<()> {
        Ok(())
    }

    /// Inject this global into a Lua environment
    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<()>;

    /// Inject this global into a JavaScript environment
    #[cfg(feature = "javascript")]
    fn inject_javascript(&self, ctx: &mut Context, context: &GlobalContext) -> Result<()>;

    /// Clean up any resources when the global is removed
    fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

/// Performance metrics for global injection
#[derive(Debug, Default)]
pub struct InjectionMetrics {
    /// Total time spent injecting globals (microseconds)
    pub total_injection_time_us: u64,
    /// Time spent per global (microseconds)
    pub per_global_times: HashMap<String, u64>,
    /// Number of globals injected
    pub globals_injected: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

impl InjectionMetrics {
    /// Check if injection meets performance requirements (<5ms)
    pub fn is_within_bounds(&self) -> bool {
        self.total_injection_time_us < 5000 // 5ms in microseconds
    }

    /// Get average injection time per global
    pub fn average_time_us(&self) -> u64 {
        if self.globals_injected == 0 {
            0
        } else {
            self.total_injection_time_us / self.globals_injected as u64
        }
    }
}
