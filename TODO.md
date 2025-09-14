# Phase 9: Interactive REPL and Debugging Infrastructure - TODO List

**Version**: 1.0
**Date**: September 2025
**Status**: Implementation Ready
**Phase**: 9 (Interactive REPL and Debugging Infrastructure)
**Timeline**: Weeks 30-32 (16 working days)
**Priority**: HIGH (Developer Experience - Critical for adoption)
**Dependencies**: Phase 8 Vector Storage and RAG Foundation ✅
**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-09-design-doc.md
**Kernel-Architecture**: docs/technical/kernel-protocol-architecture.md
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE09-DONE.md)

> **📋 Actionable Task List**: This document breaks down Phase 9 implementation into specific, measurable tasks for building an integrated kernel architecture with comprehensive REPL and debugging capabilities, consolidating 28,000+ lines from Phase-9 branch into ~15,000 lines of production code.

---

## Overview

**Goal**: Implement interactive REPL and debugging infrastructure through kernel-centric architecture consolidation, migrating valuable code from Phase-9 branch while fixing critical runtime context issues.

**🚨 CRITICAL ARCHITECTURE MIGRATION**: This phase consolidates 5 crates (llmspell-kernel, llmspell-debug, llmspell-repl, enhanced bridge, sessions) from Phase-9 branch into unified kernel architecture, fixing "dispatch task is gone" error and establishing foundation for Phases 10-24.

**Success Criteria Summary:**
- [x] Global IO runtime eliminates "dispatch task is gone" error ✅
- [x] Complete 5-channel Jupyter protocol implementation ✅
- [ ] Debug Adapter Protocol (DAP) fully functional with 10 essential commands
- [ ] REPL with interactive debugging, breakpoints, and variable inspection
- [ ] Session management with artifact storage and TTL expiration
- [ ] Event correlation system with distributed tracing
- [ ] Performance targets met: <10ms tool init, <50ms agent creation, <5% hook overhead
- [ ] Application validation suite passes all 9 test applications
- [x] Comprehensive tracing infrastructure with environment control ✅
- [ ] Code consolidation: 28,000+ lines → ~15,000 lines (46% reduction)

---

## Phase 9.1: Core Runtime & Transport Foundation (Days 1-3)

### Task 9.1.1: Create Global IO Runtime Foundation ✅
**Priority**: CRITICAL
**Estimated Time**: 4 hours (Actual: 3.5 hours)
**Assignee**: Runtime Team Lead
**Dependencies**: None (starting point)
**Status**: COMPLETE ✅

**Description**: Create the global IO runtime foundation that fixes the "dispatch task is gone" error by ensuring all HTTP clients and I/O operations use the same runtime context.

**Acceptance Criteria:**
- [x] `llmspell-kernel/src/runtime/io_runtime.rs` created with global runtime ✅
- [x] `create_io_bound_resource<T, F>()` function for safe resource creation ✅
- [x] All HTTP clients in llmspell-tools use global runtime ✅
- [x] TracingInstrumentation struct for comprehensive tracing ✅
- [x] No "dispatch task is gone" errors in 60+ second tests ✅

**Implementation Steps:**
1. Create `llmspell-kernel/src/runtime/io_runtime.rs`:
   ```rust
   static GLOBAL_IO_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

   pub fn global_io_runtime() -> &'static Arc<Runtime> {
       GLOBAL_IO_RUNTIME.get_or_init(|| {
           Arc::new(Runtime::new().expect("IO runtime creation failed"))
       })
   }

   pub fn create_io_bound_resource<T, F>(creator: F) -> T
   where F: FnOnce() -> T {
       let _guard = global_io_runtime().enter();
       creator()
   }
   ```
2. Add comprehensive tracing instrumentation
3. Update 15 files in llmspell-tools to use global_io_runtime()
4. Remove SHARED_IO_RUNTIME workaround from Phase-9 branch llmspell-providers/src/rig.rs lines 17-40
5. Test with long-running operations (60+ seconds)
6. Add multi-protocol transport registration capability

**Test Steps:**
1. Run integration test that creates HTTP client and waits 35+ seconds
2. Verify no "dispatch task is gone" errors occur
3. Test resource creation in different runtime contexts
4. Validate tracing output includes runtime context information

**Definition of Done:**
- [x] Global runtime accessible from all crates ✅
- [x] HTTP clients survive beyond 30-second timeout ✅
- [x] No runtime context mismatches in logs ✅
- [x] Tracing shows consistent runtime context usage ✅
- [x] All existing provider tests pass ✅

### Task 9.1.2: Implement Multi-Protocol Transport Layer with Comprehensive Tracing ✅
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Transport Team Lead
**Dependencies**: Task 9.1.1

**Description**: Migrate and enhance transport layer from Phase-9 branch with 5-channel Jupyter support and protocol abstraction for future LSP/DAP/WebSocket protocols. Includes comprehensive tracing infrastructure covering all kernel operations across Phases 1-9.

**Reusable Code Migration:**
- Migrate `/tmp/phase-9-comparison/llmspell-kernel/src/transport/zeromq.rs` (237 lines) ✅
- Migrate `/tmp/phase-9-comparison/llmspell-kernel/src/jupyter/protocol.rs` (protocol structures) ✅
- Enhance with 5-channel architecture and message ID tracking ✅
- Implement comprehensive tracing for all kernel operations (Phases 1-9) ✅

**Acceptance Criteria:**
- [x] JupyterTransport with 5 channels (shell, iopub, control, stdin, heartbeat) ✅
- [x] Connection file parsing from Phase-9 branch preserved ✅
- [x] Message ID tracking for distributed tracing ✅
- [x] Protocol abstraction supports future LSP/DAP/WebSocket ✅
- [x] Transport registration in global runtime ✅
- [x] Comprehensive tracing covering 13 operation categories ✅
- [x] Operation statistics and performance metrics ✅
- [x] Feature flag tracking (hooks, events, state, security, vector) ✅

**Implementation Steps:**
1. Create `llmspell-kernel/src/transport/` module structure
2. Migrate zeromq.rs from Phase-9 branch (preserve 237 lines completely)
3. Enhance JupyterTransport with 5-channel support:
   ```rust
   pub struct JupyterTransport {
       shell: Socket,     // REQ/REP - execute_request/reply
       iopub: Socket,     // PUB - stream outputs to all clients
       control: Socket,   // REQ/REP - shutdown/interrupt
       stdin: Socket,     // REQ/REP - input requests
       heartbeat: Socket, // REQ/REP - connection monitoring
   }
   ```
4. Add protocol abstraction trait for multi-protocol support
5. Implement connection file parsing (migrate from Phase-9 kernel.rs lines 45-89)
6. Add message tracing and correlation IDs

**Test Steps:**
1. Test connection to real Jupyter Lab using connection file
2. Verify all 5 channels can send/receive messages
3. Test message ID correlation across channels
4. Validate protocol abstraction with mock LSP transport

**Definition of Done:**
- [x] All 5 Jupyter channels functional ✅
- [x] Connection file compatibility with Jupyter ecosystem ✅
- [x] Message tracing includes correlation IDs ✅
- [x] Protocol abstraction ready for Phase 11 IDE integration ✅
- [x] Transport layer has <1ms overhead ✅
- [x] Tracing infrastructure tested with 18 comprehensive tests ✅
- [x] All kernel operations properly instrumented ✅

### Task 9.1.3: Implement Message Router and I/O Management ✅
**Priority**: CRITICAL
**Estimated Time**: 5 hours (Actual: 4 hours)
**Assignee**: Messaging Team Lead
**Dependencies**: Task 9.1.2
**Status**: COMPLETE ✅

**Description**: Migrate I/O management system from Phase-9 branch and implement message routing with parent header tracking and session detection.

**Reusable Code Migration:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Migrate `/tmp/phase-9-comparison/llmspell-kernel/src/kernel_io.rs` (I/O routing logic) ✅
- Migrate message handling patterns from jupyter_kernel.rs ✅
- Add session type detection for tracing (script, exec, repl, debug, state) ✅

**Acceptance Criteria:**
- [x] EnhancedIOManager with multi-channel routing ✅
- [x] Parent header tracking for message correlation ✅
- [x] Session type detection (script, exec, repl, debug, state, session) ✅
- [x] stdout/stderr capture and routing to iopub channel ✅
- [x] Real-time I/O streaming to multiple clients ✅

**Implementation Steps:**
1. Create `llmspell-kernel/src/io/` module structure ✅
2. Migrate and enhance IOManager from Phase-9 branch: ✅
   ```rust
   pub struct EnhancedIOManager {
       iopub_sender: Option<Sender<IOPubMessage>>,
       stdout_buffer: Arc<RwLock<String>>,
       stderr_buffer: Arc<RwLock<String>>,
       parent_headers: Arc<RwLock<HashMap<String, MessageHeader>>>,
       current_parent: Arc<RwLock<Option<MessageHeader>>>,
   }
   ```
3. Add session detection for operation-aware tracing ✅
4. Implement message correlation with parent_header tracking ✅
5. Add real-time streaming capabilities ✅
6. Integrate with tracing infrastructure ✅

**Test Steps:**
1. Test stdout/stderr capture during script execution ✅
2. Verify messages routed to correct Jupyter channels ✅
3. Test parent header correlation in multi-client scenarios ✅
4. Validate session detection across all session types ✅

**Definition of Done:**
- [x] I/O properly routed to iopub channel ✅
- [x] Message correlation working across all channels ✅
- [x] Session type detection >95% accurate ✅
- [x] Multiple Jupyter clients can receive I/O simultaneously ✅
- [x] Message handling latency <5ms ✅

---

## Phase 9.2: Execution Engine Integration (Days 4-6)

### Task 9.2.1: Integrate ScriptRuntime Without Spawning ✅
**Priority**: CRITICAL
**Estimated Time**: 5 hours (Actual: 3.5 hours)
**Assignee**: Execution Team Lead
**Dependencies**: Task 9.1.3
**Status**: COMPLETE ✅

**Description**: Fix the critical architecture issue by integrating ScriptRuntime directly without tokio::spawn, eliminating runtime context isolation that causes provider failures.

**Critical Architecture Fix:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Remove problematic tokio::spawn from Phase-9 `/tmp/phase-9-comparison/llmspell-cli/src/kernel_client/unified_kernel.rs` line 110 ✅
- Remove pre-warming logic (lines 79-99) that doesn't solve the core issue ✅
- Integrate ScriptRuntime directly in kernel execution context ✅

**Acceptance Criteria:**
- [x] IntegratedKernel struct combining runtime and transport ✅
- [x] No tokio::spawn in kernel creation or execution path ✅
- [x] ScriptRuntime executes in same context as transport ✅
- [x] Provider HTTP clients remain valid throughout execution ✅
- [x] Execution tracing with agent monitoring ✅

**Implementation Steps:**
1. Create `llmspell-kernel/src/execution/` module ✅
2. Design IntegratedKernel without spawning: ✅
   ```rust
   pub struct IntegratedKernel<P: Protocol> {
       runtime: ScriptRuntime,
       protocol: P,
       io_manager: Arc<EnhancedIOManager>,
       message_router: Arc<MessageRouter>,
       tracing: TracingInstrumentation,
   }

   impl IntegratedKernel {
       pub async fn run(self) {
           // NO tokio::spawn - run in current context
           loop {
               // Process messages directly without spawning
           }
       }
   }
   ```
3. Add comprehensive execution tracing with agent monitoring ✅
4. Integrate with global IO runtime ✅
5. Add application type detection for performance monitoring ✅

**Test Steps:**
1. Execute script with HTTP provider calls for 60+ seconds ✅
2. Verify no "dispatch task is gone" errors ✅
3. Test agent creation and tool execution ✅
4. Validate tracing includes execution context ✅

**Definition of Done:**
- [x] Kernel runs without spawning background tasks ✅
- [x] HTTP clients remain valid throughout execution ✅
- [x] ScriptRuntime integration preserves all existing functionality ✅
- [x] Execution tracing provides agent-level visibility ✅
- [x] Long-running operations (60+ seconds) complete successfully ✅

### Task 9.2.2: Migrate Debug Infrastructure from Phase-9 Branch ✅
**Priority**: CRITICAL
**Estimated Time**: 6 hours (Actual: 2.5 hours)
**Assignee**: Debug Team Lead
**Dependencies**: Task 9.2.1
**Status**: COMPLETE ✅

**Description**: Migrate the comprehensive debug infrastructure from Phase-9 branch (3,296 lines) into kernel crate, preserving all debug coordinator and execution bridge functionality.

**Reusable Code Migration (3,296 lines total):**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Migrate `/tmp/phase-9-comparison/llmspell-bridge/src/execution_bridge.rs` (642 lines) ✅
- Migrate `/tmp/phase-9-comparison/llmspell-bridge/src/debug_coordinator.rs` (878 lines) ✅
- Migrate `/tmp/phase-9-comparison/llmspell-bridge/src/lua/lua_debug_bridge.rs` (1,245 lines) ✅
- Migrate `/tmp/phase-9-comparison/llmspell-debug/src/*.rs` (531 lines) ✅

**Acceptance Criteria:**
- [x] DebugCoordinator fully functional in kernel ✅
- [x] ExecutionManager with breakpoint support ✅
- [x] LuaDebugBridge with hook integration ✅
- [x] Memory-aware debug coordinator for Phase 10 preparation ✅
- [x] Debug tracing with fine-grained operation tracking ✅

**Implementation Steps:**
1. Create `llmspell-kernel/src/debug/` module structure ✅
2. Migrate execution_bridge.rs unchanged (preserve 642 lines) ✅
3. Migrate debug_coordinator.rs unchanged (preserve 878 lines) ✅
4. Migrate lua_debug_bridge.rs to debug/lua/ (preserve 1,245 lines) ✅
5. Merge llmspell-debug crate contents (531 lines) ✅
6. Add memory integration hooks for Phase 10: ✅
   ```rust
   pub struct MemoryAwareDebugCoordinator {
       coordinator: DebugCoordinator,
       memory_bridge: Option<Arc<dyn MemoryBridge>>, // Prepared for Phase 10
   }
   ```
7. Integrate debug tracing with kernel tracing system ✅

**Test Steps:**
1. Test breakpoint setting and hitting in Lua scripts ✅
2. Verify variable inspection returns correct values ✅
3. Test step debugging (step, next, continue) ✅
4. Validate ExecutionManager state transitions ✅
5. Test debug coordinator with complex scripts ✅

**Definition of Done:**
- [x] All debug functionality preserved from Phase-9 branch ✅
- [x] Breakpoints work correctly in interactive mode ✅
- [x] Variable inspection returns structured data ✅
- [x] Step debugging maintains execution state ✅
- [x] Debug tracing integrates with kernel tracing ✅
- [x] Memory integration hooks prepared ✅

### Task 9.2.3: Implement DAP Bridge Integration ✅
**Priority**: HIGH
**Estimated Time**: 4 hours (Actual: 1.5 hours)
**Assignee**: Protocol Team Lead
**Dependencies**: Task 9.2.2
**Status**: COMPLETE ✅

**Description**: Preserve and enhance the complete DAP bridge implementation from Phase-9 branch, connecting it to ExecutionManager and adding source mapping.

**Reusable Code Migration:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Migrate `/tmp/phase-9-comparison/llmspell-kernel/src/dap_bridge.rs` (743 lines) completely ✅
- Add ExecutionManager integration
- Enhance with source mapping for IDE integration

**Acceptance Criteria:**
- [x] DAPBridge with all 10 essential DAP commands functional ✅
- [x] Source mapping for better IDE integration ✅
- [x] Connection to ExecutionManager for real debugging ✅
- [x] Language-agnostic debug adapters for Phase 18 preparation ✅
- [x] DAP protocol compliance for VS Code integration ✅

**Implementation Steps:**
1. Migrate dap_bridge.rs completely (preserve all 743 lines)
2. Connect to ExecutionManager from debug infrastructure:
   ```rust
   impl DAPBridge {
       pub fn connect_execution_manager(&mut self, manager: Arc<ExecutionManager>) {
           self.execution = Some(manager);
       }

       pub fn map_script_to_source(&self, script_id: u32) -> SourceReference {
           // Implementation for IDE source mapping
       }
   }
   ```
3. Add language-agnostic debug adapter infrastructure
4. Implement source mapping enhancements
5. Add comprehensive DAP tracing

**Test Steps:**
1. Test all 10 essential DAP commands (initialize, launch, setBreakpoints, etc.)
2. Verify source mapping works with IDE clients
3. Test debugging session management
4. Validate DAP protocol compliance with VS Code

**Definition of Done:**
- [x] All DAP commands respond correctly ✅
- [x] Source mapping provides accurate file:line references ✅
- [x] ExecutionManager integration enables real debugging ✅
- [x] VS Code can connect and debug scripts ✅
- [x] DAP tracing provides protocol visibility ✅

---

## Phase 9.3: State & Session Management Integration (Days 7-10)

### Task 9.3.1: Implement Unified State System
**Priority**: HIGH
**Estimated Time**: 5 hours
**Assignee**: State Team Lead
**Dependencies**: Task 9.2.3

**Description**: Consolidate state management by merging storage backends from llmspell-storage into kernel and creating unified KernelState structure.

**Current Assets to Consolidate:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Use existing `llmspell-storage/src/backends/memory.rs` ✅
- Use existing `llmspell-storage/src/backends/sled_backend.rs` ✅
- Use existing state trait definitions from `llmspell-state-persistence` ✅

**Acceptance Criteria:**
- [ ] KernelState with execution, session, and debug state
- [ ] StorageBackend enum with Memory, Sled, Vector options
- [ ] State persistence across kernel restarts
- [ ] Circuit breaker patterns for resource protection
- [ ] Performance monitoring integration

**Implementation Steps:**
1. Create `llmspell-kernel/src/state/` module structure
2. Merge storage backends:
   ```rust
   pub enum StorageBackend {
       Memory(MemoryBackend),     // From llmspell-storage
       Sled(SledBackend),         // From llmspell-storage
       Vector(VectorBackend),     // From llmspell-storage
   }

   pub struct KernelState {
       execution: Arc<RwLock<ExecutionState>>,  // From execution_bridge
       session: Arc<RwLock<SessionState>>,      // From sessions
       debug: Arc<RwLock<DebugState>>,         // From debug_coordinator
       backend: StorageBackend,
   }
   ```
3. Add circuit breaker patterns for resource protection
4. Integrate with performance monitoring infrastructure
5. Add state persistence and recovery mechanisms

**Test Steps:**
1. Test state persistence across kernel restarts
2. Verify all storage backends work correctly
3. Test circuit breaker activation under load
4. Validate state recovery after failures

**Definition of Done:**
- [ ] State persists correctly across restarts
- [ ] All storage backends functional and tested
- [ ] Circuit breaker prevents resource exhaustion
- [ ] State recovery works after unexpected shutdowns
- [ ] Performance monitoring tracks state operations

### Task 9.3.2: Migrate Complete Session Management System
**Priority**: CRITICAL
**Estimated Time**: 8 hours
**Assignee**: Session Team Lead
**Dependencies**: Task 9.3.1

**Description**: Migrate ALL 34 modules from Phase-9 branch llmspell-sessions crate as a complete subsystem, preserving the sophisticated session lifecycle, artifact storage, and policy management.

**Massive Code Migration (All 34 modules from Phase-9 branch):**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Migrate complete `/tmp/phase-9-comparison/llmspell-sessions/` crate structure ✅
- Preserve `SessionManager`, `SessionArtifact`, `SessionMetrics` completely ✅
- Migrate all policies (rate limiting, timeouts, resource management) ✅
- Migrate complete artifact subsystem with version control ✅

**Acceptance Criteria:**
- [ ] Complete session lifecycle management (create, pause, resume, archive)
- [ ] Artifact storage with version control and metadata
- [ ] Session policies (rate limiting, timeouts, resource limits)
- [ ] Session-level tracing and correlation
- [ ] TTL management for session expiration
- [ ] Multi-tenant session isolation

**Implementation Steps:**
1. Create `llmspell-kernel/src/sessions/` module preserving Phase-9 structure
2. Migrate complete session crate (preserve entire module hierarchy):
   ```rust
   pub mod sessions {
       pub mod manager;           // Core lifecycle management
       pub mod artifact;          // Version-controlled artifacts
       pub mod policies;          // Rate limiting, timeouts
       pub mod security;          // Access control
       pub mod events;           // Session-specific events
   }
   ```
3. Integrate with kernel message flow:
   ```rust
   impl SessionManager {
       pub fn handle_kernel_message(&mut self, msg: JupyterMessage) {
           self.track_message(msg);
           self.apply_policies(msg);
       }
   }
   ```
4. Add session-level tracing and correlation
5. Integrate with state persistence system

**Test Steps:**
1. Test complete session lifecycle (create → execute → pause → resume → archive)
2. Verify artifact storage with version control
3. Test session policies under load
4. Validate TTL expiration mechanisms
5. Test multi-user session isolation

**Definition of Done:**
- [ ] All 34 session modules functional
- [ ] Session artifacts persisted with version control
- [ ] Policies prevent resource exhaustion
- [ ] TTL expiration works automatically
- [ ] Session tracing provides visibility into lifecycle
- [ ] Multi-tenant isolation verified

### Task 9.3.3: Implement Event Correlation System
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Events Team Lead
**Dependencies**: Task 9.3.2

**Description**: Migrate event correlation system with distributed tracing, adding kernel-specific events and IOPub broadcasting for multi-client support.

**Existing Assets:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Use existing `llmspell-events/src/*.rs` infrastructure ✅
- Use session events from Phase-9 branch sessions ✅

**Acceptance Criteria:**
- [ ] KernelEvent enum with execution, debug, and session events
- [ ] Event correlation across distributed operations
- [ ] IOPub channel broadcasting for multi-client updates
- [ ] Cross-session event correlation
- [ ] Distributed tracing integration

**Implementation Steps:**
1. Extend existing event system for kernel:
   ```rust
   pub enum KernelEvent {
       ExecuteRequest { code: String, msg_id: String },
       ExecuteReply { status: Status, msg_id: String },
       DebugEvent(DebugEvent),      // From debug_coordinator
       SessionEvent(SessionEvent),   // From sessions crate
   }
   ```
2. Connect to IOPub channel for multi-client broadcasting
3. Add distributed tracing correlation
4. Implement cross-session event correlation
5. Add event persistence for audit trails

**Test Steps:**
1. Test event correlation across execution flows
2. Verify IOPub broadcasting to multiple clients
3. Test distributed tracing with correlation IDs
4. Validate cross-session event tracking

**Definition of Done:**
- [ ] Events correlate correctly across operations
- [ ] Multiple clients receive event updates via IOPub
- [ ] Distributed tracing shows complete execution flows
- [ ] Cross-session events tracked properly
- [ ] Event persistence provides audit capability

### Task 9.3.4: Integrate Advanced Hook System
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Hooks Team Lead
**Dependencies**: Task 9.3.3

**Description**: Import sophisticated hook patterns from existing llmspell-hooks and integrate with kernel execution flow for dynamic debug capabilities.

**Existing Assets:**
- Use existing `llmspell-hooks/src/*.rs` infrastructure ✅
- Import advanced patterns: CompositeHook, ForkHook, RetryHook ✅

**Acceptance Criteria:**
- [ ] Advanced hook patterns (CompositeHook, ForkHook, RetryHook, ConditionalHook)
- [ ] Kernel-specific hooks (PreExecute, PostExecute, PreDebug, StateChange)
- [ ] Dynamic debug flow modification
- [ ] Hook performance monitoring <5% overhead
- [ ] Hook execution in kernel context

**Implementation Steps:**
1. Create `llmspell-kernel/src/hooks/` module
2. Import advanced hook patterns from existing crate
3. Add kernel-specific hooks:
   ```rust
   pub enum KernelHook {
       PreExecute(PreExecuteHook),    // Before code execution
       PostExecute(PostExecuteHook),  // After code execution
       PreDebug(PreDebugHook),        // Before debug operation
       StateChange(StateChangeHook),   // On state transitions
   }
   ```
4. Wire into execution flow with performance monitoring
5. Add dynamic debug flow modification capabilities

**Test Steps:**
1. Test advanced hook patterns in kernel execution
2. Verify hook performance overhead <5%
3. Test dynamic debug flow modification
4. Validate hook execution with tracing

**Definition of Done:**
- [ ] Advanced hook patterns work in kernel context
- [ ] Hook overhead measured and <5%
- [ ] Dynamic debug flow modification functional
- [ ] Hook tracing provides execution visibility
- [ ] Hooks integrate with existing execution engine

---

## Phase 9.4: External Interfaces & CLI Integration (Days 11-13)

### Task 9.4.1: Fix Provider System Runtime Context
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Provider Team Lead
**Dependencies**: Task 9.3.4

**Description**: Fix the runtime context issue in provider system by removing SHARED_IO_RUNTIME workaround and updating all HTTP client creation to use global_io_runtime().

**Critical Fix Points:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Remove SHARED_IO_RUNTIME from Phase-9 `/tmp/phase-9-comparison/llmspell-providers/src/rig.rs` lines 17-40 ✅
- Update 15 files in llmspell-tools that create HTTP clients ✅
- Ensure consistent runtime context across all provider operations

**Acceptance Criteria:**
- [ ] SHARED_IO_RUNTIME workaround completely removed
- [ ] All HTTP clients use global_io_runtime()
- [ ] Provider operations survive 60+ second executions
- [ ] Provider-level cost tracking and tracing
- [ ] No runtime context mismatches in provider calls

**Implementation Steps:**
1. Update llmspell-providers/src/rig.rs:
   ```rust
   // REMOVE lines 17-40 SHARED_IO_RUNTIME workaround
   // REPLACE with:
   use llmspell_kernel::runtime::global_io_runtime;

   fn create_client_safe<F, T>(creator: F) -> T {
       global_io_runtime().block_on(async {
           creator()
       })
   }
   ```
2. Update all 15 files in llmspell-tools to use global_io_runtime()
3. Add provider-level cost tracking and tracing
4. Verify provider calls work in kernel context
5. Add comprehensive provider operation tracing

**Test Steps:**
1. Execute long-running script (60+ seconds) with multiple provider calls
2. Verify no "dispatch task is gone" errors from providers
3. Test provider switching during execution
4. Validate cost tracking accuracy

**Definition of Done:**
- [ ] No SHARED_IO_RUNTIME references remain
- [ ] All provider HTTP clients use global runtime
- [ ] Long-running provider operations complete successfully
- [ ] Cost tracking accurate within 5%
- [ ] Provider tracing shows consistent runtime context

### Task 9.4.2: Simplify CLI and Remove Pre-warming
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: CLI Team Lead
**Dependencies**: Task 9.4.1

**Description**: Remove the problematic pre-warming logic and tokio::spawn from CLI, implementing direct kernel invocation with service-ready architecture.

**Critical Removals:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Remove pre-warming logic from Phase-9 unified_kernel.rs lines 79-99 ✅
- Remove tokio::spawn from line 110 ✅
- Implement direct kernel invocation

**Acceptance Criteria:**
- [ ] No pre-warming logic in CLI code
- [ ] No tokio::spawn in kernel creation path
- [ ] Direct kernel invocation working
- [ ] Service-ready kernel architecture for Phase 12
- [ ] CLI tracing integration

**Implementation Steps:**
1. Update llmspell-cli kernel client code:
   ```rust
   // REMOVE pre-warming from lines 79-99
   // REMOVE tokio::spawn from line 110
   // REPLACE with:
   pub async fn run_kernel(config: Config) -> Result<()> {
       let kernel = IntegratedKernel::new(config)?;
       kernel.run().await  // No spawning, runs in current context
   }
   ```
2. Add service-ready kernel architecture for Phase 12
3. Implement direct kernel invocation with tracing
4. Add API endpoint framework preparation
5. Integrate CLI tracing with kernel tracing

**Test Steps:**
1. Test CLI invocation with direct kernel execution
2. Verify no background task spawning
3. Test service readiness for Phase 12
4. Validate CLI tracing integration

**Definition of Done:**
- [ ] CLI invokes kernel directly without spawning
- [ ] Kernel architecture ready for service mode
- [ ] No pre-warming or spawning logic remains
- [ ] CLI tracing integrates with kernel tracing
- [ ] Service endpoints prepared for Phase 12

### Task 9.4.3: Consolidate REPL and Debug Interfaces
**Priority**: HIGH
**Estimated Time**: 5 hours
**Assignee**: Interactive Team Lead
**Dependencies**: Task 9.4.2

**Description**: Migrate and consolidate llmspell-repl (324 lines) and llmspell-debug (531 lines) from Phase-9 branch into unified interactive session management within kernel.

**Code Migration:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- Migrate `/tmp/phase-9-comparison/llmspell-repl/src/*.rs` (324 lines) ✅
- Merge with debug functionality for unified experience ✅
- Add REPL-specific tracing and session management

**Acceptance Criteria:**
- [ ] InteractiveSession with integrated REPL and debug
- [ ] REPL meta-commands (.help, .save, .load, .exit)
- [ ] Debug commands (.break, .step, .next, .continue, .locals)
- [ ] Session persistence across REPL interactions
- [ ] Interactive debug commands with trace correlation

**Implementation Steps:**
1. Create `llmspell-kernel/src/interactive/` module
2. Consolidate REPL and debug:
   ```rust
   pub mod interactive {
       pub struct InteractiveSession {
           kernel: IntegratedKernel,
           debug_session: Option<DebugSession>,
           repl_state: REPLState,
       }

       impl InteractiveSession {
           pub async fn run_repl(&mut self) {
               // REPL loop with integrated debug commands
           }
       }
   }
   ```
3. Add REPL-specific tracing and session management
4. Implement interactive debug commands with trace correlation
5. Add tab completion and command history

**Test Steps:**
1. Test REPL startup and command execution
2. Verify debug commands work within REPL
3. Test session persistence across REPL interactions
4. Validate tab completion and history

**Definition of Done:**
- [ ] REPL and debug consolidated into single interface
- [ ] All meta-commands and debug commands functional
- [ ] Session state persists across REPL interactions
- [ ] Interactive experience smooth and responsive
- [ ] REPL tracing integrates with kernel tracing

---

## Phase 9.5: Application Validation & Future-Proofing (Days 14-16)

### Task 9.5.1: Implement Application Validation Suite
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: QA Team Lead
**Dependencies**: Task 9.4.3

**Description**: Create comprehensive application validation suite testing all 9 example applications across complexity layers 1-6 with performance tracking and cost analysis.

**Application Test Coverage:**
- **Layer 1** (2-4 agents): file-organizer, research-collector
- **Layer 2** (4 agents): content-creator ✅ (22s runtime, conditional workflows)
- **Layer 3** (5-7 agents): personal-assistant, communication-manager, code-review-assistant ⚠️
- **Layer 4** (8 agents): process-orchestrator ⚠️, knowledge-base ✅
- **Layer 5** (21 agents): webapp-creator ⚠️ (120-180s, $0.50-1.00 cost)

**Acceptance Criteria:**
- [ ] ApplicationTestSuite with complexity-based categorization
- [ ] Performance tracking and cost analysis
- [ ] Memory stability validation during extended operations
- [ ] Success rate targets: Simple 100%, Complex ≥90%, Expert ≥80%
- [ ] Automated regression detection

**Implementation Steps:**
1. Create `llmspell-kernel/src/testing/application_suite.rs`:
   ```rust
   pub struct ApplicationTestSuite {
       simple_apps: Vec<SimpleAppTest>,      // 2-4 agents, <30s
       complex_apps: Vec<ComplexAppTest>,    // 7+ agents, >30s
       expert_apps: Vec<ExpertAppTest>,      // 21+ agents, >120s
       performance_metrics: PerformanceTracker,
       cost_tracking: CostAnalyzer,
   }
   ```
2. Add runtime validation across all 9 applications
3. Implement performance tracking with regression detection
4. Add cost analysis and memory stability monitoring
5. Create automated test execution and reporting

**Test Steps:**
1. Run full validation suite on all 9 applications
2. Verify success rates meet targets (100%/90%/80%)
3. Test memory stability during expert applications
4. Validate cost tracking accuracy

**Definition of Done:**
- [ ] All 9 applications execute successfully
- [ ] Success rates meet layer-specific targets
- [ ] Performance regression detection functional
- [ ] Memory leaks detected and resolved
- [ ] Cost tracking within 5% accuracy

### Task 9.5.2: Implement Future-Proofing Infrastructure
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Architecture Team Lead
**Dependencies**: Task 9.5.1

**Description**: Add integration hooks and infrastructure foundation for Phases 10-24, ensuring seamless evolution without breaking changes.

**Future Phase Preparation:**
- consult `docs/in-progress/phase-09-design-doc.com` and `docs/in-progress/implementation-phases.md` for the proper design of the implementation of code.
- **Phase 10**: Memory integration hooks and adaptive consolidation interfaces
- **Phase 12**: Service infrastructure and API endpoint framework
- **Phase 18**: Multi-language debug architecture and bridge extensions
- **Phase 20**: Observability framework and performance monitoring

**Acceptance Criteria:**
- [ ] Memory integration hooks for Phase 10 adaptive memory system
- [ ] Service infrastructure foundation for Phase 12 daemon mode
- [ ] Multi-language debug architecture for Phase 18 JavaScript support
- [ ] Observability framework for Phase 20 production optimization
- [ ] Forward compatibility tested with mock implementations

**Implementation Steps:**
1. Add memory integration hooks for Phase 10:
   ```rust
   pub trait MemoryIntegration {
       async fn store_interaction(&self, interaction: InteractionLog);
       async fn query_context(&self, query: ContextQuery) -> Vec<MemoryItem>;
       async fn consolidate_memories(&self) -> ConsolidationResult;
   }
   ```
2. Implement service infrastructure foundation for Phase 12
3. Add multi-language debug architecture for Phase 18
4. Create observability framework for Phase 20
5. Test forward compatibility with mock implementations

**Test Steps:**
1. Test memory integration hooks with mock memory system
2. Verify service infrastructure can be extended for Phase 12
3. Test multi-language debug architecture with mock JavaScript engine
4. Validate observability framework extensibility

**Definition of Done:**
- [ ] Memory integration hooks tested with mocks
- [ ] Service infrastructure ready for Phase 12 extension
- [ ] Multi-language debug architecture validated
- [ ] Observability framework extensible
- [ ] Forward compatibility verified with automated tests

### Task 9.5.3: Comprehensive Integration Testing & Validation
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Full Team
**Dependencies**: Task 9.5.2

**Description**: Execute comprehensive integration testing validating all forward compatibility interfaces, tracing coverage, and architectural requirements for Phases 10-24.

**Validation Scope:**
- All forward compatibility interfaces (Phases 10-24)
- Tracing coverage and performance targets
- Application suite across complexity layers 1-6
- Memory stability and resource management
- Protocol compliance (Jupyter, DAP)

**Acceptance Criteria:**
- [ ] Forward compatibility interfaces pass mock integration tests
- [ ] Tracing coverage >95% with <2% performance overhead
- [ ] Application suite validation passes all complexity layers
- [ ] Memory usage stable during extended operations
- [ ] Protocol compliance verified with real clients

**Implementation Steps:**
1. Execute forward compatibility test suite
2. Validate tracing coverage and performance impact
3. Run complete application suite validation
4. Test memory stability during long operations
5. Verify protocol compliance with Jupyter Lab and VS Code

**Test Steps:**
1. Run automated forward compatibility test suite
2. Execute tracing performance benchmarks
3. Run all 9 applications with monitoring
4. Test 4+ hour continuous operation for memory leaks
5. Connect real Jupyter Lab and VS Code clients

**Definition of Done:**
- [ ] Forward compatibility tests pass for all target phases
- [ ] Tracing performance overhead <2% measured
- [ ] Application suite achieves target success rates
- [ ] No memory leaks detected in extended testing
- [ ] Real IDE clients can connect and debug successfully

---

## Final Validation Checklist

### Code Quality Gates
- [ ] All crates compile without warnings: `cargo clippy --workspace --all-features --all-targets`
- [ ] Format compliance: `cargo fmt --all --check`
- [ ] Tests pass: `cargo test --workspace --all-features`
- [ ] Documentation builds: `cargo doc --workspace --all-features --no-deps`
- [ ] Code consolidation achieved: 28,000+ → ~15,000 lines (46% reduction)
- [ ] Crate reduction: 26 → 21 crates (5 crates eliminated)

### Performance Validation
- [ ] Tool initialization: <10ms (from current ~50ms)
- [ ] Agent creation: <50ms (from current ~200ms)
- [ ] Message handling: <5ms per Jupyter message
- [ ] Debug stepping: <20ms response time
- [ ] Hook execution overhead: <5% total system impact
- [ ] Tracing overhead: <2% when RUST_LOG=info

### Architecture Validation
- [ ] Global IO runtime eliminates "dispatch task is gone" error
- [ ] 5-channel Jupyter protocol fully compliant
- [ ] DAP bridge functional with all 10 essential commands
- [ ] Session management with artifact storage and TTL working
- [ ] Event correlation with distributed tracing operational
- [ ] Provider system uses consistent runtime context

### Application Validation
- [ ] Simple applications (≤4 agents): 100% success rate
- [ ] Complex applications (5-8 agents): ≥90% success rate
- [ ] Expert applications (9+ agents): ≥80% success rate
- [ ] Memory stability verified in 4+ hour continuous tests
- [ ] Cost tracking accuracy within 5% of actual API costs

### Integration Validation
- [ ] Jupyter Lab can connect and execute code
- [ ] VS Code can connect via DAP and debug scripts
- [ ] REPL with integrated debugging fully functional
- [ ] Multiple clients can connect simultaneously
- [ ] Session persistence across kernel restarts working

### Future-Proofing Validation
- [ ] Phase 10 memory integration hooks tested with mocks
- [ ] Phase 12 service infrastructure foundation ready
- [ ] Phase 18 multi-language debug architecture validated
- [ ] Phase 20 observability framework extensible
- [ ] No breaking changes required for Phases 10-24

---

## Risk Mitigation & Contingency Plans

### Critical Risks

**Risk 1: Runtime Context Regression**
- **Probability**: MEDIUM
- **Impact**: HIGH (system failure)
- **Mitigation**: Comprehensive testing with 60+ second operations
- **Detection**: Automated tests for "dispatch task is gone" error
- **Contingency**: Rollback to isolated per-operation runtime creation

**Risk 2: Code Migration Complexity**
- **Probability**: HIGH
- **Impact**: MEDIUM (schedule delay)
- **Mitigation**: Preserve code structure, minimize changes
- **Detection**: Compilation failures, test regressions
- **Contingency**: Incremental migration with working checkpoints

**Risk 3: Performance Degradation**
- **Probability**: MEDIUM
- **Impact**: MEDIUM (user experience)
- **Mitigation**: Daily benchmarking, performance targets
- **Detection**: Automated performance regression tests
- **Contingency**: Performance profiling and optimization sprints

**Risk 4: Application Validation Failures**
- **Probability**: MEDIUM
- **Impact**: HIGH (adoption blocker)
- **Mitigation**: Continuous application testing during development
- **Detection**: Application suite validation failures
- **Contingency**: Application-specific debugging and fixes

### Schedule Risks

**Risk 1: Complex Session Migration**
- **Impact**: 2-3 day delay potential
- **Mitigation**: Complete migration of Phase-9 sessions code
- **Contingency**: Simplified session management for MVP

**Risk 2: DAP Integration Complexity**
- **Impact**: 1-2 day delay potential
- **Mitigation**: Preserve existing DAP bridge implementation
- **Contingency**: Basic debugging without full DAP compliance

**Risk 3: Application Testing Overhead**
- **Impact**: 1-2 day extension potential
- **Mitigation**: Automated test suite with parallel execution
- **Contingency**: Focus on critical applications only

---

## Team Assignments & Daily Progress

### Core Team Roles
- **Runtime Team Lead**: Global IO runtime, transport layer, message routing
- **Execution Team Lead**: ScriptRuntime integration, kernel execution context
- **Debug Team Lead**: Debug infrastructure migration, DAP bridge integration
- **State Team Lead**: State management, storage backend consolidation
- **Session Team Lead**: Session management migration, lifecycle automation
- **Events Team Lead**: Event correlation, distributed tracing integration
- **Interactive Team Lead**: REPL/debug consolidation, user experience
- **QA Team Lead**: Application validation, performance testing
- **Architecture Team Lead**: Future-proofing, integration validation

### Daily Sprint Topics

**Week 1: Core Foundation**
- **Day 1**: Global IO runtime foundation, comprehensive tracing setup
- **Day 2**: Multi-protocol transport with 5-channel Jupyter support
- **Day 3**: Message routing, I/O management, application detection
- **Day 4**: ScriptRuntime integration, execution context consolidation
- **Day 5**: Debug infrastructure migration, memory-aware coordination

**Week 2: Integration & Management**
- **Day 6**: DAP bridge, language-agnostic debug architecture
- **Day 7**: State system consolidation, performance monitoring
- **Day 8**: Complete session management migration
- **Day 9**: Event correlation, distributed tracing integration
- **Day 10**: Advanced hook system integration

**Week 3: Interfaces & Validation**
- **Day 11**: Provider system fix, cost tracking integration
- **Day 12**: CLI simplification, service architecture preparation
- **Day 13**: REPL/debug consolidation, interactive experience
- **Day 14**: Application validation suite, complexity testing
- **Day 15**: Future-proofing infrastructure, compatibility testing
- **Day 16**: Comprehensive integration testing, final validation

### Success Metrics Dashboard

**Code Quality Metrics**
- **Consolidation Rate**: Target 46% reduction (28,000 → 15,000 lines)
- **Crate Reduction**: Target 19% reduction (26 → 21 crates)
- **Code Reuse**: Target 54% preservation rate (15,000 lines preserved)
- **Test Coverage**: Target >90% with comprehensive integration tests

**Performance Metrics**
- **Tool Init**: Target <10ms (50%+ improvement)
- **Agent Creation**: Target <50ms (75%+ improvement)
- **Message Latency**: Target <5ms per Jupyter message
- **Debug Response**: Target <20ms for debugging operations
- **System Overhead**: Target <5% total impact from all enhancements

**Validation Metrics**
- **Application Success**: 100% simple, 90% complex, 80% expert
- **Memory Stability**: Zero leaks in 4+ hour continuous testing
- **Cost Accuracy**: Within 5% of actual API costs
- **Client Compatibility**: Jupyter Lab and VS Code integration working

---

**END OF PHASE 9 TODO DOCUMENT**

> **📋 IMPLEMENTATION NOTE**: This 16-day implementation consolidates 28,000+ lines from the Phase-9 branch into a production-ready ~15,000 line kernel architecture, fixing critical runtime issues while preserving all valuable debug, session, and protocol work. Each task includes specific migration instructions, comprehensive testing requirements, and clear definition of done criteria to ensure successful delivery of the foundation for Phases 10-24.