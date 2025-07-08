# LLMSpell Examples

This directory contains comprehensive examples demonstrating the Phase 2 tools library functionality.

## 📁 Example Files

### Core Examples
- **`hello.lua`** - Basic LLMSpell script example
- **`basic-math.lua`** - Lua mathematical operations
- **`agent_creation_test.lua`** - Agent creation and configuration
- **`streaming-demo.lua`** - Streaming operations example
- **`provider-info.lua`** - Provider information and configuration
- **`multimodal-stub.lua`** - Multimodal interaction example

### Phase 2 Tools Examples (Updated Naming Convention)

All tool example files follow the pattern: `tools-{category}.lua`

- **`tools-showcase.lua`** - Complete demonstration of all 26 tools
- **`tools-utility.lua`** - Utility tools (UUID, Base64, Hash, Text, Calculator, DateTime, etc.)
- **`tools-filesystem.lua`** - File system operations with security sandboxing
- **`tools-system.lua`** - System integration with security controls
- **`tools-data.lua`** - Data processing tools (JSON, CSV, HTTP, GraphQL)
- **`tools-media.lua`** - Media processing tools (Audio, Video, Image)
- **`tools-security.lua`** - Security features and sandboxing demonstrations
- **`tools-utility-reference.lua`** - Reference implementation showing correct Tool API usage

## 🚀 Running Examples

### Prerequisites
1. Ensure LLMSpell is installed and configured
2. Have appropriate API keys configured
3. Review security settings for system tools

### Basic Usage
```bash
# Run basic examples
llmspell run examples/hello.lua
llmspell run examples/basic-math.lua

# Run tool demonstrations
llmspell run examples/tools-utility.lua
llmspell run examples/tools-filesystem.lua
```

### Advanced Usage
```bash
# Run complete tools showcase
llmspell run examples/tools-showcase.lua

# Run system integration examples (requires elevated permissions)
llmspell run examples/tools-system.lua

# Run all examples
llmspell run examples/tools-run-all.lua
```

## 📋 Tools Covered

### 🔧 Utility Tools (9 tools)
- **UuidGeneratorTool** - UUID generation and validation
- **Base64EncoderTool** - Base64 encoding/decoding
- **HashCalculatorTool** - Hash calculation and verification
- **TextManipulatorTool** - Text manipulation and transformation
- **CalculatorTool** - Mathematical expression evaluation
- **DateTimeHandlerTool** - Date/time parsing and formatting
- **DiffCalculatorTool** - Text and JSON difference calculation
- **DataValidationTool** - Data validation with custom rules
- **TemplateEngineTool** - Template rendering (Handlebars/Tera)

### 📁 File System Tools (5 tools)
- **FileOperationsTool** - Secure file operations
- **ArchiveHandlerTool** - Archive creation and extraction
- **FileWatcherTool** - File system monitoring
- **FileConverterTool** - File format and encoding conversion
- **FileSearchTool** - Content search within files

### 🖥️ System Integration Tools (4 tools)
- **EnvironmentReaderTool** - Environment variable access
- **ProcessExecutorTool** - Secure command execution
- **ServiceCheckerTool** - Service availability checking
- **SystemMonitorTool** - System resource monitoring

### 📊 Data Processing Tools (4 tools)
- **JsonProcessorTool** - JSON processing with jq queries
- **CsvAnalyzerTool** - CSV analysis and statistics
- **HttpRequestTool** - HTTP client operations
- **GraphQLQueryTool** - GraphQL query execution

### 🎬 Media Processing Tools (3 tools)
- **AudioProcessorTool** - Audio file analysis
- **ImageProcessorTool** - Image metadata extraction
- **VideoProcessorTool** - Video file information

## 🔒 Security Features

All examples demonstrate secure usage:

### Input Validation
- Parameter validation for all tools
- Type checking and range validation
- Error handling for invalid inputs

### Resource Limits
- Memory usage limits
- CPU time limits
- File size restrictions
- Network rate limiting

### Sandboxing
- File system path restrictions
- Network access controls
- Process execution limits
- Environment variable filtering

### Audit Trail
- Operation logging
- Security event tracking
- Performance monitoring
- Error reporting

## 📖 Example Categories

### Beginner Examples
- `hello.lua` - Basic script structure
- `basic-math.lua` - Simple operations
- `tools-utility.lua` - Safe utility operations

### Intermediate Examples
- `tools-filesystem.lua` - File operations with security
- `tools-data.lua` - Data processing operations
- `tools-showcase.lua` - Multiple tool integration

### Advanced Examples
- `tools-system.lua` - System-level operations
- `tools-security.lua` - Security testing and validation
- Custom tool combinations and workflows

## 🎯 Use Cases

### Development
- Code generation and transformation
- File processing and analysis
- Data validation and cleaning
- Template rendering

### System Administration
- System monitoring and health checks
- File system management
- Process execution and automation
- Service availability monitoring

### Data Processing
- JSON/CSV data analysis
- HTTP API interactions
- Content searching and indexing
- Archive management

### Security
- Hash calculation and verification
- Secure file operations
- Environment variable management
- Audit trail generation

## 🔧 Configuration

### Tool Configuration
Each tool can be configured with:
- Security settings
- Resource limits
- Custom parameters
- Performance tuning

### Example Configuration
```lua
-- Configure tool with custom settings
local tool_config = {
    max_memory_mb = 100,
    timeout_ms = 5000,
    security_level = "restricted"
}

local result = agent:use_tool("tool_name", {
    operation = "example",
    parameters = {...},
    config = tool_config
})
```

## 📊 Performance Considerations

### Optimization Tips
- Use appropriate resource limits
- Implement proper error handling
- Monitor memory usage
- Cache frequently used data
- Use streaming for large files

### Benchmarking
- Tool initialization time: <10ms
- Memory usage: Controlled by limits
- CPU usage: Bounded by timeouts
- Network requests: Rate limited

## 🚨 Error Handling

All examples include:
- Proper error handling
- Informative error messages
- Graceful degradation
- Security-conscious error reporting

## 📝 Best Practices

### Security
1. Always validate inputs
2. Use appropriate security levels
3. Monitor resource usage
4. Implement proper error handling
5. Follow principle of least privilege

### Performance
1. Set appropriate resource limits
2. Use streaming for large data
3. Cache when appropriate
4. Monitor and profile operations
5. Implement proper cleanup

### Code Quality
1. Use descriptive variable names
2. Include proper documentation
3. Follow consistent formatting
4. Implement comprehensive testing
5. Use version control

## 🔍 Troubleshooting

### Common Issues
- **Permission denied**: Check security settings
- **Resource limits**: Adjust memory/CPU limits
- **Network errors**: Verify connectivity and permissions
- **File operations**: Ensure proper paths and permissions

### Debug Mode
Enable debug logging for detailed information:
```lua
-- Enable debug mode
local agent = Agent.create("claude-3-sonnet-20240229", {
    debug = true,
    log_level = "debug"
})
```

## 📚 Further Reading

- [LLMSpell Documentation](../docs/)
- [Security Guide](../SECURITY_AUDIT_REPORT.md)
- [API Reference](../docs/technical/)
- [Architecture Overview](../docs/technical/rs-llmspell-final-architecture.md)

## 🤝 Contributing

To add new examples:
1. Follow existing naming conventions
2. Include comprehensive documentation
3. Add security considerations
4. Test thoroughly
5. Update this README

---

**Phase 2 Status**: ✅ All 25 tools implemented and documented  
**Security**: ✅ Comprehensive security validation passed  
**Examples**: ✅ Complete example suite available  
**Ready for**: Phase 3 workflow orchestration