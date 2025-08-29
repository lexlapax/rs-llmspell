//! Multi-tenant infrastructure for rs-llmspell
//!
//! This crate provides comprehensive multi-tenant support that can be used across
//! all components of rs-llmspell including agents, workflows, tools, and storage.

#![allow(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::unused_async)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::explicit_iter_loop)]

pub mod manager;
pub mod registry;
pub mod traits;
pub mod usage;

pub use manager::{MultiTenantVectorManager, TenantInfo};
pub use registry::{DefaultTenantRegistry, FilteredTenantRegistry};
pub use traits::{
    IsolationMode, MultiTenancyConfig, TenantConfig, TenantExtension, TenantLifecycleHook,
    TenantLimits, TenantOperationResult, TenantRegistry, TenantResourceManager, TenantScoped,
    UsageTracked,
};
pub use usage::{CostEstimate, CostRates, TenantUsageTracker, UsageMetrics, UsageReport};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
