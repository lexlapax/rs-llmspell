# Debug Architecture

## Overview

The LLMSpell debug system implements a **Protocol-First Architecture** that separates debug capabilities from their implementation, enabling consistent debugging across CLI, kernel, and MCP transports.

## Architecture Components

### 1. Protocol Layer (`llmspell-core/src/debug.rs`)
- **DebugCapability trait**: Defines protocol for debug operations
- **DebugRequest/DebugResponse**: Protocol messages for debug operations
- **Protocol-agnostic types**: BreakpointInfo, StackFrameInfo, VariableInfo

### 2. Bridge Layer (`llmspell-bridge/src/debug_*`)
- **DebugRuntime**: Orchestrates debug sessions with script execution
- **DebugHook trait**: Defines execution interception points
- **Protocol Adapters**: Wrap existing components to implement DebugCapability
  - ExecutionManagerAdapter: Breakpoint and execution control
  - VariableInspectorAdapter: Variable inspection
  - StackNavigatorAdapter: Stack frame navigation
  - SessionManagerAdapter: Debug session management

### 3. Engine Integration (`llmspell-bridge/src/lua/engine.rs`)
- **install_debug_hooks()**: Connects DebugHook to Lua's debug API
- **ExecutionManagerHook**: Routes debug events to ExecutionManager
- Converts between DebugControl flow and Lua debug events

### 4. Runtime Integration (`llmspell-bridge/src/runtime.rs`)
- **ScriptRuntime::install_debug_hooks()**: Delegates to engine
- **ScriptEngineBridge trait**: Extended with debug hook support

## Data Flow

```
User Command (CLI/Kernel/MCP)
    ↓
DebugBridge (routes by protocol)
    ↓
DebugCapability (protocol interface)
    ↓
Protocol Adapter (e.g., ExecutionManagerAdapter)
    ↓
Bridge Component (e.g., ExecutionManager)
    ↓
DebugRuntime (orchestrates session)
    ↓
ScriptRuntime (delegates to engine)
    ↓
LuaEngine (installs Lua debug hooks)
    ↓
Script Execution with Debug Control
```

## Performance Characteristics

Based on benchmarking:
- **Debug initialization**: < 1ms (requirement: < 10ms) ✅
- **Debug operations**: Not yet measured for individual operations
- **Hook overhead**: Minimal when no breakpoints set

## Current Limitations

### 1. Pause Mechanism Not Implemented
**Status**: Script execution continues even when breakpoints are hit
**Impact**: Debugging is not fully functional - cannot pause at breakpoints
**Solution**: Requires implementing one of:
- Lua coroutine-based yielding/resuming
- Thread parking/unparking mechanism
- Async channel-based control flow

### 2. Termination Not Implemented
**Status**: Cannot forcefully terminate script execution
**Impact**: Scripts must run to completion
**Solution**: Requires script engine cooperation for clean termination

### 3. Variable Reference System
**Status**: No lazy expansion for complex objects
**Impact**: All variable data loaded eagerly
**Solution**: Implement reference-based lazy loading (optimization)

### 4. Function Names in Debug Hooks
**Status**: Using generic "<function>" for all functions
**Impact**: Less informative debug output
**Solution**: Parse Lua debug info more carefully

## Implementation Status

### Completed ✅
- Protocol definition (DebugCapability trait)
- Protocol adapters for all debug components
- Capability registry and routing
- Debug hook integration with Lua engine
- Basic debug session orchestration
- Performance meets requirements for initialization

### Not Implemented ❌
- Actual pause/resume at breakpoints
- Script termination
- Complete variable inspection
- Full integration tests

## Future Enhancements

### Phase 1: Core Functionality (Priority: High)
- Implement pause/resume mechanism using Lua coroutines
- Add proper script termination support
- Complete variable inspection with lazy loading

### Phase 2: Enhanced Features (Priority: Medium)
- Conditional breakpoints evaluation
- Watch expressions
- Call stack modification
- Hot code reload

### Phase 3: Advanced Debugging (Priority: Low)
- Time-travel debugging
- Distributed debugging
- Performance profiling integration

## Usage Example

```rust
// Create debug runtime with session
let config = LLMSpellConfig::default();
let session = DebugSession {
    session_id: "test-session".to_string(),
    script_content: script.to_string(),
    args: vec![],
    state: DebugSessionState::Created,
};

let mut debug_runtime = DebugRuntime::new(config, session, capabilities).await?;

// Set breakpoint
let request = DebugRequest::SetBreakpoints {
    source: "script.lua".to_string(),
    breakpoints: vec![(10, None)],
};
debug_runtime.process_debug_command(request).await?;

// Execute with debug hooks (Note: won't actually pause yet)
let output = debug_runtime.execute().await?;
```

## Recommendations

1. **Immediate Priority**: Implement pause/resume mechanism to make debugging functional
2. **Create separate task**: "Implement Debug Pause/Resume Mechanism" (est. 2-3 hours)
3. **Consider using**: Lua coroutines for cleanest implementation
4. **Test with**: Real debugging scenarios once pause works