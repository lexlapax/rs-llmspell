# [Document Title]

**Version**: [Version Number]  
**Status**: [Production Ready | Beta | Alpha | In Development]  
**Last Updated**: [Date]

> **[Icon] Brief Description**: One-line summary of what this document covers.

**🔗 Navigation**: [← Previous Doc](link) | [Documentation Hub](../README.md) | [Next Doc →](link)

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [Detailed Usage](#detailed-usage)
6. [Examples](#examples)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)
9. [API Reference](#api-reference)
10. [See Also](#see-also)

---

## Overview

Brief introduction explaining what this topic/feature is and why it matters. Include key benefits and use cases.

### Key Features
- Feature 1: Brief description
- Feature 2: Brief description
- Feature 3: Brief description

### When to Use
Explain scenarios where this feature/tool/concept is most appropriate.

---

## Prerequisites

### Required Knowledge
- Prerequisite 1
- Prerequisite 2

### Required Setup
```bash
# Any required installation or configuration
```

### Environment Variables
```bash
export REQUIRED_VAR="value"
```

---

## Quick Start

Minimal working example to get users started immediately:

```lua
-- Simplest possible example
local example = Feature.new()
local result = example:execute("Hello, World!")
print(result)
```

Expected output:
```
Hello, World!
```

---

## Core Concepts

### Concept 1
Explanation of fundamental concept with diagram if helpful.

### Concept 2
Another key concept users need to understand.

---

## Detailed Usage

### Basic Operations

#### Operation 1
```lua
-- Example code with comments
local instance = Module.create({
    option1 = "value1",
    option2 = "value2"
})
```

#### Operation 2
```lua
-- Another operation example
instance:method({
    param = "value"
})
```

### Advanced Features

#### Feature 1
Detailed explanation with comprehensive example.

#### Feature 2
Another advanced feature with example.

---

## Examples

### Example 1: [Use Case Name]
```lua
-- Complete, runnable example
-- Shows real-world usage
```

### Example 2: [Another Use Case]
```lua
-- Another practical example
```

### Example 3: Error Handling
```lua
-- Example showing proper error handling
local result, err = risky_operation()
if not result then
    print("Error:", err)
    -- Handle error appropriately
end
```

---

## Best Practices

### DO:
- ✅ Best practice 1
- ✅ Best practice 2
- ✅ Best practice 3

### DON'T:
- ❌ Anti-pattern 1
- ❌ Anti-pattern 2
- ❌ Anti-pattern 3

### Performance Tips
- Tip 1: Explanation
- Tip 2: Explanation

---

## Troubleshooting

### Common Issues

#### Issue 1: [Error Message or Symptom]
**Cause**: Explanation of why this happens
**Solution**: 
```lua
-- Code showing how to fix
```

#### Issue 2: [Another Error]
**Cause**: Explanation
**Solution**: Step-by-step resolution

### Debug Tips
```lua
-- How to enable debug output
Debug.enable("module_name")
```

---

## API Reference

### Functions

#### `function_name(param1, param2)`
Brief description.

**Parameters:**
- `param1` (type): Description
- `param2` (type): Description

**Returns:** (type) Description

**Example:**
```lua
local result = function_name("value1", "value2")
```

### Properties

#### `property_name`
- **Type**: string
- **Default**: "default_value"
- **Description**: What this property controls

---

## See Also

- [Related Guide 1](link) - Brief description
- [Related Guide 2](link) - Brief description
- [API Documentation](../api/lua/module.md) - Full API reference
- [Examples](../../examples/) - More code examples

---

## Changelog

### Version X.Y.Z
- Added: New feature
- Fixed: Bug description
- Changed: Breaking change notice

---

**Need Help?** Join our [Discord](link) | Report issues on [GitHub](link)