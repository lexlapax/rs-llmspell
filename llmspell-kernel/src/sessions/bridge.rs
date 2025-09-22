//! ABOUTME: Script bridge implementations for session management in Lua and other languages
//! ABOUTME: Provides Session global object for script access to session functionality

// Re-export submodules
pub mod conversions;
pub mod errors;
pub mod operations;

// Re-export key types
pub use conversions::*;
pub use errors::*;
pub use operations::*;

// Types will be used when implementing full bridge functionality

/// Script bridge stub - to be implemented in Phase 6.5
pub struct SessionBridge {
    // Implementation to be added in Phase 6.5
    _marker: std::marker::PhantomData<()>,
}

impl SessionBridge {
    /// Create new session bridge
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for SessionBridge {
    fn default() -> Self {
        Self::new()
    }
}

// Full Lua bridge implementation will be added in Phase 6.5
