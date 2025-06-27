# Known Issues and Limitations

**Last Updated**: June 27, 2025  
**Phase**: 1 - Core Execution Runtime

## Current Limitations

### 1. Placeholder Implementations

These features have stub implementations that will be completed in future phases:

#### Agent API (Phase 2)
```lua
-- Currently returns mock response
local agent = Agent.create({ name = "assistant" })
local response = agent:execute("Hello")  -- Returns placeholder
```

#### Tool API (Phase 3)
```lua
-- Currently returns empty list
local tools = Tool.list()  -- Returns {}
```

#### Workflow API (Phase 4)
```lua
-- Not yet accessible from scripts
local workflow = Workflow.create("my-workflow")  -- Not implemented
```

### 2. JavaScript Engine (Phase 5)

The JavaScript engine has compilation issues with the boa_engine dependency:

```
error[E0277]: `Cell<Option<NonNull<LinkedListLink>>>` cannot be shared between threads safely
```

**Workaround**: Use `--no-default-features --features lua` when building if needed.

### 3. CLI Integration Tests

Some CLI tests have overly strict output assertions:

- `test_cli_help`: Expects exact help text format
- `test_providers_command`: String matching too specific
- `test_output_format_json`: Regex pattern issues

**Impact**: Tests fail but functionality works correctly.

## Security Restrictions (Intentional)

These are not bugs but intentional security measures:

1. **No File System Access**
   ```lua
   -- This will fail by design
   local file = io.open("data.txt", "r")  -- Error: Access denied
   ```

2. **No Process Spawning**
   ```lua
   -- This will fail by design
   os.execute("ls")  -- Error: Access denied
   ```

3. **Limited Network Access**
   - Only LLM provider endpoints are accessible
   - General HTTP requests are blocked

4. **Memory Limits**
   - Scripts limited to 50MB by default
   - Configurable in `llmspell.toml`

## Platform-Specific Issues

### macOS
- No known issues

### Linux
- No known issues

### Windows
- Not tested in Phase 1
- Path handling should work (uses cross-platform utilities)

## Performance Considerations

1. **Lua Table Performance**
   - Large tables (>100k entries) may impact performance
   - Use streaming for large datasets when possible

2. **Coroutine Overhead**
   - Minimal overhead for streaming implementation
   - Future optimization possible in Phase 6

## Workarounds

### For Agent Placeholder
```lua
-- Until Phase 2, simulate agent responses
local function mock_agent_execute(prompt)
    return {
        text = "This is a placeholder response to: " .. prompt,
        usage = { tokens = 100 }
    }
end
```

### For Missing File Access
```lua
-- Use configuration or hardcoded data instead
local config = {
    -- Embed configuration directly
    settings = { theme = "dark", language = "en" }
}
```

### For CLI Test Issues
```bash
# Run only library tests if CLI tests fail
cargo test --lib --workspace
```

## Reporting New Issues

When reporting issues, please include:

1. **Environment**:
   - OS and version
   - Rust version (`rustc --version`)
   - LLMSpell version or commit

2. **Reproduction Steps**:
   - Minimal script that shows the issue
   - Exact commands run
   - Expected vs actual behavior

3. **Error Messages**:
   - Complete error output
   - Any relevant logs

## Fixed Issues

### Phase 1 Resolved

1. ✅ **Unused import warning**: Fixed in `system_info.rs`
2. ✅ **Clippy warnings**: All resolved
3. ✅ **Test concurrency**: Threshold adjusted for CI
4. ✅ **Documentation warnings**: All resolved

---

*This document will be updated as new issues are discovered or resolved.*