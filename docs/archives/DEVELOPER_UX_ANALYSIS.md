# Developer UX Analysis - Phase 10 Debugging Architecture

## Document History
- 2025-09-24 15:55: Created as DAP_BREAKPOINT_ANALYSIS.md
- 2025-09-24 16:10: Renamed to DEVELOPER_UX_ANALYSIS.md for broader scope

## Current Situation

### Phase 10.7 DAP Status: BLOCKED
- Protocol handling works ✅
- Breakpoint storage works ✅
- Message format correct ✅
- **Actual debugging blocked** ❌
- Scripts run without debug hooks ❌
- Breakpoints never pause execution ❌

### Root Cause
The Lua execution engine has NO connection to the DAP/debugging infrastructure. Complete architectural disconnect between:
- ScriptExecutor (llmspell-bridge)
- ExecutionManager (llmspell-kernel)
- DAP Bridge (llmspell-kernel)

## Critical Questions & Analysis

### Q1: Do we need LSP (current 10.8) right now?

**Answer: NO - LSP should be deferred**

**Rationale:**
- LSP provides code intelligence (autocomplete, go-to-definition, hover)
- Nice-to-have but NOT required for debugging
- Adds complexity without solving core debugging problem
- Can be added later once debugging works

**Recommendation:** Move LSP to Phase 10.11 or Phase 11

### Q2: How does REPL (current 10.9) factor into debugging?

**Answer: REPL is CRITICAL and should be prioritized**

**Rationale:**
- REPL provides immediate interactive testing ground
- Can implement debug commands in REPL first
- Simpler than full DAP integration
- Provides value even without IDE integration
- Natural fit for debugging workflow

**REPL Debug Commands:**
```
> break script.lua:10        # Set breakpoint
> clear script.lua:10        # Clear breakpoint
> list breaks               # List all breakpoints
> run script.lua            # Run with debugging enabled
> step                      # Step into
> next                      # Step over
> continue                  # Continue execution
> where                     # Show stack trace
> print x                   # Inspect variable
> watch x > 10             # Set watch expression
> up/down                   # Navigate stack frames
```

### Q3: What order should we implement things?

**Recommended Phase 10 Restructure:**

#### Phase 10.8: Basic REPL Implementation (Was 10.9, simplified)
**Purpose:** Establish working REPL without debug complexity
**Priority:** HIGH - Foundation for testing and user interaction

1. **Core REPL Loop**
   - Command parsing
   - Script execution via ScriptExecutor
   - Output display
   - History management
   - Multi-line input support

2. **Basic Commands**
   ```
   > run script.lua            # Execute script file
   > eval print("hello")       # Execute inline code
   > clear                     # Clear screen
   > history                   # Show command history
   > exit/quit                # Exit REPL
   ```

3. **Debug Command Placeholders**
   ```
   > break script.lua:10       # Placeholder: "Debug not yet implemented"
   > step                      # Placeholder: "Debug not yet implemented"
   > continue                  # Placeholder: "Debug not yet implemented"
   > print x                   # Placeholder: "Debug not yet implemented"
   ```

4. **Testing**
   - Execute basic Lua scripts
   - Verify output capture
   - Test error handling
   - Validate placeholder responses

#### Phase 10.9: Debug Infrastructure Foundation (NEW)
**Purpose:** Fix the architectural disconnect
**Priority:** CRITICAL - Enables actual debugging

1. **Create DebugContext trait**
   ```rust
   trait DebugContext: Send + Sync {
       async fn check_breakpoint(&self, file: &str, line: u32) -> Result<()>;
       fn should_pause(&self, file: &str, line: u32) -> bool;
       fn enable_debug_mode(&self);
       fn disable_debug_mode(&self);
       fn is_debug_enabled(&self) -> bool;
   }
   ```

2. **Modify ScriptExecutor trait**
   ```rust
   trait ScriptExecutor {
       // New method (doesn't break existing implementations if optional)
       fn set_debug_context(&mut self, context: Option<Arc<dyn DebugContext>>);

       // Existing methods unchanged
       async fn execute_script(&self, script: &str) -> Result<ScriptExecutionOutput>;
   }
   ```

3. **Wire debug context through execution chain**
   - IntegratedKernel → ScriptRuntime → LuaEngine
   - Pass ExecutionManager wrapped as DebugContext
   - Make debug optional (None = no debugging)

4. **Implement Lua debug hooks**
   - Install hooks ONLY when debug context present
   - Call DebugContext on each line
   - Handle async/sync coordination
   - Ensure no performance impact when debug disabled

5. **Test with simple script**
   - Verify hooks fire when enabled
   - Verify no hooks when disabled
   - Verify breakpoints pause
   - Verify can resume

#### Phase 10.10: Connect REPL to Debug Infrastructure (NEW)
**Purpose:** Integrate REPL with debug capabilities
**Priority:** HIGH - First user-facing debug interface

1. **Replace Debug Placeholders**
   - Connect break command to ExecutionManager
   - Implement step/next/continue
   - Wire variable inspection
   - Add stack navigation

2. **Debug Session Management in REPL**
   ```rust
   struct ReplDebugSession {
       execution_manager: Arc<ExecutionManager>,
       current_frame: Option<StackFrame>,
       paused: bool,
   }
   ```

3. **Enhanced Debug Commands**
   ```
   > break script.lua:10       # Actually sets breakpoint
   > run -d script.lua         # Run with debug enabled
   > step                      # Step into (when paused)
   > next                      # Step over (when paused)
   > continue                  # Resume execution
   > where                     # Show stack trace
   > frame 2                   # Switch to frame 2
   > locals                    # Show local variables
   > print x                   # Evaluate expression
   > clear script.lua:10      # Clear breakpoint
   > list breaks              # Show all breakpoints
   ```

4. **REPL Debug State Machine**
   - Normal mode: execute commands normally
   - Debug mode: debug context active
   - Paused mode: at breakpoint, debug commands available
   - Handle mode transitions cleanly

5. **Testing**
   - Set and hit breakpoint
   - Step through code
   - Inspect variables
   - Resume execution
   - Clear breakpoints

#### Phase 10.11: DAP Completion (Was 10.7 continuation)
**Purpose:** IDE integration using fixed infrastructure
**Priority:** HIGH - Enables IDE debugging

1. **Fix handle_launch()**
   - Actually enable debug mode
   - Store program path
   - Set up debug session

2. **Connect to debug infrastructure**
   - Use DebugContext from 10.8
   - Send stopped events
   - Handle async coordination

3. **Complete DAP commands**
   - Stack trace
   - Variable inspection
   - Step operations
   - Continue/pause

4. **Test with VS Code**
   - Full debug session
   - Breakpoints work
   - Variables visible

#### Phase 10.11: LSP Implementation (Was 10.8)
**Purpose:** Code intelligence
**Priority:** MEDIUM - Nice to have

Move existing LSP tasks here unchanged.

## Implementation Strategy

### Step 1: Build Basic REPL (Week 1)
- Implement command parser
- Connect to ScriptExecutor
- Add placeholder debug commands
- Test script execution
- Verify no regressions

### Step 2: Create DebugContext abstraction (Week 1-2)
- Define trait in llmspell-core
- Implement for ExecutionManager
- Add optional debug to ScriptExecutor trait
- Ensure backward compatibility

### Step 3: Wire debug through execution chain (Week 2)
- Modify IntegratedKernel
- Modify ScriptRuntime
- Modify LuaEngine
- Keep debug optional (None = no debug)

### Step 4: Implement Lua hooks (Week 2-3)
- Research mlua hook API
- Handle async/sync boundary
- Test basic breakpoint
- Verify no performance impact when disabled

### Step 5: Connect REPL to debug (Week 3)
- Replace placeholder commands
- Add debug session management
- Test debug commands
- Verify state transitions

### Step 6: Complete DAP (Week 3-4)
- Fix launch command
- Connect to infrastructure
- Test with VS Code

## Async/Sync Coordination Challenge

### The Problem
- Lua hooks are synchronous callbacks
- ExecutionManager.check_breakpoint() is async
- Can't await in sync context

### Solution Options

#### Option A: Polling approach
```rust
// In Lua hook
if exec_mgr.should_pause(file, line) {
    exec_mgr.set_paused(true);

    // Busy wait with yields
    while exec_mgr.is_paused() {
        thread::sleep(Duration::from_millis(10));
        thread::yield_now();
    }
}
```

#### Option B: Channel-based
```rust
// Create channel pair
let (pause_tx, pause_rx) = channel();
let (resume_tx, resume_rx) = channel();

// In hook
if should_pause {
    pause_tx.send(PauseEvent { file, line });
    resume_rx.recv(); // Block until resumed
}

// In async context
while let Some(event) = pause_rx.recv().await {
    // Handle pause
    // ... wait for resume command
    resume_tx.send(());
}
```

#### Option C: Runtime handle (Recommended)
```rust
// In Lua hook
if exec_mgr.should_pause_sync(file, line) {
    // Use runtime handle to run async code
    let handle = tokio::runtime::Handle::current();
    handle.block_on(async {
        exec_mgr.pause_and_wait(file, line).await
    });
}
```

## Risk Assessment

### High Risk
- **Async/sync coordination** - Complex, may have deadlocks
- **Performance impact** - Debug hooks slow execution
- **Thread safety** - Lua not Send, needs careful handling

### Medium Risk
- **Breaking changes** - ScriptExecutor trait change
- **Complexity increase** - More moving parts

### Low Risk
- **REPL implementation** - Well understood
- **DAP protocol** - Already working

## Success Metrics

1. **Week 1**: DebugContext wired through
2. **Week 2**: Simple breakpoint works in test
3. **Week 3**: REPL debug commands work
4. **Week 4**: VS Code debugging works

## Why This Approach is Better

### Benefits of Basic REPL First
1. **Validates ScriptExecutor works** - Ensures basic execution is solid
2. **No debug complexity** - Simpler initial implementation
3. **Immediate value** - Users get a working REPL quickly
4. **Clean separation** - Debug is clearly an add-on feature
5. **Risk reduction** - Debug issues won't block basic functionality
6. **Placeholder commands** - Show where debug will integrate

### Benefits of Debug Infrastructure Second
1. **Clear requirements** - REPL shows exactly what's needed
2. **Optional by design** - No performance impact when disabled
3. **Backward compatible** - Existing code continues to work
4. **Focused scope** - Just fix the architectural disconnect

### Benefits of Connecting Last
1. **Both pieces tested** - REPL works, debug infrastructure works
2. **Clean integration** - Clear boundaries between systems
3. **Progressive enhancement** - REPL gains debug capabilities
4. **Easy rollback** - Can disable debug if issues arise

## Recommendation

### DO NOW:
1. **Restructure Phase 10 as proposed**
2. **Start with Phase 10.8 Basic REPL** (no debug)
3. **Then Phase 10.9 Debug Infrastructure**
4. **Then Phase 10.10 Connect them**
5. **Defer LSP to Phase 10.12**

### DON'T:
1. Don't mix debug into initial REPL
2. Don't try to fix DAP without infrastructure
3. Don't implement LSP before debugging works
4. Don't make debug mandatory - keep it optional

## Phase Renumbering Plan

### Current → New Mapping:
```
Phase 10.7: DAP (blocked)           → Keep as-is (blocked status)
Phase 10.8: LSP                     → Phase 10.12: LSP (deferred)
Phase 10.9: REPL                     → Phase 10.8: Basic REPL (simplified)
                                     → Phase 10.9: Debug Infrastructure (NEW)
                                     → Phase 10.10: Connect REPL+Debug (NEW)
                                     → Phase 10.11: DAP Completion (NEW)
Phase 10.10: Example Applications   → Phase 10.13: Example Applications
Phase 10.11: Integration Testing    → Phase 10.14: Integration Testing
Phase 10.12: Documentation          → Phase 10.15: Documentation
Phase 10.13: Phase 11 Preparation   → Phase 10.16: Phase 11 Preparation
Phase 10.14: Client Registry        → Phase 10.17: Client Registry
Phase 10.15: Resource Limits        → Phase 10.18: Resource Limits
Phase 10.16: Docker                 → Phase 10.19: Docker
Phase 10.17: Metrics                → Phase 10.20: Metrics
Phase 10.18: Performance            → Phase 10.21: Performance
Phase 10.19: Additional Testing     → Phase 10.22: Additional Testing
```

### New Phase Structure Summary:
- **10.8**: Basic REPL (foundation)
- **10.9**: Debug Infrastructure (fix architecture)
- **10.10**: Connect REPL+Debug (integration)
- **10.11**: DAP Completion (IDE support)
- **10.12**: LSP (code intelligence) [deferred]
- **10.13-10.22**: Existing phases renumbered

## Decision Required

**Should we proceed with this restructuring?**

If YES:
1. Update TODO.md with new phase structure
2. Create detailed tasks for Phase 10.8 (Basic REPL)
3. Create detailed tasks for Phase 10.9 (Debug Infrastructure)
4. Create detailed tasks for Phase 10.10 (Connect)
5. Update Phase 10.7.9 to reference the new structure

If NO:
1. Mark DAP as incomplete
2. Skip to next phases
3. Return to debugging later

## Appendix: Original DAP Breakpoint Analysis

[Previous content about DAP failure chain remains below...]

---

# Original DAP Breakpoint Analysis

## Analysis Started: 2025-09-24 15:55

## Goal
Understand why breakpoints don't pause execution when debugging Lua scripts via DAP.

## Test Results
- ✅ DAP initialize works
- ✅ DAP setBreakpoints works (breakpoints stored)
- ✅ DAP launch works
- ❌ Breakpoints don't pause execution
- ❌ No "stopped" events sent when hitting breakpoints

## Components Involved

### 1. DAP Bridge (llmspell-kernel/src/debug/dap.rs)
- Handles DAP requests/responses
- Connected to ExecutionManager
- Sets breakpoints in ExecutionManager

### 2. ExecutionManager (llmspell-kernel/src/debug/execution_bridge.rs)
- Stores breakpoints
- Has `check_breakpoint()` method for async pause
- Has `PauseState` for coordination
- Has `stopped_event_tx` channel for events

### 3. LuaDebugBridge (llmspell-kernel/src/debug/lua/mod.rs)
- Separate breakpoint storage
- Hook-based debugging support
- `on_hook()` method for line hooks
- NOT connected to ExecutionManager's breakpoints

### 4. DebugHookAdapter (llmspell-kernel/src/debug/lua/mod.rs)
- Links LuaDebugBridge to ExecutionManager
- Has optional ExecutionManager reference
- Checks both systems for breakpoints

### 5. ScriptExecutor (WHERE?)
- Actually runs Lua code
- Should install debug hooks
- Should call ExecutionManager.check_breakpoint()

## Investigation Steps

### Step 1: Find ScriptExecutor implementation ✅

Found in llmspell-bridge/src/lua/engine.rs:
- `LuaEngine` implements `ScriptEngineBridge`
- `ScriptRuntime` wraps the engine and implements `ScriptExecutor`
- **CRITICAL**: `LuaEngine::engine_features()` has `debugging: false` (line 93)
- **CRITICAL**: No debug hooks installed in `execute_script()` method
- No references to ExecutionManager, DAP, or breakpoints

### Step 2: Trace IntegratedKernel connections ✅

In llmspell-kernel/src/execution/integrated.rs:
- Line 272: `ExecutionManager` created
- Line 273: `DAPBridge` created
- Line 276: `dap_bridge.connect_execution_manager(execution_manager.clone())`
- Line 296: `script_executor` stored
- **CRITICAL**: `script_executor` is NEVER connected to `execution_manager` or `dap_bridge`

### Step 3: Find the missing connection ✅

The script execution flow is completely disconnected from the debug infrastructure:

1. `IntegratedKernel::execute_code_in_context()` calls `script_executor.execute_script()`
2. `ScriptRuntime::execute_script()` calls `engine.execute_script()`
3. `LuaEngine::execute_script()` runs Lua code WITHOUT:
   - Installing debug hooks
   - Checking breakpoints
   - Connecting to ExecutionManager
   - Calling `check_breakpoint()`

## ROOT CAUSE IDENTIFIED

**The Lua execution engine has NO connection to the DAP/debugging infrastructure.**

### Missing Components:

1. **No debug hook installation in LuaEngine**
   - Should call `lua.set_hook()` with Line events
   - Should call `execution_manager.check_breakpoint(file, line)` on each line

2. **No ExecutionManager passed to ScriptExecutor**
   - IntegratedKernel creates ExecutionManager but never passes it to script_executor
   - ScriptExecutor trait has no way to receive ExecutionManager

3. **LuaDebugBridge exists but is unused**
   - Has hook support code
   - Has DebugHookAdapter
   - Never instantiated or connected

### Step 4: Check launch command ✅

In llmspell-kernel/src/debug/dap.rs line 321:
```rust
pub fn handle_launch(&self, args: &Value) -> Result<()> {
    debug!("Handling launch request: {:?}", args);
    // Extract program path and arguments
    if let Some(program) = args.get("program").and_then(|v| v.as_str()) {
        info!("Launching program: {}", program);
        // TODO: Actually launch the program
    }
    Ok(())
}
```

**The launch command does NOTHING!** It just logs and returns Ok.

## COMPLETE FAILURE CHAIN

1. **DAP launch command is a no-op** - doesn't enable debug mode
2. **ScriptExecutor has no debug context** - can't access ExecutionManager
3. **LuaEngine doesn't install hooks** - no line-by-line execution tracking
4. **ExecutionManager.check_breakpoint() never called** - breakpoints never checked
5. **No stopped events sent** - client never knows breakpoint was hit