# Complete Hook/Event Integration

## Overview

This document provides the complete implementation specification for rs-llmspell's hook and event integration system. It defines the comprehensive architecture for hooks, events, cross-engine compatibility, and production-ready integration patterns.

## Complete Hook System Architecture

### Core Hook Infrastructure

```rust
// Complete hook registry implementation
pub struct OptimizedHookManager {
    hooks: HashMap<HookPoint, PriorityQueue<Box<dyn Hook>>>,
    hook_cache: LruCache<HookCacheKey, HookResult>,
    execution_strategy: HookExecutionStrategy,
    performance_monitor: HookPerformanceMonitor,
    dependency_resolver: HookDependencyResolver,
    security_validator: HookSecurityValidator,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct HookCacheKey {
    hook_point: HookPoint,
    context_hash: u64,
    hook_chain_hash: u64,
}

#[derive(Debug, Clone)]
pub enum HookExecutionStrategy {
    Sequential,      // Execute hooks one by one
    Parallel,        // Execute compatible hooks in parallel
    Adaptive,        // Choose strategy based on hook characteristics
    Prioritized,     // Execute high-priority hooks first
}

// Complete hook execution with all features
impl OptimizedHookManager {
    pub async fn execute_hooks_complete(
        &mut self,
        point: HookPoint,
        context: &mut HookContext
    ) -> Result<CompleteHookResult> {
        let execution_id = ExecutionId::new();
        let start_time = Instant::now();
        
        // Security validation
        self.security_validator.validate_hook_execution(&point, context).await?;
        
        // Check cache first
        let cache_key = self.generate_cache_key(&point, context);
        if let Some(cached_result) = self.hook_cache.get(&cache_key) {
            self.performance_monitor.record_cache_hit(&point, execution_id);
            return Ok(CompleteHookResult::from_cached(cached_result.clone()));
        }
        
        // Get applicable hooks
        let applicable_hooks = self.get_applicable_hooks(&point)?;
        if applicable_hooks.is_empty() {
            return Ok(CompleteHookResult::empty());
        }
        
        // Resolve dependencies
        let execution_order = self.dependency_resolver.resolve_execution_order(&applicable_hooks)?;
        
        // Execute hooks based on strategy
        let hook_results = match &self.execution_strategy {
            HookExecutionStrategy::Sequential => {
                self.execute_hooks_sequential(&execution_order, context, execution_id).await?
            },
            HookExecutionStrategy::Parallel => {
                self.execute_hooks_parallel(&execution_order, context, execution_id).await?
            },
            HookExecutionStrategy::Adaptive => {
                self.execute_hooks_adaptive(&execution_order, context, execution_id).await?
            },
            HookExecutionStrategy::Prioritized => {
                self.execute_hooks_prioritized(&execution_order, context, execution_id).await?
            },
        };
        
        let total_duration = start_time.elapsed();
        
        // Create complete result
        let complete_result = CompleteHookResult {
            execution_id,
            hook_point: point,
            individual_results: hook_results,
            total_duration,
            successful_hooks: self.count_successful_hooks(&hook_results),
            failed_hooks: self.count_failed_hooks(&hook_results),
            cache_used: false,
            execution_strategy: self.execution_strategy.clone(),
        };
        
        // Cache successful results
        if complete_result.all_successful() {
            self.hook_cache.insert(cache_key, complete_result.to_cacheable());
        }
        
        // Record performance metrics
        self.performance_monitor.record_execution(
            &point,
            execution_id,
            total_duration,
            &complete_result
        );
        
        Ok(complete_result)
    }
    
    async fn execute_hooks_parallel(
        &self,
        hooks: &[ResolvedHook],
        context: &mut HookContext,
        execution_id: ExecutionId
    ) -> Result<Vec<IndividualHookResult>> {
        let mut parallel_groups = self.group_parallel_compatible_hooks(hooks);
        let mut all_results = Vec::new();
        
        for group in parallel_groups {
            if group.len() == 1 {
                // Single hook - execute directly
                let result = self.execute_single_hook(&group[0], context, execution_id).await?;
                all_results.push(result);
            } else {
                // Multiple compatible hooks - execute in parallel
                let parallel_results = self.execute_hook_group_parallel(&group, context, execution_id).await?;
                all_results.extend(parallel_results);
            }
        }
        
        Ok(all_results)
    }
    
    async fn execute_hook_group_parallel(
        &self,
        hooks: &[ResolvedHook],
        context: &HookContext,
        execution_id: ExecutionId
    ) -> Result<Vec<IndividualHookResult>> {
        let mut handles = Vec::new();
        
        // Clone context for each hook
        for hook in hooks {
            let hook_context = context.clone();
            let hook_clone = hook.clone();
            let execution_id = execution_id;
            
            let handle = tokio::spawn(async move {
                let start_time = Instant::now();
                let result = hook_clone.hook.execute(&mut hook_context.clone()).await;
                let duration = start_time.elapsed();
                
                IndividualHookResult {
                    hook_id: hook_clone.id.clone(),
                    hook_name: hook_clone.hook.name().to_string(),
                    result,
                    execution_duration: duration,
                    execution_id,
                    thread_id: format!("{:?}", std::thread::current().id()),
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all hooks to complete
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(join_error) => {
                    results.push(IndividualHookResult {
                        hook_id: "unknown".to_string(),
                        hook_name: "failed_parallel_hook".to_string(),
                        result: Err(anyhow!("Parallel execution failed: {}", join_error)),
                        execution_duration: Duration::ZERO,
                        execution_id,
                        thread_id: "unknown".to_string(),
                    });
                }
            }
        }
        
        Ok(results)
    }
}

// Hook dependency resolution
pub struct HookDependencyResolver {
    dependency_graph: HashMap<String, Vec<String>>,
    conflict_registry: HashMap<String, Vec<String>>,
}

impl HookDependencyResolver {
    pub fn resolve_execution_order(&self, hooks: &[Box<dyn Hook>]) -> Result<Vec<ResolvedHook>> {
        let mut resolved_hooks = Vec::new();
        let mut dependency_map = HashMap::new();
        
        // Build dependency map
        for hook in hooks {
            let dependencies = hook.dependencies();
            let conflicts = hook.conflicts();
            
            dependency_map.insert(hook.name().to_string(), HookDependencyInfo {
                dependencies: dependencies.to_vec(),
                conflicts: conflicts.to_vec(),
                priority: hook.priority(),
            });
        }
        
        // Topological sort with conflict detection
        let execution_order = self.topological_sort_with_conflicts(&dependency_map)?;
        
        // Create resolved hooks
        for hook_name in execution_order {
            if let Some(hook) = hooks.iter().find(|h| h.name() == hook_name) {
                resolved_hooks.push(ResolvedHook {
                    id: format!("{}_{}", hook_name, Uuid::new_v4()),
                    hook: hook.clone(),
                    dependencies_resolved: true,
                    conflicts_checked: true,
                });
            }
        }
        
        Ok(resolved_hooks)
    }
}
```

### Cross-Engine Hook Integration

```rust
// Complete cross-engine hook system
pub struct CrossEngineHookSystem {
    native_hooks: HashMap<HookPoint, Vec<Box<dyn Hook>>>,
    lua_hooks: HashMap<HookPoint, Vec<LuaHookWrapper>>,
    js_hooks: HashMap<HookPoint, Vec<JSHookWrapper>>,
    hook_translator: HookTranslator,
    execution_coordinator: CrossEngineExecutionCoordinator,
}

pub struct LuaHookWrapper {
    name: String,
    lua_function: mlua::Function<'static>,
    priority: i32,
    execution_mode: HookExecutionMode,
    lua_runtime: Arc<mlua::Lua>,
}

#[async_trait]
impl Hook for LuaHookWrapper {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn priority(&self) -> i32 {
        self.priority
    }
    
    fn execution_mode(&self) -> HookExecutionMode {
        self.execution_mode
    }
    
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Convert context to Lua table
        let lua_context = self.hook_translator.context_to_lua(context)?;
        
        // Execute Lua hook function
        let lua_result = match self.execution_mode {
            HookExecutionMode::Synchronous => {
                // Direct function call
                self.lua_function.call::<_, mlua::Value>(lua_context)?
            },
            HookExecutionMode::Asynchronous => {
                // Coroutine-based async execution
                self.execute_lua_coroutine(lua_context).await?
            }
        };
        
        // Convert result back to Rust
        self.hook_translator.lua_result_to_hook_result(lua_result)
    }
}

impl LuaHookWrapper {
    async fn execute_lua_coroutine(&self, context: mlua::Table) -> Result<mlua::Value> {
        let coroutine = self.lua_runtime.create_thread(self.lua_function.clone())?;
        
        // Start coroutine execution
        let mut result = coroutine.resume::<_, mlua::Value>(context)?;
        
        // Handle yielding coroutines
        while coroutine.status() == mlua::ThreadStatus::Resumable {
            // Yield control to allow other tasks
            tokio::task::yield_now().await;
            
            // Resume coroutine
            result = coroutine.resume::<_, mlua::Value>(())?;
        }
        
        Ok(result)
    }
}

// JavaScript hook wrapper with Promise support
pub struct JSHookWrapper {
    name: String,
    js_function: JSFunction,
    priority: i32,
    execution_mode: HookExecutionMode,
    js_runtime: Arc<JSRuntime>,
}

#[async_trait]
impl Hook for JSHookWrapper {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn priority(&self) -> i32 {
        self.priority
    }
    
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Convert context to JavaScript object
        let js_context = self.hook_translator.context_to_js(context)?;
        
        // Execute JavaScript hook function
        let js_result = match self.execution_mode {
            HookExecutionMode::Synchronous => {
                self.js_function.call_sync(js_context)?
            },
            HookExecutionMode::Asynchronous => {
                self.js_function.call_async(js_context).await?
            }
        };
        
        // Convert result back to Rust
        self.hook_translator.js_result_to_hook_result(js_result)
    }
}

// Cross-engine execution coordinator
pub struct CrossEngineExecutionCoordinator {
    execution_strategy: CrossEngineExecutionStrategy,
}

#[derive(Debug, Clone)]
pub enum CrossEngineExecutionStrategy {
    // Execute each engine sequentially
    Sequential { engine_order: Vec<ScriptEngine> },
    
    // Execute engines in parallel, merge results
    Parallel { conflict_resolution: ConflictResolution },
    
    // Primary engine with fallbacks
    PrimaryWithFallback { 
        primary: ScriptEngine, 
        fallbacks: Vec<ScriptEngine> 
    },
    
    // Vote-based execution (majority wins)
    Consensus { 
        voting_engines: Vec<ScriptEngine>,
        consensus_threshold: f64 
    },
}

impl CrossEngineExecutionCoordinator {
    pub async fn execute_cross_engine_hooks(
        &self,
        hook_point: HookPoint,
        context: &mut HookContext,
        hook_system: &CrossEngineHookSystem
    ) -> Result<CrossEngineHookResult> {
        match &self.execution_strategy {
            CrossEngineExecutionStrategy::Sequential { engine_order } => {
                self.execute_sequential(hook_point, context, hook_system, engine_order).await
            },
            CrossEngineExecutionStrategy::Parallel { conflict_resolution } => {
                self.execute_parallel(hook_point, context, hook_system, conflict_resolution).await
            },
            CrossEngineExecutionStrategy::PrimaryWithFallback { primary, fallbacks } => {
                self.execute_with_fallback(hook_point, context, hook_system, primary, fallbacks).await
            },
            CrossEngineExecutionStrategy::Consensus { voting_engines, consensus_threshold } => {
                self.execute_consensus(hook_point, context, hook_system, voting_engines, *consensus_threshold).await
            }
        }
    }
    
    async fn execute_parallel(
        &self,
        hook_point: HookPoint,
        context: &mut HookContext,
        hook_system: &CrossEngineHookSystem,
        conflict_resolution: &ConflictResolution
    ) -> Result<CrossEngineHookResult> {
        let mut handles = Vec::new();
        
        // Execute native hooks
        if let Some(native_hooks) = hook_system.native_hooks.get(&hook_point) {
            let native_context = context.clone();
            let native_hooks = native_hooks.clone();
            
            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                for hook in native_hooks {
                    let result = hook.execute(&mut native_context.clone()).await;
                    results.push(EngineHookResult::Native(result));
                }
                results
            });
            
            handles.push(handle);
        }
        
        // Execute Lua hooks
        if let Some(lua_hooks) = hook_system.lua_hooks.get(&hook_point) {
            let lua_context = context.clone();
            let lua_hooks = lua_hooks.clone();
            
            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                for hook in lua_hooks {
                    let result = hook.execute(&mut lua_context.clone()).await;
                    results.push(EngineHookResult::Lua(result));
                }
                results
            });
            
            handles.push(handle);
        }
        
        // Execute JavaScript hooks
        if let Some(js_hooks) = hook_system.js_hooks.get(&hook_point) {
            let js_context = context.clone();
            let js_hooks = js_hooks.clone();
            
            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                for hook in js_hooks {
                    let result = hook.execute(&mut js_context.clone()).await;
                    results.push(EngineHookResult::JavaScript(result));
                }
                results
            });
            
            handles.push(handle);
        }
        
        // Collect all results
        let mut all_results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(mut engine_results) => all_results.append(&mut engine_results),
                Err(error) => {
                    return Err(anyhow!("Cross-engine hook execution failed: {}", error));
                }
            }
        }
        
        // Resolve conflicts and merge results
        let merged_result = self.resolve_conflicts_and_merge(all_results, conflict_resolution, context)?;
        
        Ok(merged_result)
    }
}
```

## Complete Event System Architecture

### High-Performance Event Bus

```rust
// Production-ready event bus implementation
pub struct HighPerformanceEventBus {
    subscribers: Arc<RwLock<HashMap<String, Vec<EventSubscriber>>>>,
    wildcard_subscribers: Arc<RwLock<Vec<EventSubscriber>>>,
    event_queue: Arc<Mutex<VecDeque<Event>>>,
    worker_pool: Arc<ThreadPool>,
    metrics: Arc<Mutex<EventMetrics>>,
    rate_limiter: RateLimiter,
    event_filter: EventFilter,
    persistence: Option<EventPersistence>,
}

#[derive(Debug, Clone)]
pub struct EventSubscriber {
    id: SubscriptionId,
    handler: SubscriberHandler,
    filter: Option<EventFilter>,
    priority: i32,
    max_concurrent: Option<u32>,
    timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum SubscriberHandler {
    Native(Arc<dyn Fn(Event) -> Result<()> + Send + Sync>),
    Lua(LuaEventHandler),
    JavaScript(JSEventHandler),
    Webhook(WebhookHandler),
}

impl HighPerformanceEventBus {
    pub async fn emit_event_complete(&self, event: Event) -> Result<CompleteEmissionResult> {
        let emission_id = EmissionId::new();
        let start_time = Instant::now();
        
        // Rate limiting check
        if !self.rate_limiter.try_acquire(&event.event_type).await? {
            return Err(anyhow!("Rate limit exceeded for event type: {}", event.event_type));
        }
        
        // Event filtering
        if !self.event_filter.should_emit(&event)? {
            return Ok(CompleteEmissionResult::filtered(emission_id, event));
        }
        
        // Persist event if configured
        if let Some(persistence) = &self.persistence {
            persistence.store_event(&event).await?;
        }
        
        // Get relevant subscribers
        let relevant_subscribers = self.get_relevant_subscribers(&event).await?;
        
        if relevant_subscribers.is_empty() {
            return Ok(CompleteEmissionResult::no_subscribers(emission_id, event));
        }
        
        // Emit to all relevant subscribers
        let emission_results = self.emit_to_subscribers(&event, &relevant_subscribers, emission_id).await?;
        
        let total_duration = start_time.elapsed();
        
        // Record metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.record_emission(&event, emission_results.len(), total_duration);
        }
        
        Ok(CompleteEmissionResult {
            emission_id,
            event: event.clone(),
            subscriber_results: emission_results,
            total_duration,
            subscribers_notified: relevant_subscribers.len(),
            successful_deliveries: self.count_successful_deliveries(&emission_results),
            failed_deliveries: self.count_failed_deliveries(&emission_results),
        })
    }
    
    async fn emit_to_subscribers(
        &self,
        event: &Event,
        subscribers: &[EventSubscriber],
        emission_id: EmissionId
    ) -> Result<Vec<SubscriberEmissionResult>> {
        let mut handles = Vec::new();
        
        // Group subscribers by concurrency limits
        let grouped_subscribers = self.group_subscribers_by_concurrency(subscribers);
        
        for (max_concurrent, subscriber_group) in grouped_subscribers {
            let semaphore = Arc::new(Semaphore::new(max_concurrent));
            
            for subscriber in subscriber_group {
                let event = event.clone();
                let subscriber = subscriber.clone();
                let emission_id = emission_id;
                let semaphore = Arc::clone(&semaphore);
                
                let handle = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    let start_time = Instant::now();
                    let delivery_result = Self::deliver_to_subscriber(&event, &subscriber, emission_id).await;
                    let delivery_duration = start_time.elapsed();
                    
                    SubscriberEmissionResult {
                        subscriber_id: subscriber.id,
                        emission_id,
                        delivery_result,
                        delivery_duration,
                        delivery_timestamp: Utc::now(),
                    }
                });
                
                handles.push(handle);
            }
        }
        
        // Collect all results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(join_error) => {
                    results.push(SubscriberEmissionResult {
                        subscriber_id: SubscriptionId::unknown(),
                        emission_id,
                        delivery_result: Err(anyhow!("Delivery task failed: {}", join_error)),
                        delivery_duration: Duration::ZERO,
                        delivery_timestamp: Utc::now(),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    async fn deliver_to_subscriber(
        event: &Event,
        subscriber: &EventSubscriber,
        emission_id: EmissionId
    ) -> Result<DeliveryResult> {
        // Apply timeout if configured
        let delivery_future = Self::execute_subscriber_handler(event, subscriber, emission_id);
        
        if let Some(timeout) = subscriber.timeout {
            match tokio::time::timeout(timeout, delivery_future).await {
                Ok(result) => result,
                Err(_) => Err(anyhow!("Subscriber handler timed out after {:?}", timeout))
            }
        } else {
            delivery_future.await
        }
    }
    
    async fn execute_subscriber_handler(
        event: &Event,
        subscriber: &EventSubscriber,
        emission_id: EmissionId
    ) -> Result<DeliveryResult> {
        match &subscriber.handler {
            SubscriberHandler::Native(handler) => {
                handler(event.clone())?;
                Ok(DeliveryResult::Success)
            },
            SubscriberHandler::Lua(lua_handler) => {
                lua_handler.handle_event(event, emission_id).await
            },
            SubscriberHandler::JavaScript(js_handler) => {
                js_handler.handle_event(event, emission_id).await
            },
            SubscriberHandler::Webhook(webhook_handler) => {
                webhook_handler.deliver_event(event, emission_id).await
            }
        }
    }
}

// Cross-engine event handling
pub struct LuaEventHandler {
    lua_function: mlua::Function<'static>,
    lua_runtime: Arc<mlua::Lua>,
    error_handling: ErrorHandlingStrategy,
}

impl LuaEventHandler {
    pub async fn handle_event(&self, event: &Event, emission_id: EmissionId) -> Result<DeliveryResult> {
        // Convert event to Lua table
        let lua_event = self.convert_event_to_lua(event)?;
        
        // Execute Lua handler
        match self.lua_function.call::<_, mlua::Value>(lua_event) {
            Ok(_) => Ok(DeliveryResult::Success),
            Err(error) => {
                match &self.error_handling {
                    ErrorHandlingStrategy::FailFast => {
                        Err(anyhow!("Lua event handler failed: {}", error))
                    },
                    ErrorHandlingStrategy::LogAndContinue => {
                        log::error!("Lua event handler error: {}", error);
                        Ok(DeliveryResult::PartialSuccess)
                    },
                    ErrorHandlingStrategy::Retry { max_attempts, delay } => {
                        self.retry_lua_handler(event, *max_attempts, *delay).await
                    }
                }
            }
        }
    }
}

pub struct JSEventHandler {
    js_function: JSFunction,
    js_runtime: Arc<JSRuntime>,
    error_handling: ErrorHandlingStrategy,
}

impl JSEventHandler {
    pub async fn handle_event(&self, event: &Event, emission_id: EmissionId) -> Result<DeliveryResult> {
        // Convert event to JavaScript object
        let js_event = self.convert_event_to_js(event)?;
        
        // Execute JavaScript handler (with Promise support)
        match self.js_function.call_async(js_event).await {
            Ok(_) => Ok(DeliveryResult::Success),
            Err(error) => {
                match &self.error_handling {
                    ErrorHandlingStrategy::FailFast => {
                        Err(anyhow!("JavaScript event handler failed: {}", error))
                    },
                    ErrorHandlingStrategy::LogAndContinue => {
                        log::error!("JavaScript event handler error: {}", error);
                        Ok(DeliveryResult::PartialSuccess)
                    },
                    ErrorHandlingStrategy::Retry { max_attempts, delay } => {
                        self.retry_js_handler(event, *max_attempts, *delay).await
                    }
                }
            }
        }
    }
}
```

### Event Persistence and Replay

```rust
// Event persistence for reliability and replay
pub struct EventPersistence {
    storage_backend: StorageBackend,
    retention_policy: RetentionPolicy,
    compression: CompressionConfig,
}

#[derive(Debug, Clone)]
pub enum StorageBackend {
    InMemory { max_events: usize },
    File { directory: PathBuf, file_rotation: FileRotation },
    Database { connection_string: String, table_name: String },
    S3 { bucket: String, prefix: String, region: String },
}

impl EventPersistence {
    pub async fn store_event(&self, event: &Event) -> Result<StorageResult> {
        // Apply compression if configured
        let serialized_event = match &self.compression {
            CompressionConfig::None => serde_json::to_vec(event)?,
            CompressionConfig::Gzip => self.compress_gzip(event)?,
            CompressionConfig::Lz4 => self.compress_lz4(event)?,
        };
        
        // Store based on backend
        match &self.storage_backend {
            StorageBackend::InMemory { .. } => {
                self.store_in_memory(event).await
            },
            StorageBackend::File { directory, file_rotation } => {
                self.store_to_file(event, directory, file_rotation).await
            },
            StorageBackend::Database { connection_string, table_name } => {
                self.store_to_database(event, connection_string, table_name).await
            },
            StorageBackend::S3 { bucket, prefix, region } => {
                self.store_to_s3(event, bucket, prefix, region).await
            }
        }
    }
    
    pub async fn replay_events(
        &self,
        filter: EventReplayFilter,
        target: ReplayTarget
    ) -> Result<ReplayResult> {
        let events = self.retrieve_events_for_replay(&filter).await?;
        let mut successful_replays = 0;
        let mut failed_replays = 0;
        
        for event in events {
            match self.replay_single_event(&event, &target).await {
                Ok(_) => successful_replays += 1,
                Err(error) => {
                    failed_replays += 1;
                    log::error!("Event replay failed: {} - {}", event.sequence, error);
                }
            }
        }
        
        Ok(ReplayResult {
            total_events: successful_replays + failed_replays,
            successful_replays,
            failed_replays,
            replay_duration: filter.time_range.duration(),
        })
    }
}

// Event filtering and routing
pub struct EventFilter {
    rules: Vec<FilterRule>,
    default_action: FilterAction,
}

#[derive(Debug, Clone)]
pub struct FilterRule {
    condition: FilterCondition,
    action: FilterAction,
    priority: i32,
}

#[derive(Debug, Clone)]
pub enum FilterCondition {
    EventType(String),
    EventTypePattern(regex::Regex),
    SourceEquals(String),
    SourcePattern(regex::Regex),
    DataContains(String, serde_json::Value),
    Custom(Box<dyn Fn(&Event) -> bool + Send + Sync>),
    And(Vec<FilterCondition>),
    Or(Vec<FilterCondition>),
    Not(Box<FilterCondition>),
}

impl EventFilter {
    pub fn should_emit(&self, event: &Event) -> Result<bool> {
        // Apply rules in priority order
        let mut sorted_rules = self.rules.clone();
        sorted_rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        
        for rule in sorted_rules {
            if self.evaluate_condition(&rule.condition, event)? {
                return Ok(match rule.action {
                    FilterAction::Allow => true,
                    FilterAction::Deny => false,
                    FilterAction::Transform(_) => true, // Allow but mark for transformation
                });
            }
        }
        
        // No rule matched, use default action
        Ok(match self.default_action {
            FilterAction::Allow => true,
            FilterAction::Deny => false,
            FilterAction::Transform(_) => true,
        })
    }
}
```

## Integration with Agent Execution

### Complete Agent Hook Integration

```rust
// Complete agent execution with full hook integration
pub struct HookIntegratedAgentExecutor {
    agents: HashMap<String, Box<dyn Agent>>,
    hook_manager: OptimizedHookManager,
    event_bus: HighPerformanceEventBus,
    execution_tracker: ExecutionTracker,
}

impl HookIntegratedAgentExecutor {
    pub async fn execute_agent_with_complete_integration(
        &mut self,
        agent_id: &str,
        input: AgentInput
    ) -> Result<CompleteAgentExecutionResult> {
        let execution_id = ExecutionId::new();
        let start_time = Instant::now();
        
        // Create execution context
        let mut execution_context = CompleteExecutionContext {
            execution_id,
            agent_id: agent_id.to_string(),
            input: input.clone(),
            start_time,
            hook_context: HookContext::new(agent_id, input.clone()),
            events_emitted: Vec::new(),
            hook_executions: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // Phase 1: Pre-execution hooks
        let pre_execution_result = self.hook_manager.execute_hooks_complete(
            HookPoint::BeforeAgentExecution,
            &mut execution_context.hook_context
        ).await?;
        
        execution_context.hook_executions.push(pre_execution_result);
        
        // Emit agent execution started event
        let start_event = Event {
            event_type: "agent_execution_started".to_string(),
            data: json!({
                "execution_id": execution_id.to_string(),
                "agent_id": agent_id,
                "input": input
            }),
            timestamp: Utc::now(),
            sequence: 0,
            source: "agent_executor".to_string(),
        };
        
        let start_emission = self.event_bus.emit_event_complete(start_event.clone()).await?;
        execution_context.events_emitted.push(start_emission);
        
        // Phase 2: Tool preparation hooks
        let agent = self.agents.get_mut(agent_id)
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;
        
        if !agent.tools().is_empty() {
            let tool_prep_result = self.hook_manager.execute_hooks_complete(
                HookPoint::BeforeToolsAvailable,
                &mut execution_context.hook_context
            ).await?;
            
            execution_context.hook_executions.push(tool_prep_result);
        }
        
        // Phase 3: Agent execution with tool monitoring
        let agent_result = self.execute_agent_with_tool_monitoring(
            agent,
            &mut execution_context
        ).await;
        
        // Phase 4: Post-execution hooks (always run)
        execution_context.hook_context.agent_output = agent_result.as_ref().ok().cloned();
        execution_context.hook_context.execution_error = agent_result.as_ref().err().map(|e| e.to_string());
        
        let post_execution_result = self.hook_manager.execute_hooks_complete(
            HookPoint::AfterAgentExecution,
            &mut execution_context.hook_context
        ).await?;
        
        execution_context.hook_executions.push(post_execution_result);
        
        // Phase 5: Final event emission
        let completion_event = Event {
            event_type: "agent_execution_completed".to_string(),
            data: json!({
                "execution_id": execution_id.to_string(),
                "agent_id": agent_id,
                "success": agent_result.is_ok(),
                "total_duration_ms": start_time.elapsed().as_millis(),
                "hook_executions": execution_context.hook_executions.len(),
                "events_emitted": execution_context.events_emitted.len()
            }),
            timestamp: Utc::now(),
            sequence: 1,
            source: "agent_executor".to_string(),
        };
        
        let completion_emission = self.event_bus.emit_event_complete(completion_event).await?;
        execution_context.events_emitted.push(completion_emission);
        
        // Create complete result
        let complete_result = CompleteAgentExecutionResult {
            execution_id,
            agent_id: agent_id.to_string(),
            agent_result,
            execution_context,
            total_duration: start_time.elapsed(),
        };
        
        Ok(complete_result)
    }
    
    async fn execute_agent_with_tool_monitoring(
        &mut self,
        agent: &mut Box<dyn Agent>,
        execution_context: &mut CompleteExecutionContext
    ) -> Result<AgentOutput> {
        let mut tool_usage_tracker = ToolUsageTracker::new();
        
        // Execute agent with tool monitoring
        let result = agent.execute(execution_context.input.clone()).await;
        
        // Check for tool usage and emit events
        if let Ok(ref output) = result {
            if let Some(tools_used) = output.metadata.get("tools_used") {
                if let Value::Array(tool_names) = tools_used {
                    for tool_name in tool_names {
                        if let Value::String(name) = tool_name {
                            // Emit tool usage event
                            let tool_event = Event {
                                event_type: "tool_used".to_string(),
                                data: json!({
                                    "execution_id": execution_context.execution_id.to_string(),
                                    "agent_id": execution_context.agent_id,
                                    "tool_name": name
                                }),
                                timestamp: Utc::now(),
                                sequence: execution_context.events_emitted.len() as u64,
                                source: execution_context.agent_id.clone(),
                            };
                            
                            let tool_emission = self.event_bus.emit_event_complete(tool_event).await?;
                            execution_context.events_emitted.push(tool_emission);
                            
                            // Execute tool-specific hooks
                            execution_context.hook_context.metadata.insert(
                                "current_tool".to_string(),
                                Value::String(name.clone())
                            );
                            
                            let tool_hook_result = self.hook_manager.execute_hooks_complete(
                                HookPoint::AfterToolUsed,
                                &mut execution_context.hook_context
                            ).await?;
                            
                            execution_context.hook_executions.push(tool_hook_result);
                        }
                    }
                }
            }
        }
        
        result
    }
}
```

This complete hook/event integration provides production-ready functionality with cross-engine compatibility, comprehensive error handling, performance optimization, and full observability features.