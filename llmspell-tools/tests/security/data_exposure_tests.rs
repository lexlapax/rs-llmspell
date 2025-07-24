//! ABOUTME: Data exposure and information disclosure security tests
//! ABOUTME: Tests for sensitive data leaks, verbose errors, and information enumeration

use super::test_framework::*;
use serde_json::json;

/// Sensitive file exposure tests
pub fn sensitive_file_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "EXPOSE_BACKUP_FILES".to_string(),
            description: "Backup file exposure".to_string(),
            payload: json!({
                "paths": [
                    "config.php.bak",
                    "database.sql.backup",
                    "app.js~",
                    ".env.backup",
                    "secrets.yml.old"
                ],
                "operation": "access_backup_files"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "EXPOSE_CONFIG_FILES".to_string(),
            description: "Configuration file exposure".to_string(),
            payload: json!({
                "paths": [
                    ".env",
                    "config.yml",
                    "database.conf",
                    "app.config",
                    "settings.ini"
                ],
                "operation": "access_config_files"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "EXPOSE_LOG_FILES".to_string(),
            description: "Log file exposure".to_string(),
            payload: json!({
                "paths": [
                    "error.log",
                    "access.log",
                    "debug.log",
                    "app.log",
                    "system.log"
                ],
                "operation": "access_log_files"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "EXPOSE_SOURCE_CODE".to_string(),
            description: "Source code exposure".to_string(),
            payload: json!({
                "paths": [
                    "app.py",
                    "config.php",
                    "database.js",
                    "auth.rb",
                    "secrets.go"
                ],
                "operation": "access_source_files"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// Database information disclosure tests
pub fn database_disclosure_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "DB_ERROR_DISCLOSURE".to_string(),
            description: "Database error information disclosure".to_string(),
            payload: json!({
                "query": "SELECT * FROM users WHERE id = 'invalid'syntax'",
                "operation": "trigger_db_error"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "DB_SCHEMA_ENUMERATION".to_string(),
            description: "Database schema enumeration".to_string(),
            payload: json!({
                "queries": [
                    "SELECT table_name FROM information_schema.tables",
                    "SHOW TABLES",
                    "SELECT name FROM sqlite_master",
                    "\\dt"
                ],
                "operation": "enumerate_schema"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "DB_VERSION_DISCLOSURE".to_string(),
            description: "Database version disclosure".to_string(),
            payload: json!({
                "queries": [
                    "SELECT VERSION()",
                    "SELECT @@version",
                    "SELECT sqlite_version()"
                ],
                "operation": "get_db_version"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// Application error disclosure tests
pub fn error_disclosure_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "ERROR_STACK_TRACE".to_string(),
            description: "Stack trace information disclosure".to_string(),
            payload: json!({
                "input": "malformed_data_to_trigger_exception",
                "operation": "trigger_exception"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "ERROR_DEBUG_INFO".to_string(),
            description: "Debug information disclosure".to_string(),
            payload: json!({
                "debug_mode": true,
                "operation": "access_debug_info"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "ERROR_VERBOSE_MESSAGES".to_string(),
            description: "Verbose error messages".to_string(),
            payload: json!({
                "invalid_requests": [
                    {"field": "nonexistent"},
                    {"malformed": "json}"},
                    {"sql": "INVALID SYNTAX"}
                ],
                "operation": "trigger_verbose_errors"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// User enumeration tests
pub fn user_enumeration_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "ENUM_USERNAME_LOGIN".to_string(),
            description: "Username enumeration via login".to_string(),
            payload: json!({
                "usernames": ["admin", "user", "test", "nonexistent"],
                "password": "wrong_password",
                "operation": "enumerate_users_login"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "ENUM_USERNAME_REGISTER".to_string(),
            description: "Username enumeration via registration".to_string(),
            payload: json!({
                "usernames": ["admin", "user", "test", "available_username"],
                "operation": "enumerate_users_register"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "ENUM_EMAIL_RESET".to_string(),
            description: "Email enumeration via password reset".to_string(),
            payload: json!({
                "emails": [
                    "admin@company.com",
                    "user@company.com",
                    "nonexistent@company.com"
                ],
                "operation": "enumerate_emails_reset"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// Directory listing and file enumeration tests
pub fn directory_listing_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "DIR_LISTING_ENABLED".to_string(),
            description: "Directory listing enabled".to_string(),
            payload: json!({
                "paths": [
                    "/uploads/",
                    "/files/",
                    "/assets/",
                    "/backup/",
                    "/tmp/"
                ],
                "operation": "check_directory_listing"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "DIR_FILE_ENUMERATION".to_string(),
            description: "File enumeration attack".to_string(),
            payload: json!({
                "common_files": [
                    "index.html",
                    "robots.txt",
                    "sitemap.xml",
                    "favicon.ico",
                    "admin.php"
                ],
                "operation": "enumerate_files"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// API information disclosure tests
pub fn api_disclosure_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "API_ENDPOINT_DISCOVERY".to_string(),
            description: "API endpoint discovery".to_string(),
            payload: json!({
                "endpoints": [
                    "/api/users",
                    "/api/admin",
                    "/api/internal",
                    "/api/debug",
                    "/api/v2/secret"
                ],
                "operation": "discover_endpoints"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "API_SWAGGER_EXPOSURE".to_string(),
            description: "Swagger/OpenAPI documentation exposure".to_string(),
            payload: json!({
                "paths": [
                    "/swagger.json",
                    "/api-docs",
                    "/openapi.json",
                    "/docs",
                    "/swagger-ui.html"
                ],
                "operation": "access_api_docs"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "API_VERSION_DISCLOSURE".to_string(),
            description: "API version information disclosure".to_string(),
            payload: json!({
                "headers": ["Server", "X-Powered-By", "X-Version", "API-Version"],
                "operation": "extract_version_info"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// Metadata and comment disclosure tests
pub fn metadata_disclosure_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "META_HTML_COMMENTS".to_string(),
            description: "HTML comment information disclosure".to_string(),
            payload: json!({
                "html": "<!-- TODO: Remove debug info before production --><!-- Database password: secret123 -->",
                "operation": "extract_html_comments"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "META_JS_COMMENTS".to_string(),
            description: "JavaScript comment disclosure".to_string(),
            payload: json!({
                "js": "// API key: abc123\n/* Production server: prod.example.com */",
                "operation": "extract_js_comments"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "META_HTTP_HEADERS".to_string(),
            description: "HTTP header information disclosure".to_string(),
            payload: json!({
                "sensitive_headers": [
                    "X-Debug-Token",
                    "X-Internal-User",
                    "X-Database-Query",
                    "X-Admin-Panel"
                ],
                "operation": "check_sensitive_headers"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// Social engineering data exposure tests
pub fn social_engineering_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SOCIAL_USER_PROFILES".to_string(),
            description: "User profile information exposure".to_string(),
            payload: json!({
                "user_ids": ["1", "2", "admin", "root"],
                "fields": ["email", "phone", "address", "ssn"],
                "operation": "extract_user_profiles"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::InformationDisclosure],
        },
        SecurityTestCase {
            name: "SOCIAL_EMPLOYEE_DATA".to_string(),
            description: "Employee data exposure".to_string(),
            payload: json!({
                "endpoints": ["/api/employees", "/staff", "/directory"],
                "operation": "extract_employee_data"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::InformationDisclosure],
        },
    ]
}

/// Create all data exposure test cases
pub fn all_data_exposure_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(sensitive_file_tests());
    tests.extend(database_disclosure_tests());
    tests.extend(error_disclosure_tests());
    tests.extend(user_enumeration_tests());
    tests.extend(directory_listing_tests());
    tests.extend(api_disclosure_tests());
    tests.extend(metadata_disclosure_tests());
    tests.extend(social_engineering_tests());
    tests
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_exposure_test_creation() {
        let tests = all_data_exposure_tests();
        assert!(!tests.is_empty());

        // Verify we have tests from each category
        let categories: Vec<String> = tests
            .iter()
            .map(|t| t.name.split('_').next().unwrap_or(""))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        assert!(categories.contains(&"EXPOSE".to_string()));
        assert!(categories.contains(&"DB".to_string()));
        assert!(categories.contains(&"ERROR".to_string()));
        assert!(categories.contains(&"ENUM".to_string()));
        assert!(categories.contains(&"DIR".to_string()));
        assert!(categories.contains(&"API".to_string()));
        assert!(categories.contains(&"META".to_string()));
        assert!(categories.contains(&"SOCIAL".to_string()));
    }

    #[test]
    fn test_critical_exposure_tests() {
        let tests = all_data_exposure_tests();
        let critical_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.severity == Severity::Critical)
            .collect();

        // Should have critical tests for sensitive exposures
        assert!(!critical_tests.is_empty());

        // Config file exposure should be critical
        assert!(critical_tests
            .iter()
            .any(|t| t.name == "EXPOSE_CONFIG_FILES"));
    }

    #[test]
    fn test_information_disclosure_coverage() {
        let tests = all_data_exposure_tests();
        let disclosure_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.categories.contains(&TestCategory::InformationDisclosure))
            .collect();

        // All data exposure tests should be information disclosure
        assert_eq!(disclosure_tests.len(), tests.len());
    }
}
