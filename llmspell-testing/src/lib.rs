//! ABOUTME: Comprehensive test suite and testing utilities for rs-llmspell framework
//! ABOUTME: Provides test organization, runners, mocks, generators, benchmarks, and fixtures

//! Comprehensive test suite and utilities for the LLMSpell framework.
//!
//! This crate serves two purposes:
//! 1. **Test Suite**: Organizes and runs all tests across the llmspell workspace
//! 2. **Test Utilities**: Provides testing helpers for downstream users
//!
//! # Test Organization
//!
//! Tests are organized into categories:
//! - `unit` - Unit tests for individual components
//! - `integration` - Cross-crate integration tests
//! - `agents` - Agent-specific tests
//! - `scenarios` - End-to-end scenario tests
//! - `lua` - Lua scripting tests
//! - `performance` - Performance benchmarks
//!
//! # Running Tests
//!
//! Run specific test categories using features:
//! ```bash
//! cargo test -p llmspell-testing --features unit-tests
//! cargo test -p llmspell-testing --features integration-tests
//! cargo test -p llmspell-testing --features all-tests
//! ```
//!
//! # Test Utilities
//!
//! The crate also provides utilities for writing tests:
//! - Mock implementations of core traits
//! - Property-based test generators
//! - Benchmark helpers
//! - Test fixtures and data generators
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::mocks::MockBaseAgent;
//! use llmspell_testing::generators::component_id_strategy;
//! use proptest::prelude::*;
//! use llmspell_core::types::AgentOutput;
//!
//! // Use mock agent in tests
//! let mut mock = MockBaseAgent::new();
//! mock.expect_execute()
//!     .returning(|_, _| Ok(AgentOutput::text("test")));
//!
//! // Generate test data with proptest
//! proptest! {
//!     #[test]
//!     fn test_with_random_component_id(id in component_id_strategy()) {
//!         // Test with generated component ID
//!     }
//! }
//! ```

// Test utilities modules
pub mod agent_helpers;
pub mod attributes;
pub mod benchmarks;
pub mod bridge_helpers;
pub mod environment_helpers;
pub mod event_helpers;
pub mod fixtures;
pub mod generators;
pub mod hook_helpers;
pub mod macros;
pub mod mocks;
pub mod state_helpers;
pub mod tool_helpers;
pub mod workflow_helpers;

// Test runner support
#[cfg(feature = "test-runner")]
pub mod runner;

// Re-export commonly used test utilities when available
// TODO: Add load_fixture function in Task 5.7.5 (Test Fixtures and Data Management)
// #[cfg(feature = "test-utilities")]
// pub use fixtures::load_fixture;
#[cfg(feature = "test-utilities")]
pub use generators::component_id_strategy;

/// Test categories for selective execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    Unit,
    Integration,
    Agents,
    Scenarios,
    Lua,
    Performance,
    All,
}

impl TestCategory {
    /// Get the feature flag name for this category
    pub fn feature_name(&self) -> &'static str {
        match self {
            TestCategory::Unit => "unit-tests",
            TestCategory::Integration => "integration-tests",
            TestCategory::Agents => "agent-tests",
            TestCategory::Scenarios => "scenario-tests",
            TestCategory::Lua => "lua-tests",
            TestCategory::Performance => "performance-tests",
            TestCategory::All => "all-tests",
        }
    }

    /// Get a description of this test category
    pub fn description(&self) -> &'static str {
        match self {
            TestCategory::Unit => "Unit tests for individual components",
            TestCategory::Integration => "Cross-crate integration tests",
            TestCategory::Agents => "Agent-specific functionality tests",
            TestCategory::Scenarios => "End-to-end scenario tests",
            TestCategory::Lua => "Lua scripting tests",
            TestCategory::Performance => "Performance benchmarks",
            TestCategory::All => "All test categories",
        }
    }
}
