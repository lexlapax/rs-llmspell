# README Templates for rs-llmspell

Standardized templates for consistent documentation across the project.

## 📋 Template Categories

### 1. Documentation README Template

```markdown
# [Section Name] Documentation

**[Brief description of this documentation section]**

**🔗 Navigation**: [← Parent](../) | [Project Home](../../) | [Docs Hub](../) | [User Guide](../user-guide/) | [Developer Guide](../developer-guide/)

---

## Overview

[Purpose and scope of this documentation section]

## 📚 Contents

### Section 1: [Topic]
- **[Document]** - Description
- **[Document]** - Description

### Section 2: [Topic]
- **[Document]** - Description
- **[Document]** - Description

## ⚡ Quick Start

[Most common task or entry point]

## 📖 Related Documentation

- [Link to related section]
- [Link to related section]

---

**Status**: [Current/In Progress/Archive]  
**Last Updated**: [Date]
```

### 2. Example Category README Template

```markdown
# [Category Name] Examples

**[Brief description of example category]**

**🔗 Navigation**: [← Examples](../) | [Project Home](../../) | [Docs](../../docs/) | [Getting Started](../getting-started/)

---

## Overview

[What these examples demonstrate]

## 📚 Examples

### Example 1: [Name]
**File**: `example-1.lua`  
**Purpose**: [What it demonstrates]  
**Key Concepts**: [List of concepts]

### Example 2: [Name]
**File**: `example-2.lua`  
**Purpose**: [What it demonstrates]  
**Key Concepts**: [List of concepts]

## ⚡ Quick Start

```bash
# Run the simplest example
llmspell run examples/[category]/[example].lua
```

## 🎯 Learning Path

1. Start with [example-1]
2. Progress to [example-2]
3. Advanced: [example-3]

## 📋 Prerequisites

- [Requirement 1]
- [Requirement 2]

## 🔧 Common Patterns

[Describe common patterns demonstrated]

## 📖 Related Examples

- [Link to related category]
- [Link to related category]
```

### 3. Crate README Template

```markdown
# llmspell-[name]

**[One-line description of crate purpose]**

**🔗 Navigation**: [← Project Root](../) | [Documentation](../docs/) | [Examples](../examples/)

---

## Overview

[Detailed description of what this crate provides]

## Features

- ✅ [Feature 1]
- ✅ [Feature 2]
- ✅ [Feature 3]

## Usage

### Basic Example

```rust
use llmspell_[name]::{MainType, function};

// Example usage
let instance = MainType::new();
let result = instance.method().await?;
```

### Advanced Example

```rust
// More complex usage pattern
```

## API

### Core Types

- `MainType` - Primary interface
- `ConfigType` - Configuration
- `ResultType` - Return values

### Key Functions

- `function_1()` - Description
- `function_2()` - Description

## Dependencies

- `llmspell-core` - Core traits and types
- `[other-dep]` - Purpose

## Configuration

```toml
[package.metadata.llmspell]
feature = "value"
```

## Testing

```bash
cargo test -p llmspell-[name]
```

## Related

- [Documentation Link]
- [Example Link]
- [API Reference]
```

### 4. Application README Template

```markdown
# [Application Name]

**[Compelling one-line description]**

**🔗 Navigation**: [← Applications](../) | [Examples Home](../../) | [Documentation](../../../docs/)

---

## Overview

[Detailed description of what the application does]

## Features

- 🎯 [Key feature 1]
- 🎯 [Key feature 2]
- 🎯 [Key feature 3]

## Quick Start

```bash
# Run the application
llmspell run examples/script-users/applications/[name]/main.lua

# Or with configuration
OPENAI_API_KEY=your-key llmspell run main.lua
```

## Configuration

### Required Settings

```toml
# config.toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"

[application]
setting = "value"
```

### Environment Variables

- `OPENAI_API_KEY` - OpenAI API key (required)
- `[OTHER_VAR]` - Description (optional)

## Project Structure

```
[application-name]/
├── README.md           # This file
├── config.toml         # Configuration
├── main.lua           # Entry point
└── [other-files]      # Supporting files
```

## How It Works

1. **Step 1**: [Description]
2. **Step 2**: [Description]
3. **Step 3**: [Description]

## Example Output

```
[Sample output from running the application]
```

## Customization

[How to modify or extend the application]

## Troubleshooting

### Common Issues

**Issue**: [Description]  
**Solution**: [How to fix]

## Requirements

- LLMSpell v0.6.0+
- API keys for [providers]
- [Other requirements]

## Related Examples

- [Similar application]
- [Simpler version]
- [Advanced version]
```

## 🎨 Formatting Standards

### Navigation Bar
Always include at the top:
```markdown
**🔗 Navigation**: [← Parent](../) | [Project Home](/) | [Docs Hub](/docs/) | [Context-specific links]
```

### Emoji Usage
- 📘 User Guide
- 🔧 Developer Guide
- 📊 Technical Docs
- 📚 Examples/Contents
- ⚡ Quick Start
- 🔗 Navigation
- 🎯 Goals/Features
- ✅ Completed/Available
- 📋 Lists/Prerequisites
- 📖 Documentation/Related

### Section Ordering
1. **Title & Description**
2. **Navigation** (🔗)
3. **Overview**
4. **Quick Start** (⚡) or **Features** (✅)
5. **Main Content**
6. **Usage/Examples**
7. **Related Links** (📖)
8. **Footer** (Status/Update date if applicable)

### Code Blocks
Always specify language:
- `lua` for Lua scripts
- `rust` for Rust code
- `toml` for configuration
- `bash` for shell commands
- `markdown` for markdown examples

### Links
- Use relative paths for internal links
- Use descriptive link text (not "click here")
- Group related links in sections

## 📏 Length Guidelines

- **Root README**: 200-300 lines max
- **Documentation README**: 100-200 lines
- **Example README**: 50-150 lines
- **Crate README**: 100-150 lines
- **Application README**: 100-200 lines

## ✅ Checklist for New READMEs

- [ ] Uses appropriate template
- [ ] Has navigation breadcrumb
- [ ] Includes overview section
- [ ] Has quick start or key features
- [ ] Code examples have language tags
- [ ] Links are relative and working
- [ ] Follows emoji standards
- [ ] Appropriate length for type
- [ ] Mobile-friendly formatting
- [ ] No broken markdown

---

**Created**: Phase 7.4.6  
**Purpose**: Ensure consistent documentation UX across all 58+ README files