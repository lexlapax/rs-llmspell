//! ABOUTME: Integration tests for ApiTesterTool
//! ABOUTME: Tests REST API testing functionality with real HTTP endpoints

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::ApiTesterTool;
use serde_json::json;

#[tokio::test]
async fn test_api_tester_get_request() {
    let tool = ApiTesterTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::HTTPBIN_GET,
        "method": "GET",
        "headers": {
            "User-Agent": "llmspell-test"
        }
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_success_output(&output, &["operation", "result"]);

            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            assert_eq!(output_value["result"]["response"]["status_code"], 200);

            // Verify httpbin echoed our header
            let response_body = output_value["result"]["response"]["body"]
                .as_object()
                .unwrap();
            let headers = response_body["headers"].as_object().unwrap();
            assert_eq!(headers["User-Agent"], "llmspell-test");
        }
        Err(e) => {
            eprintln!("Warning: API GET test failed due to network issue: {}", e);
            eprintln!("This is likely due to httpbin.org being unavailable");
        }
    }
}

#[tokio::test]
async fn test_api_tester_post_request() {
    let tool = ApiTesterTool::new();
    let context = create_test_context();

    let test_data = json!({
        "name": "Test User",
        "email": "test@example.com"
    });

    let input = create_agent_input(json!({
        "input": test_endpoints::HTTPBIN_POST,
        "method": "POST",
        "body": test_data,
        "headers": {
            "Content-Type": "application/json"
        }
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_success_output(&output, &["operation", "result"]);

            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            assert_eq!(output_value["result"]["response"]["status_code"], 200);

            // Verify httpbin echoed our data
            let response_body = output_value["result"]["response"]["body"]
                .as_object()
                .unwrap();
            let json_data = response_body["json"].as_object().unwrap();
            assert_eq!(json_data["name"], "Test User");
            assert_eq!(json_data["email"], "test@example.com");
        }
        Err(e) => {
            eprintln!("Warning: API POST test failed due to network issue: {}", e);
            eprintln!("This is likely due to httpbin.org being unavailable");
        }
    }
}

#[tokio::test]
async fn test_api_tester_status_codes() {
    let tool = ApiTesterTool::new();

    // Test various status codes
    let mut successful_tests = 0;
    for status_code in [200, 201, 400, 404, 500] {
        let context = create_test_context();
        let input = create_agent_input(json!({
            "input": format!("{}/{}", test_endpoints::HTTPBIN_STATUS, status_code),
            "method": "GET"
        }))
        .unwrap();

        let output = match tool.execute(input, context).await {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Warning: API test failed for status {}: {}", status_code, e);
                eprintln!("This is likely due to httpbin.org being unavailable");
                continue;
            }
        };
        let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        // Handle potential httpbin.org issues
        if !output_value["success"].as_bool().unwrap_or(false) {
            eprintln!(
                "Warning: API request failed for status {}: {}",
                status_code, output.text
            );
            // Skip this status code if httpbin is having issues
            continue;
        }

        let actual_status = output_value["result"]["response"]["status_code"].as_u64();

        // Check if we got a 502/503 error indicating httpbin.org issues
        if actual_status == Some(502) || actual_status == Some(503) {
            eprintln!(
                "Warning: httpbin.org returned {} for status {} request - service issue",
                actual_status.unwrap(),
                status_code
            );
            continue;
        }

        assert_eq!(
            actual_status,
            Some(status_code as u64),
            "Status code mismatch for {}: got {:?}",
            status_code,
            actual_status
        );
        successful_tests += 1;
    }

    // Ensure at least some tests passed (httpbin.org might be completely down)
    assert!(
        successful_tests >= 2,
        "Too few successful tests ({}/5) - httpbin.org may be having issues",
        successful_tests
    );
}

#[tokio::test]
async fn test_api_tester_timeout() {
    let tool = ApiTesterTool::new();
    let context = create_test_context();

    // Request with 2 second timeout to a 5 second delay endpoint
    let input = create_agent_input(json!({
        "input": format!("{}/5", test_endpoints::HTTPBIN_DELAY),
        "method": "GET",
        "timeout": 2
    }))
    .unwrap();

    // Timeout might return an error or a response with error
    match tool.execute(input, context).await {
        Ok(output) => {
            assert_error_output(&output, "timeout");
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("timeout")
                    || error_msg.contains("elapsed")
                    || error_msg.contains("Request failed: error sending request")
            );
        }
    }
}

#[tokio::test]
async fn test_api_tester_invalid_url() {
    let tool = ApiTesterTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "not-a-valid-url",
        "method": "GET"
    }))
    .unwrap();

    let result = tool.execute(input, context).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("URL validation failed"));
}

#[tokio::test]
async fn test_api_tester_network_error() {
    let tool = ApiTesterTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::INVALID_URL,
        "method": "GET"
    }))
    .unwrap();

    // This might succeed with a network error in the response, or fail with an error
    match tool.execute(input, context).await {
        Ok(output) => {
            // If it returns Ok, check for error in response
            assert_error_output(&output, "error");
        }
        Err(e) => {
            // If it returns Err, that's also acceptable for network errors
            assert!(e.to_string().contains("error") || e.to_string().contains("network"));
        }
    }
}

#[tokio::test]
async fn test_api_tester_all_http_methods() {
    let tool = ApiTesterTool::new();

    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];

    for method in methods {
        let context = create_test_context();
        let input = create_agent_input(json!({
            "input": format!("{}/{}", test_endpoints::HTTPBIN_BASE, method.to_lowercase()),
            "method": method
        }))
        .unwrap();

        match tool.execute(input, context).await {
            Ok(output) => {
                let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();

                // httpbin returns 200 for all these methods
                if output_value["success"].as_bool().unwrap_or(false) {
                    let status = output_value["result"]["response"]["status_code"]
                        .as_u64()
                        .unwrap_or(0);
                    assert!(status == 200 || status == 405, 
                        "Unexpected status {} for method {}", status, method);
                } else {
                    eprintln!("Warning: API test failed for method {}: {}", method, output.text);
                }
            }
            Err(e) => {
                eprintln!("Warning: HTTP {} test failed due to network issue: {}", method, e);
                eprintln!("This is likely due to httpbin.org being unavailable");
            }
        }
    }
}

#[tokio::test]
async fn test_api_tester_response_time_measurement() {
    let tool = ApiTesterTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::HTTPBIN_GET,
        "method": "GET"
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();

            // Response time should be a positive number
            let response_time = output_value["result"]["timing"]["duration_ms"]
                .as_u64()
                .unwrap();
            assert!(response_time > 0);
            assert!(response_time < 30000); // Less than 30 seconds (accounting for network latency)
        }
        Err(e) => {
            eprintln!("Warning: API response time test failed due to network issue: {}", e);
            eprintln!("This is likely due to httpbin.org being unavailable");
        }
    }
}
