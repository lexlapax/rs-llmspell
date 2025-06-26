# Scripting Engine Concurrency and Async Patterns Analysis

## Overview

This document analyzes concurrency and async patterns for single-threaded scripting engines (Lua and JavaScript) embedded in Rust, focusing on cooperative scheduling, async abstractions, and cross-engine compatibility for rs-llmspell's agent orchestration system.

## Lua Threading Limitations and Workarounds

### mlua Async Support Capabilities

**Core Features:**
- **Async/Await Support**: mlua provides async/await for all Lua versions including LuaJIT and Luau
- **Executor Agnostic**: Works with any Rust async executor (tokio, async-std, smol)
- **Feature Flags**: Requires `features = ["async"]` in Cargo.toml
- **Send/Sync Support**: Optional `send` feature makes mlua::Lua: Send + Sync

**Architecture:**
```toml
mlua = { version = "0.9", features = ["lua54", "async", "send"] }
```

**Key Components:**
- `mlua::Thread` represents Lua coroutines
- Bridge between Rust futures and Lua coroutines
- Error handling via longjmp (requires careful stack frame management)

### Lua Coroutines vs True Async Patterns

**Lua Coroutine Model:**
- **Asymmetric Coroutines**: True asymmetric coroutines with explicit yield/resume
- **Cooperative Multitasking**: Voluntary yielding via `coroutine.yield()`
- **States**: suspended → running → normal → dead
- **Data Exchange**: Resume-yield pairs can exchange data

**Coroutine Lifecycle:**
```lua
-- Creation
co = coroutine.create(function() ... end)

-- Execution control
coroutine.resume(co, args)  -- Start/continue
coroutine.yield(values)     -- Suspend and return values
coroutine.status(co)        -- Check state
```

**Advantages:**
- Lightweight (no thread overhead)
- Simple linear programming model
- Fine-grained execution control
- Single-threaded (no synchronization needed)

**Limitations:**
- Single-threaded execution only
- No true parallelism
- Manual yield points required
- Stack-based (potential stack overflow in deep recursion)

### Cooperative Scheduling Implementation Strategies

**Strategy 1: Time-Slicing with Yield Points**
```lua
function long_running_task()
    for i = 1, 1000000 do
        -- Do work
        if i % 1000 == 0 then
            coroutine.yield()  -- Yield every 1000 iterations
        end
    end
end
```

**Strategy 2: Event-Driven Yielding**
```lua
function io_task()
    local result = start_async_io()
    while not result.ready do
        coroutine.yield()  -- Yield until IO complete
    end
    return result.data
end
```

**Strategy 3: Priority-Based Scheduling**
```lua
-- High priority tasks yield less frequently
-- Low priority tasks yield more frequently
function priority_aware_task(priority)
    local yield_frequency = priority == "high" and 10000 or 100
    for i = 1, work_size do
        -- Work
        if i % yield_frequency == 0 then
            coroutine.yield()
        end
    end
end
```

### Yield-Based Programming Models for Long Operations

**Pattern 1: Chunked Processing**
```lua
function process_large_dataset(data)
    local chunk_size = 1000
    local results = {}
    
    for i = 1, #data, chunk_size do
        local chunk = table.move(data, i, math.min(i + chunk_size - 1, #data), 1, {})
        -- Process chunk
        table.insert(results, process_chunk(chunk))
        coroutine.yield()  -- Yield after each chunk
    end
    
    return results
end
```

**Pattern 2: Stream Processing**
```lua
function stream_processor(input_stream)
    while true do
        local item = input_stream:next()
        if not item then break end
        
        -- Process item
        local result = process_item(item)
        output_stream:send(result)
        
        coroutine.yield()  -- Yield after each item
    end
end
```

**Pattern 3: State Machine with Yields**
```lua
function state_machine_task()
    local state = "init"
    local data = {}
    
    while state ~= "done" do
        if state == "init" then
            data = initialize()
            state = "processing"
        elseif state == "processing" then
            data = process_step(data)
            if processing_complete(data) then
                state = "cleanup"
            end
        elseif state == "cleanup" then
            cleanup(data)
            state = "done"
        end
        
        coroutine.yield()  -- Yield after each state transition
    end
end
```

## JavaScript Async Patterns in Embedded Engines

### Promise Implementation in Rust JS Engines

**Boa JavaScript Engine:**
- Embeddable JavaScript engine written in Rust
- Active development of "Full async/await support" project
- Passes 80%+ of ECMAScript test262 test suite
- AsyncGenerator support indicates advanced async features

**V8 Engine Integration:**
- Mature async/await implementation with optimizations
- Complex promise handling (originally 3 microtasks per await, optimized to 1)
- Sophisticated microtask queue management
- Full ECMAScript compliance

### Event Loop Integration with Tokio Runtime

**Challenge: Bridging Event Loops**
- JavaScript's event loop vs Tokio's async runtime
- Different concurrency models need integration
- Microtask vs macrotask scheduling differences

**Integration Patterns:**

**Pattern 1: Tokio-Driven Event Loop**
```rust
// Rust drives the event loop, JS responds to events
async fn run_js_with_tokio(engine: &mut JsEngine) {
    loop {
        // Process JS microtasks
        engine.run_microtasks();
        
        // Yield to Tokio
        tokio::task::yield_now().await;
        
        // Check for external events
        if let Some(event) = event_receiver.try_recv() {
            engine.emit_event(event);
        }
    }
}
```

**Pattern 2: Hybrid Scheduling**
```rust
// Alternate between JS event loop and Tokio tasks
async fn hybrid_scheduling(js_engine: &mut JsEngine, tokio_tasks: &mut TaskPool) {
    let mut js_budget = 100;  // JS gets 100ms
    let mut tokio_budget = 50; // Tokio gets 50ms
    
    loop {
        // Run JS for budget
        let js_start = Instant::now();
        while js_start.elapsed().as_millis() < js_budget && js_engine.has_work() {
            js_engine.run_microtasks();
        }
        
        // Run Tokio tasks for budget
        let tokio_start = Instant::now();
        while tokio_start.elapsed().as_millis() < tokio_budget {
            tokio_tasks.run_one().await;
        }
    }
}
```

### async/await Simulation Patterns

**Promise-like Abstractions for Scripts:**

**Pattern 1: Future-like Objects**
```javascript
// JavaScript side
class ScriptFuture {
    constructor(executor) {
        this.state = 'pending';
        this.value = undefined;
        this.callbacks = [];
        
        executor(
            (value) => this.resolve(value),
            (error) => this.reject(error)
        );
    }
    
    then(onResolve, onReject) {
        if (this.state === 'resolved') {
            onResolve(this.value);
        } else if (this.state === 'rejected') {
            onReject(this.value);
        } else {
            this.callbacks.push({ onResolve, onReject });
        }
    }
    
    resolve(value) {
        this.state = 'resolved';
        this.value = value;
        this.callbacks.forEach(cb => cb.onResolve(value));
    }
}
```

**Pattern 2: Async Function Simulation**
```javascript
// Simulated async/await using generators
function* asyncFunction() {
    const result1 = yield callAsyncOperation1();
    const result2 = yield callAsyncOperation2(result1);
    return result2;
}

function runAsync(generator) {
    const gen = generator();
    
    function step(value) {
        const result = gen.next(value);
        
        if (result.done) {
            return result.value;
        }
        
        // Assume result.value is a "future-like" object
        result.value.then(step);
    }
    
    step();
}
```

### Worker Thread Alternatives for CPU-Intensive Tasks

**Strategy 1: Chunked Execution**
```javascript
function cpuIntensiveTask(data, chunkSize = 1000) {
    return new Promise((resolve) => {
        let index = 0;
        const results = [];
        
        function processChunk() {
            const end = Math.min(index + chunkSize, data.length);
            
            for (let i = index; i < end; i++) {
                results.push(expensiveOperation(data[i]));
            }
            
            index = end;
            
            if (index < data.length) {
                setTimeout(processChunk, 0); // Yield to event loop
            } else {
                resolve(results);
            }
        }
        
        processChunk();
    });
}
```

**Strategy 2: Rust-Side Processing**
```rust
// Offload CPU-intensive work to Rust
impl JsRuntime {
    fn register_cpu_intensive_functions(&mut self) {
        self.register_function("processLargeDataset", |data: Vec<Value>| async move {
            // Process in Rust with proper async/await
            tokio::task::spawn_blocking(move || {
                // CPU-intensive work in thread pool
                data.into_iter().map(|item| expensive_rust_operation(item)).collect()
            }).await
        });
    }
}
```

## Cross-Engine Async Pattern Standardization

### Common Async Interface for Lua and JavaScript

**Unified Future/Promise Abstraction:**

```rust
// Rust side: Common async interface
#[async_trait]
pub trait ScriptAsync {
    type Value;
    type Error;
    
    async fn execute(&mut self, script: &str) -> Result<Self::Value, Self::Error>;
    async fn call_function(&mut self, name: &str, args: Vec<Self::Value>) -> Result<Self::Value, Self::Error>;
    fn create_future(&mut self) -> ScriptFuture<Self::Value>;
    fn yield_execution(&mut self) -> impl Future<Output = ()>;
}

pub struct ScriptFuture<T> {
    inner: Box<dyn Future<Output = T> + Send>,
}

impl<T> ScriptFuture<T> {
    pub fn new<F: Future<Output = T> + Send + 'static>(future: F) -> Self {
        Self { inner: Box::new(future) }
    }
}
```

**Lua Implementation:**
```rust
impl ScriptAsync for LuaRuntime {
    type Value = mlua::Value;
    type Error = mlua::Error;
    
    async fn execute(&mut self, script: &str) -> Result<Self::Value, Self::Error> {
        let thread = self.lua.create_thread(
            self.lua.load(script).into_function()?
        )?;
        
        // Cooperative execution with yields
        thread.resume_async(()).await
    }
    
    fn yield_execution(&mut self) -> impl Future<Output = ()> {
        // Yield to Tokio runtime
        tokio::task::yield_now()
    }
}
```

**JavaScript Implementation:**
```rust
impl ScriptAsync for JsRuntime {
    type Value = boa_engine::JsValue;
    type Error = boa_engine::JsError;
    
    async fn execute(&mut self, script: &str) -> Result<Self::Value, Self::Error> {
        // Run script with microtask processing
        let result = self.engine.eval(script)?;
        
        // Process microtasks until completion
        while self.engine.has_pending_microtasks() {
            self.engine.run_microtasks();
            tokio::task::yield_now().await;
        }
        
        Ok(result)
    }
}
```

### Promise/Future-like Abstractions for Scripts

**Unified Script Future API:**

```rust
pub struct UnifiedScriptFuture {
    engine_type: EngineType,
    lua_thread: Option<mlua::Thread>,
    js_promise: Option<boa_engine::JsPromise>,
    state: FutureState,
}

impl UnifiedScriptFuture {
    pub fn from_lua(thread: mlua::Thread) -> Self {
        Self {
            engine_type: EngineType::Lua,
            lua_thread: Some(thread),
            js_promise: None,
            state: FutureState::Pending,
        }
    }
    
    pub fn from_js(promise: boa_engine::JsPromise) -> Self {
        Self {
            engine_type: EngineType::JavaScript,
            lua_thread: None,
            js_promise: Some(promise),
            state: FutureState::Pending,
        }
    }
}

impl Future for UnifiedScriptFuture {
    type Output = Result<ScriptValue, ScriptError>;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.engine_type {
            EngineType::Lua => {
                // Poll Lua coroutine
                if let Some(ref thread) = self.lua_thread {
                    match thread.status() {
                        mlua::ThreadStatus::Resumable => {
                            // Resume coroutine and check if complete
                            match thread.resume(()) {
                                Ok(value) => Poll::Ready(Ok(ScriptValue::from_lua(value))),
                                Err(_) => {
                                    // Still yielded, wake later
                                    cx.waker().wake_by_ref();
                                    Poll::Pending
                                }
                            }
                        }
                        _ => Poll::Ready(Err(ScriptError::ThreadDead))
                    }
                } else {
                    Poll::Ready(Err(ScriptError::NoThread))
                }
            }
            EngineType::JavaScript => {
                // Poll JS promise
                if let Some(ref promise) = self.js_promise {
                    match promise.state() {
                        PromiseState::Fulfilled(value) => Poll::Ready(Ok(ScriptValue::from_js(value))),
                        PromiseState::Rejected(error) => Poll::Ready(Err(ScriptError::from_js(error))),
                        PromiseState::Pending => {
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    }
                } else {
                    Poll::Ready(Err(ScriptError::NoPromise))
                }
            }
        }
    }
}
```

### Error Handling in Async Script Contexts

**Unified Error Handling:**

```rust
#[derive(Debug)]
pub enum ScriptError {
    LuaError(mlua::Error),
    JsError(boa_engine::JsError),
    AsyncCancelled,
    TimeoutExceeded,
    ResourceExhausted,
}

impl ScriptError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::AsyncCancelled | Self::TimeoutExceeded => true,
            Self::LuaError(e) => !e.is_syntax_error(),
            Self::JsError(e) => !matches!(e.kind(), boa_engine::ErrorKind::Syntax),
            Self::ResourceExhausted => false,
        }
    }
}

// Error propagation in async contexts
pub async fn safe_script_execution<T>(
    future: impl Future<Output = Result<T, ScriptError>>,
    timeout: Duration,
) -> Result<T, ScriptError> {
    tokio::time::timeout(timeout, future)
        .await
        .map_err(|_| ScriptError::TimeoutExceeded)?
}
```

### Resource Cleanup in Interrupted Async Operations

**Cleanup Strategy:**

```rust
pub struct ScriptExecutor {
    lua_runtime: Option<LuaRuntime>,
    js_runtime: Option<JsRuntime>,
    active_operations: Vec<OperationHandle>,
}

impl ScriptExecutor {
    pub async fn execute_with_cleanup<T>(
        &mut self,
        operation: impl Future<Output = Result<T, ScriptError>>,
    ) -> Result<T, ScriptError> {
        let handle = self.track_operation();
        
        let result = tokio::select! {
            result = operation => result,
            _ = self.cancellation_token.cancelled() => {
                Err(ScriptError::AsyncCancelled)
            }
        };
        
        // Cleanup regardless of result
        self.cleanup_operation(handle).await;
        result
    }
    
    async fn cleanup_operation(&mut self, handle: OperationHandle) {
        // Cleanup Lua resources
        if let Some(ref mut lua) = self.lua_runtime {
            lua.cleanup_threads();
            lua.run_gc();
        }
        
        // Cleanup JS resources
        if let Some(ref mut js) = self.js_runtime {
            js.clear_microtasks();
            js.run_gc();
        }
        
        // Remove from active operations
        self.active_operations.retain(|h| h.id != handle.id);
    }
}

impl Drop for ScriptExecutor {
    fn drop(&mut self) {
        // Ensure all operations are cleaned up
        for handle in &self.active_operations {
            // Cancel any pending operations
            handle.cancel();
        }
    }
}
```

## Performance and Fairness Considerations

### Script Execution Time Slicing

**Time-Budget Scheduling:**

```rust
pub struct ScriptScheduler {
    lua_budget: Duration,
    js_budget: Duration,
    total_budget: Duration,
    current_engine: Option<EngineType>,
    last_switch: Instant,
}

impl ScriptScheduler {
    pub async fn run_with_fairness(&mut self, scripts: Vec<ScriptTask>) -> Vec<ScriptResult> {
        let mut results = Vec::new();
        let start_time = Instant::now();
        
        while !scripts.is_empty() && start_time.elapsed() < self.total_budget {
            for script in &mut scripts {
                let engine_budget = match script.engine_type {
                    EngineType::Lua => self.lua_budget,
                    EngineType::JavaScript => self.js_budget,
                };
                
                let execution_start = Instant::now();
                
                // Execute with time limit
                let result = tokio::time::timeout(
                    engine_budget,
                    script.execute_step()
                ).await;
                
                match result {
                    Ok(Ok(value)) if script.is_complete() => {
                        results.push(ScriptResult::Completed(value));
                        script.mark_done();
                    }
                    Ok(Ok(_)) => {
                        // Still running, will continue next round
                    }
                    Ok(Err(e)) => {
                        results.push(ScriptResult::Error(e));
                        script.mark_done();
                    }
                    Err(_) => {
                        // Timeout, yield to next script
                        script.yield_execution();
                    }
                }
                
                // Yield to Tokio after each script slice
                tokio::task::yield_now().await;
            }
            
            // Remove completed scripts
            scripts.retain(|s| !s.is_done());
        }
        
        results
    }
}
```

### Resource Allocation Between Concurrent Scripts

**Resource Pool Management:**

```rust
pub struct ScriptResourcePool {
    max_memory_per_script: usize,
    max_concurrent_scripts: usize,
    memory_tracker: HashMap<ScriptId, usize>,
    cpu_usage_tracker: HashMap<ScriptId, Duration>,
}

impl ScriptResourcePool {
    pub fn can_allocate(&self, script_id: ScriptId, memory_request: usize) -> bool {
        let current_memory = self.memory_tracker.get(&script_id).unwrap_or(&0);
        *current_memory + memory_request <= self.max_memory_per_script
    }
    
    pub fn allocate_memory(&mut self, script_id: ScriptId, size: usize) -> Result<(), ResourceError> {
        if !self.can_allocate(script_id, size) {
            return Err(ResourceError::MemoryExhausted);
        }
        
        *self.memory_tracker.entry(script_id).or_insert(0) += size;
        Ok(())
    }
    
    pub fn track_cpu_usage(&mut self, script_id: ScriptId, duration: Duration) {
        *self.cpu_usage_tracker.entry(script_id).or_insert(Duration::ZERO) += duration;
    }
    
    pub fn get_resource_stats(&self, script_id: ScriptId) -> ResourceStats {
        ResourceStats {
            memory_used: *self.memory_tracker.get(&script_id).unwrap_or(&0),
            cpu_time: *self.cpu_usage_tracker.get(&script_id).unwrap_or(&Duration::ZERO),
        }
    }
}
```

### Memory Management in Long-Running Async Operations

**Memory Management Strategy:**

```rust
pub struct AsyncMemoryManager {
    gc_threshold: usize,
    gc_interval: Duration,
    last_gc: Instant,
    memory_pressure: f64,
}

impl AsyncMemoryManager {
    pub async fn manage_memory(&mut self, runtimes: &mut [Box<dyn ScriptRuntime>]) {
        let current_memory = self.get_total_memory_usage();
        
        // Check if GC is needed
        if current_memory > self.gc_threshold || 
           self.last_gc.elapsed() > self.gc_interval ||
           self.memory_pressure > 0.8 {
            
            self.run_gc_cycle(runtimes).await;
        }
        
        // Update memory pressure
        self.memory_pressure = current_memory as f64 / self.gc_threshold as f64;
    }
    
    async fn run_gc_cycle(&mut self, runtimes: &mut [Box<dyn ScriptRuntime>]) {
        for runtime in runtimes {
            // Yield between each runtime GC
            runtime.run_gc();
            tokio::task::yield_now().await;
        }
        
        self.last_gc = Instant::now();
    }
    
    pub fn should_yield_for_gc(&self) -> bool {
        self.memory_pressure > 0.9
    }
}
```

### Debugging and Profiling Async Script Execution

**Debugging Infrastructure:**

```rust
pub struct AsyncScriptDebugger {
    call_stack: Vec<CallFrame>,
    execution_history: VecDeque<ExecutionEvent>,
    performance_metrics: HashMap<String, PerformanceMetrics>,
    breakpoints: HashSet<Breakpoint>,
}

#[derive(Debug)]
pub struct ExecutionEvent {
    timestamp: Instant,
    script_id: ScriptId,
    event_type: EventType,
    location: SourceLocation,
    data: serde_json::Value,
}

#[derive(Debug)]
pub enum EventType {
    FunctionCall,
    FunctionReturn,
    CoroutineYield,
    CoroutineResume,
    PromiseCreated,
    PromiseResolved,
    PromiseRejected,
    GcTriggered,
    MemoryAllocated,
    Error,
}

impl AsyncScriptDebugger {
    pub fn trace_execution(&mut self, event: ExecutionEvent) {
        self.execution_history.push_back(event);
        
        // Keep history bounded
        if self.execution_history.len() > 10000 {
            self.execution_history.pop_front();
        }
        
        // Check breakpoints
        if self.should_break(&event) {
            self.trigger_breakpoint(event);
        }
    }
    
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            total_execution_time: self.calculate_total_execution_time(),
            function_call_counts: self.get_function_call_counts(),
            memory_usage_over_time: self.get_memory_usage_history(),
            yield_frequency: self.calculate_yield_frequency(),
            gc_impact: self.calculate_gc_impact(),
        }
    }
    
    pub async fn profile_async_operation<T>(
        &mut self,
        operation: impl Future<Output = T>,
        operation_name: &str,
    ) -> T {
        let start_time = Instant::now();
        let start_memory = self.get_current_memory_usage();
        
        let result = operation.await;
        
        let end_time = Instant::now();
        let end_memory = self.get_current_memory_usage();
        
        self.record_performance_metrics(PerformanceMetrics {
            operation_name: operation_name.to_string(),
            duration: end_time - start_time,
            memory_delta: end_memory as i64 - start_memory as i64,
            yield_count: self.count_yields_in_timeframe(start_time, end_time),
        });
        
        result
    }
}
```

## Agent Orchestration Async Patterns

### Parallel Agent Execution Without True Threading

**Multi-Agent Cooperative Scheduling:**

```rust
pub struct AgentOrchestrator {
    agents: HashMap<AgentId, Box<dyn Agent>>,
    agent_states: HashMap<AgentId, AgentExecutionState>,
    execution_queue: VecDeque<AgentTask>,
    scheduler: CooperativeScheduler,
}

#[derive(Debug)]
pub enum AgentExecutionState {
    Idle,
    Running { started_at: Instant },
    Waiting { for_event: EventType },
    Yielded { resume_at: Instant },
    Blocked { on_resource: ResourceId },
    Complete { result: AgentResult },
}

impl AgentOrchestrator {
    pub async fn execute_agents_cooperatively(&mut self) -> Vec<AgentResult> {
        let mut results = Vec::new();
        
        while !self.execution_queue.is_empty() {
            let current_task = self.execution_queue.pop_front().unwrap();
            let agent_id = current_task.agent_id;
            
            // Check if agent can run
            if !self.can_agent_run(agent_id) {
                // Re-queue for later
                self.execution_queue.push_back(current_task);
                continue;
            }
            
            // Execute agent with time budget
            let execution_start = Instant::now();
            let budget = self.scheduler.get_time_budget(agent_id);
            
            match self.execute_agent_step(agent_id, current_task, budget).await {
                AgentStepResult::Completed(result) => {
                    results.push(result);
                    self.agent_states.insert(agent_id, AgentExecutionState::Complete { result });
                }
                AgentStepResult::Yielded => {
                    self.agent_states.insert(agent_id, AgentExecutionState::Yielded { 
                        resume_at: execution_start + budget 
                    });
                    // Re-queue for next cycle
                    self.execution_queue.push_back(current_task);
                }
                AgentStepResult::WaitingForEvent(event_type) => {
                    self.agent_states.insert(agent_id, AgentExecutionState::Waiting { 
                        for_event: event_type 
                    });
                    // Will be re-queued when event occurs
                }
                AgentStepResult::Blocked(resource_id) => {
                    self.agent_states.insert(agent_id, AgentExecutionState::Blocked { 
                        on_resource: resource_id 
                    });
                    // Check resource availability and re-queue if available
                }
            }
            
            // Yield to Tokio after each agent step
            tokio::task::yield_now().await;
        }
        
        results
    }
    
    async fn execute_agent_step(
        &mut self, 
        agent_id: AgentId, 
        task: AgentTask, 
        budget: Duration
    ) -> AgentStepResult {
        let agent = self.agents.get_mut(&agent_id).unwrap();
        
        // Set execution budget for the agent
        agent.set_execution_budget(budget);
        
        // Execute with timeout
        match tokio::time::timeout(budget, agent.execute_task(task)).await {
            Ok(result) => result,
            Err(_) => {
                // Budget exceeded, yield
                agent.yield_execution();
                AgentStepResult::Yielded
            }
        }
    }
}
```

**Message-Passing Between Agents:**

```rust
pub struct AgentCommunication {
    message_bus: tokio::sync::mpsc::UnboundedSender<AgentMessage>,
    agent_inboxes: HashMap<AgentId, tokio::sync::mpsc::UnboundedReceiver<AgentMessage>>,
    pending_responses: HashMap<MessageId, tokio::sync::oneshot::Sender<AgentResponse>>,
}

#[derive(Debug)]
pub struct AgentMessage {
    id: MessageId,
    from: AgentId,
    to: AgentId,
    message_type: MessageType,
    payload: serde_json::Value,
    response_required: bool,
}

impl AgentCommunication {
    pub async fn send_message(&mut self, message: AgentMessage) -> Result<(), CommunicationError> {
        if message.response_required {
            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
            self.pending_responses.insert(message.id, response_tx);
            
            // Send message
            self.message_bus.send(message)?;
            
            // Wait for response with timeout
            match tokio::time::timeout(Duration::from_secs(30), response_rx).await {
                Ok(Ok(response)) => Ok(()),
                Ok(Err(_)) => Err(CommunicationError::ResponseChannelClosed),
                Err(_) => Err(CommunicationError::ResponseTimeout),
            }
        } else {
            self.message_bus.send(message)?;
            Ok(())
        }
    }
    
    pub async fn broadcast_message(&mut self, message: AgentMessage) -> Result<(), CommunicationError> {
        // Send to all agents except sender
        for agent_id in self.agent_inboxes.keys() {
            if *agent_id != message.from {
                let mut broadcast_message = message.clone();
                broadcast_message.to = *agent_id;
                self.send_message(broadcast_message).await?;
            }
        }
        Ok(())
    }
}
```

### Tool Execution Scheduling and Queuing

**Tool Execution Pool:**

```rust
pub struct ToolExecutionPool {
    available_tools: HashMap<ToolId, Box<dyn Tool>>,
    execution_queue: tokio::sync::mpsc::UnboundedReceiver<ToolExecutionRequest>,
    result_sender: tokio::sync::mpsc::UnboundedSender<ToolExecutionResult>,
    concurrent_limit: usize,
    active_executions: tokio::task::JoinSet<ToolExecutionResult>,
}

impl ToolExecutionPool {
    pub async fn run_execution_loop(&mut self) {
        loop {
            tokio::select! {
                // New tool execution request
                Some(request) = self.execution_queue.recv() => {
                    if self.active_executions.len() < self.concurrent_limit {
                        self.start_tool_execution(request).await;
                    } else {
                        // Queue is full, handle backpressure
                        self.handle_backpressure(request).await;
                    }
                }
                
                // Tool execution completed
                Some(result) = self.active_executions.join_next() => {
                    match result {
                        Ok(execution_result) => {
                            self.result_sender.send(execution_result).unwrap();
                        }
                        Err(join_error) => {
                            eprintln!("Tool execution panicked: {:?}", join_error);
                        }
                    }
                }
                
                // Graceful shutdown
                _ = self.shutdown_signal.recv() => {
                    break;
                }
            }
        }
        
        // Wait for all active executions to complete
        while let Some(result) = self.active_executions.join_next().await {
            // Handle remaining results
        }
    }
    
    async fn start_tool_execution(&mut self, request: ToolExecutionRequest) {
        let tool = self.available_tools.get(&request.tool_id).cloned();
        
        if let Some(tool) = tool {
            let execution_future = async move {
                let start_time = Instant::now();
                let result = tool.execute(request.parameters).await;
                
                ToolExecutionResult {
                    request_id: request.id,
                    tool_id: request.tool_id,
                    result,
                    execution_time: start_time.elapsed(),
                }
            };
            
            self.active_executions.spawn(execution_future);
        }
    }
}
```

### Stream Processing with Cooperative Yielding

**Stream-Based Agent Communication:**

```rust
pub struct StreamProcessor {
    input_streams: HashMap<StreamId, Box<dyn AsyncRead + Unpin>>,
    output_streams: HashMap<StreamId, Box<dyn AsyncWrite + Unpin>>,
    processing_tasks: tokio::task::JoinSet<()>,
}

impl StreamProcessor {
    pub async fn process_stream_cooperatively(
        &mut self, 
        stream_id: StreamId,
        processor: impl Fn(Vec<u8>) -> Vec<u8> + Send + 'static,
    ) {
        let input_stream = self.input_streams.remove(&stream_id).unwrap();
        let output_stream = self.output_streams.get_mut(&stream_id).unwrap();
        
        let processing_task = async move {
            let mut reader = tokio::io::BufReader::new(input_stream);
            let mut buffer = Vec::with_capacity(4096);
            let mut yield_counter = 0;
            
            loop {
                // Read chunk from stream
                match reader.read_buf(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(bytes_read) => {
                        // Process the chunk
                        let processed_data = processor(buffer.clone());
                        
                        // Write to output stream
                        if let Err(e) = output_stream.write_all(&processed_data).await {
                            eprintln!("Stream write error: {:?}", e);
                            break;
                        }
                        
                        buffer.clear();
                        yield_counter += 1;
                        
                        // Yield periodically for cooperative multitasking
                        if yield_counter % 100 == 0 {
                            tokio::task::yield_now().await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Stream read error: {:?}", e);
                        break;
                    }
                }
            }
        };
        
        self.processing_tasks.spawn(processing_task);
    }
}
```

### Hook System Non-Blocking Execution

**Async Hook System:**

```rust
pub struct AsyncHookSystem {
    hooks: HashMap<HookPoint, Vec<Box<dyn AsyncHook>>>,
    hook_execution_pool: tokio::task::JoinSet<HookResult>,
    max_concurrent_hooks: usize,
}

#[async_trait]
pub trait AsyncHook: Send + Sync {
    async fn execute(&self, context: HookContext) -> HookResult;
    fn priority(&self) -> HookPriority;
    fn timeout(&self) -> Duration;
}

impl AsyncHookSystem {
    pub async fn execute_hooks_non_blocking(
        &mut self, 
        hook_point: HookPoint, 
        context: HookContext
    ) -> Vec<HookResult> {
        let hooks = self.hooks.get(&hook_point).cloned().unwrap_or_default();
        let mut results = Vec::new();
        
        // Sort hooks by priority
        let mut sorted_hooks = hooks;
        sorted_hooks.sort_by_key(|hook| hook.priority());
        
        for hook in sorted_hooks {
            // Check if we have room for more concurrent executions
            while self.hook_execution_pool.len() >= self.max_concurrent_hooks {
                // Wait for one hook to complete
                if let Some(result) = self.hook_execution_pool.join_next().await {
                    match result {
                        Ok(hook_result) => results.push(hook_result),
                        Err(e) => eprintln!("Hook execution error: {:?}", e),
                    }
                }
            }
            
            // Execute hook with timeout
            let hook_timeout = hook.timeout();
            let context_clone = context.clone();
            
            let hook_future = async move {
                tokio::time::timeout(hook_timeout, hook.execute(context_clone)).await
                    .unwrap_or(HookResult::Timeout)
            };
            
            self.hook_execution_pool.spawn(hook_future);
        }
        
        // Wait for remaining hooks to complete
        while let Some(result) = self.hook_execution_pool.join_next().await {
            match result {
                Ok(hook_result) => results.push(hook_result),
                Err(e) => eprintln!("Hook execution error: {:?}", e),
            }
        }
        
        results
    }
}
```

## Workflow Engine Async Design

### Parallel Workflow Step Execution Strategies

**Workflow State Machine:**

```rust
#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    pub id: WorkflowId,
    pub steps: Vec<WorkflowStep>,
    pub transitions: HashMap<StepId, Vec<Transition>>,
    pub error_handlers: HashMap<StepId, ErrorHandler>,
}

#[derive(Debug, Clone)]
pub enum WorkflowStep {
    Sequential { steps: Vec<StepId> },
    Parallel { steps: Vec<StepId>, wait_for: WaitStrategy },
    Conditional { condition: Condition, true_step: StepId, false_step: StepId },
    Loop { condition: Condition, body: StepId, max_iterations: usize },
    AgentExecution { agent_id: AgentId, parameters: serde_json::Value },
    ToolExecution { tool_id: ToolId, parameters: serde_json::Value },
}

#[derive(Debug, Clone)]
pub enum WaitStrategy {
    All,           // Wait for all parallel steps
    Any,           // Wait for any one step
    Majority,      // Wait for majority (>50%)
    Count(usize),  // Wait for specific count
}

pub struct WorkflowEngine {
    active_workflows: HashMap<WorkflowInstanceId, WorkflowInstance>,
    step_executor: StepExecutor,
    state_store: Box<dyn WorkflowStateStore>,
    event_bus: tokio::sync::broadcast::Sender<WorkflowEvent>,
}

impl WorkflowEngine {
    pub async fn execute_workflow(&mut self, definition: WorkflowDefinition) -> WorkflowResult {
        let instance_id = WorkflowInstanceId::new();
        let mut instance = WorkflowInstance::new(instance_id, definition);
        
        while !instance.is_complete() {
            let current_step = instance.get_current_step();
            
            match current_step {
                WorkflowStep::Parallel { steps, wait_for } => {
                    self.execute_parallel_steps(&mut instance, steps, wait_for).await?;
                }
                WorkflowStep::Sequential { steps } => {
                    self.execute_sequential_steps(&mut instance, steps).await?;
                }
                WorkflowStep::Conditional { condition, true_step, false_step } => {
                    self.execute_conditional_step(&mut instance, condition, true_step, false_step).await?;
                }
                WorkflowStep::Loop { condition, body, max_iterations } => {
                    self.execute_loop_step(&mut instance, condition, body, max_iterations).await?;
                }
                _ => {
                    self.execute_single_step(&mut instance, current_step).await?;
                }
            }
            
            // Persist workflow state
            self.state_store.save_workflow_state(&instance).await?;
            
            // Yield to allow other workflows to execute
            tokio::task::yield_now().await;
        }
        
        instance.get_result()
    }
    
    async fn execute_parallel_steps(
        &mut self, 
        instance: &mut WorkflowInstance, 
        steps: Vec<StepId>,
        wait_strategy: WaitStrategy
    ) -> Result<(), WorkflowError> {
        let mut step_futures = tokio::task::JoinSet::new();
        
        // Start all parallel steps
        for step_id in steps {
            let step = instance.get_step(step_id).clone();
            let step_context = instance.get_step_context(step_id);
            
            let step_future = async move {
                self.step_executor.execute_step(step, step_context).await
            };
            
            step_futures.spawn(step_future);
        }
        
        // Wait according to strategy
        match wait_strategy {
            WaitStrategy::All => {
                let mut results = Vec::new();
                while let Some(result) = step_futures.join_next().await {
                    results.push(result??);
                }
                instance.set_parallel_results(results);
            }
            WaitStrategy::Any => {
                if let Some(result) = step_futures.join_next().await {
                    instance.set_result(result??);
                    // Cancel remaining steps
                    step_futures.abort_all();
                }
            }
            WaitStrategy::Majority => {
                let total_steps = step_futures.len();
                let required_count = (total_steps / 2) + 1;
                let mut completed_count = 0;
                let mut results = Vec::new();
                
                while completed_count < required_count {
                    if let Some(result) = step_futures.join_next().await {
                        results.push(result??);
                        completed_count += 1;
                    }
                }
                
                instance.set_parallel_results(results);
                step_futures.abort_all();
            }
            WaitStrategy::Count(required_count) => {
                let mut completed_count = 0;
                let mut results = Vec::new();
                
                while completed_count < required_count {
                    if let Some(result) = step_futures.join_next().await {
                        results.push(result??);
                        completed_count += 1;
                    }
                }
                
                instance.set_parallel_results(results);
                step_futures.abort_all();
            }
        }
        
        Ok(())
    }
}
```

### Sequential Workflow with Async Steps

**Sequential Execution with Async Support:**

```rust
impl WorkflowEngine {
    async fn execute_sequential_steps(
        &mut self, 
        instance: &mut WorkflowInstance, 
        steps: Vec<StepId>
    ) -> Result<(), WorkflowError> {
        for step_id in steps {
            let step = instance.get_step(step_id).clone();
            let step_context = instance.get_step_context(step_id);
            
            // Execute step and handle async operations
            let result = match step {
                WorkflowStep::AgentExecution { agent_id, parameters } => {
                    self.execute_agent_step(agent_id, parameters, step_context).await?
                }
                WorkflowStep::ToolExecution { tool_id, parameters } => {
                    self.execute_tool_step(tool_id, parameters, step_context).await?
                }
                _ => {
                    self.step_executor.execute_step(step, step_context).await?
                }
            };
            
            // Update instance with step result
            instance.set_step_result(step_id, result);
            
            // Check for step transition conditions
            if let Some(transition) = self.evaluate_step_transitions(instance, step_id) {
                instance.transition_to(transition.target_step);
            }
            
            // Yield between steps to allow cooperative multitasking
            tokio::task::yield_now().await;
        }
        
        Ok(())
    }
    
    async fn execute_agent_step(
        &mut self,
        agent_id: AgentId,
        parameters: serde_json::Value,
        context: StepContext
    ) -> Result<StepResult, WorkflowError> {
        // Create agent task
        let agent_task = AgentTask {
            id: TaskId::new(),
            agent_id,
            parameters,
            context: context.clone(),
        };
        
        // Execute with timeout and cooperative yielding
        let execution_future = async {
            let mut yield_counter = 0;
            loop {
                // Check if agent has completed the task
                match self.agent_orchestrator.get_agent_status(agent_id) {
                    AgentStatus::Complete(result) => return Ok(StepResult::Success(result)),
                    AgentStatus::Error(error) => return Err(WorkflowError::AgentError(error)),
                    AgentStatus::Running => {
                        // Agent still running, yield and continue
                        yield_counter += 1;
                        if yield_counter % 10 == 0 {
                            tokio::task::yield_now().await;
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        };
        
        // Execute with workflow-level timeout
        tokio::time::timeout(context.step_timeout, execution_future).await
            .map_err(|_| WorkflowError::StepTimeout)?
    }
}
```

### Conditional Workflows with Async Predicates

**Async Condition Evaluation:**

```rust
#[derive(Debug, Clone)]
pub enum Condition {
    ScriptExpression { script: String, engine: ScriptEngine },
    AgentPredicate { agent_id: AgentId, predicate_function: String },
    ToolResult { tool_id: ToolId, expected_result: serde_json::Value },
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

impl WorkflowEngine {
    async fn execute_conditional_step(
        &mut self,
        instance: &mut WorkflowInstance,
        condition: Condition,
        true_step: StepId,
        false_step: StepId
    ) -> Result<(), WorkflowError> {
        let condition_result = self.evaluate_condition_async(condition, instance).await?;
        
        let next_step = if condition_result { true_step } else { false_step };
        instance.transition_to(next_step);
        
        Ok(())
    }
    
    async fn evaluate_condition_async(
        &mut self,
        condition: Condition,
        instance: &WorkflowInstance
    ) -> Result<bool, WorkflowError> {
        match condition {
            Condition::ScriptExpression { script, engine } => {
                let script_context = instance.get_script_context();
                match engine {
                    ScriptEngine::Lua => {
                        self.lua_runtime.evaluate_boolean_expression(&script, script_context).await
                    }
                    ScriptEngine::JavaScript => {
                        self.js_runtime.evaluate_boolean_expression(&script, script_context).await
                    }
                }
            }
            Condition::AgentPredicate { agent_id, predicate_function } => {
                let agent_context = instance.get_agent_context(agent_id);
                self.agent_orchestrator.evaluate_predicate(agent_id, predicate_function, agent_context).await
            }
            Condition::ToolResult { tool_id, expected_result } => {
                let tool_result = self.tool_executor.get_last_result(tool_id).await?;
                Ok(tool_result == expected_result)
            }
            Condition::And(left, right) => {
                let left_result = self.evaluate_condition_async(*left, instance).await?;
                if !left_result {
                    return Ok(false);
                }
                self.evaluate_condition_async(*right, instance).await
            }
            Condition::Or(left, right) => {
                let left_result = self.evaluate_condition_async(*left, instance).await?;
                if left_result {
                    return Ok(true);
                }
                self.evaluate_condition_async(*right, instance).await
            }
            Condition::Not(inner) => {
                let inner_result = self.evaluate_condition_async(*inner, instance).await?;
                Ok(!inner_result)
            }
        }
    }
}
```

### Loop Workflows with Async Conditions and Bodies

**Async Loop Execution:**

```rust
impl WorkflowEngine {
    async fn execute_loop_step(
        &mut self,
        instance: &mut WorkflowInstance,
        condition: Condition,
        body_step: StepId,
        max_iterations: usize
    ) -> Result<(), WorkflowError> {
        let mut iteration_count = 0;
        
        while iteration_count < max_iterations {
            // Evaluate loop condition asynchronously
            let should_continue = self.evaluate_condition_async(condition.clone(), instance).await?;
            
            if !should_continue {
                break;
            }
            
            // Execute loop body
            let body_step = instance.get_step(body_step).clone();
            let step_context = instance.get_step_context(body_step);
            
            let result = self.step_executor.execute_step(body_step, step_context).await?;
            instance.set_loop_iteration_result(iteration_count, result);
            
            iteration_count += 1;
            
            // Yield between iterations for cooperative multitasking
            tokio::task::yield_now().await;
            
            // Optional: Add iteration delay to prevent tight loops
            if let Some(delay) = instance.get_loop_delay() {
                tokio::time::sleep(delay).await;
            }
        }
        
        instance.set_loop_completion(iteration_count);
        Ok(())
    }
}
```

## Summary

This analysis provides comprehensive patterns for implementing async operations in single-threaded scripting engines:

1. **Lua Coroutines** offer simple cooperative multitasking with explicit yield points
2. **JavaScript Promises** require complex event loop integration with Tokio
3. **Cross-Engine Abstractions** enable consistent async APIs across languages
4. **Agent Orchestration** uses cooperative scheduling and message-passing for multi-agent coordination
5. **Workflow Engines** implement state machines with async step execution and parallel processing
6. **Resource Management** ensures fair allocation and cleanup in async contexts
7. **Performance Monitoring** provides visibility into async script execution

These patterns form the foundation for rs-llmspell's agent orchestration system, enabling complex multi-agent workflows while maintaining responsiveness and resource efficiency.