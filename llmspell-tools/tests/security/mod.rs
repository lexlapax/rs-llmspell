//! ABOUTME: Comprehensive security testing framework for LLMSpell tools
//! ABOUTME: Provides structured security testing with categorized test suites

pub mod auth_tests;
pub mod data_exposure_tests;
pub mod input_validation_tests;
pub mod path_security_tests;
pub mod rate_limit_tests;
pub mod test_framework;

pub use test_framework::*;

pub use auth_tests::all_auth_tests;
pub use data_exposure_tests::all_data_exposure_tests;
pub use input_validation_tests::all_input_validation_tests;
/// Import all security test suites
pub use path_security_tests::all_path_security_tests;
pub use rate_limit_tests::all_rate_limit_tests;

/// Collect all security tests into a single comprehensive suite
pub fn all_security_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(all_path_security_tests());
    tests.extend(all_input_validation_tests());
    tests.extend(all_auth_tests());
    tests.extend(all_rate_limit_tests());
    tests.extend(all_data_exposure_tests());
    tests
}

/// Get security tests by category
pub fn get_tests_by_category(category: TestCategory) -> Vec<SecurityTestCase> {
    all_security_tests()
        .into_iter()
        .filter(|test| test.categories.contains(&category))
        .collect()
}

/// Get security tests by severity
pub fn get_tests_by_severity(severity: Severity) -> Vec<SecurityTestCase> {
    all_security_tests()
        .into_iter()
        .filter(|test| test.severity == severity)
        .collect()
}

/// Get critical security tests only
pub fn get_critical_tests() -> Vec<SecurityTestCase> {
    get_tests_by_severity(Severity::Critical)
}

/// Security test statistics
pub fn get_test_statistics() -> SecurityTestStats {
    let tests = all_security_tests();
    let total = tests.len();

    let mut by_severity = std::collections::HashMap::new();
    let mut by_category = std::collections::HashMap::new();

    for test in &tests {
        *by_severity.entry(test.severity).or_insert(0) += 1;
        for category in &test.categories {
            *by_category.entry(*category).or_insert(0) += 1;
        }
    }

    SecurityTestStats {
        total_tests: total,
        by_severity,
        by_category,
    }
}

/// Security test statistics structure
#[derive(Debug, Clone)]
pub struct SecurityTestStats {
    pub total_tests: usize,
    pub by_severity: std::collections::HashMap<Severity, usize>,
    pub by_category: std::collections::HashMap<TestCategory, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_security_tests() {
        let tests = all_security_tests();
        assert!(!tests.is_empty());
        assert!(tests.len() > 100); // Should have comprehensive coverage
    }

    #[test]
    fn test_category_filtering() {
        let injection_tests = get_tests_by_category(TestCategory::Injection);
        let dos_tests = get_tests_by_category(TestCategory::DoS);
        let auth_tests = get_tests_by_category(TestCategory::Authentication);

        assert!(!injection_tests.is_empty());
        assert!(!dos_tests.is_empty());
        assert!(!auth_tests.is_empty());
    }

    #[test]
    fn test_severity_filtering() {
        let critical_tests = get_tests_by_severity(Severity::Critical);
        let high_tests = get_tests_by_severity(Severity::High);
        let medium_tests = get_tests_by_severity(Severity::Medium);
        let low_tests = get_tests_by_severity(Severity::Low);

        assert!(!critical_tests.is_empty());
        assert!(!high_tests.is_empty());
        assert!(!medium_tests.is_empty());
        assert!(!low_tests.is_empty());
    }

    #[test]
    fn test_critical_tests() {
        let critical_tests = get_critical_tests();
        assert!(!critical_tests.is_empty());

        // All should be critical severity
        for test in &critical_tests {
            assert_eq!(test.severity, Severity::Critical);
        }
    }

    #[test]
    fn test_statistics() {
        let stats = get_test_statistics();
        assert!(stats.total_tests > 0);
        assert!(!stats.by_severity.is_empty());
        assert!(!stats.by_category.is_empty());

        // Verify severity counts add up
        let severity_total: usize = stats.by_severity.values().sum();
        assert_eq!(severity_total, stats.total_tests);
    }

    #[test]
    fn test_unique_test_names() {
        let tests = all_security_tests();
        let mut names = std::collections::HashSet::new();

        for test in &tests {
            assert!(
                names.insert(&test.name),
                "Duplicate test name: {}",
                test.name
            );
        }
    }
}
