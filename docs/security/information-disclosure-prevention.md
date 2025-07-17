# Information Disclosure Prevention

## Overview

Information disclosure vulnerabilities occur when applications expose sensitive information through error messages, logs, stack traces, or debug output. This document describes the comprehensive information disclosure prevention framework implemented in LLMSpell.

## Key Components

### 1. InfoDisclosurePreventer

The core component that sanitizes sensitive information from error messages and logs.

```rust
use llmspell_utils::security::information_disclosure::{
    InfoDisclosurePreventer, InfoDisclosureConfig, ErrorInfo
};

// Production mode (most restrictive)
let preventer = InfoDisclosurePreventer::production();

// Development mode (more verbose)
let preventer = InfoDisclosurePreventer::development();

// Custom configuration
let config = InfoDisclosureConfig {
    include_stack_traces: false,
    sanitize_paths: true,
    mask_sensitive_data: true,
    filter_debug_info: true,
    max_error_length: 500,
    ..Default::default()
};
let preventer = InfoDisclosurePreventer::new(config);
```

### 2. SafeErrorHandler

Production-ready error handler that integrates with LLMSpell errors.

```rust
use llmspell_utils::error_handling::{SafeErrorHandler, ErrorContext};

// Create handler (auto-detects production/development mode)
let handler = SafeErrorHandler::new(is_production);

// Handle LLMSpell errors with context
let context = ErrorContext::new()
    .with_operation("database_query")
    .with_resource("users_table")
    .with_user_id("user123");

let safe_response = handler.handle_llmspell_error(&error, context);
```

### 3. LoggingFilter

Filters sensitive information from log messages.

```rust
use llmspell_utils::security::information_disclosure::LoggingFilter;

let filter = LoggingFilter::new(Arc::new(preventer));

// Filter log messages
let filtered = filter.filter("API key: sk-1234567890");
// Result: "API key: [REDACTED]"

// Check if message should be filtered entirely
if filter.should_filter(message) {
    // Don't log this message at all
}
```

## What Gets Sanitized

### 1. Sensitive Data Patterns

The framework automatically detects and redacts:

- **API Keys and Tokens**: `api_key=xxx`, `token: xxx`, `secret=xxx`
- **Passwords**: `password=xxx`, `pwd: xxx`
- **Email Addresses**: `user@example.com`
- **IP Addresses**: `192.168.1.1`, `10.0.0.1`
- **File Paths**: `/home/user/documents`, `C:\Users\Admin\Desktop`
- **URLs with Credentials**: `https://user:pass@example.com`
- **Credit Card Numbers**: `4111 1111 1111 1111`
- **Social Security Numbers**: `123-45-6789`
- **Database Connection Strings**: `postgres://user:pass@host/db`

### 2. Path Sanitization

File paths are sanitized to prevent information leakage:

```
/home/user/projects/secret/file.txt → [path]/.../file.txt
C:\Users\Admin\Documents\data.csv → [path]/.../data.csv
```

### 3. Debug Information Filtering

In production mode, the following are removed or sanitized:

- **Memory Addresses**: `0x7fff5fbff8c0` → `[addr]`
- **Thread IDs**: `thread 'main' panicked` → `thread panicked`
- **Version Numbers**: `v1.2.3-beta` → `[version]`
- **Line Numbers**: `at src/main.rs:42:15` → `at src/main.rs:[line]`

### 4. Stack Trace Removal

Stack traces are completely removed in production mode to prevent code structure disclosure.

## Integration with Tools

### ProcessExecutorTool

```rust
pub struct ProcessExecutorTool {
    // ... other fields
    error_handler: SafeErrorHandler,
}

async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
    let context = ErrorContext::new()
        .with_operation("process_execution")
        .with_metadata("tool", "process_executor");
        
    let safe_response = self.error_handler.handle_llmspell_error(&error, context);
    
    Ok(AgentOutput::text(serde_json::to_string_pretty(&safe_response)?))
}
```

### DatabaseConnectorTool

```rust
async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
    let context = ErrorContext::new()
        .with_operation("database_query")
        .with_metadata("tool", "database_connector");
        
    let safe_response = self.error_handler.handle_llmspell_error(&error, context);
    
    Ok(AgentOutput::text(serde_json::to_string_pretty(&safe_response)?))
}
```

### EmailSenderTool

```rust
async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
    let context = ErrorContext::new()
        .with_operation("email_send")
        .with_metadata("tool", "email_sender");
        
    let safe_response = self.error_handler.handle_llmspell_error(&error, context);
    
    Ok(AgentOutput::text(serde_json::to_string_pretty(&safe_response)?))
}
```

## Safe Error Response Format

The sanitized error response includes:

```json
{
    "error": "Sanitized error message",
    "code": "ERR_12345678",
    "category": "validation",
    "retry": false,
    "request_id": "req_abc123",
    "timestamp": "2024-01-15T10:30:00Z"
}
```

## Configuration Options

### Production Mode (Default)

```rust
InfoDisclosureConfig {
    include_stack_traces: false,      // No stack traces
    sanitize_paths: true,             // Sanitize file paths
    mask_sensitive_data: true,        // Mask API keys, passwords, etc.
    filter_debug_info: true,          // Remove debug information
    max_error_length: 500,            // Truncate long errors
    sensitive_patterns: [...],        // Default patterns
    allowed_error_details: {          // Only specific error types
        "validation",
        "permission",
        "not_found",
        "timeout",
        "rate_limit"
    }
}
```

### Development Mode

```rust
InfoDisclosureConfig {
    include_stack_traces: true,       // Include stack traces
    sanitize_paths: false,            // Show full paths
    mask_sensitive_data: true,        // Still mask sensitive data
    filter_debug_info: false,         // Include debug info
    max_error_length: 2000,           // Longer error messages
    sensitive_patterns: [...],        // Default patterns
    allowed_error_details: {}         // Allow all error details
}
```

## Best Practices

1. **Always Use SafeErrorHandler**: For any tool that handles sensitive data or external services.

2. **Provide Context**: Include operation and resource information for better error tracking:
   ```rust
   let context = ErrorContext::new()
       .with_operation("file_read")
       .with_resource(file_path)
       .with_user_id(user_id);
   ```

3. **Log Safely**: Use LoggingFilter for any logs that might contain user data:
   ```rust
   let filtered_message = log_filter.filter(&original_message);
   ```

4. **Error Codes**: Use the generated error codes for debugging without exposing details:
   ```rust
   let error_code = preventer.generate_error_code(&error_message);
   ```

5. **Audit Trails**: The framework maintains internal audit logs for security analysis.

## Testing

Run the example to see the framework in action:

```bash
cd llmspell-utils
cargo run --example information_disclosure_prevention
```

## Security Considerations

1. **Defense in Depth**: This framework is one layer of security. Also implement:
   - Input validation
   - Secure logging practices
   - Proper access controls
   - Regular security audits

2. **Configuration**: Ensure production deployments use production configuration.

3. **Updates**: Keep sensitive patterns updated as new data formats emerge.

4. **Monitoring**: Monitor for attempts to extract information through error messages.

## Performance Impact

The information disclosure prevention framework has minimal performance impact:

- Regex patterns are compiled once using lazy_static
- Path sanitization uses caching
- Error sanitization is only performed when errors occur
- Log filtering is efficient with early pattern matching

## Future Enhancements

1. **ML-Based Detection**: Use machine learning to detect new sensitive patterns
2. **Context-Aware Sanitization**: Different sanitization levels based on user roles
3. **Metrics Collection**: Track what types of sensitive data are most commonly sanitized
4. **Integration with SIEM**: Send security events to security monitoring systems