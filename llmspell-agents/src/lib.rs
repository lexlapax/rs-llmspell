//! ABOUTME: Agent infrastructure for rs-llmspell
//! ABOUTME: Provides factory, registry, and lifecycle management for agents

pub mod agents;
pub mod builder;
pub mod config;
pub mod di;
pub mod factory;
pub mod factory_registry;
pub mod lifecycle;
pub mod registry;

// Re-export main types and traits
pub use builder::AgentBuilder;
pub use config::{ConfigLoader, DefaultTemplates};
pub use di::{DIContainer, DIContainerBuilder, ScopedDIContainer};
pub use factory::{
    AgentConfig, AgentFactory, CreationHook, DefaultAgentFactory, ModelConfig, ResourceLimits,
};
pub use factory_registry::{global_registry, CustomAgentFactory, FactoryRegistry};
pub use lifecycle::hooks::{CompositeHook, LoggingHook, MetricsHook, SecurityHook, ValidationHook};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        global_registry, AgentBuilder, AgentConfig, AgentFactory, CreationHook, DIContainer,
        DefaultAgentFactory, DefaultTemplates,
    };
}
