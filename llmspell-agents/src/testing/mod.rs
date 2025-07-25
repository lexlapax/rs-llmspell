//! ABOUTME: Testing infrastructure for agent system
//! ABOUTME: Provides comprehensive testing utilities, mocks, and frameworks for agent testing

pub mod framework;
pub mod mocks;
pub mod scenarios;
pub mod utils;

// Re-export commonly used types
pub use framework::{
    AgentAssertions, LifecycleEventRecorder, TestConfig, TestHarness, TestMetrics, TestResult,
    TestScenarioBuilder,
};
pub use mocks::{
    MockAgent, MockAgentBuilder, MockAgentConfig, MockResponse, MockTool, TestDoubles,
};

/// Prelude for convenient imports in tests
pub mod prelude {
    // Re-export without conflicts
    pub use super::framework::{
        AgentAssertions, LifecycleEventRecorder, TestConfig, TestHarness, TestMetrics,
        TestScenarioBuilder,
    };
    pub use super::mocks::{
        MockAgent, MockAgentBuilder, MockAgentConfig, MockResponse, MockTool, TestDoubles,
    };
    pub use super::scenarios::*;
    pub use super::utils::*;
}
