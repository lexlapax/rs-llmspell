# Performance Optimization Patterns

## Overview

This document outlines performance optimization strategies for rs-llmspell's core systems, focusing on efficient hook execution, event system optimization, and tool execution pooling. These patterns ensure the framework can scale to production workloads while maintaining responsiveness and resource efficiency.

## Efficient Hook Execution Patterns

### 1. Hook Execution Pipeline Optimization

**Problem**: Traditional hook execution can create performance bottlenecks when many hooks are registered or when hooks perform expensive operations.

**Solution**: Implement a multi-stage hook execution pipeline with parallel execution, caching, and selective execution.

**Architecture**:
```rust
pub struct OptimizedHookManager {
    hooks: HashMap<HookPoint, HookRegistry>,
    execution_pool: ThreadPool,
    hook_cache: Arc<RwLock<HookCache>>,
    performance_monitor: HookPerformanceMonitor,
    execution_strategy: HookExecutionStrategy,
}

pub struct HookRegistry {
    hooks: Vec<RegisteredHook>,
    execution_graph: ExecutionGraph,
    parallel_groups: Vec<ParallelGroup>,
}

pub struct RegisteredHook {
    id: String,
    handler: Box<dyn Hook>,
    priority: i32,
    execution_mode: ExecutionMode,
    dependencies: Vec<String>,
    performance_profile: HookPerformanceProfile,
    cache_strategy: CacheStrategy,
}

#[derive(Clone)]
pub enum ExecutionMode {
    Synchronous,
    Asynchronous,
    Background,
    Parallel { group: String },
    Conditional { condition: Box<dyn Condition> },
}

impl OptimizedHookManager {
    pub async fn execute_hooks(&self, point: HookPoint, context: &mut HookContext) -> Result<HookExecutionResult> {
        let registry = self.hooks.get(&point)
            .ok_or_else(|| anyhow!("No hooks registered for point: {:?}", point))?;
        
        // Check cache first
        if let Some(cached_result) = self.check_cache(&point, context).await? {
            return Ok(cached_result);
        }
        
        // Plan execution based on strategy
        let execution_plan = self.create_execution_plan(registry, context).await?;
        
        // Execute hooks according to plan
        let results = match self.execution_strategy {
            HookExecutionStrategy::Sequential => {
                self.execute_sequential(&execution_plan, context).await?
            },
            HookExecutionStrategy::Parallel => {
                self.execute_parallel(&execution_plan, context).await?
            },
            HookExecutionStrategy::Adaptive => {
                self.execute_adaptive(&execution_plan, context).await?
            },
        };
        
        // Cache results if applicable
        self.cache_results(&point, context, &results).await?;
        
        // Update performance metrics
        self.performance_monitor.record_execution(&point, &results);
        
        Ok(results)
    }
    
    async fn execute_parallel(&self, plan: &ExecutionPlan, context: &mut HookContext) -> Result<HookExecutionResult> {
        let mut all_results = HookExecutionResult::new();
        
        // Execute parallel groups
        for parallel_group in &plan.parallel_groups {
            let group_futures = parallel_group.hooks.iter().map(|hook| {
                let mut group_context = context.clone();
                async move {
                    let start_time = Instant::now();
                    let result = hook.execute(&mut group_context).await;
                    let duration = start_time.elapsed();
                    
                    HookResult {
                        hook_id: hook.id.clone(),
                        result,
                        duration,
                        context: group_context,
                    }
                }
            });
            
            let group_results = try_join_all(group_futures).await?;
            
            // Merge results from parallel execution
            for hook_result in group_results {
                all_results.add_hook_result(hook_result);
                
                // Merge context changes (careful with conflicts)
                context.merge_non_conflicting(&hook_result.context)?;
            }
        }
        
        // Execute sequential hooks that depend on parallel results
        for sequential_hook in &plan.sequential_hooks {
            let hook_result = sequential_hook.execute(context).await?;
            all_results.add_hook_result(hook_result);
        }
        
        Ok(all_results)
    }
    
    async fn execute_adaptive(&self, plan: &ExecutionPlan, context: &mut HookContext) -> Result<HookExecutionResult> {
        let mut results = HookExecutionResult::new();
        let mut execution_budget = Duration::from_millis(100); // Adaptive budget
        let start_time = Instant::now();
        
        // Sort hooks by priority and expected execution time
        let mut ordered_hooks = plan.all_hooks.clone();
        ordered_hooks.sort_by(|a, b| {
            let a_score = self.calculate_execution_score(a, &execution_budget);
            let b_score = self.calculate_execution_score(b, &execution_budget);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for hook in ordered_hooks {
            let remaining_budget = execution_budget.saturating_sub(start_time.elapsed());
            
            if remaining_budget < hook.performance_profile.min_execution_time {
                // Skip expensive hooks if budget is low
                if hook.execution_mode != ExecutionMode::Background {
                    continue;
                }
            }
            
            match hook.execution_mode {
                ExecutionMode::Background => {
                    // Execute in background without blocking
                    self.execute_in_background(hook, context.clone()).await?;
                },
                _ => {
                    let hook_result = hook.execute(context).await?;
                    results.add_hook_result(hook_result);
                    
                    // Adjust budget based on actual execution time
                    if hook_result.duration > hook.performance_profile.expected_duration * 2 {
                        execution_budget = execution_budget.saturating_sub(hook_result.duration);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn calculate_execution_score(&self, hook: &RegisteredHook, budget: &Duration) -> f64 {
        let priority_score = hook.priority as f64;
        let time_score = if hook.performance_profile.expected_duration <= *budget {
            1.0
        } else {
            budget.as_millis() as f64 / hook.performance_profile.expected_duration.as_millis() as f64
        };
        let reliability_score = hook.performance_profile.success_rate;
        
        priority_score * time_score * reliability_score
    }
}

// Hook caching strategies
#[derive(Clone)]
pub enum CacheStrategy {
    None,
    InputBased { ttl: Duration },
    ResultBased { ttl: Duration, invalidation_keys: Vec<String> },
    Adaptive { base_ttl: Duration, performance_factor: f64 },
}

pub struct HookCache {
    entries: HashMap<String, CacheEntry>,
    lru_tracker: LruCache<String, ()>,
}

impl HookCache {
    fn generate_cache_key(&self, point: &HookPoint, context: &HookContext, hook_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        point.hash(&mut hasher);
        context.input_hash().hash(&mut hasher);
        hook_id.hash(&mut hasher);
        
        format!("hook_{}_{}", hook_id, hasher.finish())
    }
}
```

**Lua Usage Example**:
```lua
-- Configure optimized hook execution
local hook_manager = OptimizedHookManager.new({
    execution_strategy = "adaptive",
    max_execution_time = 200, -- milliseconds
    parallel_execution = true,
    cache_enabled = true
})

-- Register hooks with performance profiles
hook_manager:register("before_llm_call", {
    handler = function(context)
        -- Lightweight validation
        return validate_input(context.input)
    end,
    priority = 100,
    execution_mode = "parallel",
    expected_duration = 5, -- milliseconds
    cache_strategy = "input_based"
})

hook_manager:register("before_llm_call", {
    handler = function(context)
        -- Expensive preprocessing
        return preprocess_context(context)
    end,
    priority = 50,
    execution_mode = "conditional",
    condition = function(context) 
        return context.requires_preprocessing 
    end,
    expected_duration = 50,
    cache_strategy = "result_based"
})

hook_manager:register("after_llm_call", {
    handler = function(context)
        -- Background metrics collection
        collect_metrics(context)
    end,
    priority = 10,
    execution_mode = "background",
    expected_duration = 20
})

-- Hooks execute efficiently based on strategy
local agent = Agent.new({
    system_prompt = "You are a helpful assistant",
    hook_manager = hook_manager
})
```

### 2. Hook Dependency Graph Optimization

**Problem**: Hook execution order and dependencies can create inefficiencies and deadlocks.

**Solution**: Build dependency graphs and optimize execution paths.

**Architecture**:
```rust
pub struct HookDependencyGraph {
    nodes: HashMap<String, HookNode>,
    edges: Vec<DependencyEdge>,
    execution_levels: Vec<Vec<String>>,
    critical_path: Vec<String>,
}

pub struct HookNode {
    hook_id: String,
    dependencies: Vec<String>,
    dependents: Vec<String>,
    execution_weight: u64,
    parallelizable: bool,
}

impl HookDependencyGraph {
    pub fn build_execution_plan(&self) -> Result<OptimizedExecutionPlan> {
        // Topological sort to determine execution order
        let sorted_hooks = self.topological_sort()?;
        
        // Identify parallel execution opportunities
        let parallel_groups = self.find_parallel_groups(&sorted_hooks)?;
        
        // Calculate critical path for optimization
        let critical_path = self.calculate_critical_path()?;
        
        Ok(OptimizedExecutionPlan {
            execution_levels: self.execution_levels.clone(),
            parallel_groups,
            critical_path,
            estimated_duration: self.calculate_total_duration(),
        })
    }
    
    fn find_parallel_groups(&self, sorted_hooks: &[String]) -> Result<Vec<ParallelExecutionGroup>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        let mut executed_hooks = HashSet::new();
        
        for hook_id in sorted_hooks {
            let hook = &self.nodes[hook_id];
            
            // Check if all dependencies are satisfied
            let dependencies_satisfied = hook.dependencies.iter()
                .all(|dep| executed_hooks.contains(dep));
                
            if dependencies_satisfied && hook.parallelizable {
                current_group.push(hook_id.clone());
            } else {
                // Finalize current group if it has multiple hooks
                if current_group.len() > 1 {
                    groups.push(ParallelExecutionGroup {
                        hooks: current_group.clone(),
                        estimated_duration: self.calculate_group_duration(&current_group),
                    });
                }
                current_group.clear();
                current_group.push(hook_id.clone());
            }
            
            executed_hooks.insert(hook_id.clone());
        }
        
        Ok(groups)
    }
}
```

## Event System Optimization

### 1. High-Performance Event Bus

**Problem**: Traditional pub/sub event systems can become bottlenecks under high throughput.

**Solution**: Implement a lock-free, multi-producer, multi-consumer event bus with intelligent routing.

**Architecture**:
```rust
use crossbeam::channel::{Receiver, Sender};
use crossbeam::queue::SegQueue;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct HighPerformanceEventBus {
    channels: HashMap<String, EventChannel>,
    global_sequence: AtomicU64,
    routing_table: Arc<RwLock<RoutingTable>>,
    subscriber_pools: HashMap<String, SubscriberPool>,
    event_buffer: SegQueue<BufferedEvent>,
    performance_metrics: EventBusMetrics,
}

pub struct EventChannel {
    sender: Sender<Event>,
    subscriber_count: AtomicU64,
    throughput_limiter: Option<RateLimiter>,
    priority: EventPriority,
}

pub struct SubscriberPool {
    subscribers: Vec<Box<dyn EventSubscriber>>,
    load_balancer: Box<dyn LoadBalancer>,
    circuit_breaker: CircuitBreaker,
}

impl HighPerformanceEventBus {
    pub async fn emit_event(&self, event: Event) -> Result<EmissionResult> {
        // Assign sequence number for ordering
        let sequence = self.global_sequence.fetch_add(1, Ordering::Relaxed);
        let mut event = event;
        event.sequence = sequence;
        
        // Route event based on type and content
        let routes = self.routing_table.read().await.find_routes(&event)?;
        
        if routes.is_empty() {
            return Ok(EmissionResult::NoSubscribers);
        }
        
        // Emit to all relevant channels
        let mut emission_futures = Vec::new();
        
        for route in routes {
            let channel = &self.channels[&route.channel];
            
            // Check rate limits
            if let Some(limiter) = &channel.throughput_limiter {
                if !limiter.try_acquire().await {
                    continue; // Skip if rate limited
                }
            }
            
            // Priority-based emission
            match route.priority {
                EventPriority::Critical => {
                    // Immediate emission
                    emission_futures.push(self.emit_immediate(&event, &route));
                },
                EventPriority::High => {
                    // High priority queue
                    emission_futures.push(self.emit_prioritized(&event, &route));
                },
                EventPriority::Normal => {
                    // Normal queuing
                    self.event_buffer.push(BufferedEvent {
                        event: event.clone(),
                        route: route.clone(),
                        timestamp: Instant::now(),
                    });
                },
                EventPriority::Low => {
                    // Background processing
                    self.emit_background(&event, &route).await?;
                }
            }
        }
        
        // Wait for critical and high priority emissions
        let results = try_join_all(emission_futures).await?;
        
        Ok(EmissionResult::Success {
            routes_count: routes.len(),
            immediate_results: results,
        })
    }
    
    async fn emit_immediate(&self, event: &Event, route: &Route) -> Result<EmissionSuccess> {
        let subscriber_pool = &self.subscriber_pools[&route.channel];
        
        // Load balance across subscribers
        let subscriber = subscriber_pool.load_balancer.select_subscriber(&subscriber_pool.subscribers)?;
        
        // Circuit breaker protection
        if subscriber_pool.circuit_breaker.should_allow_request() {
            match subscriber.handle_event(event).await {
                Ok(result) => {
                    subscriber_pool.circuit_breaker.record_success();
                    Ok(EmissionSuccess { subscriber_id: subscriber.id(), result })
                },
                Err(error) => {
                    subscriber_pool.circuit_breaker.record_failure();
                    Err(error)
                }
            }
        } else {
            Err(anyhow!("Circuit breaker is open for channel: {}", route.channel))
        }
    }
    
    // Background event processing worker
    pub async fn start_background_processor(&self) -> Result<()> {
        let buffer = self.event_buffer.clone();
        let channels = self.channels.clone();
        let metrics = self.performance_metrics.clone();
        
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);
            let mut last_flush = Instant::now();
            
            loop {
                // Collect events into batches
                while let Some(buffered_event) = buffer.pop() {
                    batch.push(buffered_event);
                    
                    if batch.len() >= 100 || last_flush.elapsed() > Duration::from_millis(10) {
                        break;
                    }
                }
                
                if !batch.is_empty() {
                    // Process batch
                    Self::process_event_batch(&batch, &channels, &metrics).await?;
                    batch.clear();
                    last_flush = Instant::now();
                }
                
                // Small yield to prevent tight loop
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
            
            Ok::<(), anyhow::Error>(())
        });
        
        Ok(())
    }
    
    async fn process_event_batch(
        batch: &[BufferedEvent],
        channels: &HashMap<String, EventChannel>,
        metrics: &EventBusMetrics
    ) -> Result<()> {
        // Group events by channel for efficient processing
        let mut channel_groups: HashMap<String, Vec<&BufferedEvent>> = HashMap::new();
        
        for buffered_event in batch {
            channel_groups.entry(buffered_event.route.channel.clone())
                .or_default()
                .push(buffered_event);
        }
        
        // Process each channel group
        let processing_futures = channel_groups.into_iter().map(|(channel, events)| {
            let channel_sender = channels[&channel].sender.clone();
            let metrics = metrics.clone();
            
            async move {
                let start_time = Instant::now();
                let mut success_count = 0;
                
                for buffered_event in events {
                    if channel_sender.try_send(buffered_event.event.clone()).is_ok() {
                        success_count += 1;
                    }
                }
                
                let duration = start_time.elapsed();
                metrics.record_batch_processing(&channel, success_count, events.len(), duration);
                
                Ok::<(), anyhow::Error>(())
            }
        });
        
        try_join_all(processing_futures).await?;
        Ok(())
    }
}

// Smart routing table for efficient event distribution
pub struct RoutingTable {
    static_routes: HashMap<String, Vec<Route>>,
    pattern_routes: Vec<PatternRoute>,
    dynamic_routes: HashMap<String, DynamicRoute>,
    bloom_filter: BloomFilter, // Fast negative lookups
}

impl RoutingTable {
    pub fn find_routes(&self, event: &Event) -> Result<Vec<Route>> {
        // Fast bloom filter check
        if !self.bloom_filter.might_contain(&event.event_type) {
            return Ok(Vec::new());
        }
        
        let mut routes = Vec::new();
        
        // Static routes (fastest)
        if let Some(static_routes) = self.static_routes.get(&event.event_type) {
            routes.extend_from_slice(static_routes);
        }
        
        // Pattern-based routes
        for pattern_route in &self.pattern_routes {
            if pattern_route.pattern.matches(event) {
                routes.push(pattern_route.route.clone());
            }
        }
        
        // Dynamic routes (context-based)
        for (_, dynamic_route) in &self.dynamic_routes {
            if dynamic_route.condition.evaluate(event) {
                routes.push(dynamic_route.route.clone());
            }
        }
        
        Ok(routes)
    }
}
```

**JavaScript Usage Example**:
```javascript
// High-performance event system setup
const eventBus = new HighPerformanceEventBus({
    defaultChannelCapacity: 10000,
    batchProcessingSize: 100,
    batchFlushInterval: 10, // milliseconds
    enableCircuitBreaker: true,
    
    channels: {
        'user_actions': {
            priority: 'high',
            rateLimitRps: 1000,
            subscriberPool: {
                loadBalancer: 'round_robin',
                circuitBreakerThreshold: 0.5
            }
        },
        
        'system_events': {
            priority: 'critical',
            subscriberPool: {
                loadBalancer: 'least_connections',
                circuitBreakerThreshold: 0.3
            }
        },
        
        'analytics': {
            priority: 'low',
            rateLimitRps: 500,
            batchingEnabled: true
        }
    }
});

// Smart routing configuration
eventBus.addRoute({
    eventType: 'user_login',
    channel: 'user_actions',
    priority: 'high'
});

eventBus.addPatternRoute({
    pattern: /^user_.*$/,
    channel: 'user_actions',
    priority: 'normal'
});

eventBus.addDynamicRoute({
    condition: (event) => event.data.userId && event.data.premium,
    channel: 'premium_user_actions',
    priority: 'high'
});

// High-throughput event emission
async function handleHighTraffic() {
    const events = await generateEvents(1000); // 1000 events
    
    // Batch emission for efficiency
    const results = await eventBus.emitBatch(events, {
        maxConcurrency: 10,
        failureStrategy: 'continue'
    });
    
    console.log(`Emitted ${results.successCount}/${events.length} events`);
    console.log(`Avg latency: ${results.avgLatency}ms`);
}

// Performance monitoring
eventBus.on('performance_stats', (stats) => {
    console.log(`Throughput: ${stats.eventsPerSecond} events/sec`);
    console.log(`Queue depth: ${stats.queueDepth}`);
    console.log(`Circuit breaker status: ${stats.circuitBreakerStatus}`);
});
```

### 2. Event Stream Backpressure Management

**Problem**: Event producers can overwhelm consumers, leading to memory issues and system instability.

**Solution**: Implement intelligent backpressure with adaptive rate limiting and flow control.

**Architecture**:
```rust
pub struct BackpressureManager {
    flow_controllers: HashMap<String, FlowController>,
    pressure_monitors: HashMap<String, PressureMonitor>,
    adaptive_limits: HashMap<String, AdaptiveRateLimit>,
    emergency_valve: EmergencyValve,
}

pub struct FlowController {
    current_rate: f64,
    target_rate: f64,
    max_rate: f64,
    adjustment_factor: f64,
    pressure_threshold: f64,
}

impl FlowController {
    pub async fn should_allow_event(&mut self, channel: &str) -> bool {
        let current_pressure = self.measure_pressure(channel).await;
        
        if current_pressure > self.pressure_threshold {
            // Reduce rate
            self.current_rate = (self.current_rate * (1.0 - self.adjustment_factor)).max(1.0);
            false
        } else {
            // Gradually increase rate
            self.current_rate = (self.current_rate * (1.0 + self.adjustment_factor * 0.1)).min(self.max_rate);
            true
        }
    }
    
    async fn measure_pressure(&self, channel: &str) -> f64 {
        // Composite pressure metric
        let queue_pressure = self.measure_queue_pressure(channel).await;
        let processing_pressure = self.measure_processing_pressure(channel).await;
        let memory_pressure = self.measure_memory_pressure().await;
        
        (queue_pressure * 0.4 + processing_pressure * 0.4 + memory_pressure * 0.2)
    }
}
```

## Tool Execution Pooling

### 1. Intelligent Tool Pool Management

**Problem**: Tool execution can be expensive, and creating new tool instances for each request is inefficient.

**Solution**: Implement smart pooling with warm-up, scaling, and specialization.

**Architecture**:
```rust
pub struct ToolExecutionPool {
    pools: HashMap<String, SpecializedPool>,
    pool_manager: PoolManager,
    load_balancer: Box<dyn LoadBalancer>,
    scaling_policies: HashMap<String, ScalingPolicy>,
    warm_up_scheduler: WarmUpScheduler,
}

pub struct SpecializedPool {
    tool_type: String,
    instances: VecDeque<PooledTool>,
    min_size: usize,
    max_size: usize,
    current_load: AtomicU64,
    performance_stats: PoolPerformanceStats,
}

pub struct PooledTool {
    tool: Box<dyn Tool>,
    instance_id: String,
    creation_time: Instant,
    last_used: Instant,
    usage_count: u64,
    health_status: ToolHealthStatus,
    warm_up_state: WarmUpState,
}

impl ToolExecutionPool {
    pub async fn execute_tool(&self, tool_request: ToolRequest) -> Result<ToolOutput> {
        let tool_type = &tool_request.tool_type;
        
        // Get or create specialized pool
        let pool = self.pools.get(tool_type)
            .ok_or_else(|| anyhow!("No pool configured for tool type: {}", tool_type))?;
        
        // Acquire tool instance from pool
        let mut pooled_tool = self.acquire_tool_instance(pool, &tool_request).await?;
        
        // Execute with performance monitoring
        let start_time = Instant::now();
        let result = self.execute_with_monitoring(&mut pooled_tool, tool_request).await;
        let execution_time = start_time.elapsed();
        
        // Update tool statistics
        pooled_tool.last_used = Instant::now();
        pooled_tool.usage_count += 1;
        
        // Return tool to pool or retire if needed
        self.return_or_retire_tool(pool, pooled_tool, execution_time, &result).await?;
        
        result
    }
    
    async fn acquire_tool_instance(&self, pool: &SpecializedPool, request: &ToolRequest) -> Result<PooledTool> {
        // Try to get existing instance
        if let Some(mut tool) = self.try_acquire_existing(pool).await {
            // Ensure tool is warmed up
            if tool.warm_up_state != WarmUpState::Ready {
                self.complete_warm_up(&mut tool, request).await?;
            }
            return Ok(tool);
        }
        
        // Create new instance if pool can grow
        if pool.instances.len() < pool.max_size {
            let new_tool = self.create_new_tool_instance(&pool.tool_type, request).await?;
            return Ok(new_tool);
        }
        
        // Wait for instance to become available
        self.wait_for_available_instance(pool).await
    }
    
    async fn create_new_tool_instance(&self, tool_type: &str, request: &ToolRequest) -> Result<PooledTool> {
        let tool = self.tool_factory.create_tool(tool_type, &request.config).await?;
        
        let mut pooled_tool = PooledTool {
            tool,
            instance_id: Uuid::new_v4().to_string(),
            creation_time: Instant::now(),
            last_used: Instant::now(),
            usage_count: 0,
            health_status: ToolHealthStatus::Healthy,
            warm_up_state: WarmUpState::Cold,
        };
        
        // Background warm-up for future requests
        self.schedule_warm_up(&mut pooled_tool, request).await?;
        
        Ok(pooled_tool)
    }
    
    async fn schedule_warm_up(&self, tool: &mut PooledTool, request: &ToolRequest) -> Result<()> {
        tool.warm_up_state = WarmUpState::Warming;
        
        let tool_id = tool.instance_id.clone();
        let warm_up_tasks = self.get_warm_up_tasks(&tool.tool, request);
        
        // Execute warm-up tasks in background
        tokio::spawn(async move {
            for task in warm_up_tasks {
                if let Err(e) = task.execute().await {
                    warn!("Warm-up task failed for tool {}: {}", tool_id, e);
                }
            }
        });
        
        tool.warm_up_state = WarmUpState::Ready;
        Ok(())
    }
}

// Adaptive scaling policies
pub struct AdaptiveScalingPolicy {
    target_utilization: f64,
    scale_up_threshold: f64,
    scale_down_threshold: f64,
    cooldown_period: Duration,
    last_scaling_action: Instant,
}

impl AdaptiveScalingPolicy {
    pub async fn should_scale(&self, pool: &SpecializedPool) -> Option<ScalingAction> {
        if self.last_scaling_action.elapsed() < self.cooldown_period {
            return None;
        }
        
        let current_utilization = self.calculate_utilization(pool).await;
        
        if current_utilization > self.scale_up_threshold && pool.instances.len() < pool.max_size {
            Some(ScalingAction::ScaleUp {
                target_size: (pool.instances.len() as f64 * 1.5).min(pool.max_size as f64) as usize
            })
        } else if current_utilization < self.scale_down_threshold && pool.instances.len() > pool.min_size {
            Some(ScalingAction::ScaleDown {
                target_size: (pool.instances.len() as f64 * 0.8).max(pool.min_size as f64) as usize
            })
        } else {
            None
        }
    }
}
```

**Lua Usage Example**:
```lua
-- Configure intelligent tool pooling
local tool_pool = ToolExecutionPool.new({
    pools = {
        web_search = {
            min_size = 2,
            max_size = 10,
            warm_up_tasks = {"cache_common_queries", "establish_connections"},
            scaling_policy = AdaptiveScalingPolicy.new({
                target_utilization = 0.7,
                scale_up_threshold = 0.8,
                scale_down_threshold = 0.3
            })
        },
        
        llm_analysis = {
            min_size = 1,
            max_size = 5,
            warm_up_tasks = {"load_model", "initialize_tokenizer"},
            scaling_policy = PredictiveScalingPolicy.new({
                prediction_window = 300, -- seconds
                confidence_threshold = 0.8
            })
        },
        
        file_processor = {
            min_size = 3,
            max_size = 15,
            warm_up_tasks = {},
            scaling_policy = LoadBasedScalingPolicy.new({
                requests_per_instance = 10
            })
        }
    },
    
    global_settings = {
        health_check_interval = 30,
        idle_timeout = 300,
        max_pool_memory = 1024 * 1024 * 1024, -- 1GB
        performance_monitoring = true
    }
})

-- Usage with automatic pooling
local research_agent = Agent.new({
    system_prompt = "You are a research assistant",
    tool_pool = tool_pool,
    tools = {
        WebSearchTool.new(),
        DocumentAnalyzerTool.new(),
        SummarizerTool.new()
    }
})

-- Pool automatically manages tool instances
local research_result = research_agent:chat("Research quantum computing applications")

-- Pool performance monitoring
print("Pool statistics:")
print("Web search pool utilization:", tool_pool:get_utilization("web_search"))
print("Average tool execution time:", tool_pool:get_avg_execution_time())
print("Pool scaling events:", tool_pool:get_scaling_history())
```

## Performance Monitoring and Optimization

### 1. Real-Time Performance Metrics

**Architecture**:
```rust
pub struct PerformanceMonitor {
    metrics_collector: MetricsCollector,
    performance_analyzer: PerformanceAnalyzer,
    optimization_engine: OptimizationEngine,
    alerting_system: AlertingSystem,
}

impl PerformanceMonitor {
    pub async fn start_monitoring(&self) -> Result<()> {
        // Collect metrics continuously
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                let metrics = self.collect_system_metrics().await?;
                self.analyze_performance(&metrics).await?;
                
                if let Some(optimization) = self.suggest_optimization(&metrics).await? {
                    self.apply_optimization(optimization).await?;
                }
            }
        });
        
        Ok(())
    }
}
```

This comprehensive performance optimization framework ensures rs-llmspell can handle production workloads efficiently while maintaining responsiveness and resource utilization.