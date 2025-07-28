// ABOUTME: Example showing how to use test categorization
// ABOUTME: Demonstrates various categorization patterns

//! Example of test categorization usage

// Attributes will be used when macro support is ready
#[allow(unused_imports)]
use llmspell_testing::attributes::*;

// Example 1: Simple unit test categorization
#[cfg(test)]
mod unit_tests {

    #[test]
    // TODO: #[test_category(unit)]
    fn test_fast_calculation() {
        // This is a fast unit test
        assert_eq!(2 + 2, 4);
    }

    #[test]
    // TODO: #[test_category(unit)]
    fn test_string_manipulation() {
        let result = "hello".to_uppercase();
        assert_eq!(result, "HELLO");
    }
}

// Example 2: Integration tests with dependencies
#[cfg(test)]
mod integration_tests {

    #[test]
    // TODO: #[test_category(integration)]
    // TODO: Add requires_network attribute
    fn test_api_connection() {
        // This test requires network access
        // It will be skipped if network-tests feature is not enabled
        println!("API connection test would run here");
    }

    #[test]
    // TODO: #[test_category(integration)]
    // TODO: Add requires_llm and slow_test attributes
    fn test_openai_integration() {
        // This test requires OpenAI API access and is slow
        // It will be skipped unless both llm-tests and slow-tests features are enabled
        println!("OpenAI integration test would run here");
    }

    #[test]
    // TODO: #[test_category(integration)]
    // TODO: Add flaky_test attribute
    fn test_unreliable_service() {
        // This test is marked as flaky
        // It will be skipped unless flaky-tests feature is enabled
        println!("Unreliable service test would run here");
    }
}

// Example 3: Using module for categorization
#[cfg(test)]
mod agent_module_tests {

    #[test]
    // TODO: Add category: agent, tags: [integration, slow]
    fn test_agent_creation() {
        // Test within categorized module
        assert!(true);
    }

    #[test]
    // TODO: Add requires_network attribute
    fn test_agent_api_call() {
        // Test with additional requirements
        assert!(true);
    }
}

// Example 4: Using the attributes API directly
#[cfg(test)]
mod advanced_categorization {
    use super::*;
    use llmspell_testing::attributes::{Component, Priority, Scope, Speed, TestCategory};

    fn create_test_category() -> TestCategory {
        TestCategory::new(Speed::Slow, Scope::Integration)
            .with_component(Component::Agents)
            .with_component(Component::Tools)
            .with_priority(Priority::High)
            .with_dependency(Dependency::Network)
            .with_dependency(Dependency::LLM("anthropic".to_string()))
            .with_tag("regression")
            .with_tag("nightly")
    }

    #[test]
    fn test_with_complex_category() {
        let category = create_test_category();

        // This test has multiple components, dependencies, and tags
        // The category can be used for filtering and reporting
        println!("Running test with category: {:?}", category);
        assert!(true);
    }
}

// Example 5: Using skip conditions
#[cfg(test)]
mod conditional_tests {

    #[test]
    fn test_skip_in_ci() {
        // TODO: Add skip_if!(ci) when macro support is ready
        // This test will be skipped when running in CI
        println!("This won't run in CI");
    }

    #[test]
    // TODO: Add requires_network attribute
    fn test_skip_without_network() {
        // TODO: Add skip_if!(no_network) when macro support is ready
        // This test will be skipped if no network is available
        println!("This requires network connectivity");
    }

    #[test]
    fn test_skip_without_env_var() {
        // TODO: Add skip_if!(env_not_set: "SPECIAL_API_KEY") when macro support is ready
        // This test will be skipped if SPECIAL_API_KEY is not set
        println!("This requires SPECIAL_API_KEY environment variable");
    }
}

// Example 6: Creating categories programmatically
#[cfg(test)]
mod programmatic_category_tests {
    use llmspell_testing::attributes::{TestCategory, Speed, Scope, Component};

    #[test]
    fn test_creating_category_programmatically() {
        // Create categories programmatically
        let category = TestCategory::new(Speed::Slow, Scope::Integration)
            .with_component(Component::Agents)
            .with_tag("custom-tag");

        println!("Using custom category: {:?}", category);
        assert!(true);
    }
}

fn main() {
    println!("Test Categorization Examples");
    println!("===========================");
    println!();
    println!("This example demonstrates various ways to categorize tests.");
    println!("Run with different feature flags to see different tests execute:");
    println!();
    println!("  cargo test --example categorization_example --features unit-tests");
    println!(
        "  cargo test --example categorization_example --features integration-tests,network-tests"
    );
    println!(
        "  cargo test --example categorization_example --features all-tests,slow-tests,flaky-tests"
    );
}
