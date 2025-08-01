// ABOUTME: Custom test attributes and categorization system for llmspell tests
// ABOUTME: Provides macros and traits for fine-grained test selection and organization

//! Test categorization attributes and utilities.
//!
//! This module provides a comprehensive categorization system for tests,
//! allowing fine-grained selection and organization of test suites.
//!
//! # Categories
//!
//! Tests can be categorized by multiple dimensions:
//! - **Speed**: fast, slow, very_slow
//! - **Scope**: unit, integration, e2e
//! - **Dependencies**: requires_network, requires_llm, requires_database
//! - **Component**: core, agents, tools, workflows, bridge, state
//! - **Priority**: critical, high, medium, low
//! - **Stability**: stable, flaky, experimental
//!
//! # Examples
//!
//! ```rust,no_run
//! # use llmspell_testing::attributes::{Speed, Scope, Component};
//! # fn test_basic_functionality() {
//!     // Fast unit test for core functionality
//! # }
//!
//! # fn test_llm_integration() {
//!     // Slower integration test requiring network and LLM
//! # }
//! ```
//!
//! With test attributes (when macro support is ready):
//! ```text
//! #[test]
//! #[test_category(Speed::Fast, Scope::Unit, Component::Core)]
//! fn test_basic_functionality() {
//!     // Fast unit test for core functionality
//! }
//!
//! #[test]
//! #[test_category(Speed::Slow, Scope::Integration)]
//! #[requires_network]
//! #[requires_llm("openai")]
//! fn test_llm_integration() {
//!     // Slower integration test requiring network and LLM
//! }
//! ```

use std::collections::HashSet;
use std::fmt;

/// Test execution speed categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Speed {
    /// Tests that complete in <100ms
    Fast,
    /// Tests that complete in <5s
    Slow,
    /// Tests that take >5s
    VerySlow,
}

/// Test scope categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Isolated unit tests
    Unit,
    /// Cross-component integration tests
    Integration,
    /// End-to-end scenario tests
    E2E,
}

/// Component categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    /// Core traits and types
    Core,
    /// Agent functionality
    Agents,
    /// Tool implementations
    Tools,
    /// Workflow patterns
    Workflows,
    /// Script bridge (Lua/JS)
    Bridge,
    /// State persistence
    State,
    /// Event system
    Events,
    /// Hook system
    Hooks,
    /// Security features
    Security,
    /// Utilities
    Utils,
}

/// Test priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    /// Must pass for release
    Critical,
    /// Should pass for release
    High,
    /// Good to have passing
    Medium,
    /// Nice to have passing
    Low,
}

/// Test stability categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stability {
    /// Reliable tests
    Stable,
    /// Occasionally failing tests
    Flaky,
    /// New or experimental tests
    Experimental,
}

/// External dependencies required by tests
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    /// Requires network access
    Network,
    /// Requires specific LLM provider
    LLM(String),
    /// Requires database
    Database,
    /// Requires file system access
    FileSystem,
    /// Requires specific environment variable
    EnvVar(String),
    /// Requires external service
    Service(String),
}

/// Complete test categorization
#[derive(Debug, Clone)]
pub struct TestCategory {
    pub speed: Speed,
    pub scope: Scope,
    pub components: HashSet<Component>,
    pub priority: Priority,
    pub stability: Stability,
    pub dependencies: HashSet<Dependency>,
    pub tags: HashSet<String>,
}

impl Default for TestCategory {
    fn default() -> Self {
        Self {
            speed: Speed::Fast,
            scope: Scope::Unit,
            components: HashSet::new(),
            priority: Priority::Medium,
            stability: Stability::Stable,
            dependencies: HashSet::new(),
            tags: HashSet::new(),
        }
    }
}

impl TestCategory {
    /// Create a new test category
    pub fn new(speed: Speed, scope: Scope) -> Self {
        Self {
            speed,
            scope,
            ..Default::default()
        }
    }

    /// Add a component to the category
    pub fn with_component(mut self, component: Component) -> Self {
        self.components.insert(component);
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the stability
    pub fn with_stability(mut self, stability: Stability) -> Self {
        self.stability = stability;
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency: Dependency) -> Self {
        self.dependencies.insert(dependency);
        self
    }

    /// Add a custom tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Check if this test should run given the filter criteria
    pub fn matches_filter(&self, filter: &TestFilter) -> bool {
        // Check speed
        if let Some(max_speed) = &filter.max_speed {
            match (self.speed, max_speed) {
                (Speed::VerySlow, Speed::Fast) => return false,
                (Speed::VerySlow, Speed::Slow) => return false,
                (Speed::Slow, Speed::Fast) => return false,
                _ => {}
            }
        }

        // Check scope
        if !filter.scopes.is_empty() && !filter.scopes.contains(&self.scope) {
            return false;
        }

        // Check components
        if !filter.components.is_empty() {
            let has_component = self
                .components
                .iter()
                .any(|c| filter.components.contains(c));
            if !has_component {
                return false;
            }
        }

        // Check priority
        if let Some(min_priority) = &filter.min_priority {
            match (self.priority, min_priority) {
                (Priority::Low, Priority::Medium) => return false,
                (Priority::Low, Priority::High) => return false,
                (Priority::Low, Priority::Critical) => return false,
                (Priority::Medium, Priority::High) => return false,
                (Priority::Medium, Priority::Critical) => return false,
                (Priority::High, Priority::Critical) => return false,
                _ => {}
            }
        }

        // Check stability
        if filter.exclude_flaky && self.stability == Stability::Flaky {
            return false;
        }

        if filter.exclude_experimental && self.stability == Stability::Experimental {
            return false;
        }

        // Check dependencies
        if filter.no_network && self.dependencies.contains(&Dependency::Network) {
            return false;
        }

        if filter.no_external_services {
            for dep in &self.dependencies {
                match dep {
                    Dependency::LLM(_) | Dependency::Database | Dependency::Service(_) => {
                        return false
                    }
                    _ => {}
                }
            }
        }

        // Check tags
        if !filter.tags.is_empty() {
            let has_tag = self.tags.iter().any(|t| filter.tags.contains(t));
            if !has_tag {
                return false;
            }
        }

        true
    }
}

/// Filter criteria for test selection
#[derive(Debug, Clone, Default)]
pub struct TestFilter {
    pub max_speed: Option<Speed>,
    pub scopes: HashSet<Scope>,
    pub components: HashSet<Component>,
    pub min_priority: Option<Priority>,
    pub exclude_flaky: bool,
    pub exclude_experimental: bool,
    pub no_network: bool,
    pub no_external_services: bool,
    pub tags: HashSet<String>,
}

impl TestFilter {
    /// Create a filter for fast tests only
    pub fn fast_only() -> Self {
        Self {
            max_speed: Some(Speed::Fast),
            ..Default::default()
        }
    }

    /// Create a filter for stable tests only
    pub fn stable_only() -> Self {
        Self {
            exclude_flaky: true,
            exclude_experimental: true,
            ..Default::default()
        }
    }

    /// Create a filter for offline tests only
    pub fn offline_only() -> Self {
        Self {
            no_network: true,
            no_external_services: true,
            ..Default::default()
        }
    }

    /// Add a scope filter
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scopes.insert(scope);
        self
    }

    /// Add a component filter
    pub fn with_component(mut self, component: Component) -> Self {
        self.components.insert(component);
        self
    }

    /// Set minimum priority
    pub fn with_min_priority(mut self, priority: Priority) -> Self {
        self.min_priority = Some(priority);
        self
    }

    /// Add a tag filter
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }
}

/// Trait for types that can be categorized
pub trait Categorizable {
    /// Get the test category
    fn category(&self) -> &TestCategory;

    /// Check if this item matches the filter
    fn matches_filter(&self, filter: &TestFilter) -> bool {
        self.category().matches_filter(filter)
    }
}

// Display implementations
impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Speed::Fast => write!(f, "fast"),
            Speed::Slow => write!(f, "slow"),
            Speed::VerySlow => write!(f, "very_slow"),
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Unit => write!(f, "unit"),
            Scope::Integration => write!(f, "integration"),
            Scope::E2E => write!(f, "e2e"),
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Component::Core => write!(f, "core"),
            Component::Agents => write!(f, "agents"),
            Component::Tools => write!(f, "tools"),
            Component::Workflows => write!(f, "workflows"),
            Component::Bridge => write!(f, "bridge"),
            Component::State => write!(f, "state"),
            Component::Events => write!(f, "events"),
            Component::Hooks => write!(f, "hooks"),
            Component::Security => write!(f, "security"),
            Component::Utils => write!(f, "utils"),
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "testing")]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let category = TestCategory::new(Speed::Fast, Scope::Unit)
            .with_component(Component::Core)
            .with_priority(Priority::High)
            .with_tag("regression");

        assert_eq!(category.speed, Speed::Fast);
        assert_eq!(category.scope, Scope::Unit);
        assert!(category.components.contains(&Component::Core));
        assert_eq!(category.priority, Priority::High);
        assert!(category.tags.contains("regression"));
    }

    #[test]
    fn test_filter_matching() {
        let category = TestCategory::new(Speed::Slow, Scope::Integration)
            .with_component(Component::Agents)
            .with_dependency(Dependency::Network);

        // Should match empty filter
        assert!(category.matches_filter(&TestFilter::default()));

        // Should not match fast-only filter
        assert!(!category.matches_filter(&TestFilter::fast_only()));

        // Should not match offline-only filter
        assert!(!category.matches_filter(&TestFilter::offline_only()));

        // Should match integration scope filter
        let filter = TestFilter::default().with_scope(Scope::Integration);
        assert!(category.matches_filter(&filter));
    }

    #[test]
    fn test_priority_filtering() {
        let high_priority =
            TestCategory::new(Speed::Fast, Scope::Unit).with_priority(Priority::High);

        let low_priority = TestCategory::new(Speed::Fast, Scope::Unit).with_priority(Priority::Low);

        let filter = TestFilter::default().with_min_priority(Priority::Medium);

        assert!(high_priority.matches_filter(&filter));
        assert!(!low_priority.matches_filter(&filter));
    }
}
