//! Protocol registry for future extensibility
//!
//! This module provides a registry pattern for protocols,
//! preparing for Phase 18 multi-language support.

use crate::protocols::ProtocolConfig;
use crate::traits::Protocol;
use anyhow::Result;
use std::collections::HashMap;

/// Factory trait for creating protocol instances
pub trait ProtocolFactory: Send + Sync {
    /// Create a new protocol instance
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol cannot be created
    fn create(&self, config: ProtocolConfig) -> Result<Box<dyn Protocol>>;

    /// Get the name of this protocol
    fn name(&self) -> &str;
}

/// Registry for protocol implementations
pub struct ProtocolRegistry {
    protocols: HashMap<String, Box<dyn ProtocolFactory>>,
}

impl ProtocolRegistry {
    /// Create a new protocol registry
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
        }
    }

    /// Register a protocol factory
    pub fn register(&mut self, factory: Box<dyn ProtocolFactory>) {
        self.protocols.insert(factory.name().to_string(), factory);
    }

    /// Create a protocol instance by name
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol is not registered or creation fails
    pub fn create(&self, name: &str, config: ProtocolConfig) -> Result<Box<dyn Protocol>> {
        self.protocols
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown protocol: {}", name))?
            .create(config)
    }

    /// List all registered protocols
    pub fn list(&self) -> Vec<String> {
        self.protocols.keys().cloned().collect()
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
