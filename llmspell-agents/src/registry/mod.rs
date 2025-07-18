//! ABOUTME: Agent registry module organization
//! ABOUTME: Provides discovery, categorization, and persistence for agents

// Core types module
pub mod types;

// Re-export core types at module level
pub use types::{
    AgentMetadata, AgentMetrics, AgentQuery, AgentRegistry, AgentStatus, InMemoryAgentRegistry,
};

// Sub-modules
pub mod categories;
pub mod discovery;
pub mod metadata;
pub mod persistence;
pub mod registration;

// Re-export key types from submodules
pub use categories::{AgentCategory, AgentTag, CategoryManager, StandardCategories, TagType};
pub use discovery::{
    DiscoveryService, RecommendationContext, SearchBuilder, SearchCriteria, SearchResult,
    SortField, SortOrder,
};
pub use metadata::{
    ExtendedAgentMetadata, ExtendedMetadataBuilder, HealthState, HealthStatus, MetadataManager,
    VersionInfo,
};
pub use persistence::PersistentAgentRegistry;
pub use registration::{AgentRegistrar, RegistrationBuilder, RegistrationOptions};
