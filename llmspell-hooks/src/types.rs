// ABOUTME: Core types for the hook system including HookPoint enum and Language enum
// ABOUTME: Defines all hook points and language identifiers for cross-language support

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// All possible hook points in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookPoint {
    // Agent lifecycle hooks (7 states)
    BeforeAgentInit,
    AfterAgentInit,
    BeforeAgentExecution,
    AfterAgentExecution,
    AgentError,
    BeforeAgentShutdown,
    AfterAgentShutdown,

    // Tool execution hooks (6 states)
    BeforeToolDiscovery,
    AfterToolDiscovery,
    BeforeToolExecution,
    AfterToolExecution,
    ToolValidation,
    ToolError,

    // Workflow hooks (8 states)
    BeforeWorkflowStart,
    WorkflowStageTransition,
    BeforeWorkflowStage,
    AfterWorkflowStage,
    WorkflowCheckpoint,
    WorkflowRollback,
    AfterWorkflowComplete,
    WorkflowError,

    // State management hooks (6 states)
    BeforeStateRead,
    AfterStateRead,
    BeforeStateWrite,
    AfterStateWrite,
    StateConflict,
    StateMigration,

    // System hooks (5 states)
    SystemStartup,
    SystemShutdown,
    ConfigurationChange,
    ResourceLimitExceeded,
    SecurityViolation,

    // Session hooks (5 states)
    SessionStart,
    SessionEnd,
    SessionCheckpoint,
    SessionRestore,
    SessionSave,

    // Event hooks (3 states)
    BeforeEventEmit,
    AfterEventEmit,
    EventHandlerError,

    // Performance hooks (3 states)
    PerformanceThresholdExceeded,
    MemoryUsageHigh,
    CpuUsageHigh,

    // Custom hooks
    Custom(String),
}

impl fmt::Display for HookPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookPoint::Custom(name) => write!(f, "custom:{}", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Languages supported by the hook system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Lua,
    JavaScript,
    Python,
    Native,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Lua => write!(f, "lua"),
            Language::JavaScript => write!(f, "javascript"),
            Language::Python => write!(f, "python"),
            Language::Native => write!(f, "native"),
        }
    }
}

/// Component identifier for hook context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
    pub id: Uuid,
    pub component_type: ComponentType,
    pub name: String,
}

impl ComponentId {
    pub fn new(component_type: ComponentType, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_type,
            name,
        }
    }
}

/// Types of components that can trigger hooks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Agent,
    Tool,
    Workflow,
    System,
    Custom(String),
}

/// Hook priority for execution ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Priority(pub i32);

impl Priority {
    pub const HIGHEST: Priority = Priority(i32::MIN);
    pub const HIGH: Priority = Priority(-100);
    pub const NORMAL: Priority = Priority(0);
    pub const LOW: Priority = Priority(100);
    pub const LOWEST: Priority = Priority(i32::MAX);
}

impl Default for Priority {
    fn default() -> Self {
        Priority::NORMAL
    }
}

/// Hook metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    pub name: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub language: Language,
    pub tags: Vec<String>,
    pub version: String,
}

impl Default for HookMetadata {
    fn default() -> Self {
        Self {
            name: String::from("unnamed"),
            description: None,
            priority: Priority::default(),
            language: Language::Native,
            tags: Vec::new(),
            version: String::from("1.0.0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_point_display() {
        assert_eq!(HookPoint::BeforeAgentInit.to_string(), "BeforeAgentInit");
        assert_eq!(
            HookPoint::Custom("test".to_string()).to_string(),
            "custom:test"
        );
    }

    #[test]
    fn test_language_display() {
        assert_eq!(Language::Lua.to_string(), "lua");
        assert_eq!(Language::JavaScript.to_string(), "javascript");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::HIGHEST < Priority::HIGH);
        assert!(Priority::HIGH < Priority::NORMAL);
        assert!(Priority::NORMAL < Priority::LOW);
        assert!(Priority::LOW < Priority::LOWEST);
    }

    #[test]
    fn test_component_id_creation() {
        let id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        assert_eq!(id.component_type, ComponentType::Agent);
        assert_eq!(id.name, "test-agent");
    }
}
