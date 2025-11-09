//! ABOUTME: PostgreSQL configuration types
//! ABOUTME: Connection settings, pool configuration, and RLS options

use serde::{Deserialize, Serialize};

/// PostgreSQL backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database connection string (format: postgresql://user:pass@host:port/database)
    pub connection_string: String,

    /// Maximum number of connections in the pool (default: 20)
    pub max_pool_size: u32,

    /// Connection timeout in milliseconds (default: 5000)
    pub connection_timeout_ms: u64,

    /// Enable Row-Level Security (RLS) for multi-tenancy (default: true)
    pub enable_rls: bool,

    /// Tenant ID for RLS context (optional, set via set_tenant_context)
    #[serde(skip)]
    pub tenant_id: Option<String>,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            connection_string: String::new(),
            max_pool_size: 20,
            connection_timeout_ms: 5000,
            enable_rls: true,
            tenant_id: None,
        }
    }
}

impl PostgresConfig {
    /// Create a new PostgreSQL configuration with connection string
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            ..Default::default()
        }
    }

    /// Set the maximum pool size
    pub fn with_max_pool_size(mut self, max_pool_size: u32) -> Self {
        self.max_pool_size = max_pool_size;
        self
    }

    /// Set the connection timeout
    pub fn with_connection_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout_ms = timeout_ms;
        self
    }

    /// Enable or disable RLS
    pub fn with_rls(mut self, enable_rls: bool) -> Self {
        self.enable_rls = enable_rls;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.connection_string.is_empty() {
            return Err("Connection string cannot be empty".to_string());
        }

        if self.max_pool_size == 0 {
            return Err("Max pool size must be greater than 0".to_string());
        }

        if self.max_pool_size > 100 {
            return Err("Max pool size should not exceed 100".to_string());
        }

        Ok(())
    }
}
