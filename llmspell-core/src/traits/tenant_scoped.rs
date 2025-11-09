//! Tenant-scoped trait for multi-tenant resource management
//!
//! # Phase 13b.3.4: Async Trait Migration
//!
//! This trait enables components to be scoped to specific tenants, providing:
//! - Tenant context management
//! - State scope definition
//! - Async operations for backends with async tenant context (e.g., PostgreSQL RLS)

use crate::state::StateScope;
use anyhow::Result;
use async_trait::async_trait;

/// Tenant-aware resource that can be scoped to a specific tenant
///
/// This trait was migrated to async to support backends with async tenant context management
/// (e.g., PostgreSQL with session variables). All methods that interact with backend state
/// are now async to allow for I/O operations without blocking.
///
/// # Example
///
/// ```rust
/// use llmspell_core::{TenantScoped, state::StateScope};
/// use async_trait::async_trait;
/// use anyhow::Result;
/// use std::sync::{Arc, RwLock};
///
/// struct MyBackend {
///     tenant_id: Arc<RwLock<Option<String>>>,
///     scope: StateScope,
/// }
///
/// #[async_trait]
/// impl TenantScoped for MyBackend {
///     async fn tenant_id(&self) -> Option<String> {
///         self.tenant_id.read().unwrap().clone()
///     }
///
///     fn scope(&self) -> &StateScope {
///         &self.scope  // Return reference to field
///     }
///
///     async fn set_tenant_context(&self, tenant_id: String, _scope: StateScope) -> Result<()> {
///         *self.tenant_id.write().unwrap() = Some(tenant_id);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait TenantScoped: Send + Sync {
    /// Get the tenant ID this resource belongs to
    ///
    /// Returns owned String to support async backends that may need to query
    /// tenant context from external sources (databases, Redis, etc.)
    async fn tenant_id(&self) -> Option<String>;

    /// Get the state scope for this tenant (sync - simple getter)
    fn scope(&self) -> &StateScope;

    /// Set the tenant context
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant identifier to set
    /// * `scope` - The state scope for this tenant context
    ///
    /// # Returns
    /// * `Result<()>` - Success or error (enables proper error propagation)
    ///
    /// # Phase 13b.3.4 Changes
    /// - Now async to support backends with async context management
    /// - Changed from `&mut self` to `&self` (interior mutability pattern)
    /// - Returns Result for explicit error handling (no silent failures)
    async fn set_tenant_context(&self, tenant_id: String, scope: StateScope) -> Result<()>;
}
