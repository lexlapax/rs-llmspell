//! ABOUTME: Unified discovery pattern for bridge components
//! ABOUTME: Provides consistent discovery interface across all component types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for unified discovery across bridge components
#[async_trait::async_trait]
pub trait BridgeDiscovery<T>: Send + Sync
where
    T: Clone + Send + Sync,
{
    /// Get all available types with their information
    async fn discover_types(&self) -> Vec<(String, T)>;

    /// Get information about a specific type
    async fn get_type_info(&self, type_name: &str) -> Option<T>;

    /// Check if a type is available
    async fn has_type(&self, type_name: &str) -> bool;

    /// List all available type names
    async fn list_types(&self) -> Vec<String>;

    /// Filter types by criteria (note: cannot be used with dyn trait objects)
    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, T)>
    where
        F: Fn(&str, &T) -> bool + Send,
        Self: Sized;
}

/// Common discovery information fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryInfo {
    /// Type name
    pub type_name: String,
    /// Human-readable description
    pub description: String,
    /// Supported features
    pub features: Vec<String>,
    /// Required parameters
    pub required_params: Vec<String>,
    /// Optional parameters
    pub optional_params: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Default implementation for HashMap-based discovery
#[async_trait::async_trait]
impl<T, S> BridgeDiscovery<T> for HashMap<String, T, S>
where
    T: Clone + Send + Sync,
    S: std::hash::BuildHasher + Send + Sync,
{
    async fn discover_types(&self) -> Vec<(String, T)> {
        self.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    async fn get_type_info(&self, type_name: &str) -> Option<T> {
        self.get(type_name).cloned()
    }

    async fn has_type(&self, type_name: &str) -> bool {
        self.contains_key(type_name)
    }

    async fn list_types(&self) -> Vec<String> {
        self.keys().cloned().collect()
    }

    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, T)>
    where
        F: Fn(&str, &T) -> bool + Send,
    {
        self.iter()
            .filter(|(k, v)| predicate(k, v))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
