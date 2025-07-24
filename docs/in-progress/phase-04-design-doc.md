# Phase 4: Hook and Event System - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: Implementation Ready  
**Phase**: 4 (Hook and Event System)  
**Timeline**: Weeks 17-18  
**Priority**: HIGH (Production Essential)  
**Dependencies**: Phase 3 (Tool Enhancement & Agent Infrastructure) ✅ COMPLETE

> **📋 Detailed Implementation Guide**: This document provides complete specifications for implementing Phase 4 hook and event system for rs-llmspell.

---

## Phase Overview

### Goal
Implement comprehensive hooks and events system that enables extensibility, monitoring, and reactive programming patterns across all components, leveraging the infrastructure delivered in Phase 3.

### Core Principles
- **Leverage Existing Infrastructure**: Build on Phase 3's event emission (don't rebuild)
- **Unified System**: Event-Driven Hook System eliminates overlap between hooks and events
- **Performance First**: <5% overhead with regression testing from day 1
- **Script Integration**: Hooks accessible from Lua, JavaScript, and Python
- **Production Ready**: Built-in hooks for logging, metrics, and debugging

### Success Criteria
- [ ] Pre/post execution hooks work for agents and tools
- [ ] Hook execution works for 6 agent states, 34 tools, and 4 workflow patterns
- [ ] Event emission and subscription functional
- [ ] Built-in logging and metrics hooks operational
- [ ] Scripts can register custom hooks
- [ ] Hook execution doesn't significantly impact performance (<5% overhead)

---

## 1. Implementation Specifications

### 1.1 Hook System Architecture

**Core Hook Infrastructure:**

```rust
// llmspell-hooks/src/lib.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookPoint {
    // Agent lifecycle hooks (6 states)
    BeforeAgentInit,
    AfterAgentInit,
    BeforeAgentExecution,
    AfterAgentExecution,
    AgentError,
    BeforeAgentShutdown,
    AfterAgentShutdown,
    
    // Tool execution hooks (34 tools)
    BeforeToolDiscovery,
    AfterToolDiscovery,
    BeforeToolExecution,
    AfterToolExecution,
    ToolValidation,
    ToolError,
    
    // Workflow hooks (4 patterns)
    BeforeWorkflowStart,
    WorkflowStageTransition,
    BeforeWorkflowStage,
    AfterWorkflowStage,
    WorkflowCheckpoint,
    WorkflowRollback,
    AfterWorkflowComplete,
    WorkflowError,
    
    // State management hooks
    BeforeStateRead,
    AfterStateRead,
    BeforeStateWrite,
    AfterStateWrite,
    StateConflict,
    StateMigration,
    
    // System hooks
    SystemStartup,
    SystemShutdown,
    ConfigurationChange,
    ResourceLimitExceeded,
    SecurityViolation,
    
    // Custom hooks
    Custom(String),
}
```

### 1.2 Event Bus Implementation

**Event System Using tokio-stream and crossbeam:**

```rust
// llmspell-events/src/lib.rs
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventPattern, Vec<Arc<dyn EventHandler>>>>>,
    event_store: Option<Arc<dyn EventStore>>,
    dispatcher: Arc<EventDispatcher>,
    metrics: Arc<EventMetrics>,
}

impl EventBus {
    pub async fn publish(&self, event: Event) -> Result<()> {
        // Store event if persistence enabled
        if let Some(store) = &self.event_store {
            store.append(&event).await?;
        }
        
        // Collect matching handlers
        let handlers = self.collect_handlers(&event).await?;
        
        // Dispatch to handlers with performance tracking
        let start = Instant::now();
        self.dispatcher.dispatch(event, handlers).await?;
        
        // Update metrics
        self.metrics.record_event(&event, start.elapsed());
        
        Ok(())
    }
}
```

### 1.3 Unified Hook-Event System

**Unifying Hooks and Events to Eliminate Overlap:**

```rust
// llmspell-hooks/src/unified.rs
pub struct UnifiedHookEventSystem {
    hook_registry: Arc<HookRegistry>,
    event_bus: Arc<EventBus>,
    performance_monitor: Arc<PerformanceMonitor>,
}

impl UnifiedHookEventSystem {
    /// Hooks are synchronous interception points that can modify behavior
    pub async fn execute_hook(&self, 
                             point: HookPoint, 
                             context: &mut HookContext) -> Result<HookResult> {
        // Track performance
        let _guard = self.performance_monitor.track_hook(&point);
        
        // Execute hooks in priority order
        let hooks = self.hook_registry.get_hooks(&point).await?;
        
        for hook in hooks {
            match hook.execute(context).await? {
                HookResult::Continue => continue,
                HookResult::Modified(value) => {
                    context.data.insert("modified_value".to_string(), value);
                }
                HookResult::Cancel(reason) => {
                    // Convert cancellation to event for async handling
                    self.event_bus.publish(Event::hook_cancelled(point, reason)).await?;
                    return Ok(HookResult::Cancel(reason));
                }
                HookResult::Redirect(target) => {
                    return Ok(HookResult::Redirect(target));
                }
            }
        }
        
        Ok(HookResult::Continue)
    }
    
    /// Events are asynchronous notifications for loose coupling
    pub async fn emit_event(&self, event: Event) -> Result<()> {
        self.event_bus.publish(event).await
    }
}
```

### 1.4 Script Integration

**Lua Hook Registration API:**

```lua
-- Global Hook API
Hook.register("agent:before_execution", function(context)
    -- Synchronous hook logic
    Logger.info("Agent starting", {agent_id = context.agent_id})
    
    -- Can modify context
    context.metadata.custom_field = "value"
    
    -- Return control flow
    return {
        continue_execution = true,
        state_updates = {
            last_execution = os.time()
        }
    }
end)

-- Event subscription
Event.subscribe("agent:state_changed", function(event)
    -- Asynchronous event handling
    if event.data.new_state == "failed" then
        Alert.send("Agent failed", event.data)
    end
end)

-- Built-in hook discovery
local hooks = Hook.list()
for _, hook_point in ipairs(hooks) do
    print("Available hook: " .. hook_point)
end
```

### 1.5 Built-in Hooks

**Standard Hooks for Production Use:**

```rust
// llmspell-hooks/src/builtin/mod.rs
pub mod logging;
pub mod metrics;
pub mod debugging;
pub mod security;

// Logging hook
pub struct LoggingHook {
    logger: Arc<dyn Logger>,
    log_level: LogLevel,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl Hook for LoggingHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Smart logging based on context
        match context.point {
            HookPoint::BeforeAgentExecution => {
                self.logger.info("Agent execution starting", context.metadata);
            }
            HookPoint::ToolError => {
                self.logger.error("Tool execution failed", context.data);
            }
            _ => {
                self.logger.debug("Hook point triggered", context);
            }
        }
        
        Ok(HookResult::Continue)
    }
}

// Metrics hook
pub struct MetricsHook {
    collector: Arc<MetricsCollector>,
    histogram_buckets: Vec<f64>,
}

impl Hook for MetricsHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Collect metrics based on hook point
        match context.point {
            HookPoint::AfterToolExecution => {
                if let Some(duration) = context.data.get("duration_ms") {
                    self.collector.record_histogram(
                        "tool_execution_duration",
                        duration.as_f64().unwrap_or(0.0),
                        &context.metadata
                    );
                }
            }
            HookPoint::AgentError => {
                self.collector.increment_counter(
                    "agent_errors_total",
                    &context.metadata
                );
            }
            _ => {}
        }
        
        Ok(HookResult::Continue)
    }
}
```

### 1.6 Performance Optimization

**Ensuring <5% Overhead:**

```rust
// llmspell-hooks/src/performance.rs
pub struct PerformanceMonitor {
    overhead_threshold: Duration,
    batch_size: usize,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl PerformanceMonitor {
    pub fn track_hook(&self, point: &HookPoint) -> PerformanceGuard {
        PerformanceGuard::new(point.clone(), self.metrics.clone())
    }
    
    pub async fn check_overhead(&self) -> Result<OverheadReport> {
        let metrics = self.metrics.lock().await;
        
        // Calculate overhead percentage
        let total_hook_time = metrics.total_hook_duration;
        let total_execution_time = metrics.total_execution_duration;
        let overhead_percent = (total_hook_time.as_secs_f64() / 
                               total_execution_time.as_secs_f64()) * 100.0;
        
        Ok(OverheadReport {
            overhead_percent,
            total_hooks_executed: metrics.hook_count,
            average_hook_duration: total_hook_time / metrics.hook_count as u32,
            recommendation: if overhead_percent > 5.0 {
                "Consider batching hooks or reducing hook complexity"
            } else {
                "Performance within acceptable limits"
            }
        })
    }
}

// Hook batching for high-frequency events
pub struct BatchedHookExecutor {
    batch_size: usize,
    batch_timeout: Duration,
    pending: Arc<Mutex<Vec<(HookPoint, HookContext)>>>,
}

impl BatchedHookExecutor {
    pub async fn queue_hook(&self, point: HookPoint, context: HookContext) -> Result<()> {
        let mut pending = self.pending.lock().await;
        pending.push((point, context));
        
        if pending.len() >= self.batch_size {
            self.flush_batch().await?;
        }
        
        Ok(())
    }
    
    async fn flush_batch(&self) -> Result<()> {
        let mut pending = self.pending.lock().await;
        let batch = std::mem::take(&mut *pending);
        
        // Execute hooks in parallel for same hook point
        let grouped = self.group_by_hook_point(batch);
        
        for (point, contexts) in grouped {
            self.execute_hook_batch(point, contexts).await?;
        }
        
        Ok(())
    }
}
```

---

## 2. Integration Points

### 2.1 Agent Lifecycle Integration

**Leveraging Phase 3 State Transitions:**

```rust
// Integration with existing agent state machine
impl Agent {
    async fn transition_state(&mut self, new_state: AgentState) -> Result<()> {
        let old_state = self.state.clone();
        
        // Pre-transition hook
        let mut context = HookContext::new(
            self.determine_hook_point(&old_state, &new_state),
            self.id.clone()
        );
        
        match self.hook_system.execute_hook(context.point.clone(), &mut context).await? {
            HookResult::Cancel(reason) => {
                return Err(Error::StateTransitionCancelled(reason));
            }
            _ => {}
        }
        
        // Perform transition (existing Phase 3 code)
        self.state = new_state;
        
        // Post-transition event (existing Phase 3 infrastructure)
        self.event_bus.emit_event(Event::agent_state_changed(
            self.id.clone(),
            old_state,
            new_state
        )).await?;
        
        Ok(())
    }
    
    fn determine_hook_point(&self, old: &AgentState, new: &AgentState) -> HookPoint {
        match (old, new) {
            (AgentState::Uninitialized, AgentState::Initializing) => HookPoint::BeforeAgentInit,
            (AgentState::Initializing, AgentState::Ready) => HookPoint::AfterAgentInit,
            (AgentState::Ready, AgentState::Running) => HookPoint::BeforeAgentExecution,
            (AgentState::Running, AgentState::Ready) => HookPoint::AfterAgentExecution,
            (_, AgentState::Failed) => HookPoint::AgentError,
            (_, AgentState::Stopped) => HookPoint::BeforeAgentShutdown,
            _ => HookPoint::Custom(format!("{:?}_to_{:?}", old, new))
        }
    }
}
```

### 2.2 Tool Execution Integration

**Adding Hooks to 34 Tools:**

```rust
// Integration with existing tool infrastructure
impl Tool {
    async fn execute(&self, params: ToolParams) -> Result<ToolResponse> {
        // Pre-execution hook
        let mut context = HookContext::new(
            HookPoint::BeforeToolExecution,
            self.name()
        );
        context.data.insert("params", serde_json::to_value(&params)?);
        
        self.hook_system.execute_hook(context.point.clone(), &mut context).await?;
        
        // Execute tool (existing Phase 3 code)
        let start = Instant::now();
        let result = self.inner_execute(params).await;
        let duration = start.elapsed();
        
        // Post-execution hook
        let mut context = HookContext::new(
            match &result {
                Ok(_) => HookPoint::AfterToolExecution,
                Err(_) => HookPoint::ToolError,
            },
            self.name()
        );
        context.data.insert("duration_ms", duration.as_millis().into());
        context.data.insert("result", serde_json::to_value(&result)?);
        
        self.hook_system.execute_hook(context.point.clone(), &mut context).await?;
        
        result
    }
}
```

### 2.3 Workflow Integration

**Hooks for 4 Workflow Patterns:**

```rust
// Integration with workflow patterns
impl Workflow {
    async fn execute_step(&mut self, step: &WorkflowStep) -> Result<StepResult> {
        // Before step hook
        let mut context = HookContext::new(
            HookPoint::BeforeWorkflowStage,
            self.id.clone()
        );
        context.data.insert("step_name", step.name.clone().into());
        context.data.insert("step_index", self.current_step_index.into());
        
        self.hook_system.execute_hook(context.point.clone(), &mut context).await?;
        
        // Execute step (existing Phase 3 code)
        let result = match &self.pattern {
            WorkflowPattern::Sequential => self.execute_sequential_step(step).await,
            WorkflowPattern::Conditional => self.execute_conditional_step(step).await,
            WorkflowPattern::Loop => self.execute_loop_step(step).await,
            WorkflowPattern::Parallel => self.execute_parallel_step(step).await,
        };
        
        // After step hook
        let mut context = HookContext::new(
            HookPoint::AfterWorkflowStage,
            self.id.clone()
        );
        context.data.insert("step_result", serde_json::to_value(&result)?);
        
        self.hook_system.execute_hook(context.point.clone(), &mut context).await?;
        
        result
    }
}
```

---

## 3. Testing Strategy

### 3.1 Performance Regression Testing

**Day 1 Performance Monitoring:**

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hook_overhead_under_5_percent() {
        let system = create_test_system().await;
        
        // Baseline: execution without hooks
        let baseline_duration = measure_baseline_execution(&system).await;
        
        // Register standard hooks
        system.register_builtin_hooks().await;
        
        // Execution with hooks
        let hooked_duration = measure_hooked_execution(&system).await;
        
        // Calculate overhead
        let overhead_percent = ((hooked_duration.as_secs_f64() - baseline_duration.as_secs_f64()) 
                               / baseline_duration.as_secs_f64()) * 100.0;
        
        assert!(overhead_percent < 5.0, 
                "Hook overhead {}% exceeds 5% limit", overhead_percent);
    }
    
    #[tokio::test]
    async fn test_high_frequency_hook_batching() {
        let system = create_test_system().await;
        let executor = BatchedHookExecutor::new(100, Duration::from_millis(10));
        
        // Simulate high-frequency events
        let start = Instant::now();
        for i in 0..10000 {
            executor.queue_hook(
                HookPoint::AfterToolExecution,
                create_test_context(i)
            ).await.unwrap();
        }
        executor.flush_batch().await.unwrap();
        
        let duration = start.elapsed();
        
        // Should complete in under 100ms with batching
        assert!(duration < Duration::from_millis(100),
                "Batched execution took {:?}", duration);
    }
}
```

### 3.2 Integration Testing

**Testing All Integration Points:**

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_agent_lifecycle_hooks() {
        let mut agent = create_test_agent().await;
        let hook_calls = Arc::new(Mutex::new(Vec::new()));
        
        // Register tracking hook
        agent.hook_system.register(
            TrackingHook::new(hook_calls.clone())
        ).await;
        
        // Trigger all state transitions
        agent.initialize().await.unwrap();
        agent.start().await.unwrap();
        agent.execute("test").await.unwrap();
        agent.stop().await.unwrap();
        
        // Verify all 6 states triggered hooks
        let calls = hook_calls.lock().await;
        assert_eq!(calls.len(), 6);
        assert!(calls.contains(&HookPoint::BeforeAgentInit));
        assert!(calls.contains(&HookPoint::AfterAgentInit));
        assert!(calls.contains(&HookPoint::BeforeAgentExecution));
        assert!(calls.contains(&HookPoint::AfterAgentExecution));
        assert!(calls.contains(&HookPoint::BeforeAgentShutdown));
    }
    
    #[tokio::test]
    async fn test_tool_execution_hooks() {
        // Test hooks for all 34 tools
        for tool_name in get_all_tool_names() {
            let tool = Tool::get(&tool_name).unwrap();
            let hook_called = Arc::new(AtomicBool::new(false));
            
            tool.hook_system.register(
                FlagHook::new(HookPoint::BeforeToolExecution, hook_called.clone())
            ).await;
            
            tool.execute(create_test_params()).await.unwrap();
            
            assert!(hook_called.load(Ordering::SeqCst),
                   "Hook not called for tool: {}", tool_name);
        }
    }
}
```

---

## 4. Migration and Quick Wins

### 4.1 Phase 3 Quick Wins

**From Handoff Package:**

1. **Fix tool invocation parameter format** (~2 hours)
   - Location: `llmspell-bridge/src/lua/globals/agent.rs` line ~153
   - Update parameter wrapping to match tool expectations

2. **Complete documentation** (~4 hours)
   - Create `/CHANGELOG_v0.3.0.md` for breaking changes
   - Update `/docs/providers/README.md` with enhancement docs

### 4.2 Implementation Order

**Recommended Sequence:**

1. **Week 1 - Core Infrastructure**
   - Implement HookRegistry and HookPoint enum
   - Set up EventBus with tokio-stream/crossbeam
   - Create UnifiedHookEventSystem
   - Implement performance monitoring

2. **Week 1 - Integration Points**
   - Agent lifecycle hooks (6 states)
   - Tool execution hooks (34 tools)
   - Basic workflow hooks (4 patterns)
   - Built-in hooks (logging, metrics)

3. **Week 2 - Script Integration**
   - Lua Hook API and Event API
   - Hook discovery and registration
   - Script examples and testing
   - Performance optimization

4. **Week 2 - Testing and Polish**
   - Performance regression suite
   - Integration test coverage
   - Documentation updates
   - Final optimization pass

---

## 5. Risk Mitigation

### 5.1 Performance Risks

**Mitigation Strategies:**

1. **Hook Batching**: Batch high-frequency hooks
2. **Lazy Registration**: Only load hooks when needed
3. **Circuit Breakers**: Disable slow hooks automatically
4. **Monitoring**: Real-time overhead tracking

### 5.2 Compatibility Risks

**Ensuring Backward Compatibility:**

1. **Optional Hooks**: All hooks are opt-in
2. **Graceful Degradation**: System works without hooks
3. **Version Detection**: Check for hook support
4. **Migration Tools**: Helper functions for updates

---

## 6. Documentation Requirements

### 6.1 API Documentation

- Complete rustdoc for all hook/event APIs
- Script integration examples
- Performance tuning guide
- Hook development guide

### 6.2 User Documentation

- Hook system overview
- Built-in hooks reference
- Custom hook tutorial
- Performance best practices

---

## 7. Deliverables

### Phase 4 Completion Checklist

- [ ] Core hook infrastructure implemented
- [ ] Event bus operational with tokio-stream/crossbeam
- [ ] Unified hook-event system eliminating overlap
- [ ] 20+ hook points across system
- [ ] Integration with 6 agent states
- [ ] Integration with 34 tools
- [ ] Integration with 4 workflow patterns
- [ ] Built-in hooks (logging, metrics, debugging)
- [ ] Lua script integration complete
- [ ] Performance <5% overhead verified
- [ ] Comprehensive test coverage
- [ ] Documentation complete
- [ ] Examples for all patterns
- [ ] Migration from Phase 3 smooth

---

## 8. Future Considerations

### Post-Phase 4 Enhancements

1. **JavaScript/Python Integration** (Phase 15)
   - Extend hook APIs to other languages
   - Cross-language event propagation

2. **Persistent Event Store** (Phase 5)
   - Event sourcing patterns
   - Hook execution replay

3. **Distributed Hooks** (Phase 16-17)
   - A2A protocol hook propagation
   - Remote hook registration

4. **Advanced Patterns**
   - Conditional hooks
   - Hook composition
   - Dynamic hook generation

---

## Conclusion

Phase 4 delivers a production-ready hook and event system by leveraging the solid foundation from Phase 3. The unified approach eliminates complexity while providing powerful extensibility. With performance as a primary concern and comprehensive testing from day 1, the system will meet the <5% overhead target while enabling rich monitoring, debugging, and customization capabilities.

The implementation follows a pragmatic approach: start with the most mature infrastructure (agent lifecycle hooks), progressively add integration points, and continuously monitor performance. This ensures a stable, performant system that enhances rather than hinders the core LLMSpell functionality.