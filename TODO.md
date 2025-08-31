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
- [x] Kernel service starts as standalone process in <100ms
- [x] Multiple clients (CLI, web, IDE) connect to same kernel
- [x] LRP/LDP protocols enable message-based communication
- [x] Connection discovery via JSON files works
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

---

## 🏗️ **PHASE 9.2 ARCHITECTURAL FOUNDATION COMPLETE**

### ✅ **CORE ARCHITECTURAL ACHIEVEMENTS:**

**1. Three-Layer Architecture (9.2.7b)** - **FOUNDATION FOR ALL FUTURE WORK**
- **Bridge Layer** (`src/*.rs`): Script-agnostic traits - ZERO script engine imports
- **Shared Layer**: Common implementation logic for all languages  
- **Script Layer** (`src/lua/*_impl.rs`): Language-specific implementations
- **Implemented Traits**:
  - `ConditionEvaluator` trait → `LuaConditionEvaluator` implementation
  - `VariableInspector` trait → `LuaVariableInspector` implementation
  - `DebugStateCache` trait → `LuaDebugStateCache` implementation
  - `StackNavigator` trait → `LuaStackNavigator` implementation

**2. Execution vs Diagnostics Separation**
- **ExecutionBridge**: Breakpoints, stepping, debugging, execution control
- **DiagnosticsBridge**: Logging, profiling, error reporting, tracing, observability
- **SharedExecutionContext**: Unified state with performance metrics and correlation IDs
- **Clean separation maintained**: No mixing of execution and diagnostics concerns

**3. Complete Debug Infrastructure Components**
- **Interactive Debugger (9.2.1)**: Multi-client ExecutionManager-based debugging
- **Session Management (9.2.2)**: Resource isolation and concurrent session handling
- **Debug Hooks (9.2.3)**: Lua hook integration with `block_on_async` pattern
- **Conditional Breakpoints (9.2.4)**: Bytecode-cached condition evaluation
- **Step Debugging (9.2.5)**: Automatic mode transitions and step execution
- **Variable Inspector (9.2.6)**: Slow path inspection with LRU caching
- **Watch Expressions (9.2.8)**: Cached expression evaluation with batching
- **Stack Navigation (9.2.9)**: Read-only frame navigation using cached stacks
- **Async Integration (9.2.10)**: Context preservation across async boundaries
- **Distributed Tracing (9.2.11)**: OpenTelemetry + OTLP with trace enrichment

**4. Performance Architecture**
- **Fast Path**: Atomic operations, <1ms execution when debugging inactive
- **Slow Path**: Complex operations only triggered during active debugging
- **Performance Targets Achieved**: <10% debug overhead, <5% tracing overhead
- **Sync/Async Bridging**: `block_on_async` for Lua hooks (multi-threaded runtime required)

**5. Quality Gates and Testing (9.2.12)**
- **Architecture Compliance**: Zero mlua imports verified in bridge layer
- **Comprehensive Testing**: 151+ tests with trait-based patterns
- **Protocol Compliance**: LRP/LDP protocol definitions validated
- **Documentation Coverage**: Complete API documentation for trait-based architecture
- **Performance Validation**: All targets met and measured

### 🔧 **MANDATORY PATTERNS FOR REMAINING TASKS (9.3-9.6):**

**Architecture Compliance (NON-NEGOTIABLE)**:
```bash
# Verify zero script imports in bridge layer:
find src -maxdepth 1 -name "*.rs" -exec grep -l "use mlua" {} \;  # Must return empty

# Use unified types from execution_bridge.rs:
StackFrame, Breakpoint, Variable (not custom debug types)

# Follow diagnostics vs execution separation:
DiagnosticsBridge: Hot reload, validation, profiling, error reporting
ExecutionBridge: Breakpoints, stepping, debugging, execution control
```

**Integration Patterns for Specific Task Types**:
- **Hot Reload/Validation (9.3.1-9.3.2)** → DiagnosticsBridge for error reporting
- **Profiling/Performance (9.3.3, 9.6.1)** → SharedExecutionContext.performance_metrics
- **LSP/DAP/IDE (9.4.6-9.4.7)** → ExecutionManager for debugging features
- **CLI Commands (9.5.2)** → ExecutionBridge and DiagnosticsBridge interfaces

**Testing Requirements**:
- **Multi-threaded runtime**: `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`
- **No test behavior changes**: Separate integration test binaries, no `#[cfg(test)]`
- **Architecture validation**: Verify trait usage and layer separation in all tests

### 🎯 **IMPLEMENTATION PRIORITY FOR REMAINING PHASES:**
1. **Phase 9.3**: DevEx features using DiagnosticsBridge (hot reload, validation, profiling)
2. **Phase 9.4**: Kernel integration using ExecutionBridge (LSP/DAP, multi-client debugging)
3. **Phase 9.5**: CLI and configuration updates using established bridge interfaces
4. **Phase 9.6**: Final validation, optimization, and architectural compliance testing

### 🚀 **ARCHITECTURAL FOUNDATION IS COMPLETE**
**All remaining tasks can now build confidently on this solid three-layer foundation with clear separation of concerns, comprehensive debugging capabilities, and future-ready extensibility for JavaScript and Python support.**

---

## Phase 9.3: Development Experience Features (Days 7-9)

### Task 9.3.1: Hot Reload System
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: DevEx Team Lead

**Description**: File watching and hot reload with state preservation using Phase 9.2 three-layer architecture, trait-based validation, and distributed tracing integration.

**ARCHITECTURE FOUNDATION (Phase 9.2):**
- **DiagnosticsBridge Integration**: Hot reload validation and error reporting via established tracing
- **SharedExecutionContext**: State preservation with async integration (9.2.10) and performance metrics
- **Trait-Based Validation**: Uses DiagnosticsBridge validation methods (no custom validators)
- **Three-Layer Pattern**: Bridge layer traits, shared logic, script-specific implementations

**Acceptance Criteria:**
- [x] File watching integrated with DiagnosticsBridge tracing for observability
- [x] State preservation uses SharedExecutionContext async integration patterns
- [x] Validation leverages established DiagnosticsBridge.validate_script methods
- [x] Error reporting via DiagnosticsBridge with trace enrichment (9.2.11)
- [x] Debouncing prevents reload storms with performance metrics tracking
- [x] Multi-file watching preserves separate SharedExecutionContext per file

**Implementation Steps:**
1. **Enhance DiagnosticsBridge with hot reload capabilities** (follows 9.2 pattern):
   ```rust
   // llmspell-bridge/src/diagnostics_bridge.rs - add hot reload methods
   use crate::execution_context::{SharedExecutionContext, SourceLocation};
   use notify::{Watcher, RecommendedWatcher, Event};
   
   impl DiagnosticsBridge {
       pub fn enable_hot_reload(&mut self, paths: Vec<PathBuf>) -> Result<()> {
           let (tx, rx) = mpsc::channel();
           let mut watcher = RecommendedWatcher::new(tx, Duration::from_millis(100))?;
           
           for path in paths {
               watcher.watch(&path, RecursiveMode::NonRecursive)?;
           }
           
           self.hot_reload_watcher = Some(watcher);
           self.hot_reload_receiver = Some(rx);
           Ok(())
       }
       
       pub async fn handle_file_change(
           &self, 
           path: &Path, 
           context: &mut SharedExecutionContext
       ) -> Result<bool> {
           // Create trace span for hot reload operation
           let _span = self.trace_execution("hot_reload", context);
           
           // Preserve async context using 9.2.10 patterns
           let snapshot = context.preserve_across_async_boundary();
           
           // Read and validate using established DiagnosticsBridge methods
           let script_content = fs::read_to_string(path).await?;
           match self.validate_script(&script_content, context) {
               Ok(_) => {
                   // Log successful reload via tracing
                   self.log_with_metadata(
                       "info", 
                       &format!("Hot reload: {}", path.display()),
                       Some("hot_reload"),
                       serde_json::json!({ "file": path, "success": true })
                   );
                   
                   // Restore context after async boundary
                   context.restore_from_async_boundary(snapshot);
                   Ok(true)
               },
               Err(validation_errors) => {
                   // Report validation errors via tracing
                   for error in validation_errors {
                       self.trace_diagnostic(&error, "error");
                   }
                   Ok(false) // Don't reload on validation failure
               }
           }
       }
   }
   ```
2. **Implement file watching within DiagnosticsBridge architecture**
3. **Use SharedExecutionContext async patterns from 9.2.10**
4. **Leverage validation methods from enhanced DiagnosticsBridge**
5. **Add distributed tracing for hot reload observability**
6. **Test with multiple files and async context preservation**

**Definition of Done:**
- [x] File changes detected
- [x] State preserved on reload
- [x] Validation prevents bad reloads
- [x] <500ms reload time
- [x] `cargo fmt --all --check` passes
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes

### Task 9.3.2: Script Validation System
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: DevEx Team

**Description**: Comprehensive script validation using Phase 9.2 three-layer architecture, trait-based evaluation patterns, and distributed tracing for validation observability.

**ARCHITECTURE FOUNDATION (Phase 9.2):**
- **DiagnosticsBridge Extension**: Built-in validation leveraging distributed tracing (9.2.11)
- **ConditionEvaluator Trait**: Reuses bytecode compilation patterns for syntax validation
- **SharedExecutionContext Integration**: Context-aware validation with async boundaries (9.2.10)
- **Three-Layer Validation**: Bridge traits, shared validation logic, Lua-specific syntax checking

**Acceptance Criteria:**
- [ ] Syntax validation reuses ConditionEvaluator bytecode compilation patterns
- [ ] API validation leverages VariableInspector trait for context analysis
- [ ] Security pattern detection via DiagnosticsBridge tracing integration
- [ ] Performance validation uses SharedExecutionContext.performance_metrics
- [ ] Validation reports enriched with trace spans for observability
- [ ] Multi-file validation preserves SharedExecutionContext per script

**Implementation Steps:**
1. **Enhance DiagnosticsBridge with validation capabilities** (don't create ScriptValidator):
   ```rust
   // llmspell-bridge/src/diagnostics_bridge.rs - add validation methods
   use crate::execution_context::SharedExecutionContext;
   
   impl DiagnosticsBridge {
       pub fn validate_script(
           &self,
           script: &str, 
           context: &mut SharedExecutionContext
       ) -> Result<ValidationReport> {
           // Create trace span for validation operation
           let _span = self.trace_execution("script_validation", context);
           
           let mut report = ValidationReport::new();
           
           // Syntax validation reusing ConditionEvaluator compilation patterns
           if let Some(condition_evaluator) = &self.condition_evaluator {
               match condition_evaluator.compile_condition(script) {
                   Err(compilation_error) => {
                       report.add_error(self.enrich_diagnostic(&compilation_error));
                       self.trace_diagnostic(&compilation_error, "error");
                   },
                   Ok(_) => {
                       self.trace_diagnostic("Syntax validation passed", "info");
                   }
               }
           }
           
           // API validation using VariableInspector trait patterns
           if let Some(variable_inspector) = &self.variable_inspector {
               let api_violations = variable_inspector.validate_api_usage(script, context)?;
               for violation in api_violations {
                   report.add_warning(violation);
                   self.trace_diagnostic(&violation, "warning");
               }
           }
           
           // Performance validation using established metrics
           if context.performance_metrics.execution_count > 10000 {
               let perf_warning = "High execution count - consider optimization";
               report.add_warning(perf_warning);
               self.trace_diagnostic(perf_warning, "warning");
           }
           
           // Security validation with trace enrichment
           let security_issues = self.detect_security_patterns(script);
           for issue in security_issues {
               report.add_error(issue.clone());
               self.trace_diagnostic(&issue, "error");
           }
           
           Ok(report)
       }
   }
   
   pub struct ValidationReport {
       errors: Vec<String>,
       warnings: Vec<String>,
       suggestions: Vec<String>,
   }
   ```
2. **Integrate ConditionEvaluator trait for syntax validation** (reuse compilation logic)
3. **Use VariableInspector trait for API usage analysis** (leverage existing patterns)
4. **Add security pattern detection with trace enrichment**
5. **Implement performance validation using established metrics**
6. **Test validation with distributed tracing observability**

**Definition of Done:**
- [ ] Validation comprehensive
- [ ] All check types work
- [ ] Reports actionable
- [ ] Performance acceptable
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.3.3: Performance Profiling
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: DevEx Team

**Description**: CPU and memory profiling using Phase 9.2 diagnostics architecture, stack navigation traits, and distributed tracing integration for comprehensive performance observability.

**ARCHITECTURE FOUNDATION (Phase 9.2):**
- **DiagnosticsBridge Integration**: Profiling as diagnostics with distributed tracing (9.2.11)
- **StackNavigator Trait**: Enhanced flamegraphs using trait-based stack capture
- **SharedExecutionContext Metrics**: Leverages established performance_metrics without duplication
- **DebugStateCache Integration**: LRU caching patterns for profiling data management

**Acceptance Criteria:**
- [ ] CPU profiling via DiagnosticsBridge with pprof and trace span integration
- [ ] Flamegraphs enhanced using StackNavigator trait for frame details
- [ ] Memory profiling coordinates with DebugStateCache LRU patterns
- [ ] Performance analysis enriches SharedExecutionContext.performance_metrics
- [ ] Profiling data exported via DiagnosticsBridge trace infrastructure
- [ ] <5% profiling overhead maintaining established performance targets

**Implementation Steps:**
1. **Enhance DiagnosticsBridge with profiling capabilities** (don't create separate PerformanceProfiler):
   ```rust
   // llmspell-bridge/src/diagnostics_bridge.rs - add profiling methods
   use crate::execution_context::{SharedExecutionContext, PerformanceMetrics};
   
   impl DiagnosticsBridge {
       pub fn start_profiling(
           &mut self, 
           context: Arc<RwLock<SharedExecutionContext>>
       ) -> Result<()> {
           // Create trace span for profiling session
           let _span = self.trace_execution("profiling_start", &*context.read().await);
           
           self.profiler_guard = Some(pprof::ProfilerGuard::new(100)?); // 100Hz sampling
           self.profiling_context = Some(context);
           
           // Initialize profiling with DebugStateCache for data management
           if let Some(debug_cache) = &self.debug_state_cache {
               debug_cache.enable_profiling_mode();
           }
           
           Ok(())
       }
       
       pub fn generate_flamegraph(&self) -> Result<Vec<u8>> {
           if let Some(guard) = &self.profiler_guard {
               let context = self.profiling_context.as_ref().unwrap().read().await;
               
               // Use StackNavigator trait for enhanced flamegraph data
               let enhanced_frames = if let Some(stack_navigator) = &self.stack_navigator {
                   context.stack.iter().map(|frame| {
                       FlameGraphFrame {
                           function: stack_navigator.format_frame(frame),
                           file: frame.source.clone(),
                           line: frame.line,
                           execution_count: context.performance_metrics.execution_count,
                       }
                   }).collect()
               } else {
                   Vec::new()
               };
               
               let profile = guard.report().build()?;
               let mut flamegraph_data = Vec::new();
               
               // Create trace-enriched flamegraph
               profile.flamegraph_with_context(&mut flamegraph_data, &enhanced_frames)?;
               
               // Log flamegraph generation via tracing
               self.trace_diagnostic(
                   &format!("Generated flamegraph with {} frames", enhanced_frames.len()),
                   "info"
               );
               
               Ok(flamegraph_data)
           } else {
               Err(anyhow!("Profiling not active"))
           }
       }
       
       pub fn update_performance_metrics(
           &self, 
           operation: &str, 
           duration: Duration
       ) {
           // Update SharedExecutionContext performance metrics
           if let Some(context_ref) = &self.profiling_context {
               let mut context = context_ref.write().await;
               context.update_metrics(duration.as_micros() as u64, 0);
               
               // Report via diagnostics
               context.add_diagnostic(DiagnosticEntry {
                   level: "trace".to_string(),
                   message: format!("Operation '{}' took {}μs", operation, duration.as_micros()),
                   location: context.location.clone(),
                   timestamp: chrono::Utc::now().timestamp_millis() as u64,
               });
           }
       }
   }
   ```
2. **Integrate pprof with StackNavigator trait for enhanced flame graphs**
3. **Use DebugStateCache patterns for profiling data management** 
4. **Add distributed tracing integration for profiling observability**
5. **Test profiling overhead against established <5% performance targets**
3. **Generate flamegraphs enhanced with SharedExecutionContext stack data**
4. **Track memory allocations via SharedExecutionContext.performance_metrics**
5. **Detect potential leaks through diagnostics reporting**
6. **Export multiple formats via DiagnosticsBridge infrastructure**

**Definition of Done:**
- [ ] Profiling functional
- [ ] Flamegraphs generated
- [ ] Memory leaks detected
- [ ] Multiple export formats
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.3.4: Performance Profiler Hooks
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: DevEx Team

**Description**: Enhanced profiling integration with Phase 9.2 debug hooks using block_on_async patterns, StackNavigator trait, and DiagnosticsBridge tracing.

**ARCHITECTURE FOUNDATION (Phase 9.2):**
- **Debug Hooks Integration**: Uses established lua/globals/execution.rs hooks with block_on_async (9.2.3)
- **StackNavigator Trait**: Professional stack sampling via trait-based frame capture
- **DiagnosticsBridge Coordination**: Profiling data flows through distributed tracing (9.2.11)
- **Async Context Preservation**: Profiling data preserved across async boundaries (9.2.10)

**Acceptance Criteria:**
- [ ] Profiling hooks integrated via established debug hook patterns (9.2.3)
- [ ] Stack sampling uses StackNavigator trait for frame capture
- [ ] Function timing leverages block_on_async for sync/async bridging
- [ ] Memory tracking coordinates with DebugStateCache LRU patterns
- [ ] <5% overhead maintained using fast path/slow path architecture
- [ ] Profiling controlled via DiagnosticsBridge with trace observability

**Implementation Steps:**
1. **Enhance existing debug hooks with profiling integration**:
   ```rust
   // llmspell-bridge/src/lua/globals/execution.rs - extend established hooks
   use crate::{
       diagnostics_bridge::DiagnosticsBridge,
       execution_context::SharedExecutionContext,
       stack_navigator::StackNavigator,
       lua::sync_utils::block_on_async,
   };
   
   pub fn install_hooks_with_profiling(
       lua: &Lua,
       execution_manager: Arc<ExecutionManager>,
       shared_context: Arc<RwLock<SharedExecutionContext>>,
       diagnostics_bridge: Arc<DiagnosticsBridge>,
   ) -> Result<()> {
       // Clone for hook closure
       let context_clone = shared_context.clone();
       let diagnostics_clone = diagnostics_bridge.clone();
       
       lua.set_hook(HookTriggers {
           every_line: true,
           on_calls: true, 
           on_returns: true,
           every_nth_instruction: Some(1000), // Profiling sample rate
       }, move |lua, debug| {
           // Use block_on_async for sync/async bridge (established pattern)
           block_on_async("profiling_hook", async move {
               let mut context = context_clone.write().await;
               
               match debug.event() {
                   DebugEvent::Call => {
                       let func_name = debug.name().unwrap_or("<anonymous>");
                       let start_time = std::time::Instant::now();
                       
                       // Track function entry with trace span
                       diagnostics_clone.trace_execution(
                           &format!("function_call:{}", func_name),
                           &context
                       );
                       
                       context.function_entry_time = Some(start_time);
                   },
                   DebugEvent::Return => {
                       let func_name = debug.name().unwrap_or("<anonymous>");
                       
                       if let Some(start_time) = context.function_entry_time {
                           let duration = start_time.elapsed();
                           
                           // Update metrics using established patterns
                           context.update_metrics(duration.as_micros() as u64, 0);
                           
                           // Report via DiagnosticsBridge tracing
                           diagnostics_clone.update_performance_metrics(func_name, duration);
                       }
                   },
                   DebugEvent::Line => {
                       // Sample stack using StackNavigator trait
                       if debug.line_count().unwrap_or(0) % 1000 == 0 {
                           if let Some(stack_navigator) = &diagnostics_clone.stack_navigator {
                               let current_frame = stack_navigator.navigate_to_frame(0, &context.stack)?;
                               diagnostics_clone.sample_stack_for_profiling(current_frame);
                           }
                       }
                   },
                   _ => {}
               }
               
               Ok(())
           }, None) // Use established block_on_async pattern
       });
           
           Ok(())
       });
   }
   ```
       
2. **Add profiling methods to DiagnosticsBridge** (integrate with existing architecture):
   ```rust
   // llmspell-bridge/src/diagnostics_bridge.rs - add sampling methods
   impl DiagnosticsBridge {
       pub fn sample_stack_for_profiling(&self, stack: Vec<StackFrame>) {
           if let Some(context_ref) = &self.profiling_context {
               let mut context = context_ref.write().await;
               
               // Add to profiling data
               self.cpu_samples.lock().push(CpuSample {
                   timestamp: std::time::Instant::now(),
                   stack, // Use unified StackFrame type
               });
               
               // Update context stack
               context.stack = stack;
           }
       }
       
       pub fn sample_memory(&self, lua: &mlua::Lua) -> Result<()> {
           // Sample memory usage and update SharedExecutionContext
           let memory_usage = lua.used_memory();
           
           if let Some(context_ref) = &self.profiling_context {
               let mut context = context_ref.write().await;
               context.update_metrics(0, memory_usage);
           }
           
           Ok(())
       }
   }
   ```
       
3. **Use output.rs for stack capture** (leverage existing functionality):
   ```rust
   // llmspell-bridge/src/lua/output.rs - add profiling stack capture
   pub fn capture_stack_for_profiling(
       lua: &mlua::Lua, 
       debug: &mlua::Debug
   ) -> Result<Vec<StackFrame>> {
       // Use existing stack capture logic but return unified StackFrame type
       let stack = capture_stack_trace(lua, debug)?;
       
       // Convert to unified StackFrame type from execution_bridge.rs
       Ok(stack.into_iter().map(|frame| StackFrame {
           id: uuid::Uuid::new_v4().to_string(),
           name: frame.name,
           source: frame.source,
           line: frame.line,
           column: frame.column,
           locals: Vec::new(), // Profiling doesn't need locals
           is_user_code: true,
       }).collect())
   }
   ```
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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.3.5: Hook Introspection & Circuit Breakers
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: DevEx Team

**Description**: Integration with Phase 4 hooks via diagnostics_bridge.rs monitoring, using SharedExecutionContext for performance metrics.

**ARCHITECTURE ALIGNMENT with Phase 9.1:**
- **Monitoring integrates with diagnostics_bridge.rs** (monitoring is diagnostics)
- **Performance metrics use SharedExecutionContext** (avoid duplication)
- **Execution tracing via diagnostics** infrastructure
- **Real-time updates through DiagnosticsBridge** event system
- **Circuit breaker status as diagnostics** reporting

**Acceptance Criteria:**
- [ ] Hook listing via DiagnosticsBridge integration
- [ ] Hook details retrievable through diagnostics reporting
- [ ] Execution tracing integrated with SharedExecutionContext
- [ ] Circuit breaker status visible via diagnostics events
- [ ] Real-time monitoring through DiagnosticsBridge
- [ ] Performance metrics from SharedExecutionContext.performance_metrics

**Implementation Steps:**
1. **Integrate with DiagnosticsBridge for hook monitoring** (don't create HookInspector):
   ```rust
   // llmspell-bridge/src/diagnostics_bridge.rs - add hook monitoring
   impl DiagnosticsBridge {
       pub fn monitor_phase4_hooks(
           &mut self, 
           hook_manager: Arc<HookManager>,
           shared_context: Arc<RwLock<SharedExecutionContext>>
       ) {
           self.hook_manager = Some(hook_manager);
           self.hook_monitoring_context = Some(shared_context);
       }
       
       pub fn get_hook_status(&self) -> HookStatusReport {
           let mut report = HookStatusReport::new();
           
           if let Some(manager) = &self.hook_manager {
               // Get hook list from Phase 4 HookManager
               let hooks = manager.list_active_hooks();
               
               // Use SharedExecutionContext for performance data
               if let Some(context_ref) = &self.hook_monitoring_context {
                   let context = context_ref.read().await;
                   report.performance_summary = self.get_performance_summary();
                   
                   // Add execution traces from diagnostics
                   report.execution_traces = context.recent_logs
                       .iter()
                       .filter(|log| log.message.contains("hook"))
                       .cloned()
                       .collect();
               }
               
               // Circuit breaker status via diagnostics reporting
               for hook in hooks {
                   if hook.circuit_breaker_triggered {
                       self.report_circuit_breaker_event(&hook);
                   }
               }
           }
           
           report
       }
   }
   ```
2. **Implement circuit breaker state management using DebugStateCache**
3. **Add distributed tracing for hook monitoring observability**
4. **Integrate with SharedExecutionContext for performance metrics**
5. **Test hook introspection with multi-threaded runtime patterns**

**Definition of Done:**
- [ ] Hooks introspectable
- [ ] Circuit breakers monitored
- [ ] Real-time updates work
- [ ] Metrics accurate
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.3.6: Session Recording/Replay
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: DevEx Team

**Description**: Complete session recording integrated with diagnostics_bridge.rs, using unified types and SharedExecutionContext for comprehensive replay.

**ARCHITECTURE ALIGNMENT with Phase 9.1:**
- **Recording integrates with diagnostics_bridge.rs** (event recording is diagnostics)
- **Uses unified types** from execution_bridge.rs (StackFrame, Variable, etc.)
- **Leverages SharedExecutionContext** for comprehensive state capture
- **Coordinates with ExecutionManager** for debugging state
- **Uses output.rs** for value serialization in recordings

**Acceptance Criteria:**
- [ ] Sessions recorded via DiagnosticsBridge to JSON
- [ ] All event types captured using unified types
- [ ] Interactive replay restores SharedExecutionContext
- [ ] Stepping through events coordinated with ExecutionManager
- [ ] Environment restoration via SharedExecutionContext state
- [ ] Compression supported through diagnostics infrastructure

**Implementation Steps:**
1. **Enhance DiagnosticsBridge with recording capabilities** (don't create separate SessionRecorder):
   ```rust
   // llmspell-bridge/src/diagnostics_bridge.rs - add session recording
   use crate::{
       execution_bridge::{StackFrame, Variable, DebugState},
       execution_context::SharedExecutionContext,
   };
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum SessionEvent {
       ScriptStart { 
           script_path: String, 
           context: SharedExecutionContext 
       },
       VariableChange { 
           variable: Variable,           // Use unified Variable type
           location: SourceLocation 
       },
       FunctionCall { 
           stack_frame: StackFrame,      // Use unified StackFrame type
           arguments: Vec<Variable> 
       },
       ToolInvocation { 
           tool_name: String, 
           arguments: serde_json::Value,
           context: SharedExecutionContext 
       },
       BreakpointHit { 
           location: SourceLocation, 
           stack: Vec<StackFrame>,       // Use unified types
           locals: Vec<Variable> 
       },
       DebugStateChange { 
           old_state: DebugState, 
           new_state: DebugState         // Use unified DebugState
       },
   }
   
   impl DiagnosticsBridge {
       pub fn start_session_recording(&mut self, session_id: String) {
           self.recording_session = Some(SessionRecording {
               session_id,
               events: Vec::new(),
               start_time: chrono::Utc::now(),
               context_snapshots: HashMap::new(),
           });
       }
       
       pub fn record_event(&mut self, event: SessionEvent) {
           if let Some(session) = &mut self.recording_session {
               session.events.push(TimestampedEvent {
                   timestamp: chrono::Utc::now(),
                   event,
               });
           }
       }
   }
   ```
2. **Implement event serialization using VariableInspector trait patterns**
3. **Add replay functionality with SharedExecutionContext restoration**
4. **Integrate distributed tracing correlation IDs for session tracking**
5. **Test recording/replay with multi-threaded async context preservation**

**Definition of Done:**
- [ ] Recording comprehensive
- [ ] Replay accurate
- [ ] Interactive stepping works
- [ ] Environment restored
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.3.7: Section 9.3 Quality Gates and Testing
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Comprehensive quality validation of DevEx features using Phase 9.2 architectural compliance testing, trait validation, and distributed tracing verification.

**ARCHITECTURE FOUNDATION (Phase 9.2):**
- **Three-Layer Architecture Validation**: Tests verify trait usage and zero script engine imports in bridge layer
- **Trait Integration Testing**: Validates ConditionEvaluator, VariableInspector, StackNavigator usage patterns
- **Distributed Tracing Compliance**: Verifies trace enrichment and observability integration
- **Performance Architecture Testing**: Validates fast path/slow path patterns and <5% overhead targets

**Acceptance Criteria:**
- [ ] Hot reload tests validate DiagnosticsBridge integration with <500ms targets
- [ ] Validation system tests verify ConditionEvaluator/VariableInspector trait usage
- [ ] Profiling tests confirm <5% overhead with no metrics duplication
- [ ] Recording/replay tests validate unified types and trace correlation
- [ ] Hook monitoring tests verify DebugStateCache circuit breaker patterns
- [ ] Multi-threaded runtime tests for block_on_async patterns
- [ ] Zero clippy warnings with three-layer architecture compliance
- [ ] Complete documentation for trait-based DevEx patterns

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
   # Focus on new code in development experience features
   ```

3. **Write and Run Feature Tests**:
   ```bash
   # Write hot reload tests
   # Write validation system tests
   # Write profiling accuracy tests
   # Write session recording/replay tests
   cargo test --workspace --all-features
   ```

4. **Validate Phase 9.2 Architecture Compliance**:
   ```bash
   # Verify zero script engine imports in bridge layer
   find llmspell-bridge/src -maxdepth 1 -name "*.rs" -exec grep -l "use mlua" {} \;
   # Must return empty
   
   # Test trait implementations
   cargo test --package llmspell-bridge -- condition_evaluator
   cargo test --package llmspell-bridge -- variable_inspector  
   cargo test --package llmspell-bridge -- stack_navigator
   
   # Benchmark performance targets
   cargo bench --package llmspell-repl -- hot_reload  # <500ms
   cargo bench --package llmspell-bridge -- profiling # <5% overhead
   
   # Verify multi-threaded runtime compatibility
   cargo test --package llmspell-bridge -- async_context
   ```

5. **Run Quality Check Scripts**:
   ```bash
   ./scripts/quality-check-minimal.sh  # Format, clippy, compile
   ./scripts/quality-check-fast.sh     # Adds unit tests & docs
   ```

6. **Document Phase 9.2 DevEx Integration Patterns**:
   ```bash
   # Document trait-based architecture usage
   cargo doc --package llmspell-bridge --no-deps  
   cargo doc --package llmspell-repl --no-deps
   
   # Verify documentation covers:
   # - Three-layer architecture patterns in DevEx features
   # - ConditionEvaluator/VariableInspector trait usage
   # - DiagnosticsBridge integration patterns
   # - Distributed tracing integration examples
   # - Multi-threaded runtime requirements
   ```

**Definition of Done:**
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with `cargo test --workspace --all-features`
- [ ] Hot reload <500ms, profiling overhead <5% verified
- [ ] Quality check scripts pass
- [ ] DevEx feature documentation complete

---

## Phase 9.4: Multi-Client Implementation (Days 10-11)

### Task 9.4.1: CLI Client Integration
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: CLI Team Lead

**Description**: Update llmspell-cli to connect to kernel service, integrating with Phase 9.1 architecture for debugging and error display.

**ARCHITECTURE ALIGNMENT with Phase 9.1:**
- **Debug workflow support** uses ExecutionManager and ExecutionBridge
- **Enhanced error display** integrates with diagnostics_bridge.rs
- **REPL commands** align with established LRP/LDP protocols
- **Uses unified types** (StackFrame, Variable, Breakpoint) for consistency

**Acceptance Criteria:**
- [ ] CLI connects to kernel service with established protocols
- [ ] All REPL commands implemented using ExecutionManager
- [ ] Command history with search
- [ ] Enhanced error display via DiagnosticsBridge integration
- [ ] Debug workflow support using ExecutionBridge architecture
- [ ] Media display capability via established IOPub channels

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

**ARCHITECTURE FOUNDATION (Phase 9.2):**
- **Interactive Debugger Initialization**: Uses established multi-client session patterns (9.2.2)
- **Debug State Management**: Leverages DebugStateCache and step debugging patterns (9.2.5)
- **SharedExecutionContext**: Performance monitoring and async context preservation (9.2.10)
- **Distributed Tracing**: Debug run observability via trace enrichment (9.2.11)

**Acceptance Criteria:**
- [ ] Debug mode initializes InteractiveDebugger with session management
- [ ] Kernel discovery uses established LRP/LDP protocol patterns
- [ ] Debug state initialization via DebugStateCache LRU patterns
- [ ] Script execution preserves SharedExecutionContext async boundaries
- [ ] Non-debug mode maintains fast path performance (<1ms overhead)
- [ ] Debug execution integrates distributed tracing for observability

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

**ARCHITECTURE ALIGNMENT with Phase 9.1:**
- **Uses unified types** (StackFrame, Variable) instead of generic types
- **Error formatting** integrates with diagnostics_bridge.rs patterns
- **Debug interface** coordinates with ExecutionManager
- **Output formatting** uses output.rs functions

**Acceptance Criteria:**
- [ ] IOPub events received using established protocol types
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

**Description**: LSP/DAP integration using Phase 9.2 interactive debugging infrastructure, ExecutionManager patterns, and trait-based variable inspection for IDE integration.

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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes

### Task 9.4.7: VS Code Extension
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: IDE Team

**Description**: VS Code extension using Phase 9.2 LSP/DAP integration, interactive debugging UI, conditional breakpoint support, and distributed tracing visualization.

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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


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
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes


### Task 9.4.9: Section 9.4 Quality Gates and Testing
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: QA Team

**Description**: Comprehensive quality checks and testing of multi-client implementation.

**Acceptance Criteria:**
- [ ] Multi-client tests pass (10+ clients)
- [ ] Protocol compliance verified
- [ ] Security tests complete
- [ ] Performance benchmarks met
- [ ] Integration tests pass
- [ ] Zero clippy warnings
- [ ] Code properly formatted
- [ ] Documentation complete
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
   # - Debug overhead <10%
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