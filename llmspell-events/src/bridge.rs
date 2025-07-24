// ABOUTME: Cross-language event bridge for propagating events between languages
// ABOUTME: Stub implementation - will be fully implemented in Phase 15

use crate::universal_event::{Language, UniversalEvent};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for cross-language event bridges
#[async_trait]
pub trait EventBridge: Send + Sync {
    /// Propagate an event to another language
    async fn propagate_event(&self, event: UniversalEvent, target_language: Language)
        -> Result<()>;

    /// Get supported target languages
    fn supported_languages(&self) -> Vec<Language>;
}

/// Cross-language event bridge (stub implementation)
pub struct CrossLanguageEventBridge {
    // Stub - will be implemented in Phase 15
}

impl CrossLanguageEventBridge {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CrossLanguageEventBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBridge for CrossLanguageEventBridge {
    async fn propagate_event(
        &self,
        _event: UniversalEvent,
        _target_language: Language,
    ) -> Result<()> {
        // Stub implementation - will be completed in Phase 15
        Ok(())
    }

    fn supported_languages(&self) -> Vec<Language> {
        // Currently only Rust is supported
        vec![Language::Rust]
    }
}
