//! ABOUTME: Composition patterns for agents and tools
//! ABOUTME: Enables complex workflows through agent and tool orchestration

pub mod capabilities;
pub mod delegation;
pub mod hierarchical;
pub mod lifecycle;
pub mod tool_composition;
pub mod traits;

// Re-export main composition types
pub use tool_composition::{
    CompositionError, CompositionErrorStrategy, CompositionMetrics, CompositionResult,
    CompositionStep, ConditionType, DataFlow, DataTransform, ExecutionCondition, OutputTransform,
    RetryConfig, StepErrorStrategy, StepMetrics, StepResult, ToolComposition, ToolProvider,
};

pub use traits::{
    Capability, CapabilityCategory, Composable, CompositeAgent, CompositeAgentBuilder,
    CompositionError as CompositionTraitError, CompositionMetadata, CompositionType,
    ExecutionCondition as AgentExecutionCondition, ExecutionPattern, HierarchicalAgent,
    HierarchyEvent,
};

pub use capabilities::{
    CapabilityAggregator, CapabilityEntry, CapabilityMatch, CapabilityRequirement,
    CapabilityRequirementBuilder, CapabilityScorer, CapabilityStatistics, CapabilityUsageStats,
    DefaultCapabilityScorer,
};
pub use delegation::{
    DelegatingAgent, DelegatingAgentBuilder, DelegationConfig, DelegationRequest, DelegationResult,
    DelegationStrategy,
};
pub use hierarchical::{HierarchicalAgentBuilder, HierarchicalCompositeAgent, HierarchicalConfig};

pub use lifecycle::{
    CascadeDirection, ComponentHealth, ComponentLifecycle, CompositeLifecycleManager,
    ErrorSeverity, HealthCheckResult, HierarchicalLifecycleManager, LifecycleConfig,
    LifecycleEvent, LifecycleEventHandler, LifecycleState,
};
