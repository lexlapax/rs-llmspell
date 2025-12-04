// ABOUTME: Category definitions and test organization helpers
// ABOUTME: Provides predefined test categories and utility functions

//! Test category definitions and organization.
//!
//! This module provides predefined test categories that can be used
//! across the test suite for consistent categorization.

use llmspell_testing::attributes::*;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Predefined category for basic unit tests
pub fn unit_test_category() -> TestCategory {
    TestCategory::new(Speed::Fast, Scope::Unit)
        .with_priority(Priority::High)
        .with_stability(Stability::Stable)
}

/// Predefined category for integration tests
pub fn integration_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_priority(Priority::High)
        .with_stability(Stability::Stable)
}

/// Predefined category for end-to-end tests
pub fn e2e_test_category() -> TestCategory {
    TestCategory::new(Speed::VerySlow, Scope::E2E)
        .with_priority(Priority::Medium)
        .with_stability(Stability::Stable)
}

/// Category for agent-specific tests
pub fn agent_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_component(Component::Agents)
        .with_priority(Priority::High)
}

/// Category for tool tests
pub fn tool_test_category() -> TestCategory {
    TestCategory::new(Speed::Fast, Scope::Unit)
        .with_component(Component::Tools)
        .with_priority(Priority::High)
}

/// Category for workflow tests
pub fn workflow_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_component(Component::Workflows)
        .with_priority(Priority::High)
}

/// Category for Lua bridge tests
pub fn lua_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_component(Component::Bridge)
        .with_tag("lua")
}

/// Category for state persistence tests
pub fn state_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_component(Component::State)
        .with_priority(Priority::Critical)
}

/// Category for security tests
pub fn security_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_component(Component::Security)
        .with_priority(Priority::Critical)
        .with_tag("security")
}

/// Category for performance-sensitive tests
pub fn performance_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_tag("performance")
        .with_stability(Stability::Experimental)
}

/// Category for tests requiring network access
pub fn network_test_category() -> TestCategory {
    TestCategory::new(Speed::Slow, Scope::Integration)
        .with_dependency(Dependency::Network)
        .with_stability(Stability::Flaky)
}

/// Category for tests requiring LLM providers
pub fn llm_test_category(provider: &str) -> TestCategory {
    TestCategory::new(Speed::VerySlow, Scope::Integration)
        .with_dependency(Dependency::Network)
        .with_dependency(Dependency::LLM(provider.to_string()))
        .with_stability(Stability::Flaky)
}

/// Registry of all test categories
static CATEGORY_REGISTRY: LazyLock<HashMap<&'static str, TestCategory>> = LazyLock::new(|| {
    let mut registry = HashMap::new();

    // Basic categories
    registry.insert("unit", unit_test_category());
    registry.insert("integration", integration_test_category());
    registry.insert("e2e", e2e_test_category());

    // Component categories
    registry.insert("agent", agent_test_category());
    registry.insert("tool", tool_test_category());
    registry.insert("workflow", workflow_test_category());
    registry.insert("lua", lua_test_category());
    registry.insert("state", state_test_category());
    registry.insert("security", security_test_category());

    // Special categories
    registry.insert("performance", performance_test_category());
    registry.insert("network", network_test_category());

    registry
});

/// Get a category by name
pub fn get_category(name: &str) -> Option<&TestCategory> {
    CATEGORY_REGISTRY.get(name)
}

/// Create a filter for CI environments
pub fn ci_filter() -> TestFilter {
    TestFilter::stable_only().with_min_priority(Priority::Medium)
}

/// Create a filter for local development
pub fn dev_filter() -> TestFilter {
    TestFilter::fast_only().with_scope(Scope::Unit)
}

/// Create a filter for pre-release testing
pub fn prerelease_filter() -> TestFilter {
    TestFilter::default().with_min_priority(Priority::High)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_registry() {
        assert!(get_category("unit").is_some());
        assert!(get_category("integration").is_some());
        assert!(get_category("unknown").is_none());
    }

    #[test]
    fn test_ci_filter() {
        let filter = ci_filter();

        // Flaky test should not match
        let flaky = TestCategory::new(Speed::Fast, Scope::Unit).with_stability(Stability::Flaky);
        assert!(!flaky.matches_filter(&filter));

        // Low priority test should not match
        let low_priority = TestCategory::new(Speed::Fast, Scope::Unit).with_priority(Priority::Low);
        assert!(!low_priority.matches_filter(&filter));
    }
}
