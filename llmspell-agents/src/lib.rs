//! ABOUTME: Agent infrastructure for rs-llmspell
//! ABOUTME: Provides factory, registry, and lifecycle management for agents

pub mod agent_wrapped_tool;
pub mod agents;
pub mod builder;
pub mod composition;
pub mod config;
pub mod di;
pub mod factory;
pub mod factory_registry;
pub mod health;
pub mod lifecycle;
pub mod registry;
pub mod templates;
pub mod tool_context;
pub mod tool_discovery;
pub mod tool_errors;
pub mod tool_invocation;
pub mod tool_manager;

// Re-export main types and traits
pub use agent_wrapped_tool::{
    AgentWrappedTool, ParameterMappingConfig, ParameterTransform, ToolMetadata, TransformType,
};
pub use builder::AgentBuilder;
pub use composition::{CompositionStep, DataFlow, DataTransform, ToolComposition, ToolProvider};
pub use config::{ConfigLoader, DefaultTemplates};
pub use di::{DIContainer, DIContainerBuilder, ScopedDIContainer};
pub use factory::{
    AgentConfig, AgentFactory, CreationHook, DefaultAgentFactory, ModelConfig, ResourceLimits,
};
pub use factory_registry::{global_registry, CustomAgentFactory, FactoryRegistry};
pub use health::{
    AgentHealthMonitor, HealthCheckResult, HealthIssue, HealthMonitorConfig, HealthStatus,
    ResourceHealthCheck, ResponsivenessHealthCheck, StateMachineHealthCheck,
};
pub use lifecycle::{
    events::{
        EventSubscription, LifecycleEvent, LifecycleEventData, LifecycleEventSystem,
        LifecycleEventType, LoggingEventListener, MetricsEventListener,
    },
    hooks::{CompositeHook, LoggingHook, MetricsHook, SecurityHook, ValidationHook},
    middleware::{
        LifecycleMiddleware, LifecycleMiddlewareChain, LifecyclePhase, LoggingMiddleware,
        MetricsMiddleware, MiddlewareConfig, MiddlewareContext,
    },
    resources::{
        LoggingResourceHook, ResourceAllocation, ResourceManager, ResourceRequest, ResourceType,
    },
    shutdown::{
        LoggingShutdownHook, ResourceCleanupHook, ShutdownConfig, ShutdownCoordinator,
        ShutdownPriority, ShutdownRequest, ShutdownResult,
    },
    state_machine::{AgentState, AgentStateMachine, StateMachineConfig, StateMachineMetrics},
};
pub use tool_context::{
    ContextInheritanceRule, ToolContextManager, ToolExecutionContext, ToolExecutionRecord,
};
pub use tool_discovery::{RecommendationContext, ToolDiscoveryService, ToolSearchCriteria};
pub use tool_errors::{
    ErrorContext, ErrorRecoveryStrategy, RecoveryAction, ToolErrorHandler, ToolIntegrationError,
};
pub use tool_invocation::{InvocationConfig, InvocationMetrics, InvocationResult, ToolInvoker};
pub use tool_manager::{ToolManager, ToolManagerConfig};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        global_registry, AgentBuilder, AgentConfig, AgentFactory, CreationHook, DIContainer,
        DefaultAgentFactory, DefaultTemplates,
    };
}
