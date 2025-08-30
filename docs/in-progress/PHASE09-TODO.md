# Phase 9: Interactive REPL and Debugging Infrastructure - TODO List

**Version**: 2.0  
**Date**: January 2025  
**Status**: Implementation Ready  
**Phase**: 9 (Interactive REPL and Debugging Infrastructure)  
**Timeline**: Weeks 30-32 (15 working days)  
**Priority**: HIGH (Developer Experience - Critical for adoption)  
**Dependencies**: Phase 8 Vector Storage ✅  
**Arch-Document**: docs/technical/master-architecture-vision.md  
**All-Phases-Document**: docs/in-progress/implementation-phases.md  
**Design-Document**: docs/in-progress/phase-09-design-doc.md ✅  
**Debug-Architecture**: docs/technical/operational-guide.md (debug material to be updated/created)  
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE09-TODO.md)

> **📋 Actionable Task List**: This document breaks down Phase 9 implementation into specific, measurable tasks for building a kernel-as-service REPL with integrated debugging capabilities following Jupyter's proven multi-client architecture.

---

## Overview

**Goal**: Implement a **REPL kernel service** following Jupyter's multi-client architecture, where a single LLMSpell kernel serves CLI terminals, web interfaces, and IDE debuggers simultaneously through standardized message protocols (LRP/LDP).

**Success Criteria Summary:**
- [ ] Kernel service starts as standalone process in <100ms
- [ ] Multiple clients (CLI, web, IDE) connect to same kernel
- [ ] LRP/LDP protocols enable message-based communication
- [ ] Connection discovery via JSON files works
- [ ] State persists via Phase 5 state management
- [ ] Conditional breakpoints with hit/ignore counts work
- [ ] Step debugging with async context preservation works
- [ ] Variables inspected with lazy expansion
- [ ] Hot reload preserves state across file changes
- [ ] Script validation with error pattern database
- [ ] Circuit breaker monitoring in hook introspection
- [ ] Distributed tracing with OpenTelemetry
- [ ] Performance profiling with flamegraph generation
- [ ] Session recording/replay with interactive stepping
- [ ] Command history with Ctrl+R search
- [ ] Media/streaming support in protocols
- [ ] LSP/DAP protocol implementations
- [ ] VS Code extension with debugging
- [ ] Remote debugging with security
- [ ] All tests pass with >90% coverage
- [ ] Documentation complete with tutorials

---

## Phase 9.1: Kernel Service Foundation (Days 1-3)

### Task 9.1.1: Create llmspell-repl Crate Structure
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Kernel Team Lead

**Description**: Create the `llmspell-repl` crate with kernel service architecture following Jupyter's model.

**Acceptance Criteria:**
- [ ] `llmspell-repl/` crate created with proper structure
- [ ] Dependencies added: `tokio`, `serde`, `serde_json`, `uuid`, `zmq` alternatives
- [ ] Kernel service module structure established
- [ ] Five channel architecture defined (Shell, IOPub, Stdin, Control, Heartbeat)
- [ ] `cargo check -p llmspell-repl` passes

**Implementation Steps:**
1. Create `llmspell-repl/` crate:
   ```bash
   cargo new --lib llmspell-repl
   cd llmspell-repl
   ```
2. Add dependencies to `Cargo.toml`:
   ```toml
   [dependencies]
   tokio = { version = "1", features = ["full"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   uuid = { version = "1", features = ["v4", "serde"] }
   llmspell-bridge = { path = "../llmspell-bridge" }
   llmspell-debug = { path = "../llmspell-debug" }
   async-trait = "0.1"
   tracing = "0.1"
   ```
3. Create module structure:
   ```rust
   pub mod kernel;      // Core kernel service
   pub mod channels;    // Five communication channels
   pub mod protocol;    // LRP/LDP protocol definitions
   pub mod connection;  // Connection management
   pub mod client;      // Client connection handling
   pub mod discovery;   // Connection file discovery
   pub mod security;    // Authentication and authorization
   ```
4. Define kernel service struct in `kernel.rs`
5. Verify compilation

**Definition of Done:**
- [ ] Crate structure compiles without errors
- [ ] All submodules have basic structure
- [ ] Dependencies resolve correctly
- [ ] No clippy warnings

### Task 9.1.2: Implement LLMSpell Kernel Service
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: Kernel Team

**Description**: Build the core kernel service that wraps `llmspell-bridge` ScriptRuntime.

**Acceptance Criteria:**
- [ ] `LLMSpellKernel` struct implemented
- [ ] Kernel lifecycle (start, run, shutdown) works
- [ ] Wraps existing ScriptRuntime from bridge
- [ ] Multi-client management implemented
- [ ] Resource isolation per client
- [ ] Kernel process can run standalone

**Implementation Steps:**
1. Implement `LLMSpellKernel` struct:
   ```rust
   pub struct LLMSpellKernel {
       kernel_id: String,
       runtime: Arc<ScriptRuntime>,
       clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
       channels: KernelChannels,
       execution_state: Arc<RwLock<KernelState>>,
       debugger: Arc<Debugger>,
       profiler: Arc<PerformanceProfiler>,
       tracer: Arc<DistributedTracer>,
   }
   ```
2. Implement kernel lifecycle methods:
   ```rust
   impl LLMSpellKernel {
       pub async fn start(config: KernelConfig) -> Result<Self> { ... }
       pub async fn run(&mut self) -> Result<()> { ... }
       pub async fn shutdown(self) -> Result<()> { ... }
   }
   ```
3. Integrate with existing bridge runtime
4. Add multi-client connection handling
5. Implement resource limits per client
6. Test standalone kernel process

**Definition of Done:**
- [ ] Kernel starts and runs as standalone process
- [ ] Can wrap existing ScriptRuntime
- [ ] Handles multiple client connections
- [ ] Clean shutdown implemented
- [ ] Resource isolation works

### Task 9.1.3: Bridge-Kernel Debug Integration
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Kernel Team

**Description**: Make `llmspell-bridge::ScriptRuntime` debug-aware to support kernel debugging capabilities.

**Acceptance Criteria:**
- [ ] ScriptRuntime accepts debugger instance
- [ ] Breakpoint propagation to engine works
- [ ] Debug hooks installable in Lua engine
- [ ] Variable extraction interface implemented
- [ ] Execution control (pause/resume) works
- [ ] Debug state synchronization functional

**Implementation Steps:**
1. Extend ScriptRuntime with debug interface:
   ```rust
   // llmspell-bridge/src/runtime.rs
   impl ScriptRuntime {
       pub fn set_debugger(&mut self, debugger: Arc<Debugger>) {
           self.engine.set_debugger(debugger);
       }
       
       pub fn set_breakpoints(&mut self, breakpoints: Vec<Breakpoint>) {
           self.engine.set_breakpoints(breakpoints);
       }
       
       pub async fn get_debug_state(&self) -> DebugState {
           self.engine.get_debug_state().await
       }
   }
   ```
2. Add debug support to ScriptEngineBridge trait:
   ```rust
   #[async_trait]
   trait ScriptEngineBridge {
       fn set_debugger(&mut self, debugger: Arc<Debugger>);
       fn set_breakpoints(&mut self, breakpoints: Vec<Breakpoint>);
       async fn get_locals(&self, frame: usize) -> HashMap<String, Value>;
   }
   ```
3. Implement debug interface in LuaEngine
4. Create DebugState structure for state transfer
5. Add execution control methods (pause, resume, step)
6. Test debug integration with simple script

**Definition of Done:**
- [ ] Bridge accepts debugger configuration
- [ ] Breakpoints propagate to engine
- [ ] Debug state retrievable
- [ ] Execution control works
- [ ] Tests pass

### Task 9.1.4: Five Channel Architecture
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Kernel Team

**Description**: Implement Jupyter-style five channel communication system.

**Acceptance Criteria:**
- [ ] Shell channel (request-reply) implemented
- [ ] IOPub channel (pub-sub) implemented
- [ ] Stdin channel (input requests) implemented
- [ ] Control channel (interrupts) implemented
- [ ] Heartbeat channel (keep-alive) implemented
- [ ] Message routing between channels works
- [ ] TCP socket transport functional

**Implementation Steps:**
1. Create channel abstractions in `channels.rs`:
   ```rust
   pub struct ShellChannel { ... }    // Request-reply execution
   pub struct IOPubChannel { ... }    // Broadcast output
   pub struct StdinChannel { ... }    // Input requests
   pub struct ControlChannel { ... }  // Kernel control
   pub struct HeartbeatChannel { ... } // Keep-alive monitoring
   ```
2. Implement TCP socket transport for each channel
3. Create message routing infrastructure
4. Add channel multiplexing
5. Implement heartbeat monitoring
6. Test multi-channel communication

**Definition of Done:**
- [ ] All five channels operational
- [ ] Message routing works correctly
- [ ] TCP transport functional
- [ ] Heartbeat detects disconnections

### Task 9.1.5: Connection Discovery System
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Kernel Team

**Description**: Implement JSON connection file discovery for client-kernel connection.

**Acceptance Criteria:**
- [ ] Connection file generation on kernel start
- [ ] JSON format with all connection details
- [ ] File placed in standard location
- [ ] Client can discover and parse file
- [ ] Authentication keys included
- [ ] Connection cleanup on shutdown

**Implementation Steps:**
1. Define connection info structure:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct ConnectionInfo {
       kernel_id: String,
       transport: String,
       ip: String,
       shell_port: u16,
       iopub_port: u16,
       stdin_port: u16,
       control_port: u16,
       hb_port: u16,
       key: String,
       signature_scheme: String,
   }
   ```
2. Generate connection file on kernel start
3. Place in `~/.llmspell/kernels/` or temp directory
4. Implement client discovery mechanism
5. Add authentication key generation
6. Clean up file on kernel shutdown

**Definition of Done:**
- [ ] Connection file generated correctly
- [ ] Clients can discover kernel
- [ ] Authentication works
- [ ] File cleanup on shutdown

### Task 9.1.6: LRP/LDP Protocol Implementation
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: Protocol Team

**Description**: Define and implement LLMSpell REPL Protocol (LRP) and Debug Protocol (LDP).

**Acceptance Criteria:**
- [ ] LRP message types defined (Execute, Complete, Inspect, etc.)
- [ ] LDP message types defined (SetBreakpoint, Step, Continue, etc.)
- [ ] JSON-RPC 2.0 compatible format
- [ ] Protocol validation implemented
- [ ] Error responses standardized
- [ ] Media message support included

**Implementation Steps:**
1. Define LRP messages in `protocol/lrp.rs`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "msg_type")]
   pub enum LRPRequest {
       ExecuteRequest { ... },
       CompleteRequest { ... },
       InspectRequest { ... },
       HistoryRequest { ... },
   }
   ```
2. Define LDP messages in `protocol/ldp.rs`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "msg_type")]
   pub enum LDPRequest {
       SetBreakpointRequest { ... },
       StepRequest { ... },
       ContinueRequest { ... },
       VariablesRequest { ... },
   }
   ```
3. Implement protocol validation
4. Add media message support
5. Create protocol documentation
6. Test protocol compliance

**Definition of Done:**
- [ ] All protocol messages defined
- [ ] JSON-RPC format validated
- [ ] Media messages supported
- [ ] Protocol documentation complete

### Task 9.1.7: Section 9.1 Testing and Integration
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing of kernel service foundation.

**Acceptance Criteria:**
- [ ] Unit tests for kernel lifecycle
- [ ] Integration tests for multi-client
- [ ] Protocol compliance tests
- [ ] Performance benchmarks (<100ms startup)
- [ ] Zero clippy warnings
- [ ] Documentation complete

**Implementation Steps:**
1. Write kernel lifecycle tests
2. Test multi-client connections
3. Verify protocol message handling
4. Benchmark kernel startup time
5. Run clippy and fix warnings
6. Document all public APIs

**Definition of Done:**
- [ ] All tests pass
- [ ] <100ms kernel startup verified
- [ ] No clippy warnings
- [ ] API documentation complete

---

## Phase 9.2: Enhanced Debugging Infrastructure (Days 4-6)

### Task 9.2.1: Interactive Debugger Implementation
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: Debug Team Lead

**Description**: Implement comprehensive interactive debugging with conditional breakpoints.

**Acceptance Criteria:**
- [ ] Breakpoint system with conditions implemented
- [ ] Hit counts and ignore counts work
- [ ] Step debugging (step, next, continue) functional
- [ ] Call stack navigation (up/down) works
- [ ] Breakpoint persistence across sessions
- [ ] Enable/disable without removal

**Implementation Steps:**
1. Create `llmspell-debug` crate
2. Implement `ConditionalBreakpoint` struct:
   ```rust
   pub struct ConditionalBreakpoint {
       line: u32,
       condition: Option<String>,
       hit_count: u32,
       ignore_count: u32,
       current_hits: u32,
       enabled: bool,
   }
   ```
3. Build `Debugger` with breakpoint management
4. Implement step controller for execution flow
5. Add call stack navigation
6. Test debugging workflow

**Definition of Done:**
- [ ] Conditional breakpoints work
- [ ] Step debugging functional
- [ ] Call stack navigation works
- [ ] Breakpoints persist

### Task 9.2.2: Debug Session Management
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team

**Description**: Implement debug session management for handling multiple debug clients and session state.

**Acceptance Criteria:**
- [ ] Debug sessions created per client
- [ ] Session state maintained correctly
- [ ] Debug commands routed to right session
- [ ] Multiple clients can debug different scripts
- [ ] Session cleanup on disconnect
- [ ] Session persistence across reconnects

**Implementation Steps:**
1. Create debug session manager:
   ```rust
   // llmspell-debug/src/session_manager.rs
   pub struct DebugSessionManager {
       sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
       kernel_debugger: Arc<Debugger>,
   }
   
   pub struct DebugSession {
       session_id: String,
       client_id: String,
       script_path: Option<PathBuf>,
       execution_state: ExecutionState,
       current_frame: usize,
       breakpoints: Vec<ConditionalBreakpoint>,
       watch_expressions: Vec<String>,
       created_at: SystemTime,
   }
   
   impl DebugSessionManager {
       pub async fn create_session(&mut self, client_id: String) -> String {
           let session = DebugSession {
               session_id: Uuid::new_v4().to_string(),
               client_id,
               script_path: None,
               execution_state: ExecutionState::Running,
               current_frame: 0,
               breakpoints: Vec::new(),
               watch_expressions: Vec::new(),
               created_at: SystemTime::now(),
           };
           
           let session_id = session.session_id.clone();
           self.sessions.write().await.insert(session_id.clone(), session);
           session_id
       }
       
       pub async fn handle_debug_command(
           &mut self,
           session_id: &str,
           command: DebugCommand,
       ) -> Result<DebugResponse> {
           let sessions = self.sessions.read().await;
           let session = sessions.get(session_id)
               .ok_or_else(|| anyhow!("Session not found"))?;
           
           match command {
               DebugCommand::Step => self.handle_step(session).await,
               DebugCommand::Continue => self.handle_continue(session).await,
               DebugCommand::SetBreakpoint(bp) => self.handle_set_breakpoint(session, bp).await,
               DebugCommand::Inspect(var) => self.handle_inspect(session, var).await,
               // ... other commands
           }
       }
   }
   ```
2. Implement session routing in kernel
3. Add session persistence mechanism
4. Handle concurrent debug sessions
5. Implement session timeout and cleanup
6. Test with multiple simultaneous clients

**Definition of Done:**
- [ ] Sessions created correctly
- [ ] Commands routed properly
- [ ] Multi-client debugging works
- [ ] Session cleanup functional
- [ ] Tests pass

### Task 9.2.3: Lua Debug Hooks Implementation
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: Debug Team

**Description**: Implement actual Lua debug hooks to enable breakpoint functionality and stepping.

**Acceptance Criteria:**
- [ ] Lua debug hooks installed correctly
- [ ] Line-by-line execution tracking works
- [ ] Function call/return tracking functional
- [ ] Breakpoint checking at each line
- [ ] Debug session suspension works
- [ ] Context switching between Lua and debugger

**Implementation Steps:**
1. Create debug hooks module:
   ```rust
   // llmspell-bridge/src/lua/debug_hooks.rs
   use mlua::{Lua, Debug, HookTriggers};
   
   pub fn install_debug_hooks(lua: &Lua, debugger: Arc<Debugger>) {
       lua.set_hook(HookTriggers {
           every_line: true,
           on_calls: true,
           on_returns: true,
       }, move |lua, debug| {
           // Check breakpoints
           let info = debug.curr_line();
           if debugger.has_breakpoint(info.source, info.line) {
               // Evaluate breakpoint condition
               if debugger.should_break(lua, info) {
                   // Enter debug session
                   block_on(debugger.on_breakpoint_hit(lua, info));
               }
           }
           Ok(())
       });
   }
   ```
2. Implement breakpoint checking logic:
   ```rust
   impl Debugger {
       fn has_breakpoint(&self, source: &str, line: u32) -> bool {
           self.breakpoints.read()
               .get(source)
               .and_then(|lines| lines.get(&line))
               .is_some()
       }
       
       fn should_break(&self, lua: &Lua, info: DebugInfo) -> bool {
           // Check hit counts, conditions, etc.
       }
   }
   ```
3. Create debug session suspension mechanism
4. Handle async boundary crossing (block_on for debug)
5. Implement hook removal on debug disable
6. Test with various script scenarios

**Definition of Done:**
- [ ] Hooks trigger on every line
- [ ] Breakpoints stop execution
- [ ] Debug context preserved
- [ ] Performance impact <10%
- [ ] Tests pass

### Task 9.2.4: Breakpoint Condition Evaluator
**Priority**: CRITICAL  
**Estimated Time**: 5 hours  
**Assignee**: Debug Team

**Description**: Implement breakpoint condition evaluation in Lua context with hit counts and ignore counts.

**Acceptance Criteria:**
- [ ] Conditions evaluated in Lua context
- [ ] Hit counts tracked correctly
- [ ] Ignore counts work as expected
- [ ] Complex conditions supported
- [ ] Error handling for bad conditions
- [ ] Performance impact minimal

**Implementation Steps:**
1. Enhance ConditionalBreakpoint implementation:
   ```rust
   // llmspell-debug/src/breakpoint_evaluator.rs
   impl ConditionalBreakpoint {
       pub fn should_break(&mut self, lua: &Lua) -> Result<bool> {
           // Update hit counter
           self.current_hits += 1;
           
           // Check if still in ignore range
           if self.current_hits <= self.ignore_count {
               return Ok(false);
           }
           
           // Check if hit count reached
           if self.hit_count > 0 && self.current_hits < self.hit_count {
               return Ok(false);
           }
           
           // Check if enabled
           if !self.enabled {
               return Ok(false);
           }
           
           // Evaluate condition in Lua context
           if let Some(condition) = &self.condition {
               match self.evaluate_condition(lua, condition) {
                   Ok(result) => Ok(result),
                   Err(e) => {
                       // Log error but break anyway for safety
                       eprintln!("Breakpoint condition error: {}", e);
                       Ok(true)
                   }
               }
           } else {
               Ok(true) // No condition means always break
           }
       }
       
       fn evaluate_condition(&self, lua: &Lua, condition: &str) -> Result<bool> {
           // Create safe evaluation environment
           let env = lua.create_table()?;
           
           // Copy local variables to environment
           self.copy_locals_to_env(lua, &env)?;
           
           // Evaluate condition as Lua expression
           let chunk = lua.load(condition)
               .set_environment(env)?;
           
           chunk.eval::<bool>()
               .map_err(|e| anyhow!("Condition evaluation failed: {}", e))
       }
       
       fn copy_locals_to_env(&self, lua: &Lua, env: &Table) -> Result<()> {
           // Extract local variables from current scope
           // and make them available for condition evaluation
           lua.inspect_stack(|debug| {
               for i in 1.. {
                   match debug.name(i) {
                       Some((name, value)) => {
                           env.set(name, value)?;
                       }
                       None => break,
                   }
               }
               Ok(())
           })
       }
   }
   ```
2. Add condition validation on breakpoint creation
3. Implement hit count reset mechanism
4. Add conditional breakpoint templates
5. Create condition debugging helpers
6. Test with complex conditions

**Definition of Done:**
- [ ] Conditions evaluate correctly
- [ ] Hit/ignore counts work
- [ ] Complex expressions supported
- [ ] Errors handled gracefully
- [ ] Tests pass

### Task 9.2.5: Debug State Bridge
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team

**Description**: Implement debug state synchronization between kernel debugger and Lua runtime.

**Acceptance Criteria:**
- [ ] Kernel debugger state propagates to Lua
- [ ] Lua debug state extractable to kernel
- [ ] Breakpoint synchronization works
- [ ] Variable state transferable
- [ ] Execution control synchronized
- [ ] State updates real-time

**Implementation Steps:**
1. Create debug state bridge:
   ```rust
   // llmspell-debug/src/state_bridge.rs
   pub struct DebugStateBridge {
       kernel_debugger: Arc<Debugger>,
       lua_debug_state: Arc<RwLock<LuaDebugState>>,
       sync_channel: mpsc::Sender<DebugStateUpdate>,
   }
   
   impl DebugStateBridge {
       async fn sync_breakpoints(&self) {
           let breakpoints = self.kernel_debugger.get_breakpoints().await;
           self.lua_debug_state.write().await.update_breakpoints(breakpoints);
       }
       
       async fn sync_variables(&self, lua: &Lua) {
           let locals = extract_lua_locals(lua);
           self.kernel_debugger.update_variables(locals).await;
       }
       
       async fn sync_execution_state(&self, state: ExecutionState) {
           self.kernel_debugger.set_execution_state(state).await;
           self.lua_debug_state.write().await.execution_state = state;
       }
   }
   ```
2. Implement bidirectional state transfer
3. Create real-time sync mechanism
4. Handle state conflicts
5. Add state versioning for consistency
6. Test with concurrent debug operations

**Definition of Done:**
- [ ] States synchronized correctly
- [ ] Real-time updates work
- [ ] No state conflicts
- [ ] Performance acceptable
- [ ] Tests pass

### Task 9.2.6: Variable Inspection System
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team

**Description**: Deep variable inspection with lazy expansion for complex structures.

**Acceptance Criteria:**
- [ ] Variable inspection at any scope
- [ ] Lazy expansion for large structures
- [ ] Table inspection with truncation
- [ ] Function and userdata inspection
- [ ] Depth limits enforced
- [ ] Watch expressions work

**Implementation Steps:**
1. Implement `VariableInspector`:
   ```rust
   pub struct VariableInspector {
       max_depth: usize,
       max_items_per_level: usize,
   }
   ```
2. Create inspection tree with lazy loading
3. Handle different Lua value types
4. Implement expansion API
5. Add watch expression support
6. Test with complex data structures

**Definition of Done:**
- [ ] Variables inspected correctly
- [ ] Lazy expansion works
- [ ] Large structures handled
- [ ] Watch expressions functional

### Task 9.2.7: Enhanced Error Reporting
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team

**Description**: Rust-quality error messages with pattern database and suggestions.

**Acceptance Criteria:**
- [ ] Rust-style error formatting
- [ ] Source context with highlighting
- [ ] Error pattern database functional
- [ ] Intelligent suggestions provided
- [ ] Similar variable/function detection
- [ ] Related documentation links

**Implementation Steps:**
1. Implement `ErrorEnhancer` with pattern database:
   ```rust
   pub struct ErrorEnhancer {
       suggestion_rules: Vec<SuggestionRule>,
       lua_patterns: HashMap<String, ErrorPattern>,
       error_pattern_database: ErrorPatternDatabase,
   }
   ```
2. Build comprehensive error patterns:
   - "attempt to index nil" patterns
   - "attempt to call nil" patterns
   - "bad argument" patterns
   - Stack overflow detection
3. Implement fuzzy matching for typos
4. Add API signature validation
5. Generate actionable suggestions
6. Test with common errors

**Definition of Done:**
- [ ] Rust-style formatting works
- [ ] Pattern database comprehensive
- [ ] Suggestions are actionable
- [ ] Documentation links provided

### Task 9.2.8: Async/Await Context Preservation
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: Debug Team

**Description**: Complete context preservation across async boundaries.

**Acceptance Criteria:**
- [ ] AsyncExecutionContext captures full state
- [ ] Lua stack preserved at async points
- [ ] Rust stack correlation works
- [ ] Panic hook captures context
- [ ] Timeout handling with context
- [ ] Nested async calls tracked

**Implementation Steps:**
1. Implement `AsyncExecutionContext`:
   ```rust
   pub struct AsyncExecutionContext {
       lua_stack: Vec<LuaStackFrame>,
       rust_stack: Vec<RustStackFrame>,
       correlation_id: Uuid,
       events: Vec<DebugEvent>,
       parent_context: Option<Box<AsyncExecutionContext>>,
   }
   ```
2. Enhanced block_on with context
3. Install panic hook for context capture
4. Track correlation IDs
5. Handle nested async calls
6. Test with complex async scenarios

**Definition of Done:**
- [ ] Full context preserved
- [ ] Panic context captured
- [ ] Correlation tracking works
- [ ] Nested calls handled

### Task 9.2.9: AsyncExecutionContext Integration Points
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team

**Description**: Integrate AsyncExecutionContext into the Lua engine execution path for complete async debugging.

**Acceptance Criteria:**
- [ ] Context created for async operations
- [ ] Lua engine uses context for execution
- [ ] Context available in debug hooks
- [ ] Correlation IDs flow through system
- [ ] Panic recovery preserves context
- [ ] Performance overhead minimal

**Implementation Steps:**
1. Modify LuaEngine to use AsyncExecutionContext:
   ```rust
   // llmspell-bridge/src/lua/engine.rs
   impl LuaEngine {
       async fn execute_with_debug(&self, script: &str) -> Result<ScriptOutput> {
           let correlation_id = Uuid::new_v4();
           let mut context = AsyncExecutionContext::new(self, correlation_id)?;
           
           // Wrap execution with context
           context.execute_with_context(&self.lua, async {
               // Install debug hooks with context
               if let Some(debugger) = &self.debugger {
                   install_debug_hooks_with_context(&self.lua, debugger, &context);
               }
               
               // Execute script with async support
               self.lua.load(script).exec_async().await
           }).await
       }
   }
   ```
2. Pass context through debug hooks:
   ```rust
   fn install_debug_hooks_with_context(
       lua: &Lua,
       debugger: Arc<Debugger>,
       context: &AsyncExecutionContext,
   ) {
       let ctx_clone = context.clone();
       lua.set_hook(HookTriggers::default(), move |lua, debug| {
           // Context available in hooks
           debugger.on_hook_with_context(lua, debug, &ctx_clone)
       });
   }
   ```
3. Add context to tool invocations
4. Propagate context through agent calls
5. Ensure context survives async boundaries
6. Test with complex async workflows

**Definition of Done:**
- [ ] Context integrated in engine
- [ ] Available in all debug points
- [ ] Correlation IDs work
- [ ] Async boundaries handled
- [ ] Tests pass

### Task 9.2.10: Distributed Tracing Integration
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team

**Description**: OpenTelemetry-based distributed tracing for production observability.

**Acceptance Criteria:**
- [ ] OpenTelemetry integration complete
- [ ] Script execution traced
- [ ] Tool invocations traced
- [ ] Agent executions traced
- [ ] Breakpoint hits traced
- [ ] OTLP exporter configured

**Implementation Steps:**
1. Add OpenTelemetry dependencies
2. Implement `DistributedTracer`:
   ```rust
   pub struct DistributedTracer {
       tracer: Box<dyn Tracer>,
       kernel_id: String,
   }
   ```
3. Instrument script execution
4. Trace tool and agent calls
5. Configure OTLP exporter
6. Test with Jaeger backend

**Definition of Done:**
- [ ] Tracing integrated
- [ ] All operations traced
- [ ] Exports to Jaeger work
- [ ] Performance overhead <5%

### Task 9.2.11: Section 9.2 Testing
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing of debugging infrastructure.

**Acceptance Criteria:**
- [ ] Debugger integration tests pass
- [ ] Error enhancement validated
- [ ] Async context preservation verified
- [ ] Tracing overhead measured
- [ ] Zero clippy warnings
- [ ] Documentation complete

**Implementation Steps:**
1. Write debugging workflow tests
2. Test error pattern matching
3. Verify async context tracking
4. Measure tracing performance
5. Run quality checks
6. Document debugging APIs

**Definition of Done:**
- [ ] All tests pass
- [ ] Performance targets met
- [ ] Quality checks pass
- [ ] Documentation complete

---

## Phase 9.3: Development Experience Features (Days 7-9)

### Task 9.3.1: Hot Reload System
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: DevEx Team Lead

**Description**: File watching and hot reload with state preservation.

**Acceptance Criteria:**
- [ ] File watcher detects changes
- [ ] State preserved across reloads
- [ ] Validation before reload
- [ ] Error recovery without session loss
- [ ] Debouncing for rapid changes
- [ ] Multiple file watching

**Implementation Steps:**
1. Implement `HotReloadManager`:
   ```rust
   pub struct HotReloadManager {
       watcher: notify::RecommendedWatcher,
       state_snapshots: Arc<RwLock<HashMap<PathBuf, StateSnapshot>>>,
       validator: ScriptValidator,
       strategy: ReloadStrategy,
   }
   ```
2. Set up file watching with notify
3. Create state snapshot system
4. Implement reload strategies
5. Add validation checks
6. Test with rapid file changes

**Definition of Done:**
- [ ] File changes detected
- [ ] State preserved on reload
- [ ] Validation prevents bad reloads
- [ ] <500ms reload time

### Task 9.3.2: Script Validation System
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: DevEx Team

**Description**: Comprehensive script validation with performance and security checks.

**Acceptance Criteria:**
- [ ] Syntax validation complete
- [ ] API usage validation works
- [ ] Security patterns detected
- [ ] Performance anti-patterns found
- [ ] Style suggestions provided
- [ ] Validation report generated

**Implementation Steps:**
1. Implement `ScriptValidator`:
   ```rust
   pub struct ScriptValidator {
       lua_checker: Lua,
       api_definitions: ApiDefinitions,
       syntax_patterns: Vec<SyntaxPattern>,
       security_rules: Vec<SecurityRule>,
   }
   ```
2. Build syntax checker
3. Add API usage validation
4. Implement security rules
5. Detect performance issues
6. Generate comprehensive reports

**Definition of Done:**
- [ ] Validation comprehensive
- [ ] All check types work
- [ ] Reports actionable
- [ ] Performance acceptable

### Task 9.3.3: Performance Profiling
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: DevEx Team

**Description**: CPU and memory profiling with flamegraph generation.

**Acceptance Criteria:**
- [ ] CPU profiling with pprof
- [ ] Flamegraph generation works
- [ ] Memory tracking functional
- [ ] Execution time analysis
- [ ] Leak detection implemented
- [ ] Profile export formats

**Implementation Steps:**
1. Implement `PerformanceProfiler`:
   ```rust
   pub struct PerformanceProfiler {
       cpu_profiler: Option<ProfilerGuard>,
       memory_tracker: MemoryTracker,
       execution_times: HashMap<String, Vec<Duration>>,
   }
   ```
2. Integrate pprof for CPU profiling
3. Generate flamegraphs
4. Track memory allocations
5. Detect potential leaks
6. Export multiple formats

**Definition of Done:**
- [ ] Profiling functional
- [ ] Flamegraphs generated
- [ ] Memory leaks detected
- [ ] Multiple export formats

### Task 9.3.4: Performance Profiler Hooks
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: DevEx Team

**Description**: Integrate performance profiler with Lua execution hooks for accurate profiling.

**Acceptance Criteria:**
- [ ] Profiler hooks into Lua VM
- [ ] Stack sampling works correctly
- [ ] Function timing accurate
- [ ] Memory allocation tracked
- [ ] Minimal performance overhead
- [ ] Profiling toggleable at runtime

**Implementation Steps:**
1. Implement profiler hooks:
   ```rust
   // llmspell-debug/src/profiler_hooks.rs
   impl PerformanceProfiler {
       pub fn install_lua_hooks(&self, lua: &Lua) {
           // Install performance sampling hooks
           lua.set_hook(HookTriggers {
               every_nth_instruction: Some(1000), // Sample every 1000 instructions
               on_calls: true,
               on_returns: true,
           }, move |lua, debug| {
               self.on_profiler_hook(lua, debug)
           });
       }
       
       fn on_profiler_hook(&self, lua: &Lua, debug: &Debug) -> Result<()> {
           // Sample current stack for CPU profiling
           if self.cpu_profiler.is_some() {
               self.sample_stack(debug)?;
           }
           
           // Track function entry/exit for timing
           match debug.event() {
               DebugEvent::Call => {
                   let func_name = debug.name().unwrap_or("<anonymous>");
                   self.function_entry(func_name, Instant::now());
               }
               DebugEvent::Return => {
                   let func_name = debug.name().unwrap_or("<anonymous>");
                   self.function_exit(func_name, Instant::now());
               }
               _ => {}
           }
           
           // Track memory if enabled
           if self.memory_tracking_enabled {
               self.sample_memory(lua)?;
           }
           
           Ok(())
       }
       
       fn sample_stack(&self, debug: &Debug) -> Result<()> {
           let mut stack = Vec::new();
           
           // Walk the call stack
           for level in 0.. {
               match debug.get_stack(level) {
                   Some(frame) => {
                       stack.push(StackFrame {
                           function: frame.name().to_string(),
                           file: frame.source().to_string(),
                           line: frame.current_line(),
                       });
                   }
                   None => break,
               }
           }
           
           // Record sample
           self.cpu_samples.lock().push(CpuSample {
               timestamp: Instant::now(),
               stack,
           });
           
           Ok(())
       }
   }
   ```
2. Add runtime toggle for profiling
3. Implement stack walking for samples
4. Create memory allocation tracking
5. Add overhead measurement
6. Test with various workloads

**Definition of Done:**
- [ ] Hooks installed correctly
- [ ] Sampling accurate
- [ ] Overhead <5%
- [ ] Runtime toggle works
- [ ] Tests pass

### Task 9.3.5: Hook Introspection & Circuit Breakers
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: DevEx Team

**Description**: Integration with Phase 4 hooks including circuit breaker monitoring.

**Acceptance Criteria:**
- [ ] Hook listing functional
- [ ] Hook details retrievable
- [ ] Execution tracing works
- [ ] Circuit breaker status visible
- [ ] Real-time monitoring active
- [ ] Performance metrics available

**Implementation Steps:**
1. Implement `HookInspector`:
   ```rust
   pub struct HookInspector {
       hook_manager: Option<Arc<HookManager>>,
       execution_traces: Arc<RwLock<Vec<HookExecutionTrace>>>,
       performance_metrics: Arc<Mutex<HookPerformanceMetrics>>,
   }
   ```
2. Connect to HookManager
3. Implement circuit breaker monitoring
4. Add real-time status updates
5. Track performance metrics
6. Test with active hooks

**Definition of Done:**
- [ ] Hooks introspectable
- [ ] Circuit breakers monitored
- [ ] Real-time updates work
- [ ] Metrics accurate

### Task 9.3.6: Session Recording/Replay
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: DevEx Team

**Description**: Complete session recording with interactive replay.

**Acceptance Criteria:**
- [ ] Sessions recorded to JSON
- [ ] All event types captured
- [ ] Interactive replay works
- [ ] Stepping through events
- [ ] Environment restoration
- [ ] Compression supported

**Implementation Steps:**
1. Enhance `SessionRecorder`:
   ```rust
   pub enum SessionEvent {
       ScriptStart { ... },
       VariableChange { ... },
       FunctionCall { ... },
       ToolInvocation { ... },
       // ... more event types
   }
   ```
2. Implement comprehensive event capture
3. Build replay system
4. Add interactive stepping
5. Restore environment state
6. Test with complex sessions

**Definition of Done:**
- [ ] Recording comprehensive
- [ ] Replay accurate
- [ ] Interactive stepping works
- [ ] Environment restored

### Task 9.3.7: Section 9.3 Testing
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Test all development experience features.

**Acceptance Criteria:**
- [ ] Hot reload tests pass
- [ ] Validation tests complete
- [ ] Profiling verified
- [ ] Recording/replay tested
- [ ] Performance targets met
- [ ] Documentation complete

**Implementation Steps:**
1. Test hot reload scenarios
2. Validate script checker
3. Verify profiling accuracy
4. Test session replay
5. Benchmark performance
6. Document all features

**Definition of Done:**
- [ ] All tests pass
- [ ] Performance verified
- [ ] Documentation complete

---

## Phase 9.4: Multi-Client Implementation (Days 10-11)

### Task 9.4.1: CLI Client Integration
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: CLI Team Lead

**Description**: Update llmspell-cli to connect to kernel service.

**Acceptance Criteria:**
- [ ] CLI connects to kernel service
- [ ] All REPL commands implemented
- [ ] Command history with search
- [ ] Enhanced error display
- [ ] Debug workflow support
- [ ] Media display capability

**Implementation Steps:**
1. Update CLI to use kernel connection:
   ```rust
   pub async fn start_repl(
       engine: ScriptEngine,
       runtime_config: LLMSpellConfig,
       history_file: Option<PathBuf>,
   ) -> Result<()> {
       let kernel = connect_or_start_kernel().await?;
       let cli_client = CLIReplInterface::new(kernel).await?;
       cli_client.run_interactive_loop().await
   }
   ```
2. Implement all REPL commands (.break, .step, .locals, etc.)
3. Add Ctrl+R history search
4. Enhance error display
5. Support media output
6. Test debugging workflows

**Definition of Done:**
- [ ] CLI fully integrated
- [ ] All commands work
- [ ] History search functional
- [ ] Media display works

### Task 9.4.2: CLI Run Command Mode Selection
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: CLI Team

**Description**: Modify `llmspell run` command to support debug mode via kernel service.

**Acceptance Criteria:**
- [ ] Run command detects --debug flag
- [ ] Kernel connection attempted in debug mode
- [ ] Fallback to embedded runtime works
- [ ] Script execution via kernel functional
- [ ] Debug state properly initialized
- [ ] Performance acceptable for non-debug

**Implementation Steps:**
1. Modify run command handler:
   ```rust
   // llmspell-cli/src/commands/run.rs
   pub async fn execute_script_file(
       script_path: PathBuf,
       engine: ScriptEngine,
       runtime_config: LLMSpellConfig,
       stream: bool,
       args: Vec<String>,
       output_format: OutputFormat,
       debug_mode: bool,  // New parameter
   ) -> Result<()> {
       if debug_mode {
           // Try kernel connection first
           match discover_kernel().await {
               Ok(kernel) => {
                   execute_via_kernel(kernel, script_path, args).await?
               }
               Err(_) => {
                   // Start new kernel
                   let kernel = start_kernel_service(&runtime_config).await?;
                   execute_via_kernel(kernel, script_path, args).await?
               }
           }
       } else {
           // Current direct execution path
           let runtime = create_runtime(engine, runtime_config).await?;
           let result = runtime.execute_script(&script_content).await?;
           println!("{}", format_output(&result, output_format)?);
       }
   }
   ```
2. Implement kernel execution path:
   ```rust
   async fn execute_via_kernel(
       kernel: KernelConnection,
       script_path: PathBuf,
       args: Vec<String>,
   ) -> Result<()> {
       // Send execute request via shell channel
       let req = LRPRequest::ExecuteRequest {
           code: fs::read_to_string(&script_path).await?,
           debug_mode: true,
           args: Some(args),
       };
       kernel.shell_channel.send(req).await?;
       
       // Handle responses and debug events
       kernel.handle_execution_responses().await
   }
   ```
3. Add debug mode detection to CLI args
4. Create kernel connection utilities
5. Implement response handling
6. Test both debug and non-debug paths

**Definition of Done:**
- [ ] Debug mode detected correctly
- [ ] Kernel execution works
- [ ] Fallback functional
- [ ] Performance unchanged for non-debug
- [ ] Tests pass

### Task 9.4.3: CLI Debug Event Handler
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: CLI Team

**Description**: Implement debug event handling from IOPub channel in CLI.

**Acceptance Criteria:**
- [ ] IOPub events received correctly
- [ ] Breakpoint hits trigger debug REPL
- [ ] Output streams displayed properly
- [ ] Error events formatted nicely
- [ ] Progress events shown
- [ ] State changes reflected

**Implementation Steps:**
1. Create debug event handler:
   ```rust
   // llmspell-cli/src/kernel/debug_handler.rs
   pub struct DebugEventHandler {
       iopub_receiver: broadcast::Receiver<IOPubMessage>,
       debug_interface: Arc<DebugInterface>,
       output_formatter: OutputFormatter,
   }
   
   impl DebugEventHandler {
       pub async fn handle_events(&mut self) {
           while let Ok(event) = self.iopub_receiver.recv().await {
               match event {
                   IOPubMessage::DebugEvent(DebugEvent::BreakpointHit { location, stack, locals }) => {
                       self.on_breakpoint_hit(location, stack, locals).await?;
                   }
                   IOPubMessage::StreamOutput { name, text } => {
                       self.display_output(name, text);
                   }
                   IOPubMessage::ExecuteResult { data, .. } => {
                       self.display_result(data);
                   }
                   IOPubMessage::Error { traceback, .. } => {
                       self.display_error(traceback);
                   }
               }
           }
       }
       
       async fn on_breakpoint_hit(&mut self, location: Location, stack: Stack, locals: Locals) {
           println!("🔴 Breakpoint hit at {}:{}", location.file, location.line);
           self.display_stack(&stack);
           self.display_locals(&locals);
           
           // Enter interactive debug mode
           self.debug_interface.enter_debug_repl().await?;
       }
   }
   ```
2. Implement interactive debug REPL:
   ```rust
   impl DebugInterface {
       async fn enter_debug_repl(&mut self) {
           loop {
               let cmd = self.prompt_debug_command().await?;
               match cmd.as_str() {
                   "step" | "s" => self.send_step_request().await?,
                   "next" | "n" => self.send_next_request().await?,
                   "continue" | "c" => break,
                   "locals" | "l" => self.send_locals_request().await?,
                   "backtrace" | "bt" => self.send_backtrace_request().await?,
                   cmd if cmd.starts_with("inspect ") => {
                       let var = cmd.strip_prefix("inspect ").unwrap();
                       self.send_inspect_request(var).await?;
                   }
                   _ => println!("Unknown command. Try: step, next, continue, locals, backtrace, inspect <var>"),
               }
           }
       }
   }
   ```
3. Format debug output nicely
4. Handle concurrent events properly
5. Add event filtering options
6. Test with various debug scenarios

**Definition of Done:**
- [ ] Events handled correctly
- [ ] Debug REPL works
- [ ] Output formatted nicely
- [ ] All event types handled
- [ ] Tests pass

### Task 9.4.4: Kernel Discovery Logic
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: CLI Team

**Description**: Implement CLI-side kernel discovery and connection logic.

**Acceptance Criteria:**
- [ ] Connection files discovered
- [ ] Existing kernels detected
- [ ] Connection attempted correctly
- [ ] New kernel started if needed
- [ ] Connection info cached
- [ ] Cleanup on exit

**Implementation Steps:**
1. Implement kernel discovery:
   ```rust
   // llmspell-cli/src/kernel/discovery.rs
   pub struct KernelDiscovery {
       connection_dir: PathBuf,
       connection_cache: HashMap<String, ConnectionInfo>,
   }
   
   impl KernelDiscovery {
       pub async fn discover_or_start() -> Result<KernelConnection> {
           // 1. Check for connection files
           let conn_files = self.find_connection_files().await?;
           
           // 2. Try connecting to existing kernels
           for file in conn_files {
               let info = self.read_connection_info(&file).await?;
               if let Ok(conn) = self.try_connect(&info).await {
                   println!("Connected to existing kernel: {}", info.kernel_id);
                   return Ok(conn);
               }
           }
           
           // 3. Start new kernel
           println!("Starting new kernel...");
           self.start_new_kernel().await
       }
       
       async fn find_connection_files(&self) -> Result<Vec<PathBuf>> {
           let pattern = self.connection_dir.join("llmspell-kernel-*.json");
           glob::glob(&pattern.to_string_lossy())?
               .filter_map(Result::ok)
               .collect()
       }
       
       async fn try_connect(&self, info: &ConnectionInfo) -> Result<KernelConnection> {
           // Try to connect to all channels
           let shell = connect_channel(info.ip, info.shell_port).await?;
           let iopub = connect_channel(info.ip, info.iopub_port).await?;
           let control = connect_channel(info.ip, info.control_port).await?;
           
           // Verify kernel is alive via heartbeat
           if !self.check_heartbeat(info).await? {
               return Err(anyhow!("Kernel not responding"));
           }
           
           Ok(KernelConnection {
               info: info.clone(),
               shell,
               iopub,
               control,
           })
       }
   }
   ```
2. Implement connection caching
3. Add connection retry logic
4. Handle stale connection files
5. Implement cleanup on exit
6. Test discovery scenarios

**Definition of Done:**
- [ ] Discovery works correctly
- [ ] Connections established
- [ ] New kernels started
- [ ] Cleanup functional
- [ ] Tests pass

### Task 9.4.5: Web Client Foundation
**Priority**: MEDIUM  
**Estimated Time**: 6 hours  
**Assignee**: Web Team

**Description**: Create foundation for web-based REPL client.

**Acceptance Criteria:**
- [ ] WebSocket connection to kernel
- [ ] Basic web UI scaffolding
- [ ] Message handling works
- [ ] Output streaming functional
- [ ] Media display supported
- [ ] Multi-client aware

**Implementation Steps:**
1. Create web client structure
2. Implement WebSocket transport
3. Build basic HTML/JS interface
4. Handle LRP messages
5. Display streamed output
6. Test multi-client scenarios

**Definition of Done:**
- [ ] Web client connects
- [ ] Messages handled
- [ ] Output displayed
- [ ] Multi-client works

### Task 9.4.6: IDE Integration (LSP/DAP)
**Priority**: HIGH  
**Estimated Time**: 10 hours  
**Assignee**: IDE Team

**Description**: Implement LSP server and DAP adapter for IDE integration.

**Acceptance Criteria:**
- [ ] LSP server implemented
- [ ] Completion provider works
- [ ] Hover provider functional
- [ ] Diagnostics published
- [ ] DAP adapter implemented
- [ ] Breakpoint management works

**Implementation Steps:**
1. Create `llmspell-lsp` crate
2. Implement LanguageServer trait:
   ```rust
   impl LanguageServer for LLMSpellLanguageServer {
       async fn initialize(...) -> Result<InitializeResult> { ... }
       async fn completion(...) -> Result<Option<CompletionResponse>> { ... }
       async fn hover(...) -> Result<Option<Hover>> { ... }
   }
   ```
3. Build DAP adapter
4. Connect to kernel service
5. Implement all providers
6. Test with VS Code

**Definition of Done:**
- [ ] LSP server functional
- [ ] All providers work
- [ ] DAP debugging works
- [ ] VS Code integration tested

### Task 9.4.7: VS Code Extension
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: IDE Team

**Description**: Create VS Code extension for LLMSpell development.

**Acceptance Criteria:**
- [ ] Extension manifest complete
- [ ] Language configuration done
- [ ] Debug adapter integrated
- [ ] Syntax highlighting works
- [ ] Snippets provided
- [ ] Commands implemented

**Implementation Steps:**
1. Create extension structure
2. Write package.json manifest
3. Implement extension activation
4. Connect to LSP server
5. Integrate debug adapter
6. Add syntax highlighting
7. Create useful snippets
8. Test in VS Code

**Definition of Done:**
- [ ] Extension installable
- [ ] All features work
- [ ] Debugging functional
- [ ] Good developer experience

### Task 9.4.8: Remote Debugging Security
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Security Team

**Description**: Implement security for remote debugging connections.

**Acceptance Criteria:**
- [ ] Authentication token system
- [ ] TLS encryption support
- [ ] Permission model implemented
- [ ] Audit logging functional
- [ ] Session isolation works
- [ ] Security documentation

**Implementation Steps:**
1. Implement `RemoteDebugSecurity`:
   ```rust
   pub struct RemoteDebugSecurity {
       auth_tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
       tls_config: Option<Arc<ServerConfig>>,
       audit_log: Arc<Mutex<AuditLog>>,
   }
   ```
2. Build token authentication
3. Add TLS support
4. Implement permissions
5. Create audit logging
6. Document security model

**Definition of Done:**
- [ ] Authentication works
- [ ] TLS encryption functional
- [ ] Permissions enforced
- [ ] Audit trail complete

### Task 9.4.9: Section 9.4 Testing
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Test multi-client implementation thoroughly.

**Acceptance Criteria:**
- [ ] Multi-client tests pass
- [ ] Protocol compliance verified
- [ ] Security tests complete
- [ ] Performance benchmarks met
- [ ] Integration tests pass
- [ ] Documentation complete

**Implementation Steps:**
1. Test multi-client scenarios
2. Verify protocol handling
3. Test security measures
4. Benchmark performance
5. Run integration tests
6. Document client APIs

**Definition of Done:**
- [ ] All tests pass
- [ ] Security verified
- [ ] Performance acceptable
- [ ] Documentation complete

---

## Phase 9.5: Configuration and CLI Commands (Days 12-13)

### Task 9.5.1: Configuration System
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Config Team

**Description**: Implement comprehensive configuration for debugging and REPL.

**Acceptance Criteria:**
- [ ] TOML configuration parsing
- [ ] Debug settings configurable
- [ ] REPL settings configurable
- [ ] Remote settings supported
- [ ] Environment variable override
- [ ] Configuration validation

**Implementation Steps:**
1. Define configuration structure
2. Implement TOML parsing
3. Add environment variable support
4. Create validation logic
5. Document all settings
6. Test configuration loading

**Definition of Done:**
- [ ] Configuration loads correctly
- [ ] All settings work
- [ ] Validation comprehensive
- [ ] Documentation complete

### Task 9.5.2: CLI Debug Commands
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: CLI Team

**Description**: Implement all CLI debug commands.

**Acceptance Criteria:**
- [ ] `llmspell debug` command works
- [ ] `llmspell debug-server` implemented
- [ ] `llmspell debug-attach` functional
- [ ] `llmspell record` captures sessions
- [ ] `llmspell replay` works
- [ ] `llmspell validate` validates scripts
- [ ] `llmspell profile` generates profiles

**Implementation Steps:**
1. Implement debug command
2. Add debug-server mode
3. Build debug-attach client
4. Create recording command
5. Implement replay command
6. Add validation command
7. Build profiling command

**Definition of Done:**
- [ ] All commands implemented
- [ ] Help text complete
- [ ] Error handling robust
- [ ] Documentation updated

### Task 9.5.3: Media and Streaming Support
**Priority**: MEDIUM  
**Estimated Time**: 6 hours  
**Assignee**: Protocol Team

**Description**: Add media handling and streaming to protocols.

**Acceptance Criteria:**
- [ ] Media messages in LRP
- [ ] Streaming protocol defined
- [ ] Image display support
- [ ] Audio/video handling
- [ ] Progress streaming works
- [ ] Large file transfers

**Implementation Steps:**
1. Extend LRP with media messages
2. Define streaming protocol
3. Implement media handlers
4. Add progress tracking
5. Support large transfers
6. Test with various media

**Definition of Done:**
- [ ] Media messages work
- [ ] Streaming functional
- [ ] All media types handled
- [ ] Performance acceptable

### Task 9.5.4: Command History Enhancement
**Priority**: MEDIUM  
**Estimated Time**: 4 hours  
**Assignee**: CLI Team

**Description**: Implement enhanced command history with reverse search.

**Acceptance Criteria:**
- [ ] History persistence works
- [ ] Ctrl+R search implemented
- [ ] Fuzzy matching functional
- [ ] History size configurable
- [ ] Search highlighting works
- [ ] History management commands

**Implementation Steps:**
1. Implement `EnhancedHistory`
2. Add reverse search
3. Integrate fuzzy matching
4. Build search UI
5. Add history commands
6. Test search functionality

**Definition of Done:**
- [ ] History search works
- [ ] Fuzzy matching accurate
- [ ] UI responsive
- [ ] Commands functional

### Task 9.5.5: Documentation and Tutorials
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: Documentation Team

**Description**: Create comprehensive documentation and tutorials.

**Acceptance Criteria:**
- [ ] Architecture documentation
- [ ] Protocol specifications
- [ ] Client implementation guide
- [ ] Debugging tutorial
- [ ] Configuration reference
- [ ] Troubleshooting guide

**Implementation Steps:**
1. Document kernel architecture
2. Write protocol specs
3. Create client guides
4. Build debugging tutorial
5. Document configuration
6. Add troubleshooting

**Definition of Done:**
- [ ] Documentation comprehensive
- [ ] Examples work
- [ ] Tutorials clear
- [ ] Reference complete

### Task 9.5.6: Section 9.5 Testing
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Final testing and validation.

**Acceptance Criteria:**
- [ ] All CLI commands tested
- [ ] Configuration validated
- [ ] Media handling verified
- [ ] History search tested
- [ ] Documentation reviewed
- [ ] Performance benchmarked

**Implementation Steps:**
1. Test all CLI commands
2. Validate configurations
3. Test media scenarios
4. Verify history search
5. Review documentation
6. Run benchmarks

**Definition of Done:**
- [ ] All tests pass
- [ ] Documentation accurate
- [ ] Performance targets met

---

## Phase 9.6: Final Integration and Polish (Days 14-15)

### Task 9.6.1: Performance Optimization
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Performance Team

**Description**: Optimize performance to meet all targets.

**Acceptance Criteria:**
- [ ] Kernel startup <100ms
- [ ] Message handling <50ms
- [ ] Tab completion <100ms
- [ ] Breakpoint checking <1ms
- [ ] Hot reload <500ms
- [ ] Memory overhead <50MB

**Implementation Steps:**
1. Profile kernel startup
2. Optimize message handling
3. Speed up completions
4. Optimize breakpoint checks
5. Improve hot reload
6. Reduce memory usage

**Definition of Done:**
- [ ] All targets met
- [ ] Benchmarks pass
- [ ] No regressions

### Task 9.6.2: End-to-End Testing
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: QA Team

**Description**: Comprehensive end-to-end testing of all features.

**Acceptance Criteria:**
- [ ] Multi-client scenarios tested
- [ ] Debugging workflows verified
- [ ] Recording/replay tested
- [ ] Security validated
- [ ] Performance confirmed
- [ ] All integrations work

**Implementation Steps:**
1. Test complete debugging session
2. Verify multi-client workflows
3. Test session recording
4. Validate security measures
5. Run performance suite
6. Test all integrations

**Definition of Done:**
- [ ] All scenarios pass
- [ ] No critical bugs
- [ ] Performance acceptable

### Task 9.6.3: Quality Assurance
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Final quality checks and polish.

**Acceptance Criteria:**
- [ ] >90% test coverage
- [ ] Zero clippy warnings
- [ ] All TODOs resolved
- [ ] Documentation complete
- [ ] Examples working
- [ ] No memory leaks

**Implementation Steps:**
1. Run coverage analysis
2. Fix clippy warnings
3. Resolve all TODOs
4. Complete documentation
5. Test all examples
6. Check for memory leaks

**Definition of Done:**
- [ ] Quality targets met
- [ ] No warnings
- [ ] Documentation complete

### Task 9.6.4: Release Preparation
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Release Team

**Description**: Prepare for Phase 9 release.

**Acceptance Criteria:**
- [ ] CHANGELOG updated
- [ ] Version bumped
- [ ] Migration guide written
- [ ] Release notes prepared
- [ ] Breaking changes documented
- [ ] Announcement drafted

**Implementation Steps:**
1. Update CHANGELOG
2. Bump version numbers
3. Write migration guide
4. Prepare release notes
5. Document breaking changes
6. Draft announcement

**Definition of Done:**
- [ ] Release ready
- [ ] Documentation complete
- [ ] Announcement prepared

### Task 9.6.5: Stakeholder Demo
**Priority**: HIGH  
**Estimated Time**: 2 hours  
**Assignee**: Team Lead

**Description**: Demonstrate Phase 9 features to stakeholders.

**Acceptance Criteria:**
- [ ] Demo script prepared
- [ ] All features demonstrated
- [ ] Questions answered
- [ ] Feedback collected
- [ ] Issues documented
- [ ] Next steps defined

**Implementation Steps:**
1. Prepare demo script
2. Set up demo environment
3. Conduct demonstration
4. Collect feedback
5. Document issues
6. Plan next steps

**Definition of Done:**
- [ ] Demo completed
- [ ] Feedback positive
- [ ] Next steps clear

### Task 9.6.6: Phase 9 Completion
**Priority**: CRITICAL  
**Estimated Time**: 2 hours  
**Assignee**: Project Manager

**Description**: Official Phase 9 completion and handoff.

**Acceptance Criteria:**
- [ ] All tasks completed
- [ ] Documentation finalized
- [ ] Code reviewed and merged
- [ ] Tests passing in CI
- [ ] Performance validated
- [ ] Phase 10 ready to start

**Implementation Steps:**
1. Verify all tasks done
2. Final documentation review
3. Merge all code
4. Verify CI green
5. Validate performance
6. Hand off to Phase 10

**Definition of Done:**
- [ ] Phase 9 complete
- [ ] All criteria met
- [ ] Ready for Phase 10

---

## Risk Mitigation

### Technical Risks
1. **Protocol Complexity**: LRP/LDP may be complex
   - Mitigation: Start with minimal protocol, iterate
   - Fallback: Simplified protocol version

2. **Multi-client Conflicts**: State synchronization issues
   - Mitigation: Rust Arc/RwLock patterns
   - Monitoring: Conflict detection logging

3. **Performance Overhead**: Debugging may slow execution
   - Mitigation: Conditional compilation, lazy evaluation
   - Target: <10% overhead when enabled

### Schedule Risks
1. **Kernel Architecture Complexity**: May take longer than estimated
   - Mitigation: Early prototyping, parallel development
   - Buffer: 2 days contingency built in

2. **IDE Integration Challenges**: VS Code extension complexity
   - Mitigation: Start with minimal viable extension
   - Fallback: Command-line debugging only

---

## Success Metrics

### Performance
- Kernel startup: <100ms ✅
- Message handling: <50ms ✅  
- Multi-client scaling: 10+ clients ✅
- Debug overhead: <10% ✅

### Quality
- Test coverage: >90% ✅
- Documentation: 100% public APIs ✅
- Zero critical bugs ✅

### Developer Experience
- 80% reduction in debug time ✅
- 90% of errors show suggestions ✅
- 95% can debug without docs ✅

---

## Dependencies

### External
- `tokio`: Async runtime
- `serde`: Serialization
- `tower-lsp`: LSP implementation
- `notify`: File watching
- `pprof`: CPU profiling
- `opentelemetry`: Distributed tracing

### Internal
- Phase 4: Hook system integration
- Phase 5: State management
- Phase 7: Session management
- Phase 8: RAG for context

---

## Completion Checklist

### Week 1 (Days 1-3): Kernel Foundation
- [ ] llmspell-repl crate created
- [ ] Kernel service implemented
- [ ] Five channels working
- [ ] Connection discovery functional
- [ ] Protocols defined

### Week 2 (Days 4-9): Core Features
- [ ] Debugging infrastructure complete
- [ ] Error enhancement working
- [ ] Hot reload functional
- [ ] Profiling implemented
- [ ] Session recording works

### Week 3 (Days 10-15): Integration & Polish
- [ ] Multi-client support complete
- [ ] CLI fully integrated
- [ ] IDE support working
- [ ] All commands implemented
- [ ] Performance targets met
- [ ] Documentation complete

---

**🚀 Phase 9 transforms LLMSpell from a powerful scripting platform into a developer-friendly system with world-class debugging capabilities through its kernel-as-service architecture.**