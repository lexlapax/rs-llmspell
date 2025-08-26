//! ABOUTME: Rate limiting and `DoS` protection security tests
//! ABOUTME: Tests for rate limit bypass, resource exhaustion, and denial of service attacks

use super::test_framework::*;
use serde_json::json;

/// Basic rate limit bypass tests
#[must_use]
pub fn rate_limit_bypass_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "RATE_HEADER_SPOOF".to_string(),
            description: "Rate limit bypass via header spoofing".to_string(),
            payload: json!({
                "requests": 1000,
                "headers": {
                    "X-Forwarded-For": "127.0.0.1",
                    "X-Real-IP": "10.0.0.1",
                    "X-Originating-IP": "192.168.1.1"
                },
                "operation": "flood_requests"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "RATE_USER_AGENT_ROTATION".to_string(),
            description: "Rate limit bypass via User-Agent rotation".to_string(),
            payload: json!({
                "requests": 500,
                "user_agents": [
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
                    "Mozilla/5.0 (X11; Linux x86_64)"
                ],
                "operation": "rotate_user_agents"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "RATE_DISTRIBUTED_ATTACK".to_string(),
            description: "Distributed rate limit bypass".to_string(),
            payload: json!({
                "source_ips": ["1.1.1.1", "2.2.2.2", "3.3.3.3", "4.4.4.4"],
                "requests_per_ip": 100,
                "operation": "distributed_flood"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "RATE_SESSION_ROTATION".to_string(),
            description: "Rate limit bypass via session rotation".to_string(),
            payload: json!({
                "requests": 200,
                "session_rotation": true,
                "operation": "session_flood"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Resource exhaustion tests
#[must_use]
pub fn resource_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "RESOURCE_MEMORY_BOMB".to_string(),
            description: "Memory exhaustion attack".to_string(),
            payload: json!({
                "data_size": 100_000_000,
                "operation": "allocate_memory",
                "payload": "A".repeat(10000)
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "RESOURCE_CPU_INTENSIVE".to_string(),
            description: "CPU exhaustion attack".to_string(),
            payload: json!({
                "iterations": 1_000_000,
                "operation": "cpu_intensive_task",
                "complexity": "exponential"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "RESOURCE_DISK_FILL".to_string(),
            description: "Disk space exhaustion".to_string(),
            payload: json!({
                "file_size": 10_000_000_000i64,
                "operation": "create_large_file",
                "path": "/tmp/large_file.dat"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "RESOURCE_CONNECTION_POOL".to_string(),
            description: "Connection pool exhaustion".to_string(),
            payload: json!({
                "connections": 10000,
                "operation": "exhaust_connections",
                "hold_time": 3600
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Slowloris and slow attack tests
#[must_use]
pub fn slow_attack_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SLOW_LORIS_HTTP".to_string(),
            description: "Slowloris HTTP attack".to_string(),
            payload: json!({
                "connections": 1000,
                "send_interval": 10,
                "partial_headers": true,
                "operation": "slowloris"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "SLOW_POST_ATTACK".to_string(),
            description: "Slow POST attack".to_string(),
            payload: json!({
                "content_length": 1_000_000,
                "send_rate": 1,
                "operation": "slow_post"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "SLOW_READ_ATTACK".to_string(),
            description: "Slow read attack".to_string(),
            payload: json!({
                "read_rate": 1,
                "connection_hold": 3600,
                "operation": "slow_read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Application-layer `DoS` tests
#[must_use]
pub fn application_dos_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "APP_REGEX_BOMB".to_string(),
            description: "Regular expression DoS".to_string(),
            payload: json!({
                "pattern": "(a+)+$",
                "input": "a".repeat(100) + "X",
                "operation": "regex_match"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "APP_XML_BOMB".to_string(),
            description: "XML billion laughs attack".to_string(),
            payload: json!({
                "xml": "<?xml version=\"1.0\"?><!DOCTYPE lolz [<!ENTITY lol \"lol\"><!ENTITY lol2 \"&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;\"><!ENTITY lol3 \"&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;\">]><lolz>&lol3;</lolz>",
                "operation": "parse_xml"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "APP_JSON_DEPTH_BOMB".to_string(),
            description: "JSON depth bomb".to_string(),
            payload: json!({
                "json": "{".repeat(10000) + "\"key\":\"value\"" + &"}".repeat(10000),
                "operation": "parse_json"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "APP_HASH_COLLISION".to_string(),
            description: "Hash collision DoS".to_string(),
            payload: json!({
                "keys": ["Aa", "BB", "C#", "D4"],
                "operation": "hash_attack",
                "collision_count": 1_000_000
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Protocol-specific `DoS` tests
#[must_use]
pub fn protocol_dos_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "PROTO_HTTP_PIPELINE".to_string(),
            description: "HTTP pipeline abuse".to_string(),
            payload: json!({
                "pipelined_requests": 1000,
                "operation": "http_pipeline_flood"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "PROTO_WEBSOCKET_FLOOD".to_string(),
            description: "WebSocket message flood".to_string(),
            payload: json!({
                "messages_per_second": 10000,
                "message_size": 64000,
                "operation": "websocket_flood"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "PROTO_DNS_AMPLIFICATION".to_string(),
            description: "DNS amplification attack".to_string(),
            payload: json!({
                "query_type": "ANY",
                "domain": "large-response.example.com",
                "operation": "dns_amplify"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Rate limit evasion techniques
#[must_use]
pub fn rate_limit_evasion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "EVASION_CASE_VARIATION".to_string(),
            description: "Case variation in endpoints".to_string(),
            payload: json!({
                "endpoints": ["/api/users", "/API/users", "/api/USERS", "/Api/Users"],
                "requests_per_endpoint": 250,
                "operation": "case_evasion"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "EVASION_ENCODING_BYPASS".to_string(),
            description: "URL encoding bypass".to_string(),
            payload: json!({
                "endpoints": ["/api/users", "/api%2Fusers", "/%61%70%69/users"],
                "requests_per_endpoint": 250,
                "operation": "encoding_evasion"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "EVASION_PARAMETER_VARIATION".to_string(),
            description: "Parameter variation bypass".to_string(),
            payload: json!({
                "base_url": "/api/search",
                "parameters": [
                    {"q": "test"},
                    {"query": "test"},
                    {"search": "test"},
                    {"term": "test"}
                ],
                "requests_per_param": 250,
                "operation": "param_evasion"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Burst attack tests
#[must_use]
pub fn burst_attack_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "BURST_IMMEDIATE".to_string(),
            description: "Immediate burst attack".to_string(),
            payload: json!({
                "requests": 1000,
                "time_window": 1,
                "operation": "immediate_burst"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "BURST_COORDINATED".to_string(),
            description: "Coordinated burst from multiple sources".to_string(),
            payload: json!({
                "sources": 10,
                "requests_per_source": 100,
                "coordination_time": 5,
                "operation": "coordinated_burst"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::DoS],
        },
        SecurityTestCase {
            name: "BURST_GRADUAL_RAMP".to_string(),
            description: "Gradual ramp-up attack".to_string(),
            payload: json!({
                "initial_rate": 10,
                "final_rate": 1000,
                "ramp_time": 60,
                "operation": "gradual_ramp"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::DoS],
        },
    ]
}

/// Create all rate limiting and `DoS` test cases
#[must_use]
pub fn all_rate_limit_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(rate_limit_bypass_tests());
    tests.extend(resource_exhaustion_tests());
    tests.extend(slow_attack_tests());
    tests.extend(application_dos_tests());
    tests.extend(protocol_dos_tests());
    tests.extend(rate_limit_evasion_tests());
    tests.extend(burst_attack_tests());
    tests
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rate_limit_test_creation() {
        let tests = all_rate_limit_tests();
        assert!(!tests.is_empty());

        // Verify we have tests from each category
        let categories: Vec<String> = tests
            .iter()
            .map(|t| t.name.split('_').next().unwrap_or(""))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();

        assert!(categories.contains(&"RATE".to_string()));
        assert!(categories.contains(&"RESOURCE".to_string()));
        assert!(categories.contains(&"SLOW".to_string()));
        assert!(categories.contains(&"APP".to_string()));
        assert!(categories.contains(&"PROTO".to_string()));
        assert!(categories.contains(&"EVASION".to_string()));
        assert!(categories.contains(&"BURST".to_string()));
    }
    #[test]
    fn test_critical_dos_tests() {
        let tests = all_rate_limit_tests();
        let critical_tests: Vec<_> = tests
            .iter()
            .filter(|t| t.severity == Severity::Critical)
            .collect();

        // Should have critical tests for dangerous DoS attacks
        assert!(!critical_tests.is_empty());

        // Memory bomb should be critical
        assert!(critical_tests
            .iter()
            .any(|t| t.name == "RESOURCE_MEMORY_BOMB"));

        // XML bomb should be critical
        assert!(critical_tests.iter().any(|t| t.name == "APP_XML_BOMB"));
    }
    #[test]
    fn test_dos_category_coverage() {
        let tests = all_rate_limit_tests();
        let dos_test_count = tests
            .iter()
            .filter(|t| t.categories.contains(&TestCategory::DoS))
            .count();

        // All rate limit tests should be DoS category
        assert_eq!(dos_test_count, tests.len());
    }
}
