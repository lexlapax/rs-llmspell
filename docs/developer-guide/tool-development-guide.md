# Tool Development Guide

✅ **Current Implementation**: This guide reflects actual Phase 3.3 and Phase 7 APIs and patterns used in the 37 existing tools.

This guide explains how to create new tools for rs-llmspell using current APIs and established patterns.

## Tool Development Overview

### **What is a Tool?**
Tools are reusable components that perform specific tasks like file operations, API calls, data processing, or system interactions. Each tool:
- Implements the `Tool` trait
- Declares security requirements
- Uses resource limits for safety
- Provides structured responses
- Integrates with the agent and workflow systems

### **Current Tool Ecosystem**
rs-llmspell has **37 production tools** across 10 categories:
- **API Tools** (2): GraphQL, HTTP requests
- **Communication** (2): Database, Email
- **Data Processing** (3): CSV, JSON, Graph builder
- **Document & Academic** (2): PDF processor, Citation formatter
- **File System** (5): Operations, Search, Watch, Convert, Archive
- **Media** (3): Audio, Image, Video processing
- **Search** (1): Web search
- **System** (4): Environment, Process, Service, Monitor
- **Utility** (9): Calculator, UUID, Hash, Base64, Text, etc.
- **Web** (6): URL analyzer, Scraper, API tester, Webhook, Monitor, Sitemap

---

## Creating a New Tool

### **Step 1: Tool Structure**

```rust
use llmspell_core::traits::tool::{Tool, ToolCategory, SecurityLevel, ToolResult};
use llmspell_utils::resource_limits::{ResourceLimits, ResourceTracker};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct YourTool {
    // Tool configuration
}

#[derive(Debug, Deserialize)]
pub struct YourToolInput {
    // Input parameters
    pub input: String,  // Primary input parameter (standardized)
    pub option1: Option<String>,
    pub option2: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct YourToolOutput {
    // Output data
    pub result: String,
    pub metadata: Option<serde_json::Value>,
}
```

### **Step 2: Implement Tool Trait**

```rust
impl Tool for YourTool {
    fn name(&self) -> &str {
        "your_tool"  // Registry name (use hyphens for web tools, underscores for others)
    }

    fn description(&self) -> &str {
        "Brief description of what your tool does"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Utility  // Choose appropriate category
    }

    // ✅ REQUIRED: Declare security level
    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe  // Safe, Restricted, or Privileged
    }

    // ✅ OPTIONAL: Custom security requirements (if needed)
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
            .with_file_access("/tmp/your-tool")  // If file access needed
            .with_network_access("api.example.com")  // If network access needed
    }

    // ✅ OPTIONAL: Custom resource limits (if needed)
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(50 * 1024 * 1024)  // 50MB
            .with_cpu_limit(10000)  // 10 seconds
    }

    // ✅ REQUIRED: Main execution method
    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        // 1. Parse input
        let params: YourToolInput = serde_json::from_value(input)?;
        
        // 2. Create resource tracker
        let tracker = ResourceTracker::new(self.resource_limits());
        
        // 3. Validate input
        self.validate_input(&params)?;
        
        // 4. Perform operation with resource tracking
        let result = self.perform_operation(&params, &tracker).await?;
        
        // 5. Return structured response
        Ok(ToolResult::success(serde_json::to_value(result)?))
    }
}
```

### **Step 3: Implementation Methods**

```rust
impl YourTool {
    pub fn new() -> Self {
        Self {
            // Initialize configuration
        }
    }

    fn validate_input(&self, input: &YourToolInput) -> Result<(), LLMError> {
        if input.input.is_empty() {
            return Err(LLMError::validation("Input cannot be empty"));
        }
        
        // Additional validation as needed
        Ok(())
    }

    async fn perform_operation(
        &self, 
        input: &YourToolInput, 
        tracker: &ResourceTracker
    ) -> Result<YourToolOutput, LLMError> {
        // Track this operation
        track_operation!(tracker)?;
        
        // Track memory if processing large data
        let _memory_guard = if input.input.len() > 1024 * 1024 {
            Some(MemoryGuard::new(tracker, input.input.len())?)
        } else {
            None
        };
        
        // Perform your tool's logic
        let result = self.do_actual_work(input).await?;
        
        Ok(YourToolOutput {
            result,
            metadata: Some(serde_json::json!({
                "processed_at": chrono::Utc::now().to_rfc3339(),
                "input_size": input.input.len()
            }))
        })
    }

    async fn do_actual_work(&self, input: &YourToolInput) -> Result<String, LLMError> {
        // Your tool's specific implementation
        Ok(format!("Processed: {}", input.input))
    }
}
```

---

## Tool Registration

### **Step 4: Register in Bridge**

Add your tool to `llmspell-bridge/src/tools.rs`:

```rust
// Import your tool
use llmspell_tools::your_category::YourTool;

pub fn register_all_tools() -> HashMap<String, Arc<dyn Tool>> {
    let mut tools = HashMap::new();
    
    // ... existing tools ...
    
    // Register your tool
    tools.insert("your_tool".to_string(), Arc::new(YourTool::new()));
    
    tools
}
```

### **Step 5: Add Tests**

Create tests in your tool module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_your_tool_basic() {
        let tool = YourTool::new();
        let input = json!({
            "input": "test data"
        });
        
        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        
        let output: YourToolOutput = serde_json::from_value(result.data).unwrap();
        assert_eq!(output.result, "Processed: test data");
    }

    #[tokio::test]
    async fn test_your_tool_validation() {
        let tool = YourTool::new();
        let input = json!({
            "input": ""  // Empty input should fail
        });
        
        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_security_level() {
        let tool = YourTool::new();
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
    }
}
```

---

## Common Tool Patterns

### **File Operations Tool Pattern**

```rust
use llmspell_security::{SandboxContext, FileSandbox};
use std::path::Path;

impl YourFileTool {
    async fn safe_file_operation(&self, path: &str) -> Result<String, LLMError> {
        // Create sandbox context
        let context = SandboxContext::new(
            "file-operation".to_string(),
            self.security_requirements(),
            self.resource_limits(),
        );
        
        // Validate path using file sandbox
        let file_sandbox = FileSandbox::new(context)?;
        let validated_path = file_sandbox.validate_path(Path::new(path))?;
        
        // Perform file operation
        let content = tokio::fs::read_to_string(validated_path).await?;
        Ok(content)
    }
}
```

### **Network Tool Pattern**

```rust
impl YourNetworkTool {
    async fn safe_http_request(&self, url: &str) -> Result<String, LLMError> {
        let tracker = ResourceTracker::new(self.resource_limits());
        
        // Basic URL validation
        let parsed_url = url::Url::parse(url)
            .map_err(|e| LLMError::validation(&format!("Invalid URL: {}", e)))?;
        
        // Check if domain is allowed (if restricted)
        if self.security_level() == SecurityLevel::Restricted {
            // Implement domain validation based on security_requirements()
        }
        
        // Make request with timeout
        let client = reqwest::Client::new();
        let response = tracker.with_timeout(async {
            client.get(url).send().await
        }).await??;
        
        let text = response.text().await?;
        Ok(text)
    }
}
```

### **Data Processing Tool Pattern**

```rust
use llmspell_utils::resource_limits::track_data_processing;

impl YourDataTool {
    async fn process_data(&self, data: &str) -> Result<String, LLMError> {
        let tracker = ResourceTracker::new(self.resource_limits());
        
        // Track data processing with estimated memory usage
        track_data_processing(&tracker, data.len() * 2, || {
            // Your data processing logic
            let processed = data.to_uppercase();
            Ok(processed)
        })
    }
}
```

---

## Security Guidelines

### **Security Level Selection**

**Safe (SecurityLevel::Safe)**:
- No file or network access
- Memory-only operations
- Examples: Calculator, UUID generator, Hash calculator

**Restricted (SecurityLevel::Restricted)**:  
- Limited file access to specific directories  
- Limited network access to specific domains
- Most tools should use this level
- Examples: File operations, Web scraper, API tester

**Privileged (SecurityLevel::Privileged)**:
- Full system access
- Requires security review
- Use sparingly
- Examples: Process executor, System monitor

### **Input Validation**

```rust
fn validate_input(&self, input: &YourToolInput) -> Result<(), LLMError> {
    // Always validate required fields
    if input.input.is_empty() {
        return Err(LLMError::validation("Input cannot be empty"));
    }
    
    // Validate length limits
    if input.input.len() > 1_000_000 {
        return Err(LLMError::validation("Input too large (max 1MB)"));
    }
    
    // Validate format if applicable
    if input.url.is_some() {
        url::Url::parse(input.url.as_ref().unwrap())
            .map_err(|_| LLMError::validation("Invalid URL format"))?;
    }
    
    Ok(())
}
```

---

## Testing Your Tool

### **Unit Tests**
```bash
# Test your specific tool
cargo test -p llmspell-tools --test test_your_tool

# Test with resource limits
RUST_LOG=debug cargo test -p llmspell-tools --test test_your_tool -- --nocapture
```

### **Integration Tests**
```bash
# Test tool registration
cargo test --test tool_registry_test

# Test through bridge
cargo test -p llmspell-bridge --test lua_tool_integration -- your_tool
```

### **Performance Tests**
```bash
# Benchmark your tool
cargo bench -p llmspell-tools -- your_tool

# Check resource usage
cargo test --test resource_usage_test -- your_tool
```

---

## Tool Categories and Naming

### **Naming Conventions**
- **Underscore naming**: Most tools (`file_operations`, `uuid_generator`)
- **Hyphen naming**: Web/external tools (`url-analyzer`, `api-tester`)
- **Descriptive names**: Clear purpose (`hash_calculator` not `hasher`)

### **Category Guidelines**
- **ToolCategory::Filesystem**: File/directory operations
- **ToolCategory::Network**: HTTP, web, API tools  
- **ToolCategory::DataProcessing**: Transform, analyze, convert data
- **ToolCategory::System**: Process, environment, monitoring
- **ToolCategory::Utility**: General purpose, calculations, generators
- **ToolCategory::Media**: Audio, video, image processing
- **ToolCategory::Communication**: Email, messaging, databases

---

## Common Mistakes to Avoid

### **❌ Don't Do This**
```rust
// Missing security level
fn security_level(&self) -> SecurityLevel {
    SecurityLevel::Privileged  // DON'T default to privileged!
}

// No input validation
async fn execute(&self, input: serde_json::Value) -> ToolResult {
    let params: YourToolInput = serde_json::from_value(input)?; // Can panic!
    // ... rest of implementation
}

// No resource tracking
async fn process_large_data(&self, data: &[u8]) -> Result<Vec<u8>, LLMError> {
    // Processing large data without tracking memory usage
    let processed = some_memory_intensive_operation(data);
    Ok(processed)
}
```

### **✅ Do This Instead**
```rust
// Appropriate security level
fn security_level(&self) -> SecurityLevel {
    SecurityLevel::Safe  // Start with Safe and escalate only if needed
}

// Proper input validation
async fn execute(&self, input: serde_json::Value) -> ToolResult {
    let params: YourToolInput = serde_json::from_value(input)
        .map_err(|e| LLMError::validation(&format!("Invalid input: {}", e)))?;
    
    self.validate_input(&params)?;
    // ... rest of implementation
}

// Resource tracking for large operations
async fn process_large_data(&self, data: &[u8]) -> Result<Vec<u8>, LLMError> {
    let tracker = ResourceTracker::new(self.resource_limits());
    let _memory_guard = MemoryGuard::new(&tracker, data.len() * 2)?; // Estimated output size
    
    let processed = some_memory_intensive_operation(data);
    Ok(processed)
}
```

---

## Getting Help

### **Tool Development Questions**
- **Review existing tools**: Look at similar tools in `llmspell-tools/src/`
- **Check implementation guides**: [Security Guide](security-guide.md), [Resource Limits](implementing-resource-limits.md)
- **Test thoroughly**: Use the test patterns above
- **Ask for review**: Request feedback on tool design before implementation

### **Common Resources**
- **Tool trait definition**: `llmspell-core/src/traits/tool.rs`
- **Existing tool examples**: `llmspell-tools/src/*/`
- **Registration logic**: `llmspell-bridge/src/tools.rs`
- **Test examples**: Look for `#[cfg(test)]` modules in existing tools

---

**Happy tool development! 🔧**

*For more implementation details, see [Security Development Guide](security-guide.md) and [Resource Limits Implementation](implementing-resource-limits.md)*