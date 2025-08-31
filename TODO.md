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
- [x] Kernel service starts as standalone process in <100ms (verified via llmspell-kernel binary)
- [x] Multiple clients (CLI, web, IDE) connect to same kernel (ClientManager implemented)
- [x] LRP/LDP protocols enable message-based communication (full protocol.rs implementation)
- [x] Connection discovery via JSON files works (ConnectionInfo + KernelDiscovery)
- [x] State persists via SharedExecutionContext with async integration (Phase 9.2.10)
- [x] Conditional breakpoints with hit/ignore counts work (ConditionEvaluator + two-tier architecture)
- [x] Step debugging with async context preservation works (StepDebugger with mode transitions)
- [x] Variables inspected with lazy expansion (VariableInspector trait + caching)
- [x] Hot reload preserves state across file changes (DiagnosticsBridge integration)
- [x] Script validation with error pattern database (three-layer validation system)
- [x] Circuit breaker monitoring with adaptive thresholds (CircuitBreaker trait + WorkloadClassifier)
- [x] Distributed tracing with trace enrichment (DiagnosticsBridge trace_execution)
- [x] Performance profiling with adaptive overhead limits (HookProfiler + ProfilingConfig)
- [x] Session recording/replay with adaptive compression (SessionRecorder trait)
- [x] Hook multiplexer allows profiling + debugging simultaneously (HookMultiplexer innovation)
- [x] Two-tier debug system achieves <1% overhead when disabled (DebugStateCache fast path)
- [x] Three-layer bridge architecture maintained (Bridge → Shared → Script layers)
- [x] Dependency injection pattern with Null implementations for testing (DiagnosticsBridgeBuilder)
- [x] Adaptive performance configuration (no hardcoded thresholds, environment presets)
- [ ] Command history with Ctrl+R search (Phase 9.4)
- [ ] Media/streaming support in protocols (Phase 9.4)
- [ ] LSP/DAP protocol implementations (Phase 9.4)
- [ ] VS Code extension with debugging (Phase 9.4)
- [ ] Remote debugging with security (Phase 9.4)
- [x] All Phase 9.1-9.3 tests pass with zero clippy warnings
- [x] Architecture documentation complete (dependency-injection.md, adaptive-performance.md)

---

# see TODO-DONE.md for details of the below sub-tasks that are now done.
### Task 9.1.1: Create llmspell-repl Crate Structure
### Task 9.1.2: Implement LLMSpell Kernel Service
### Task 9.1.3: Bridge-Kernel Debug Integration
### Task 9.1.4: Five Channel Architecture
### Task 9.1.5: Connection Discovery System
### Task 9.1.6: LRP/LDP Protocol Implementation
### ✅ Task 9.1.7: Debug/Diagnostics Architecture Refactoring [COMPLETE]
### Task 9.1.8: Foundation Quality Gates and Testing ✅ COMPLETE
### ✅ Task 9.2.1: Interactive Debugger Implementation with Bridge Integration - COMPLETE
### Task 9.2.2: Debug Session Management with Multi-Client Integration ✅
### Task 9.2.3: Lua Debug Hooks Implementation ✅
### Task 9.2.4: Debug Performance Optimization & Hook Multiplexer Architecture ✅
### Task 9.2.5: Breakpoint Condition Evaluator (Two-Tier Integration) ✅ COMPLETED
### Task 9.2.6: Step Debugging with Mode Transitions ✅ COMPLETED
### Task 9.2.7: Variable Inspection System (Slow Path Only) ✅ COMPLETED
### Task 9.2.7b: Architecture Refactoring - Three-Layer Bridge Compliance ✅ COMPLETED
### Task 9.2.8: Watch Expressions (Slow Path Evaluation) ✅ COMPLETED
### Task 9.2.9: Call Stack Navigator (Read-Only Operations) ✅ COMPLETED
### Task 9.2.10: SharedExecutionContext Async Integration Points ✅ COMPLETED
### Task 9.2.11: Distributed Tracing Integration
### Task 9.2.12: Section 9.2 Quality Gates and Testing
### Task 9.3.1: Hot Reload System
### Task 9.3.2: Script Validation System ✓
### Task 9.3.3: Performance Profiling ✅ COMPLETE
### Task 9.3.4: Unified Test Execution Framework
### Task 9.3.5: Performance Profiler Hooks
### Task 9.3.6: Hook Introspection & Circuit Breakers
### Task 9.3.7: Session Recording/Replay
### Task 9.3.8: Section 9.3 Quality Gates and Testing

---

## Phase 9.4: Multi-Client Implementation (Days 10-11)

### Task 9.4.1: CLI Client Integration
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: CLI Team Lead

**Description**: Update llmspell-cli to connect to kernel service with workload-aware performance expectations.

**ARCHITECTURE ALIGNMENT (Phase 9.1 + 9.3 Patterns):**
- **Dependency Injection**: Use builder pattern for CLI components
- **Test Safety**: Create `NullKernelConnection` for testing
- **Three-Layer Architecture**: CLI → KernelConnection → Script Runtime
- **Workload Categorization**: Interactive commands = Micro, batch ops = Heavy
- **Adaptive Performance**: Response time expectations based on WorkloadClassifier
- **Debug workflow support**: Uses ExecutionManager and ExecutionBridge
- **Enhanced error display**: Integrates with DiagnosticsBridge

**Acceptance Criteria:**
- [ ] CLI uses dependency injection (builder pattern)
- [ ] NullKernelConnection implemented for tests
- [ ] CLI operations categorized by WorkloadClassifier (Micro/Light/Medium/Heavy)
- [ ] Interactive commands use Micro workload thresholds (adaptive, not fixed)
- [ ] Batch operations use Heavy workload thresholds (adaptive)
- [ ] Tab completion responsive within Micro thresholds
- [ ] Debug operations measured with appropriate workload category
- [ ] Media display performance adapted to content size
- [ ] Test helpers use `create_test_cli()` pattern

**Implementation Steps:**
1. Update CLI to use dependency injection:
   ```rust
   // Use builder pattern instead of direct construction
   pub async fn start_repl(
       engine: ScriptEngine,
       runtime_config: LLMSpellConfig,
       history_file: Option<PathBuf>,
   ) -> Result<()> {
       let kernel = KernelConnection::builder()
           .discovery(Box::new(KernelDiscovery::new()))
           .circuit_breaker(Box::new(ExponentialBackoffBreaker::default()))
           .build()
           .connect_or_start().await?;
           
       let cli_client = CLIReplInterface::builder()
           .kernel(kernel)
           .diagnostics(DiagnosticsBridge::builder().build())
           .build();
           
       cli_client.run_interactive_loop().await
   }
   ```
2. **Implement REPL commands using ExecutionManager**:
   ```rust
   // Commands that interact with debugging
   match command {
       ".break" => {
           // Use ExecutionManager from execution_bridge.rs
           let bp = Breakpoint::new(current_file, line_number);
           kernel.execution_manager.add_breakpoint(bp).await?
       },
       ".step" => {
           kernel.execution_manager.send_command(DebugCommand::StepInto).await?
       },
       ".locals" => {
           // Get variables via ExecutionManager
           let vars = kernel.execution_manager.get_variables(current_frame).await?;
           display_variables_using_output_formatting(vars);
       },
   }
   ```

3. **Integrate multi-client debugging session management**
4. **Add distributed tracing for CLI command observability**
5. **Enhanced error display via DiagnosticsBridge with trace enrichment**
6. **Test CLI integration with established interactive debugging patterns**

**Definition of Done:**
- [ ] CLI fully integrated
- [ ] All commands work
- [ ] History search functional
- [ ] Media display works
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.2: CLI Run Command Mode Selection
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: CLI Team

**Description**: Debug mode CLI execution using Phase 9.2 interactive debugging infrastructure, session management patterns, and distributed tracing integration.

**ARCHITECTURE ALIGNMENT (Phase 9.2 + 9.3 Patterns):**
- **Dependency Injection**: Use builder pattern for debug components
- **Test Safety**: Create `NullDebugSession` for testing
- **Workload Classification**: Debug mode = Medium, non-debug = Light
- **Interactive Debugger**: Uses established multi-client session patterns (9.2.2)
- **Debug State Management**: Leverages DebugStateCache and step debugging patterns (9.2.5)
- **SharedExecutionContext**: Performance monitoring and async context preservation (9.2.10)
- **Session Recording**: Integrate SessionRecorder for debug replay
- **Circuit Breaker**: Use for kernel connection failures

**Acceptance Criteria:**
- [ ] Debug components use dependency injection
- [ ] NullDebugSession implemented for tests
- [ ] Debug mode initializes InteractiveDebugger with session management
- [ ] Kernel discovery uses CircuitBreaker for retry logic
- [ ] Debug state initialization via DebugStateCache LRU patterns
- [ ] Script execution preserves SharedExecutionContext async boundaries
- [ ] Non-debug mode maintains Light workload performance (adaptive)
- [ ] Debug mode uses Medium workload thresholds
- [ ] SessionRecorder captures debug session for replay
- [ ] No hardcoded performance thresholds

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
           // Initialize debug session using Phase 9.2 patterns
           let debug_session_id = uuid::Uuid::new_v4().to_string();
           
           match discover_kernel().await {
               Ok(mut kernel) => {
                   // Initialize InteractiveDebugger with session management
                   kernel.initialize_debug_session(debug_session_id.clone()).await?;
                   
                   // Create SharedExecutionContext with performance metrics
                   let mut shared_context = SharedExecutionContext::new();
                   shared_context.correlation_id = Some(debug_session_id);
                   
                   // Execute with distributed tracing
                   execute_via_kernel_with_tracing(kernel, script_path, args, shared_context).await?
               }
               Err(_) => {
                   // Start kernel with interactive debugging enabled
                   let kernel = start_debug_kernel(&runtime_config, debug_session_id).await?;
                   let shared_context = SharedExecutionContext::new();
                   execute_via_kernel_with_tracing(kernel, script_path, args, shared_context).await?
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
   async fn execute_via_kernel_with_tracing(
       kernel: KernelConnection,
       script_path: PathBuf,
       args: Vec<String>,
       mut shared_context: SharedExecutionContext,
   ) -> Result<()> {
       // Create trace span for script execution
       let trace_id = shared_context.correlation_id.clone().unwrap_or_default();
       
       // Send execute request with trace correlation
       let req = LRPRequest::ExecuteRequest {
           code: fs::read_to_string(&script_path).await?,
           debug_mode: true,
           args: Some(args),
           trace_id: Some(trace_id),
           context: Some(shared_context.clone()),
       };
       
       kernel.shell_channel.send(req).await?;
       
       // Handle debug events using established patterns
       while let Some(event) = kernel.debug_channel.recv().await {
           match event {
               DebugEvent::BreakpointHit { location, stack, locals } => {
                   // Display using VariableInspector formatting
                   println!("Breakpoint hit at {}:{}", location.source, location.line);
                   kernel.variable_inspector.display_locals(&locals)?;
               },
               DebugEvent::StepComplete { new_location } => {
                   println!("Step complete: {}:{}", new_location.source, new_location.line);
               },
               _ => {} // Handle other debug events
           }
       }
       
       Ok(())
   }
   ```
3. **Implement debug session management with multi-client support**
4. **Add distributed tracing integration for debug run observability**
5. **Create kernel connection with InteractiveDebugger initialization**
6. **Test debug/non-debug paths with performance validation**

**Definition of Done:**
- [ ] Debug mode detected correctly
- [ ] Kernel execution works
- [ ] Fallback functional
- [ ] Performance unchanged for non-debug
- [ ] Tests pass
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.3: CLI Debug Event Handler
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: CLI Team

**Description**: Implement debug event handling using unified types and ExecutionManager, integrating with Phase 9.1 architecture.

**ARCHITECTURE ALIGNMENT (Phase 9.1 + 9.3 Patterns):**
- **Dependency Injection**: Use builder pattern for event handler
- **Test Safety**: Create `NullDebugEventHandler` for testing
- **Three-Layer Architecture**: Event Handler → ExecutionBridge → Script Runtime
- **Uses unified types**: StackFrame, Variable from execution_bridge.rs
- **Error formatting**: Integrates with DiagnosticsBridge patterns
- **Debug interface**: Coordinates with ExecutionManager
- **Hook Integration**: Uses HookProfiler for event performance monitoring
- **Circuit Breaker**: Handles event flooding scenarios
- **Output formatting** uses output.rs functions

**Acceptance Criteria:**
- [ ] Event handler uses dependency injection
- [ ] NullDebugEventHandler implemented for tests
- [ ] IOPub events received using established protocol types
- [ ] Event performance monitored with HookProfiler
- [ ] Circuit breaker prevents event flooding
- [ ] No hardcoded thresholds for event handling
- [ ] Breakpoint hits trigger debug REPL via ExecutionManager
- [ ] Output streams displayed using output.rs formatting
- [ ] Error events formatted via diagnostics_bridge.rs patterns
- [ ] Progress events shown with SharedExecutionContext metrics
- [ ] State changes reflected using unified DebugState type

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
                   IOPubMessage::DebugEvent(DebugEvent::BreakpointHit { 
                       location, 
                       stack,    // Vec<StackFrame> - unified type
                       locals    // Vec<Variable> - unified type
                   }) => {
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
       
       async fn on_breakpoint_hit(
           &mut self, 
           location: SourceLocation, 
           stack: Vec<StackFrame>,        // Use unified StackFrame type
           locals: Vec<Variable>          // Use unified Variable type
       ) {
           println!("🔴 Breakpoint hit at {}:{}", location.source, location.line);
           
           // Use output.rs for display formatting
           self.display_stack_using_output_formatting(&stack);
           self.display_variables_using_output_formatting(&locals);
           
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
3. **Integrate VariableInspector trait for professional variable display**
4. **Add StackNavigator trait integration for stack formatting**
5. **Implement distributed tracing correlation for debug events**
6. **Test debug event handling with multi-client session patterns**

**Definition of Done:**
- [ ] Events handled correctly
- [ ] Debug REPL works
- [ ] Output formatted nicely
- [ ] All event types handled
- [ ] Tests pass
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.4: Kernel Discovery Logic
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: CLI Team

**Description**: CLI kernel discovery using established LRP/LDP protocols, connection authentication, and multi-client session management patterns.

**ARCHITECTURE ALIGNMENT (Phase 9.3 Patterns):**
- **Dependency Injection**: Use builder pattern for discovery component
- **Test Safety**: Create `NullKernelDiscovery` for testing
- **Circuit Breaker**: Use for connection retry logic
- **Session Recording**: Record discovery attempts for debugging
- **Adaptive Retry**: Use WorkloadClassifier for retry intervals

**Acceptance Criteria:**
- [ ] Discovery component uses dependency injection
- [ ] NullKernelDiscovery implemented for tests
- [ ] Connection files discovered
- [ ] Existing kernels detected with CircuitBreaker retry
- [ ] Connection attempted with adaptive retry intervals
- [ ] New kernel started if needed
- [ ] Connection info cached
- [ ] SessionRecorder logs discovery attempts
- [ ] Cleanup on exit
- [ ] No hardcoded retry intervals

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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.5: Web Client Foundation
**Priority**: MEDIUM  
**Estimated Time**: 6 hours  
**Assignee**: Web Team

**Description**: Web REPL client using Phase 9.2 kernel protocols, interactive debugging WebSocket integration, and distributed tracing visualization.

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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.6: IDE Integration (LSP/DAP)
**Priority**: HIGH  
**Estimated Time**: 10 hours  
**Assignee**: IDE Team

**Description**: LSP/DAP integration with workload-aware responsiveness metrics for different IDE operations.

**ARCHITECTURE ALIGNMENT (Phase 9.3 Patterns):**
- **Dependency Injection**: Use builder pattern for LSP/DAP servers
- **Test Safety**: Create `NullLanguageServer`, `NullDebugAdapter`
- **Adaptive Performance**: Use WorkloadClassifier, NOT hardcoded thresholds
- **Workload Classification**:
  - Autocomplete: Micro workload (fast, interactive)
  - Hover Info: Micro workload (tooltip display)
  - Diagnostics: Medium workload (background analysis)
  - Debug Steps: Heavy workload (complex state changes)
- **NO HARDCODED THRESHOLDS**: All performance expectations adaptive

**Acceptance Criteria:**
- [ ] LSP/DAP servers use dependency injection
- [ ] Null implementations created for testing
- [ ] LSP operations categorized by WorkloadClassifier
- [ ] Autocomplete uses Micro workload category (adaptive thresholds)
- [ ] Hover provider uses Micro workload category (adaptive)
- [ ] Diagnostics use Medium workload category (adaptive)
- [ ] DAP operations use Heavy workload for complex operations
- [ ] Performance metrics reported via HookProfiler
- [ ] CircuitBreaker handles LSP/DAP overload
- [ ] NO hardcoded millisecond thresholds anywhere

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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes

### Task 9.4.7: VS Code Extension
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: IDE Team

**Description**: VS Code extension using Phase 9.2 LSP/DAP integration, interactive debugging UI, conditional breakpoint support, and distributed tracing visualization.

**Architectural Alignment (from Phase 9.1-9.3):**
- Use dependency injection for VS Code API wrappers
- Create Null implementations for testing without VS Code
- Integrate with DiagnosticsBridge for performance monitoring
- Use adaptive thresholds for extension performance
- Apply WorkloadClassifier to extension operations
- Implement CircuitBreaker for remote connections

**Acceptance Criteria:**
- [ ] Extension manifest complete
- [ ] Language configuration done
- [ ] Debug adapter integrated
- [ ] Syntax highlighting works
- [ ] Snippets provided
- [ ] Commands implemented
- [ ] Dependency injection for VS Code APIs
- [ ] Null implementations for testing
- [ ] DiagnosticsBridge integration
- [ ] Adaptive performance monitoring

**Implementation Steps:**
1. Create extension structure with DI pattern
2. Write package.json manifest
3. Implement extension activation with DiagnosticsBridge
4. Connect to LSP server with CircuitBreaker
5. Integrate debug adapter with SessionRecorder
6. Add syntax highlighting with performance monitoring
7. Create useful snippets
8. Test with Null implementations
9. Test in VS Code

**Definition of Done:**
- [ ] Extension installable
- [ ] All features work with DI pattern
- [ ] Debugging functional with SessionRecorder
- [ ] Good developer experience
- [ ] Performance monitoring integrated
- [ ] Test coverage >90% using Null implementations
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.8: Remote Debugging Security
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Security Team

**Description**: Implement security for remote debugging connections.

**Architectural Alignment (from Phase 9.1-9.3):**
- Use dependency injection for security providers
- Create Null implementations for testing without crypto
- Integrate with DiagnosticsBridge for security monitoring
- Apply CircuitBreaker pattern to auth failures
- Use SessionRecorder for security audit trails
- No hardcoded timeouts or retry limits

**Acceptance Criteria:**
- [ ] Authentication token system
- [ ] TLS encryption support
- [ ] Permission model implemented
- [ ] Audit logging functional
- [ ] Session isolation works
- [ ] Security documentation
- [ ] Dependency injection for auth providers
- [ ] Null implementations for testing
- [ ] CircuitBreaker for auth failures
- [ ] SessionRecorder integration

**Implementation Steps:**
1. Implement `RemoteDebugSecurity` with DI:
   ```rust
   pub struct RemoteDebugSecurity {
       auth_provider: Box<dyn AuthProvider>,
       tls_provider: Box<dyn TlsProvider>,
       audit_recorder: Box<dyn SessionRecorder>,
       circuit_breaker: Box<dyn CircuitBreaker>,
   }
   ```
2. Build token authentication with adaptive rate limiting
3. Add TLS support with configurable cipher suites
4. Implement permissions with role-based access
5. Create audit logging via SessionRecorder
6. Add CircuitBreaker for failed auth attempts
7. Create Null implementations for testing
8. Document security model

**Definition of Done:**
- [ ] Authentication works
- [ ] TLS encryption functional
- [ ] Permissions enforced
- [ ] Audit trail complete
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.9: Section 9.4 Quality Gates and Testing
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Comprehensive quality checks and testing of multi-client implementation.

**ARCHITECTURE VALIDATION (Phase 9.3 Requirements):**
- **Dependency Injection**: Verify all components use builder pattern
- **Test Safety**: Confirm Null implementations exist for all traits
- **No Factory Functions**: Check no `create_*` functions in src/
- **No Hardcoded Thresholds**: Validate all performance uses WorkloadClassifier
- **Three-Layer Architecture**: Verify proper separation of concerns

**Acceptance Criteria:**
- [ ] All components use dependency injection (no direct construction)
- [ ] Null implementations exist for all injectable traits
- [ ] No factory functions in src/ (except documented exceptions)
- [ ] No hardcoded performance thresholds (all adaptive)
- [ ] No #[cfg(test)] in production code
- [ ] Multi-client tests pass (10+ clients)
- [ ] Protocol compliance verified
- [ ] Security tests complete with CircuitBreaker
- [ ] Performance measured with WorkloadClassifier
- [ ] Integration tests use test helpers from tests/common
- [ ] Zero clippy warnings
- [ ] Code properly formatted
- [ ] Documentation explains DI patterns
- [ ] Quality scripts pass

**Implementation Steps:**
1. **Validate Architectural Requirements**:
   ```bash
   # Check for factory functions (should be none except documented)
   rg "pub fn create_" --type rust src/
   
   # Check for hardcoded thresholds (should use WorkloadClassifier)
   rg "\d+ms|\d+ ms" --type rust src/
   
   # Verify Null implementations exist
   rg "struct Null" --type rust src/
   
   # Check for #[cfg(test)] in src (should be in test modules only)
   rg "#\[cfg\(test\)\]" --type rust src/
   ```

2. **Run Code Formatting**:
   ```bash
   cargo fmt --all --check
   # Fix any formatting issues:
   cargo fmt --all
   ```

3. **Run Clippy Linting**:
   ```bash
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   # Focus on CLI, LSP, and client code
   ```

3. **Write and Run Multi-Client Tests**:
   ```bash
   # Write multi-client scenario tests
   # Write protocol compliance tests
   # Write security validation tests
   # Write CLI integration tests
   # Write LSP/DAP integration tests
   cargo test --workspace --all-features
   ```

4. **Test Multi-Client Scenarios**:
   ```bash
   # Test with 10+ simultaneous clients
   cargo test --package llmspell-repl -- --ignored multi_client
   # Verify no resource leaks or conflicts
   ```

5. **Verify Security Measures**:
   ```bash
   # Test authentication and authorization
   cargo test --package llmspell-repl -- security
   # Test TLS encryption
   # Test audit logging
   ```

6. **Run Quality Check Scripts**:
   ```bash
   ./scripts/quality-check-minimal.sh  # Format, clippy, compile
   ./scripts/quality-check-fast.sh     # Adds unit tests & docs
   ```

7. **Document Client APIs**:
   ```bash
   cargo doc --package llmspell-cli --no-deps
   cargo doc --package llmspell-lsp --no-deps
   # Document all client integration APIs
   ```

**Definition of Done:**
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with `cargo test --workspace --all-features`
- [ ] 10+ simultaneous clients verified
- [ ] Security measures validated
- [ ] Quality check scripts pass
- [ ] Client API documentation complete

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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.5.6: Section 9.5 Quality Gates and Testing
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Comprehensive quality checks and final testing of configuration and CLI commands.

**Acceptance Criteria:**
- [ ] All CLI commands tested
- [ ] Configuration validated
- [ ] Media handling verified
- [ ] History search tested
- [ ] Documentation reviewed
- [ ] Performance benchmarked
- [ ] Zero clippy warnings
- [ ] Code properly formatted
- [ ] Quality scripts pass

**Implementation Steps:**
1. **Run Code Formatting**:
   ```bash
   cargo fmt --all --check
   # Fix any formatting issues:
   cargo fmt --all
   ```

2. **Run Clippy Linting**:
   ```bash
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   # Focus on configuration and CLI command code
   ```

3. **Test All CLI Commands**:
   ```bash
   # Test each new CLI command
   cargo test --package llmspell-cli -- cli_commands
   # Test llmspell debug
   # Test llmspell debug-server
   # Test llmspell debug-attach
   # Test llmspell record
   # Test llmspell replay
   # Test llmspell validate
   # Test llmspell profile
   ```

4. **Validate Configuration System**:
   ```bash
   # Test TOML configuration loading
   # Test environment variable overrides
   # Test configuration validation
   cargo test --package llmspell-repl -- config
   ```

5. **Test Media and History**:
   ```bash
   # Test media message handling
   # Test streaming protocol
   # Test Ctrl+R history search
   cargo test --workspace -- media history
   ```

6. **Run Quality Check Scripts**:
   ```bash
   ./scripts/quality-check-minimal.sh  # Format, clippy, compile
   ./scripts/quality-check-fast.sh     # Adds unit tests & docs
   ```

7. **Review Documentation**:
   ```bash
   # Verify all new CLI commands documented
   # Check configuration reference complete
   cargo doc --workspace --no-deps
   ```

**Definition of Done:**
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with `cargo test --workspace --all-features`
- [ ] All CLI commands work correctly
- [ ] Configuration system validated
- [ ] Quality check scripts pass
- [ ] Documentation complete and accurate

---

## Phase 9.6: Final Integration and Polish (Days 14-15)

### Task 9.6.1: Performance Optimization
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Performance Team

**Description**: Optimize performance with adaptive, resource-aware limits based on system capabilities.

**ARCHITECTURE ALIGNMENT (9.3.3 ProfilingConfig Pattern):**
- **MemoryConfig**: Adaptive limits based on available system resources
- **Resource-Aware**: Different limits for development/production/embedded
- **Percentage-Based**: Memory as % of available RAM, not fixed MB
- **Environment Presets**: Configurable for different deployment scenarios

**Acceptance Criteria:**
- [ ] MemoryConfig with environment-specific presets
- [ ] Memory limits as percentage of available RAM
- [ ] Development: generous limits (20% RAM)
- [ ] Production: configurable limits (10% RAM default)
- [ ] Embedded: strict fixed limits (configurable)
- [ ] Performance targets workload-aware, not fixed

**Implementation Steps:**
1. **Create MemoryConfig with adaptive limits**:
   ```rust
   pub struct MemoryConfig {
       pub max_memory_mode: MemoryLimitMode,
       pub cache_size_percentage: f32,
       pub gc_threshold_percentage: f32,
       pub environment: Environment,
   }
   
   pub enum MemoryLimitMode {
       Percentage(f32),      // % of available RAM
       Fixed(usize),        // Fixed bytes (embedded)
       Adaptive {           // Dynamic based on pressure
           min_mb: usize,
           max_percentage: f32,
       },
   }
   ```
2. Profile operations and categorize by workload
3. Apply appropriate thresholds per operation type
4. Report metrics without hard failures

**Definition of Done:**
- [ ] MemoryConfig implemented
- [ ] Resource-aware limits working
- [ ] Performance metrics reported
- [ ] No fixed thresholds in code

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

### Task 9.6.3: Final Quality Assurance
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Comprehensive final quality checks and polish for Phase 9.

**Acceptance Criteria:**
- [ ] >90% test coverage
- [ ] Zero clippy warnings
- [ ] Zero formatting issues
- [ ] All TODOs resolved
- [ ] Documentation complete (>95% coverage)
- [ ] Examples working
- [ ] No memory leaks
- [ ] All quality scripts pass

**Implementation Steps:**
1. **Run Complete Code Formatting**:
   ```bash
   # Check formatting across entire workspace
   cargo fmt --all --check
   # Fix any remaining formatting issues:
   cargo fmt --all
   ```

2. **Run Comprehensive Clippy Analysis**:
   ```bash
   # Run with all features and strict settings
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   # Fix any remaining clippy warnings
   # Pay special attention to:
   # - Unused code
   # - Inefficient patterns
   # - Missing documentation
   ```

3. **Run Coverage Analysis**:
   ```bash
   # Install tarpaulin if needed
   cargo install cargo-tarpaulin
   # Run coverage analysis
   cargo tarpaulin --workspace --all-features --out Html
   # Verify >90% coverage
   # Add tests for uncovered code paths
   ```

4. **Search and Resolve TODOs**:
   ```bash
   # Find all TODO comments
   grep -r "TODO" --include="*.rs" .
   # Resolve or convert to tracked issues
   # No TODOs should remain in code
   ```

5. **Verify Documentation Coverage**:
   ```bash
   # Generate documentation
   cargo doc --workspace --no-deps
   # Check for missing docs warnings
   cargo doc --workspace --no-deps 2>&1 | grep warning
   # Aim for >95% documentation coverage
   ```

6. **Test All Examples**:
   ```bash
   # Run all example scripts
   cargo run --example debug_example
   cargo run --example repl_example
   cargo run --example multi_client_example
   # Verify all examples work correctly
   ```

7. **Check for Memory Leaks**:
   ```bash
   # Run with valgrind (Linux/macOS)
   valgrind --leak-check=full cargo test --workspace
   # Or use built-in sanitizers
   RUSTFLAGS="-Z sanitizer=address" cargo test --workspace
   ```

8. **Run Full Quality Suite**:
   ```bash
   # Run all quality check scripts in sequence
   ./scripts/quality-check-minimal.sh  # Format, clippy, compile
   ./scripts/quality-check-fast.sh     # Adds unit tests & docs
   ./scripts/quality-check.sh          # Full validation suite
   # All must pass with zero errors
   ```

9. **Final Verification Checklist**:
   ```bash
   # Verify all acceptance criteria met:
   # - Kernel startup <100ms
   # - Debug overhead within development thresholds
   # - Multi-client support (10+ clients)
   # - All protocols implemented
   # - All CLI commands working
   # - VS Code extension functional
   ```

**Definition of Done:**
- [ ] `cargo fmt --all --check` passes with zero changes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage >90% verified
- [ ] Zero TODO comments in codebase
- [ ] Documentation coverage >95%
- [ ] All examples run successfully
- [ ] No memory leaks detected
- [ ] All quality scripts pass
- [ ] Performance targets met

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
   - Target: Adaptive overhead based on workload characteristics

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
- Debug overhead: Adaptive thresholds ✅

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
- [x] Profiling implemented ✅ (Task 9.3.3 with ProfilingConfig)
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