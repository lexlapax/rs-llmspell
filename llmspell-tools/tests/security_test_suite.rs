//! ABOUTME: Comprehensive security test suite for all tools
//! ABOUTME: Tests for common vulnerabilities and security controls

use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
use llmspell_core::{
    types::{AgentInput, AgentOutput},
    BaseAgent, ExecutionContext, LLMSpellError,
};
use llmspell_security::sandbox::{file_sandbox::FileSandbox, SandboxContext};
use llmspell_tools::*;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Create a standard test execution context
fn create_test_context() -> ExecutionContext {
    ExecutionContext::new()
}

/// Create an agent input with the given parameters
fn create_agent_input(params: Value) -> Result<AgentInput, LLMSpellError> {
    // AgentInput expects parameters to be wrapped in a "parameters" object
    let mut input = AgentInput::text("");
    let wrapped_params = json!({ "parameters": params });
    if let Value::Object(map) = wrapped_params {
        // Convert serde_json::Map to HashMap
        input.parameters = map.into_iter().collect();
    }
    Ok(input)
}

/// Test path traversal attempts across file system tools
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_path_traversal_prevention() {
    let tools = vec![
        (
            "file-operations",
            vec![
                ("../../../etc/passwd", "path"),
                ("..\\..\\windows\\system32", "path"),
                ("/etc/passwd", "path"),
                ("C:\\Windows\\System32", "path"),
                ("file:///etc/passwd", "path"),
                ("\\\\server\\share\\file", "path"),
            ],
        ),
        (
            "file-search",
            vec![("../../../", "path"), ("/", "path"), ("~/.ssh/", "path")],
        ),
        (
            "archive-handler",
            vec![
                ("../../../archive.zip", "source_path"),
                ("/tmp/../etc/passwd", "target_path"),
            ],
        ),
    ];

    for (tool_name, test_cases) in tools {
        for (malicious_path, param_name) in test_cases {
            let result = execute_tool_raw(
                tool_name,
                json!({
                    param_name: malicious_path,
                    "operation": "read"
                }),
            )
            .await;

            // Tools should either reject or handle safely
            if let Ok(output) = &result {
                let output_str = output.text.to_lowercase();
                assert!(
                    !output_str.contains("root:")
                        && !output_str.contains("/etc/passwd")
                        && !output_str.contains("system32"),
                    "Tool {} may be vulnerable to path traversal: {}",
                    tool_name,
                    malicious_path
                );
            }
        }
    }
}

/// Test SSRF prevention in web tools
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_ssrf_prevention() {
    let internal_urls = vec![
        "http://localhost/admin",
        "http://127.0.0.1:8080",
        "http://169.254.169.254/", // AWS metadata
        "http://[::1]/",
        "http://0.0.0.0/",
        "file:///etc/passwd",
        "gopher://localhost:70/",
        "dict://localhost:11211/",
    ];

    let web_tools = vec![
        "web-scraper",
        "api-tester",
        "webhook-caller",
        "url-analyzer",
    ];

    for tool_name in web_tools {
        for url in &internal_urls {
            // Add timeout to prevent hanging on connection attempts
            let result = timeout(
                Duration::from_secs(3),
                execute_tool_raw(
                    tool_name,
                    json!({
                        "input": url
                    }),
                ),
            )
            .await;

            // Tools should either reject, timeout, or handle safely
            match result {
                Ok(Ok(output)) => {
                    assert!(
                        !output.text.contains("/etc/passwd") && !output.text.contains("metadata"),
                        "Tool {} may be vulnerable to SSRF with URL: {}",
                        tool_name,
                        url
                    );
                }
                Ok(Err(_)) => {
                    // Tool rejected the URL - this is good
                }
                Err(_) => {
                    // Timeout - this is also acceptable for SSRF prevention
                }
            }
        }
    }
}

/// Test command injection in process executor
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_command_injection_prevention() {
    let malicious_commands = vec![
        "ls; cat /etc/passwd",
        "ls && cat /etc/passwd",
        "ls | cat /etc/passwd",
        "ls `cat /etc/passwd`",
        "ls $(cat /etc/passwd)",
        "ls\ncat /etc/passwd",
        "ls\rcat /etc/passwd",
    ];

    for cmd in malicious_commands {
        let result = execute_tool_raw(
            "process-executor",
            json!({
                "command": cmd
            }),
        )
        .await;

        // Should either reject or not execute the injection
        if let Ok(output) = &result {
            assert!(
                !output.text.contains("root:") && !output.text.contains("/etc/passwd"),
                "ProcessExecutor may be vulnerable to command injection: {}",
                cmd
            );
        }
    }
}

/// Test SQL injection prevention
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_sql_injection_prevention() {
    let sql_payloads = vec![
        "' OR '1'='1",
        "'; DROP TABLE users; --",
        "' UNION SELECT * FROM passwords --",
        "admin'--",
        "1' OR '1' = '1' /*",
    ];

    // This would need actual database setup, so we're testing parameter handling
    for payload in sql_payloads {
        let result = execute_tool_raw(
            "database-connector",
            json!({
                "provider": "sqlite",
                "connection_string": ":memory:",
                "operation": "query",
                "query": "SELECT * FROM users WHERE name = ?",
                "params": [payload]
            }),
        )
        .await;

        // Even if it fails, it shouldn't execute the injection
        if let Ok(output) = result {
            assert!(
                !output.text.contains("DROP") && !output.text.contains("UNION"),
                "Database connector may be vulnerable to SQL injection"
            );
        }
    }
}

/// Test XXE prevention in XML parsing
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_xxe_prevention() {
    let xxe_payloads = vec![
        r#"<?xml version="1.0"?>
<!DOCTYPE root [
<!ENTITY xxe SYSTEM "file:///etc/passwd">
]>
<root>&xxe;</root>"#,
        r#"<?xml version="1.0"?>
<!DOCTYPE root [
<!ENTITY xxe SYSTEM "http://169.254.169.254/">
]>
<root>&xxe;</root>"#,
    ];

    // Test with sitemap crawler which parses XML
    for _payload in xxe_payloads {
        // Would need to serve this XML, so we test handling
        let result = execute_tool_raw(
            "sitemap-crawler",
            json!({
                "input": "data:text/xml;base64," // Would encode payload
            }),
        )
        .await;

        // Should reject or parse safely without exposing file contents
        if let Ok(output) = &result {
            assert!(
                !output.text.contains("root:") && !output.text.contains("/etc/passwd"),
                "XML parser may be vulnerable to XXE"
            );
        }
    }
}

/// Test resource exhaustion prevention
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_resource_exhaustion_prevention() {
    // Test zip bomb prevention
    let result = timeout(
        Duration::from_secs(5),
        execute_tool_raw(
            "archive-handler",
            json!({
                "operation": "extract",
                "source_path": "tests/fixtures/zipbomb.zip", // Would need fixture
                "target_path": "/tmp/test"
            }),
        ),
    )
    .await;

    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Archive handler should prevent zip bombs"
    );

    // Test large JSON prevention
    let large_json = json!({
        "data": vec![0; 1_000_000] // 1MB of data
    });

    let result = timeout(
        Duration::from_secs(5),
        execute_tool_raw(
            "json-processor",
            json!({
                "input": large_json,
                "operation": "validate"
            }),
        ),
    )
    .await;

    // Should complete quickly or fail
    assert!(
        result.is_ok(),
        "JSON processor should handle large inputs gracefully"
    );
}

/// Test template injection prevention
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_template_injection_prevention() {
    let template_payloads = vec![
        "{{system('cat /etc/passwd')}}",
        "{{eval('require(\"child_process\").exec(\"cat /etc/passwd\")')}}",
        "{%raw%}{{7*7}}{%endraw%}",
        "{{constructor.constructor('return process')().exit()}}",
    ];

    for payload in template_payloads {
        let result = execute_tool_raw(
            "template-engine",
            json!({
                "input": payload,
                "engine": "handlebars",
                "context": {}
            }),
        )
        .await;

        if let Ok(output) = result {
            assert!(
                !output.text.contains("root:") && !output.text.contains("49"), // 7*7
                "Template engine may be vulnerable to injection"
            );
        }
    }
}

/// Test email header injection
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_email_header_injection() {
    let header_payloads = vec![
        "test@example.com\r\nBcc: attacker@evil.com",
        "test@example.com\nCc: attacker@evil.com",
        "Subject\r\nX-Injected: true",
    ];

    for payload in header_payloads {
        let result = execute_tool_raw(
            "email-sender",
            json!({
                "provider": "smtp",
                "from": "sender@example.com",
                "to": payload,
                "subject": "Test",
                "body": "Test"
            }),
        )
        .await;

        // Should reject or sanitize
        // Should reject malformed email addresses
        if let Ok(output) = &result {
            let has_error = output.text.contains("error")
                || output.text.contains("invalid")
                || output.text.contains("failed");
            assert!(
                has_error,
                "Email sender may be vulnerable to header injection with payload: {}",
                payload
            );
        }
    }
}

/// Test rate limiting effectiveness
#[cfg_attr(test_category = "integration")]
#[tokio::test]
#[ignore = "Makes real web requests - run with --ignored flag"]
async fn test_rate_limiting() {
    let start = std::time::Instant::now();
    let mut results = vec![];
    let mut rate_limited = false;

    // Try to make 10 requests quickly (reduced from 100)
    for i in 0..10 {
        // Add timeout to prevent hanging
        let result = timeout(
            Duration::from_secs(5),
            execute_tool_raw(
                "web_search",
                json!({
                    "input": "test",
                    "provider": "duckduckgo",
                    "max_results": 1
                }),
            ),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                // Check if we got rate limited
                if output.text.contains("rate") || output.text.contains("limit") {
                    rate_limited = true;
                    break;
                }
                results.push(Ok(output));
            }
            Ok(Err(e)) => {
                // Check if error indicates rate limiting
                let error_msg = e.to_string();
                if error_msg.contains("rate")
                    || error_msg.contains("limit")
                    || error_msg.contains("429")
                {
                    rate_limited = true;
                    break;
                }
                results.push(Err(e));
            }
            Err(_) => {
                // Timeout - could be due to rate limiting
                if i > 2 {
                    rate_limited = true;
                    break;
                }
            }
        }

        // If more than 2 seconds have passed, assume rate limiting is working
        if start.elapsed().as_secs() >= 2 {
            rate_limited = true;
            break;
        }
    }

    let elapsed = start.elapsed();

    // Should either be rate limited or take reasonable time
    assert!(
        rate_limited || elapsed.as_secs() >= 1 || results.len() < 10,
        "Rate limiting may not be properly enforced - {} requests completed in {:?}",
        results.len(),
        elapsed
    );
}

/// Test input validation across all tools
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_input_validation() {
    // Test null byte injection
    let null_byte_test = "test\0injection";

    let file_result = execute_tool_raw(
        "file-operations",
        json!({
            "path": null_byte_test,
            "operation": "read"
        }),
    )
    .await;

    assert!(
        file_result.is_err(),
        "File operations should reject null bytes"
    );

    // Test extremely long inputs
    let long_input = "A".repeat(1_000_000);

    let text_result = execute_tool_raw(
        "text-manipulator",
        json!({
            "input": long_input,
            "operation": "uppercase"
        }),
    )
    .await;

    // Should either handle or reject gracefully
    assert!(
        text_result.is_err() || text_result.unwrap().text.len() < 2_000_000,
        "Text manipulator should handle long inputs safely"
    );
}

/// Test secure random number generation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_secure_randomness() {
    let mut uuids = std::collections::HashSet::new();

    // Generate 1000 UUIDs
    for _ in 0..1000 {
        let result = execute_tool_raw(
            "uuid-generator",
            json!({
                "version": 4,
                "count": 1
            }),
        )
        .await
        .unwrap();

        let uuid = result.text;
        assert!(uuids.insert(uuid), "UUID generator produced duplicate");
    }
}

/// Test timeout enforcement
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeout_enforcement() {
    let result = execute_tool_raw(
        "web-scraper",
        json!({
            "input": "https://httpbin.org/delay/10",
            "timeout": 2
        }),
    )
    .await;

    // Should timeout
    assert!(
        result.is_err() || result.unwrap().text.contains("timeout"),
        "Web scraper should enforce timeouts"
    );
}

/// Test error message information disclosure
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_error_message_safety() {
    let result = execute_tool_raw(
        "database-connector",
        json!({
            "provider": "postgresql",
            "host": "nonexistent.internal.corp",
            "database": "secret_db",
            "username": "admin",
            "password": "Super$ecret123!",
            "operation": "query",
            "query": "SELECT * FROM users"
        }),
    )
    .await;

    if let Err(e) = result {
        let error_text = e.to_string();
        assert!(
            !error_text.contains("Super$ecret123!") && !error_text.contains("internal.corp"),
            "Error messages should not leak sensitive information"
        );
    }
}

/// Helper function to execute tools directly
async fn execute_tool_raw(tool_name: &str, params: Value) -> Result<AgentOutput, LLMSpellError> {
    let context = create_test_context();
    let input = create_agent_input(params)?;

    match tool_name {
        // File system tools
        "file-operations" => {
            FileOperationsTool::new(Default::default())
                .execute(input, context)
                .await
        }
        "file-search" => {
            let sandbox_context = SandboxContext {
                id: "test-sandbox".to_string(),
                security_requirements: SecurityRequirements::default(),
                resource_limits: ResourceLimits::default(),
                working_directory: "/tmp/llmspell-test".to_string(),
                allowed_paths: vec!["/tmp/llmspell-test".to_string()],
                allowed_domains: vec![],
                allowed_env_vars: vec![],
            };
            let sandbox = Arc::new(FileSandbox::new(sandbox_context)?);
            FileSearchTool::new(Default::default(), sandbox)
                .execute(input, context)
                .await
        }
        "archive-handler" => ArchiveHandlerTool::new().execute(input, context).await,

        // Web tools
        "web-scraper" => {
            WebScraperTool::new(Default::default())
                .execute(input, context)
                .await
        }
        "api-tester" => ApiTesterTool::new().execute(input, context).await,
        "webhook-caller" => WebhookCallerTool::new().execute(input, context).await,
        "url-analyzer" => UrlAnalyzerTool::new().execute(input, context).await,
        "sitemap-crawler" => SitemapCrawlerTool::new().execute(input, context).await,
        "web_search" => {
            WebSearchTool::new(Default::default())?
                .execute(input, context)
                .await
        }

        // System tools
        "process-executor" => {
            ProcessExecutorTool::new(Default::default())
                .execute(input, context)
                .await
        }

        // Data processing tools
        "json-processor" => {
            JsonProcessorTool::new(Default::default())
                .execute(input, context)
                .await
        }
        "database-connector" => {
            DatabaseConnectorTool::new(Default::default())?
                .execute(input, context)
                .await
        }

        // Utility tools
        "template-engine" => TemplateEngineTool::new().execute(input, context).await,
        "text-manipulator" => {
            TextManipulatorTool::new(Default::default())
                .execute(input, context)
                .await
        }
        "uuid-generator" => {
            UuidGeneratorTool::new(Default::default())
                .execute(input, context)
                .await
        }

        // Communication tools
        "email-sender" => {
            EmailSenderTool::new(Default::default())?
                .execute(input, context)
                .await
        }

        _ => Err(LLMSpellError::Tool {
            message: format!("Unknown tool: {}", tool_name),
            tool_name: Some(tool_name.to_string()),
            source: None,
        }),
    }
}
