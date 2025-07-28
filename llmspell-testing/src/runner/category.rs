// ABOUTME: Test category definitions and metadata
// ABOUTME: Provides information about available test categories

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    Unit,
    Integration,
    Agent,
    Scenario,
    Lua,
    Performance,
    All,
}

impl FromStr for TestCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unit" => Ok(TestCategory::Unit),
            "integration" => Ok(TestCategory::Integration),
            "agent" => Ok(TestCategory::Agent),
            "scenario" => Ok(TestCategory::Scenario),
            "lua" => Ok(TestCategory::Lua),
            "performance" => Ok(TestCategory::Performance),
            "all" => Ok(TestCategory::All),
            _ => Err(format!("Unknown test category: {}", s)),
        }
    }
}

impl TestCategory {
    pub fn name(&self) -> &'static str {
        match self {
            TestCategory::Unit => "unit",
            TestCategory::Integration => "integration",
            TestCategory::Agent => "agent",
            TestCategory::Scenario => "scenario",
            TestCategory::Lua => "lua",
            TestCategory::Performance => "performance",
            TestCategory::All => "all",
        }
    }

    pub fn feature_name(&self) -> &'static str {
        match self {
            TestCategory::Unit => "unit-tests",
            TestCategory::Integration => "integration-tests",
            TestCategory::Agent => "agent-tests",
            TestCategory::Scenario => "scenario-tests",
            TestCategory::Lua => "lua-tests",
            TestCategory::Performance => "performance-tests",
            TestCategory::All => "all-tests",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TestCategory::Unit => "Unit tests for individual components",
            TestCategory::Integration => "Integration tests across components",
            TestCategory::Agent => "Agent-specific functionality tests",
            TestCategory::Scenario => "End-to-end scenario tests",
            TestCategory::Lua => "Lua scripting bridge tests",
            TestCategory::Performance => "Performance benchmarks",
            TestCategory::All => "All test categories",
        }
    }

    pub fn test_files(&self) -> Vec<&'static str> {
        match self {
            TestCategory::Unit => vec![
                "tests/unit/core_tests.rs",
                "tests/unit/agents_tests.rs",
                "tests/unit/tools_tests.rs",
                "tests/unit/workflows_tests.rs",
                "tests/unit/bridge_tests.rs",
                "tests/unit/events_tests.rs",
                "tests/unit/hooks_tests.rs",
                "tests/unit/state_tests.rs",
                "tests/unit/storage_tests.rs",
                "tests/unit/utils_tests.rs",
            ],
            TestCategory::Integration => vec![
                "tests/integration/backup_recovery.rs",
                "tests/integration/component_state_integration.rs",
                "tests/integration/state_migration.rs",
            ],
            TestCategory::Agent => vec!["tests/agents/isolation_tests.rs"],
            TestCategory::Scenario => vec!["tests/scenarios/disaster_recovery.rs"],
            TestCategory::Lua => vec!["tests/lua/mod.rs", "tests/integration/*.lua"],
            TestCategory::Performance => vec!["benches/*.rs"],
            TestCategory::All => vec!["all test files"],
        }
    }

    pub fn all_categories() -> Vec<TestCategory> {
        vec![
            TestCategory::Unit,
            TestCategory::Integration,
            TestCategory::Agent,
            TestCategory::Scenario,
            TestCategory::Lua,
            TestCategory::Performance,
        ]
    }
}

impl fmt::Display for TestCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
