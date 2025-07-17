//! Example demonstrating information disclosure prevention

use llmspell_utils::error_handling::{ErrorContext, SafeErrorHandler};
use llmspell_utils::security::information_disclosure::{
    ErrorInfo, InfoDisclosurePreventer, LoggingFilter,
};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("=== Information Disclosure Prevention Demo ===\n");

    // Demo 1: Error sanitization in production vs development
    demo_error_sanitization();

    // Demo 2: Log filtering
    demo_log_filtering();

    // Demo 3: Safe error handling with LLMSpell
    demo_safe_error_handling();
}

fn demo_error_sanitization() {
    println!("1. Error Sanitization Demo:");
    println!("----------------------------");

    // Create production and development preventers
    let prod_preventer = InfoDisclosurePreventer::production();
    let dev_preventer = InfoDisclosurePreventer::development();

    // Create an error with sensitive information
    let mut context = HashMap::new();
    context.insert("user_id".to_string(), "user123".to_string());
    context.insert("api_key".to_string(), "sk-1234567890abcdef".to_string());

    let error_info = ErrorInfo {
        message:
            "Failed to connect to database at postgres://user:password123@192.168.1.100:5432/mydb"
                .to_string(),
        kind: Some("database_error".to_string()),
        stack_trace: Some("at src/db/connection.rs:42:15\nat src/main.rs:100:20".to_string()),
        context,
        source_location: Some(("src/db/connection.rs".to_string(), 42)),
    };

    // Sanitize in production mode
    println!("Production Mode:");
    let prod_error = prod_preventer.sanitize_error(&error_info);
    println!("  Message: {}", prod_error.message);
    println!("  Category: {:?}", prod_error.category);
    println!("  Error Code: {}", prod_error.error_code);
    println!("  Retriable: {}", prod_error.retriable);

    // Sanitize in development mode
    println!("\nDevelopment Mode:");
    let dev_error = dev_preventer.sanitize_error(&error_info);
    println!("  Message: {}", dev_error.message);
    println!("  Category: {:?}", dev_error.category);
    println!("  Error Code: {}", dev_error.error_code);
    println!("  Note: Stack traces would be included in dev mode\n");
}

fn demo_log_filtering() {
    println!("\n2. Log Filtering Demo:");
    println!("----------------------");

    let preventer = Arc::new(InfoDisclosurePreventer::production());
    let log_filter = LoggingFilter::new(preventer.clone());

    // Test various log messages
    let test_logs = vec![
        "Starting server on port 8080",
        "User authentication failed for email: john.doe@example.com",
        "API request with key: sk-1234567890abcdef",
        "Database connection string: postgres://admin:secret123@localhost/db",
        "Processing file: /home/user/documents/secret.pdf",
        "Credit card validation failed for: 4111 1111 1111 1111",
    ];

    println!("Original -> Filtered:");
    for log in test_logs {
        let filtered = log_filter.filter(log);
        println!("  {} -> {}", log, filtered);

        if log_filter.should_filter(log) {
            println!("    [This message would be filtered out entirely]");
        }
    }
}

fn demo_safe_error_handling() {
    println!("\n\n3. Safe Error Handling Demo:");
    println!("----------------------------");

    // Simulate production mode
    let handler = SafeErrorHandler::new(true);

    // Create various LLMSpell errors
    use llmspell_core::LLMSpellError;

    let errors = vec![
        (
            LLMSpellError::Validation {
                message: "Invalid path: /etc/passwd - permission denied".to_string(),
                field: Some("file_path".to_string()),
            },
            ErrorContext::new()
                .with_operation("file_read")
                .with_resource("/etc/passwd"),
        ),
        (
            LLMSpellError::Tool {
                tool_name: Some("database_connector".to_string()),
                message: "Connection failed: postgres://admin:password@192.168.1.100/sensitive_db"
                    .to_string(),
                source: None,
            },
            ErrorContext::new()
                .with_operation("database_query")
                .with_user_id("user456"),
        ),
        (
            LLMSpellError::Network {
                message:
                    "Failed to connect to API at https://api.example.com with key sk-abcdef123456"
                        .to_string(),
                source: None,
            },
            ErrorContext::new()
                .with_operation("api_call")
                .with_metadata("endpoint", "/v1/users"),
        ),
    ];

    println!("LLMSpell Error -> Safe Response:");
    for (error, context) in errors {
        let safe_response = handler.handle_llmspell_error(&error, &context);
        println!("\n  Original: {:?}", error);
        println!("  Safe Response:");
        println!("    Error: {}", safe_response.error);
        println!("    Code: {}", safe_response.code);
        println!("    Category: {:?}", safe_response.category);
        println!("    Retry: {}", safe_response.retry);
    }

    println!("\n=== Demo Complete ===");
}
