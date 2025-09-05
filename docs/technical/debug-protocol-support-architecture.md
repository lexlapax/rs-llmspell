# Debug Protocol Support Architecture

**Date**: 2025-09-05  
**Status**: Architecture Decision  
**Impact**: Enables IDE debugging, fixes REPL debug commands  
**Approach**: Hybrid DAP Bridge  

## Executive Summary

This document outlines the architecture for implementing debug protocol support in LLMSpell, addressing three critical gaps:
1. Non-functional `.locals` REPL command
2. Missing standalone `debug` CLI command
3. Absent DAP (Debug Adapter Protocol) support for IDE integration

**Recommendation**: Implement a hybrid DAP bridge that translates between DAP requests and our existing ExecutionManager infrastructure.

## Current State Analysis

### What Works ✅
```
Infrastructure exists but isn't connected:
- ExecutionManager tracks breakpoints, stack frames, variables
- capture_locals() extracts Lua variables (output.rs:311)
- Breakpoints pause execution (Phase 9.8.9 complete)
- GenericKernel has handle_debug_request() endpoint
- Jupyter protocol tunnels debug messages
```

### What's Broken ❌
```
1. REPL .locals command returns "not yet implemented"
2. No llmspell debug <script> CLI command
3. No DAP server for VS Code/IDE integration
4. Variables not wired to debug commands
5. Stack inspection incomplete
```

### Root Cause
The debug infrastructure was built in layers (Phase 9.1-9.8) but never connected end-to-end. Each component works in isolation but lacks the protocol layer to communicate.

## Architecture Options Evaluated

### Option 1: Direct Wiring (Quick Fix)
**Approach**: Directly connect REPL commands to ExecutionManager
```rust
// Just wire existing pieces
handle_locals_command() → execution_manager.get_variables()
```
**Verdict**: ❌ Solves immediate issue but creates technical debt, no IDE support

### Option 2: Full DAP Server
**Approach**: Implement complete DAP specification
```rust
// Full 50+ command DAP implementation
struct DAPServer implements all DAP commands
```
**Verdict**: ❌ Overkill, 90% of DAP unused, weeks of work

### Option 3: Hybrid DAP Bridge (Recommended) ✅
**Approach**: Minimal DAP adapter bridging to ExecutionManager
```rust
// Translate essential DAP commands to our infrastructure
DAPBridge translates 10 core commands
```
**Verdict**: ✅ Right-sized, enables IDEs, leverages existing code

## Proposed Architecture: Hybrid DAP Bridge

### Core Design Principles
1. **Minimal Surface**: Implement only essential DAP commands (10 vs 50+)
2. **Translation Layer**: DAP semantics → ExecutionManager operations
3. **Protocol Agnostic**: Same bridge for REPL, CLI, Jupyter, direct DAP
4. **Progressive Enhancement**: Start simple, add DAP commands as needed

### Architecture Diagram
```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   VS Code   │  │    REPL     │  │     CLI     │  │   Jupyter   │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                 │                 │                 │
       │ DAP            │ .locals        │ debug cmd      │ debug_request
       ▼                 ▼                 ▼                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                     GenericKernel                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   DAP Bridge                              │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │   │
│  │  │ Variables  │  │ StackTrace │  │Breakpoints │  ...    │   │
│  │  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘         │   │
│  └────────┼───────────────┼───────────────┼─────────────────┘   │
│           ▼               ▼               ▼                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              ExecutionManager                             │   │
│  │  • Breakpoint management                                  │   │
│  │  • Stack frame tracking                                   │   │
│  │  • Variable storage                                       │   │
│  │  • Execution control (pause/resume/step)                 │   │
│  └──────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────┘
```

### Essential DAP Commands

Only 10 commands needed for functional debugging:

| Command | Purpose | Maps To |
|---------|---------|---------|
| `initialize` | Handshake with client | Return capabilities |
| `launch` | Start debug session | Initialize ExecutionManager |
| `setBreakpoints` | Set breakpoints | `execution_manager.add_breakpoint()` |
| `setExceptionBreakpoints` | Break on errors | Configure error handling |
| `stackTrace` | Get call stack | `execution_manager.get_stack_frames()` |
| `scopes` | Get variable scopes | Return frame scopes |
| `variables` | Get variables | `capture_locals()` + cached vars |
| `continue` | Resume execution | `execution_manager.resume()` |
| `next` | Step over | `execution_manager.step_over()` |
| `stepIn` | Step into | `execution_manager.step_into()` |

### DAP Bridge Implementation

```rust
// llmspell-kernel/src/dap_bridge.rs

use dap::*;  // Use existing dap crate
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Minimal DAP adapter for ExecutionManager
pub struct DAPBridge {
    execution_manager: Arc<ExecutionManager>,
    sequence: AtomicI64,
    initialized: AtomicBool,
}

impl DAPBridge {
    pub fn new(execution_manager: Arc<ExecutionManager>) -> Self {
        Self {
            execution_manager,
            sequence: AtomicI64::new(1),
            initialized: AtomicBool::new(false),
        }
    }

    /// Main entry point for all debug requests
    pub async fn handle_request(&self, request: Value) -> Result<Value> {
        // Parse as DAP request
        let dap_request: Request = serde_json::from_value(request)?;
        
        // Route to appropriate handler
        let response = match dap_request.command.as_str() {
            "initialize" => self.handle_initialize(dap_request).await,
            "launch" => self.handle_launch(dap_request).await,
            "setBreakpoints" => self.handle_set_breakpoints(dap_request).await,
            "stackTrace" => self.handle_stack_trace(dap_request).await,
            "scopes" => self.handle_scopes(dap_request).await,
            "variables" => self.handle_variables(dap_request).await,
            "continue" => self.handle_continue(dap_request).await,
            "next" => self.handle_next(dap_request).await,
            "stepIn" => self.handle_step_in(dap_request).await,
            "disconnect" => self.handle_disconnect(dap_request).await,
            _ => self.handle_unsupported(dap_request),
        }?;
        
        Ok(serde_json::to_value(response)?)
    }

    async fn handle_initialize(&self, req: Request) -> Result<Response> {
        self.initialized.store(true, Ordering::SeqCst);
        
        Ok(Response {
            request_seq: req.seq,
            success: true,
            command: req.command,
            body: Some(json!({
                "supportsConfigurationDoneRequest": true,
                "supportsFunctionBreakpoints": false,
                "supportsConditionalBreakpoints": true,
                "supportsEvaluateForHovers": true,
                "supportsStepBack": false,
                "supportsSetVariable": false,
                "supportsRestartFrame": false,
                "supportsModulesRequest": false,
                "supportsDelayedStackTraceLoading": false,
            })),
            ..Default::default()
        })
    }

    async fn handle_set_breakpoints(&self, req: Request) -> Result<Response> {
        let args: SetBreakpointsArguments = serde_json::from_value(req.arguments)?;
        
        // Clear existing breakpoints for this source
        self.execution_manager.clear_breakpoints_for_source(&args.source.path).await;
        
        // Add new breakpoints
        let mut verified_breakpoints = Vec::new();
        for bp in args.breakpoints.unwrap_or_default() {
            let our_bp = Breakpoint::new(args.source.path.clone(), bp.line as u32)
                .with_condition(bp.condition.unwrap_or_default());
            
            let id = self.execution_manager.add_breakpoint(our_bp).await;
            
            verified_breakpoints.push(json!({
                "id": id,
                "verified": true,
                "line": bp.line,
            }));
        }
        
        Ok(Response {
            request_seq: req.seq,
            success: true,
            command: req.command,
            body: Some(json!({
                "breakpoints": verified_breakpoints
            })),
            ..Default::default()
        })
    }

    async fn handle_stack_trace(&self, req: Request) -> Result<Response> {
        let frames = self.execution_manager.get_stack_frames().await;
        
        let dap_frames: Vec<_> = frames
            .iter()
            .enumerate()
            .map(|(i, frame)| json!({
                "id": i,
                "name": frame.name,
                "source": {
                    "path": frame.source,
                },
                "line": frame.line,
                "column": frame.column.unwrap_or(0),
            }))
            .collect();
        
        Ok(Response {
            request_seq: req.seq,
            success: true,
            command: req.command,
            body: Some(json!({
                "stackFrames": dap_frames,
                "totalFrames": dap_frames.len(),
            })),
            ..Default::default()
        })
    }

    async fn handle_variables(&self, req: Request) -> Result<Response> {
        let args: VariablesArguments = serde_json::from_value(req.arguments)?;
        
        // Get variables for the requested reference
        // Reference 1000+ = stack frame ID
        let variables = if args.variables_reference >= 1000 {
            let frame_id = (args.variables_reference - 1000) as usize;
            self.execution_manager.get_frame_variables(frame_id).await
        } else {
            HashMap::new()
        };
        
        let dap_variables: Vec<_> = variables
            .iter()
            .map(|(name, var)| json!({
                "name": name,
                "value": var.value,
                "type": var.var_type,
                "variablesReference": 0,  // No children for now
            }))
            .collect();
        
        Ok(Response {
            request_seq: req.seq,
            success: true,
            command: req.command,
            body: Some(json!({
                "variables": dap_variables,
            })),
            ..Default::default()
        })
    }

    async fn handle_continue(&self, req: Request) -> Result<Response> {
        self.execution_manager.resume().await;
        
        Ok(Response {
            request_seq: req.seq,
            success: true,
            command: req.command,
            body: Some(json!({
                "allThreadsContinued": true,
            })),
            ..Default::default()
        })
    }
}
```

## Integration Points

### 1. REPL `.locals` Command
```rust
// llmspell-repl/src/session.rs
async fn handle_locals_command(&mut self) -> Result<ReplResponse> {
    // Create DAP variables request
    let dap_request = json!({
        "seq": 1,
        "type": "request",
        "command": "variables",
        "arguments": {
            "variablesReference": 1000,  // Current frame
        }
    });
    
    let response = self.kernel.send_debug_command(dap_request).await?;
    let variables = response["body"]["variables"].as_array()
        .ok_or_else(|| anyhow!("Invalid variables response"))?;
    
    // Format for display
    let mut output = String::from("Local variables:\n");
    for var in variables {
        writeln!(output, "  {} = {} ({})", 
            var["name"], var["value"], var["type"])?;
    }
    
    Ok(ReplResponse::Info(output))
}
```

### 2. CLI `debug` Command
```rust
// llmspell-cli/src/cli.rs
Debug {
    script: PathBuf,
    #[arg(long)]
    break_at: Vec<String>,  // file:line format
    #[arg(long)]
    watch: Vec<String>,     // expressions to watch
    #[arg(long)]
    port: Option<u16>,      // DAP server port (for IDE attach)
    #[arg(last = true)]
    args: Vec<String>,
}

// llmspell-cli/src/commands/debug.rs
pub async fn handle_debug_command(cmd: DebugCommand) -> Result<()> {
    // Start kernel with DAP bridge enabled
    let mut kernel = ZmqKernelClient::new_with_dap(config).await?;
    
    // Set initial breakpoints
    for bp in cmd.break_at {
        let parts: Vec<_> = bp.split(':').collect();
        kernel.set_breakpoint(parts[0], parts[1].parse()?).await?;
    }
    
    // If port specified, start DAP server for IDE attachment
    if let Some(port) = cmd.port {
        kernel.start_dap_server(port).await?;
        println!("DAP server listening on port {}", port);
    }
    
    // Start interactive debug session
    kernel.launch_debug(cmd.script, cmd.args).await?;
    
    // Enter debug REPL
    debug_repl(kernel).await
}
```

### 3. Jupyter Integration
```rust
// llmspell-kernel/src/kernel.rs
impl<T: Transport, P: Protocol> GenericKernel<T, P> {
    pub async fn handle_debug_request(&self, request: Value) -> Result<Value> {
        // Jupyter passes DAP payloads in debug_request messages
        // Route directly to DAP bridge
        self.dap_bridge.handle_request(request).await
    }
}
```

### 4. VS Code Integration
```json
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "llmspell",
            "request": "launch",
            "name": "Debug Lua Script",
            "program": "${file}",
            "debugServer": 9555,  // DAP port
            "stopOnEntry": false
        }
    ]
}
```

## Implementation Phases

### Phase 1: Wire Existing Infrastructure (1 hour)
**Goal**: Fix immediate `.locals` command

1. Connect `handle_locals_command()` to ExecutionManager
2. Format and display variables
3. Test with existing Lua scripts

**Deliverables**:
- [ ] `.locals` command works in REPL
- [ ] Variables displayed with name, value, type

### Phase 2: DAP Bridge Core (3 hours)
**Goal**: Implement minimal DAP adapter

1. Create `dap_bridge.rs` with 10 essential commands
2. Wire to ExecutionManager operations
3. Add to GenericKernel

**Deliverables**:
- [ ] DAP bridge handles core commands
- [ ] ExecutionManager operations mapped
- [ ] Unit tests for command translation

### Phase 3: CLI Debug Command (2 hours)
**Goal**: Add standalone debug command

1. Add `debug` command to CLI
2. Parse breakpoint specifications
3. Launch debug session with DAP
4. Interactive debug REPL

**Deliverables**:
- [ ] `llmspell debug script.lua --break main.lua:10` works
- [ ] Interactive stepping (continue, next, step)
- [ ] Variable inspection during pause

### Phase 4: IDE Integration (2 hours)
**Goal**: Enable VS Code debugging

1. Start DAP server on specified port
2. Handle IDE attachment
3. Create VS Code extension config
4. Test with VS Code

**Deliverables**:
- [ ] VS Code can attach to debug session
- [ ] Breakpoints set from VS Code work
- [ ] Variables visible in VS Code UI

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_dap_bridge_initialize() {
    let bridge = DAPBridge::new(execution_manager);
    let response = bridge.handle_request(json!({
        "type": "request",
        "command": "initialize",
    })).await.unwrap();
    
    assert!(response["success"].as_bool().unwrap());
    assert!(response["body"]["supportsConditionalBreakpoints"].as_bool().unwrap());
}

#[tokio::test]
async fn test_locals_command() {
    let mut repl = create_test_repl().await;
    repl.execute("local x = 42; local y = 'hello'").await;
    
    let response = repl.handle_command(".locals").await.unwrap();
    assert!(response.contains("x = 42"));
    assert!(response.contains("y = hello"));
}
```

### Integration Tests
```bash
# Test REPL locals
echo "local x = 42; .locals" | llmspell repl

# Test debug command
llmspell debug test.lua --break test.lua:5

# Test VS Code attachment
llmspell debug test.lua --port 9555 &
code --open-url "vscode://debug/attach?port=9555"
```

### End-to-End Scenarios
1. **REPL Debugging**: Set breakpoint, run code, inspect locals
2. **CLI Debugging**: Debug script with breakpoints and stepping
3. **IDE Debugging**: Full VS Code debugging experience
4. **Jupyter Debugging**: Debug cells in Jupyter notebook

## Benefits of This Approach

### Immediate Benefits
- ✅ `.locals` command works (Phase 1 - 1 hour)
- ✅ Standalone debug command (Phase 3 - 2 hours)
- ✅ Leverages existing ExecutionManager
- ✅ Minimal code addition (~500 lines)

### Medium-term Benefits
- ✅ VS Code debugging support (Phase 4)
- ✅ Jupyter notebook debugging
- ✅ Consistent debug experience across interfaces
- ✅ Future-proof DAP foundation

### Long-term Benefits
- ✅ Remote debugging capability
- ✅ Multi-language debug support (JS, Python via DAP)
- ✅ Debug protocol extensibility
- ✅ IDE ecosystem compatibility

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| `.locals` functionality | 100% working | REPL test suite |
| Debug command availability | Implemented | CLI test suite |
| DAP command coverage | 10 commands | Unit tests |
| VS Code integration | Breakpoints work | Manual testing |
| Performance overhead | <5ms per command | Benchmarks |
| Code addition | <1000 lines | Line count |

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| DAP spec complexity | Implement only essential subset |
| ExecutionManager gaps | Fill gaps as discovered, existing code works |
| Performance overhead | Cache variables, batch operations |
| IDE compatibility | Test with VS Code first, others later |

## Alternatives Considered

### Why Not Full DAP?
- 50+ commands vs 10 needed
- Weeks of implementation
- Most commands irrelevant for scripting

### Why Not Direct Wiring?
- No IDE support
- Technical debt
- Multiple code paths

### Why Not Custom Protocol?
- Reinventing wheel
- No tool support
- Maintenance burden

## Conclusion

The hybrid DAP bridge architecture provides the optimal balance of:
- **Immediate fixes** for broken functionality
- **IDE integration** via standard protocol
- **Minimal implementation** effort
- **Future extensibility** for additional features

This approach leverages our existing ExecutionManager infrastructure while providing a standard DAP interface that tools understand, solving all three identified problems with ~8 hours of implementation.

---

**Decision**: Implement hybrid DAP bridge architecture in 4 phases over 8 hours total.