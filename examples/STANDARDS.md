# LLMSpell Example Standards

This document defines the standards and conventions for all LLMSpell examples.

## 📋 Metadata Header

Every example MUST start with a metadata header:

### Lua Examples
```lua
-- Example: [Name of Example]
-- Purpose: [What this example demonstrates]
-- Audience: [Script Users|Rust Developers|System Integrators]
-- Prerequisites: [What users need before running this]
-- Expected Output: [What should happen when run successfully]
-- Version: [LLMSpell version this works with]
-- Tags: [agent, tool, workflow, state, error-handling, etc.]
```

### Rust Examples
```rust
//! Example: [Name of Example]
//! Purpose: [What this example demonstrates]
//! Audience: [Rust Developers|Library Users]
//! Prerequisites: [Required dependencies or setup]
//! Expected Output: [What should happen when run successfully]
//! Version: [LLMSpell version this works with]
//! Tags: [agent, tool, workflow, state, error-handling, etc.]
```

## 🏗️ Structure Requirements

### File Organization
1. **Naming Convention**:
   - Use kebab-case for files: `error-handling.lua`, `custom-tool.rs`
   - Prefix with numbers for sequential learning: `00-hello-world.lua`
   - Be descriptive but concise

2. **Directory Placement**:
   - Place in appropriate audience directory
   - Use correct scope subdirectory
   - Group related examples together

### Code Structure
1. **Imports/Requires First**:
   ```lua
   -- All requires at the top
   local json = require("json")
   ```

2. **Configuration Section**:
   ```lua
   -- Configuration (use environment variables)
   local API_KEY = os.getenv("OPENAI_API_KEY")
   local CONFIG = {
       timeout = 30,
       retries = 3
   }
   ```

3. **Main Logic**:
   - Clear section comments
   - Logical flow from simple to complex
   - Group related operations

4. **Error Handling**:
   - Always include error handling
   - Show both success and failure paths
   - Provide helpful error messages

## ✅ Quality Requirements

### Documentation
1. **Inline Comments**:
   - Explain WHY, not just WHAT
   - Comment complex logic
   - Note important assumptions

2. **Expected Output**:
   - Document what successful execution looks like
   - Include sample output in comments
   - Note any side effects

### Error Handling
1. **Required Patterns**:
   ```lua
   -- Good: Proper error handling
   local success, result = pcall(function()
       return Tool.execute("FileReader", {path = "data.txt"})
   end)
   
   if not success then
       print("Error reading file: " .. tostring(result))
       -- Graceful degradation or recovery
   end
   ```

2. **Never Do**:
   ```lua
   -- Bad: No error handling
   local data = Tool.execute("FileReader", {path = "data.txt"})
   ```

### Security
1. **No Hardcoded Secrets**:
   ```lua
   -- Good: Use environment variables
   local api_key = os.getenv("OPENAI_API_KEY")
   
   -- Bad: Hardcoded secrets
   local api_key = "sk-abc123..."  -- NEVER DO THIS
   ```

2. **Input Validation**:
   - Validate user inputs
   - Sanitize file paths
   - Check boundaries

## 🧪 Testing Requirements

### Self-Contained
- Examples must be runnable without external dependencies
- Include test data or generate it
- Clean up after execution

### CI Integration
- Must pass in CI environment
- Handle missing API keys gracefully
- Timeout appropriately

### Verification
```lua
-- Include verification at the end
assert(result ~= nil, "Result should not be nil")
assert(type(result) == "table", "Result should be a table")
print("✅ Example completed successfully!")
```

## 📝 Documentation Requirements

### README Files
Each directory must have a README.md that includes:
1. Purpose of the examples
2. Prerequisites
3. How to run
4. Common issues
5. Learning path

### Cross-References
- Link to related examples
- Reference documentation
- Point to next steps

## 🚀 Performance Considerations

### Resource Usage
1. **Clean Up Resources**:
   ```lua
   -- Always clean up
   if session then
       session:close()
   end
   ```

2. **Reasonable Limits**:
   - Don't create 1000s of objects
   - Use appropriate timeouts
   - Limit retry attempts

### Efficiency
- Show efficient patterns
- Avoid unnecessary operations
- Demonstrate caching where appropriate

## 🔄 Maintenance

### Version Compatibility
- Tag examples with version
- Update when APIs change
- Note deprecations

### Regular Testing
- Test with each release
- Update for breaking changes
- Keep output current

## 📏 Style Guide

### Lua Style
```lua
-- Use local variables
local my_var = "value"

-- Use snake_case for variables
local user_name = "Alice"

-- Use PascalCase for globals (Tool, Agent, etc.)
local result = Tool.execute("Calculator", {})

-- Consistent indentation (2 spaces)
if condition then
  do_something()
end
```

### Rust Style
```rust
// Follow Rust conventions
use llmspell::prelude::*;

// Use snake_case for functions
fn process_data(input: &str) -> Result<String> {
    // Implementation
}

// Use CamelCase for types
struct CustomAgent {
    // Fields
}

// Proper error handling
let result = operation().context("Failed to perform operation")?;
```

## ✨ Example Template

### Lua Template
```lua
-- Example: Template Example
-- Purpose: Demonstrates example structure
-- Audience: Script Users
-- Prerequisites: None
-- Expected Output: Prints "Hello, World!"
-- Version: 0.7.0
-- Tags: basic, template

-- Configuration
local CONFIG = {
    message = "Hello, World!"
}

-- Main function
local function main()
    -- Example logic here
    print(CONFIG.message)
    
    -- Verification
    assert(CONFIG.message ~= nil, "Message should not be nil")
    print("✅ Example completed successfully!")
end

-- Error handling wrapper
local success, err = pcall(main)
if not success then
    print("❌ Error: " .. tostring(err))
    os.exit(1)
end
```

### Rust Template
```rust
//! Example: Template Example
//! Purpose: Demonstrates example structure
//! Audience: Rust Developers
//! Prerequisites: llmspell = "0.7"
//! Expected Output: Prints "Hello, World!"
//! Version: 0.7.0
//! Tags: basic, template

use anyhow::Result;
use llmspell::prelude::*;

fn main() -> Result<()> {
    // Configuration
    let config = Config {
        message: "Hello, World!".to_string(),
    };
    
    // Example logic
    println!("{}", config.message);
    
    // Verification
    assert!(!config.message.is_empty());
    println!("✅ Example completed successfully!");
    
    Ok(())
}

struct Config {
    message: String,
}
```

## 🎯 Goals

Every example should:
1. **Teach** - Demonstrate a concept clearly
2. **Work** - Run successfully out of the box
3. **Inspire** - Show what's possible
4. **Guide** - Lead to next steps
5. **Document** - Be self-explanatory

Following these standards ensures consistent, high-quality examples that help users succeed with LLMSpell.