# Complete Async Patterns Integration

## Overview

This document provides the complete implementation specification for rs-llmspell's async patterns integration across Rust, Lua, and JavaScript environments. It defines unified async interfaces, cooperative scheduling patterns, cross-engine coordination, and production-ready async execution strategies.

## Unified Async Architecture

### Core Async Abstraction Layer

```rust
// Universal async interface for all execution contexts
#[async_trait]
pub trait AsyncExecutionContext: Send + Sync {
    type Handle: Send + Sync + Clone;
    type Result: Send + Sync;
    type Error: Send + Sync + std::error::Error;
    
    // Core async operations
    async fn execute_async(&self, task: AsyncTask) -> Result<Self::Handle, Self::Error>;
    async fn await_completion(&self, handle: Self::Handle) -> Result<Self::Result, Self::Error>;
    async fn cancel_execution(&self, handle: Self::Handle) -> Result<(), Self::Error>;
    
    // Status and monitoring
    fn is_completed(&self, handle: &Self::Handle) -> bool;
    fn get_progress(&self, handle: &Self::Handle) -> Option<AsyncProgress>;
    
    // Scheduling and coordination
    async fn yield_execution(&self) -> Result<(), Self::Error>;
    async fn sleep(&self, duration: Duration) -> Result<(), Self::Error>;
    async fn coordinate_with(&self, other: &Self, strategy: CoordinationStrategy) -> Result<(), Self::Error>;
}

// Async task representation
#[derive(Debug, Clone)]
pub struct AsyncTask {
    pub id: TaskId,
    pub kind: TaskKind,
    pub payload: TaskPayload,
    pub priority: TaskPriority,
    pub timeout: Option<Duration>,
    pub dependencies: Vec<TaskId>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum TaskKind {
    AgentExecution,
    ToolInvocation,
    WorkflowStep,
    HookExecution,
    EventEmission,
    ScriptEvaluation,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum TaskPayload {
    AgentInput(AgentInput),
    ToolParameters(serde_json::Value),
    ScriptCode { code: String, language: ScriptLanguage },
    WorkflowData(WorkflowStepData),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct AsyncProgress {
    pub completed_steps: u32,
    pub total_steps: u32,
    pub current_operation: String,
    pub estimated_completion: Option<Instant>,
    pub resource_usage: ResourceUsage,
}

// Coordination strategies for cross-engine async operations
#[derive(Debug, Clone)]
pub enum CoordinationStrategy {
    Sequential,           // Execute one after another
    Parallel,            // Execute simultaneously
    PipelineParallel,    // Results of first feed into second
    RaceToCompletion,    // First to complete wins
    AllOrNothing,        // All must succeed or all fail
    BestEffort,          // Collect successful results, ignore failures
}
```

### Rust Async Foundation

```rust
// Native Rust async execution context
pub struct RustAsyncContext {
    executor: Arc<AsyncExecutor>,
    task_tracker: Arc<TaskTracker>,
    resource_manager: Arc<ResourceManager>,
    cancellation_registry: Arc<CancellationRegistry>,
}

#[async_trait]
impl AsyncExecutionContext for RustAsyncContext {
    type Handle = RustAsyncHandle;
    type Result = serde_json::Value;
    type Error = AsyncExecutionError;
    
    async fn execute_async(&self, task: AsyncTask) -> Result<Self::Handle, Self::Error> {
        let handle = RustAsyncHandle::new(task.id);
        
        // Register for cancellation
        self.cancellation_registry.register(&handle, task.timeout).await?;
        
        // Start task execution
        let executor = Arc::clone(&self.executor);
        let task_tracker = Arc::clone(&self.task_tracker);
        let handle_clone = handle.clone();
        
        let join_handle = tokio::spawn(async move {
            task_tracker.track_start(&handle_clone).await;
            
            let result = match task.kind {
                TaskKind::AgentExecution => {
                    executor.execute_agent_async(task.payload).await
                },
                TaskKind::ToolInvocation => {
                    executor.execute_tool_async(task.payload).await
                },
                TaskKind::WorkflowStep => {
                    executor.execute_workflow_step_async(task.payload).await
                },
                TaskKind::HookExecution => {
                    executor.execute_hook_async(task.payload).await
                },
                TaskKind::EventEmission => {
                    executor.emit_event_async(task.payload).await
                },
                TaskKind::ScriptEvaluation => {
                    executor.evaluate_script_async(task.payload).await
                },
                TaskKind::Custom(ref custom_type) => {
                    executor.execute_custom_async(custom_type, task.payload).await
                }
            };
            
            task_tracker.track_completion(&handle_clone, &result).await;
            result
        });
        
        // Store join handle for later awaiting
        self.task_tracker.store_join_handle(&handle, join_handle).await?;
        
        Ok(handle)
    }
    
    async fn await_completion(&self, handle: Self::Handle) -> Result<Self::Result, Self::Error> {
        let join_handle = self.task_tracker.get_join_handle(&handle).await
            .ok_or_else(|| AsyncExecutionError::HandleNotFound(handle.clone()))?;
        
        match join_handle.await {
            Ok(result) => {
                self.task_tracker.cleanup_handle(&handle).await;
                result
            },
            Err(join_error) => {
                self.task_tracker.cleanup_handle(&handle).await;
                Err(AsyncExecutionError::TaskPanicked(join_error.to_string()))
            }
        }
    }
    
    async fn cancel_execution(&self, handle: Self::Handle) -> Result<(), Self::Error> {
        // Cancel the task
        if let Some(join_handle) = self.task_tracker.get_join_handle(&handle).await {
            join_handle.abort();
        }
        
        // Cleanup resources
        self.cancellation_registry.unregister(&handle).await;
        self.task_tracker.cleanup_handle(&handle).await;
        
        Ok(())
    }
    
    async fn yield_execution(&self) -> Result<(), Self::Error> {
        tokio::task::yield_now().await;
        Ok(())
    }
    
    async fn sleep(&self, duration: Duration) -> Result<(), Self::Error> {
        tokio::time::sleep(duration).await;
        Ok(())
    }
}

// Rust-specific async handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RustAsyncHandle {
    pub task_id: TaskId,
    pub created_at: Instant,
    pub thread_id: String,
}

impl RustAsyncHandle {
    fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            created_at: Instant::now(),
            thread_id: format!("{:?}", std::thread::current().id()),
        }
    }
}
```

### Lua Coroutine Integration

```rust
// Lua async execution context using coroutines
pub struct LuaAsyncContext {
    lua_runtime: Arc<mlua::Lua>,
    coroutine_scheduler: Arc<CoroutineScheduler>,
    yield_manager: Arc<YieldManager>,
    cooperative_executor: Arc<CooperativeExecutor>,
}

#[async_trait]
impl AsyncExecutionContext for LuaAsyncContext {
    type Handle = LuaCoroutineHandle;
    type Result = mlua::Value;
    type Error = LuaAsyncError;
    
    async fn execute_async(&self, task: AsyncTask) -> Result<Self::Handle, Self::Error> {
        let handle = LuaCoroutineHandle::new(task.id);
        
        // Convert task to Lua script
        let lua_script = self.convert_task_to_lua(&task)?;
        
        // Create coroutine
        let coroutine = self.lua_runtime.create_thread(
            self.lua_runtime.load(&lua_script).into_function()?
        )?;
        
        // Register with scheduler
        self.coroutine_scheduler.register_coroutine(
            handle.clone(),
            coroutine,
            task.priority
        ).await?;
        
        Ok(handle)
    }
    
    async fn await_completion(&self, handle: Self::Handle) -> Result<Self::Result, Self::Error> {
        loop {
            match self.coroutine_scheduler.poll_coroutine(&handle).await? {
                CoroutineStatus::Completed(result) => {
                    self.coroutine_scheduler.cleanup_coroutine(&handle).await;
                    return Ok(result);
                },
                CoroutineStatus::Yielded(yield_reason) => {
                    // Handle different yield reasons
                    match yield_reason {
                        YieldReason::CooperativeYield => {
                            // Allow other coroutines to run
                            self.yield_execution().await?;
                        },
                        YieldReason::AwaitingIO => {
                            // Wait for I/O completion
                            self.wait_for_io_completion(&handle).await?;
                        },
                        YieldReason::SleepUntil(wake_time) => {
                            // Sleep until specified time
                            let now = Instant::now();
                            if wake_time > now {
                                tokio::time::sleep(wake_time - now).await;
                            }
                        },
                        YieldReason::WaitingForEvent(event_name) => {
                            // Wait for specific event
                            self.wait_for_event(&handle, &event_name).await?;
                        },
                    }
                    
                    // Resume coroutine
                    self.coroutine_scheduler.resume_coroutine(&handle).await?;
                },
                CoroutineStatus::Error(error) => {
                    self.coroutine_scheduler.cleanup_coroutine(&handle).await;
                    return Err(LuaAsyncError::CoroutineError(error));
                },
                CoroutineStatus::Suspended => {
                    // Wait for external resumption
                    self.yield_execution().await?;
                },
            }
        }
    }
    
    async fn yield_execution(&self) -> Result<(), Self::Error> {
        // In Lua context, yielding means allowing other coroutines to run
        self.cooperative_executor.yield_to_scheduler().await?;
        Ok(())
    }
}

// Lua coroutine handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaCoroutineHandle {
    pub task_id: TaskId,
    pub coroutine_id: CoroutineId,
    pub created_at: Instant,
}

// Coroutine scheduler for cooperative multitasking
pub struct CoroutineScheduler {
    active_coroutines: Arc<Mutex<HashMap<LuaCoroutineHandle, CoroutineInfo>>>,
    ready_queue: Arc<Mutex<VecDeque<LuaCoroutineHandle>>>,
    blocked_coroutines: Arc<Mutex<HashMap<LuaCoroutineHandle, BlockedReason>>>,
    scheduler_state: Arc<Mutex<SchedulerState>>,
}

#[derive(Debug, Clone)]
pub struct CoroutineInfo {
    pub coroutine: mlua::Thread,
    pub priority: TaskPriority,
    pub last_execution: Instant,
    pub total_execution_time: Duration,
    pub yield_count: u32,
    pub status: CoroutineStatus,
}

#[derive(Debug, Clone)]
pub enum CoroutineStatus {
    Ready,
    Running,
    Yielded(YieldReason),
    Blocked(BlockedReason),
    Completed(mlua::Value),
    Error(String),
    Suspended,
}

#[derive(Debug, Clone)]
pub enum YieldReason {
    CooperativeYield,
    AwaitingIO,
    SleepUntil(Instant),
    WaitingForEvent(String),
    ResourceWait(String),
}

#[derive(Debug, Clone)]
pub enum BlockedReason {
    WaitingForResource(String),
    WaitingForEvent(String),
    WaitingForCallback,
    Deadlock,
}

impl CoroutineScheduler {
    pub async fn run_scheduler_loop(&self) -> Result<(), LuaAsyncError> {
        loop {
            // Check if scheduler should continue
            {
                let state = self.scheduler_state.lock().await;
                if matches!(*state, SchedulerState::Stopped) {
                    break;
                }
            }
            
            // Get next ready coroutine
            let next_coroutine = {
                let mut ready_queue = self.ready_queue.lock().await;
                ready_queue.pop_front()
            };
            
            if let Some(handle) = next_coroutine {
                // Execute coroutine
                match self.execute_coroutine_step(&handle).await {
                    Ok(CoroutineStepResult::Completed(result)) => {
                        self.mark_completed(&handle, result).await;
                    },
                    Ok(CoroutineStepResult::Yielded(reason)) => {
                        self.handle_yield(&handle, reason).await;
                    },
                    Ok(CoroutineStepResult::Blocked(reason)) => {
                        self.mark_blocked(&handle, reason).await;
                    },
                    Err(error) => {
                        self.mark_error(&handle, error).await;
                    }
                }
            } else {
                // No ready coroutines, check for unblocked ones
                self.check_blocked_coroutines().await;
                
                // Brief sleep to prevent busy waiting
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        
        Ok(())
    }
    
    async fn execute_coroutine_step(&self, handle: &LuaCoroutineHandle) -> Result<CoroutineStepResult, LuaAsyncError> {
        let coroutine_info = {
            let active = self.active_coroutines.lock().await;
            active.get(handle).cloned()
                .ok_or_else(|| LuaAsyncError::CoroutineNotFound(handle.clone()))?
        };
        
        let start_time = Instant::now();
        
        // Resume coroutine execution
        let resume_result = coroutine_info.coroutine.resume::<_, mlua::Value>(())?;
        
        let execution_time = start_time.elapsed();
        
        // Update execution statistics
        {
            let mut active = self.active_coroutines.lock().await;
            if let Some(info) = active.get_mut(handle) {
                info.last_execution = Instant::now();
                info.total_execution_time += execution_time;
                info.yield_count += 1;
            }
        }
        
        // Determine result based on coroutine status
        match coroutine_info.coroutine.status() {
            mlua::ThreadStatus::Resumable => {
                // Coroutine yielded - determine why
                let yield_reason = self.determine_yield_reason(&resume_result)?;
                Ok(CoroutineStepResult::Yielded(yield_reason))
            },
            mlua::ThreadStatus::Unresumable => {
                // Coroutine completed
                Ok(CoroutineStepResult::Completed(resume_result))
            },
            mlua::ThreadStatus::Error => {
                // Coroutine errored
                Err(LuaAsyncError::CoroutineError(format!("Coroutine error: {:?}", resume_result)))
            }
        }
    }
}

// Lua-JavaScript async bridge
pub struct LuaJavaScriptAsyncBridge {
    lua_context: Arc<LuaAsyncContext>,
    js_context: Arc<JavaScriptAsyncContext>,
    cross_engine_coordinator: Arc<CrossEngineAsyncCoordinator>,
}

impl LuaJavaScriptAsyncBridge {
    pub async fn execute_cross_engine_workflow(
        &self,
        workflow: CrossEngineAsyncWorkflow
    ) -> Result<CrossEngineResult, AsyncBridgeError> {
        let coordination_id = CoordinationId::new();
        
        // Start tasks in both engines
        let mut handles = Vec::new();
        
        for step in workflow.steps {
            match step.engine {
                ScriptEngine::Lua => {
                    let task = AsyncTask {
                        id: step.id.clone(),
                        kind: TaskKind::ScriptEvaluation,
                        payload: TaskPayload::ScriptCode {
                            code: step.code.clone(),
                            language: ScriptLanguage::Lua,
                        },
                        priority: step.priority,
                        timeout: step.timeout,
                        dependencies: step.dependencies.clone(),
                        metadata: step.metadata.clone(),
                    };
                    
                    let handle = self.lua_context.execute_async(task).await?;
                    handles.push(CrossEngineHandle::Lua(handle));
                },
                ScriptEngine::JavaScript => {
                    let task = AsyncTask {
                        id: step.id.clone(),
                        kind: TaskKind::ScriptEvaluation,
                        payload: TaskPayload::ScriptCode {
                            code: step.code.clone(),
                            language: ScriptLanguage::JavaScript,
                        },
                        priority: step.priority,
                        timeout: step.timeout,
                        dependencies: step.dependencies.clone(),
                        metadata: step.metadata.clone(),
                    };
                    
                    let handle = self.js_context.execute_async(task).await?;
                    handles.push(CrossEngineHandle::JavaScript(handle));
                }
            }
        }
        
        // Coordinate execution based on workflow strategy
        let coordination_result = self.cross_engine_coordinator.coordinate_execution(
            coordination_id,
            handles,
            workflow.coordination_strategy
        ).await?;
        
        Ok(coordination_result)
    }
}
```

### JavaScript Promise Integration

```rust
// JavaScript async execution context using Promises
pub struct JavaScriptAsyncContext {
    js_runtime: Arc<JSRuntime>,
    promise_scheduler: Arc<PromiseScheduler>,
    event_loop: Arc<EventLoop>,
    async_coordinator: Arc<JSAsyncCoordinator>,
}

#[async_trait]
impl AsyncExecutionContext for JavaScriptAsyncContext {
    type Handle = JavaScriptPromiseHandle;
    type Result = serde_json::Value;
    type Error = JavaScriptAsyncError;
    
    async fn execute_async(&self, task: AsyncTask) -> Result<Self::Handle, Self::Error> {
        let handle = JavaScriptPromiseHandle::new(task.id);
        
        // Convert task to JavaScript
        let js_code = self.convert_task_to_javascript(&task)?;
        
        // Create Promise-based execution
        let promise = self.js_runtime.evaluate_promise(&js_code).await?;
        
        // Register with promise scheduler
        self.promise_scheduler.register_promise(
            handle.clone(),
            promise,
            task.priority
        ).await?;
        
        Ok(handle)
    }
    
    async fn await_completion(&self, handle: Self::Handle) -> Result<Self::Result, Self::Error> {
        loop {
            match self.promise_scheduler.poll_promise(&handle).await? {
                PromiseStatus::Resolved(result) => {
                    self.promise_scheduler.cleanup_promise(&handle).await;
                    return Ok(result);
                },
                PromiseStatus::Rejected(error) => {
                    self.promise_scheduler.cleanup_promise(&handle).await;
                    return Err(JavaScriptAsyncError::PromiseRejected(error));
                },
                PromiseStatus::Pending => {
                    // Allow event loop to process
                    self.event_loop.tick().await?;
                    
                    // Brief yield to allow other tasks
                    self.yield_execution().await?;
                },
            }
        }
    }
    
    async fn yield_execution(&self) -> Result<(), Self::Error> {
        // In JavaScript context, yielding means allowing event loop to process
        self.event_loop.yield_to_event_loop().await?;
        Ok(())
    }
}

// JavaScript Promise handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JavaScriptPromiseHandle {
    pub task_id: TaskId,
    pub promise_id: PromiseId,
    pub created_at: Instant,
}

// Promise scheduler for JavaScript async coordination
pub struct PromiseScheduler {
    active_promises: Arc<Mutex<HashMap<JavaScriptPromiseHandle, PromiseInfo>>>,
    resolution_queue: Arc<Mutex<VecDeque<PromiseResolution>>>,
    microtask_queue: Arc<Mutex<VecDeque<Microtask>>>,
}

#[derive(Debug, Clone)]
pub struct PromiseInfo {
    pub promise: JSPromise,
    pub priority: TaskPriority,
    pub created_at: Instant,
    pub status: PromiseStatus,
    pub resolvers: Vec<PromiseResolver>,
    pub rejection_handlers: Vec<RejectionHandler>,
}

#[derive(Debug, Clone)]
pub enum PromiseStatus {
    Pending,
    Resolved(serde_json::Value),
    Rejected(String),
}

impl PromiseScheduler {
    pub async fn process_promise_queue(&self) -> Result<(), JavaScriptAsyncError> {
        // Process microtasks first (higher priority)
        self.process_microtasks().await?;
        
        // Then process promise resolutions
        self.process_promise_resolutions().await?;
        
        Ok(())
    }
    
    async fn process_microtasks(&self) -> Result<(), JavaScriptAsyncError> {
        loop {
            let microtask = {
                let mut queue = self.microtask_queue.lock().await;
                queue.pop_front()
            };
            
            match microtask {
                Some(task) => {
                    self.execute_microtask(task).await?;
                },
                None => break, // No more microtasks
            }
        }
        
        Ok(())
    }
    
    async fn process_promise_resolutions(&self) -> Result<(), JavaScriptAsyncError> {
        let resolutions = {
            let mut queue = self.resolution_queue.lock().await;
            let current_resolutions: Vec<_> = queue.drain(..).collect();
            current_resolutions
        };
        
        for resolution in resolutions {
            self.resolve_promise(resolution).await?;
        }
        
        Ok(())
    }
}
```

### Cross-Engine Async Coordination

```rust
// Cross-engine async coordinator
pub struct CrossEngineAsyncCoordinator {
    rust_context: Arc<RustAsyncContext>,
    lua_context: Arc<LuaAsyncContext>, 
    js_context: Arc<JavaScriptAsyncContext>,
    coordination_strategies: HashMap<CoordinationStrategy, Box<dyn CoordinationExecutor>>,
    resource_manager: Arc<AsyncResourceManager>,
}

impl CrossEngineAsyncCoordinator {
    pub async fn coordinate_execution(
        &self,
        coordination_id: CoordinationId,
        handles: Vec<CrossEngineHandle>,
        strategy: CoordinationStrategy
    ) -> Result<CrossEngineResult, CoordinationError> {
        let start_time = Instant::now();
        
        // Get appropriate coordination executor
        let executor = self.coordination_strategies.get(&strategy)
            .ok_or_else(|| CoordinationError::UnsupportedStrategy(strategy.clone()))?;
        
        // Execute coordination strategy
        let coordination_result = executor.execute_coordination(
            coordination_id,
            handles,
            self
        ).await?;
        
        let total_duration = start_time.elapsed();
        
        Ok(CrossEngineResult {
            coordination_id,
            strategy,
            results: coordination_result.results,
            total_duration,
            successful_engines: coordination_result.successful_engines,
            failed_engines: coordination_result.failed_engines,
            resource_usage: self.resource_manager.get_usage_summary(coordination_id).await?,
        })
    }
}

// Sequential coordination executor
pub struct SequentialCoordinationExecutor;

#[async_trait]
impl CoordinationExecutor for SequentialCoordinationExecutor {
    async fn execute_coordination(
        &self,
        coordination_id: CoordinationId,
        handles: Vec<CrossEngineHandle>,
        coordinator: &CrossEngineAsyncCoordinator
    ) -> Result<CoordinationExecutionResult, CoordinationError> {
        let mut results = Vec::new();
        let mut successful_engines = Vec::new();
        let mut failed_engines = Vec::new();
        
        for handle in handles {
            match handle {
                CrossEngineHandle::Rust(rust_handle) => {
                    match coordinator.rust_context.await_completion(rust_handle).await {
                        Ok(result) => {
                            results.push(EngineResult::Rust(result));
                            successful_engines.push(ScriptEngine::Rust);
                        },
                        Err(error) => {
                            failed_engines.push(EngineFailure {
                                engine: ScriptEngine::Rust,
                                error: error.to_string(),
                            });
                        }
                    }
                },
                CrossEngineHandle::Lua(lua_handle) => {
                    match coordinator.lua_context.await_completion(lua_handle).await {
                        Ok(result) => {
                            // Convert Lua value to JSON
                            let json_result = lua_value_to_json(result)?;
                            results.push(EngineResult::Lua(json_result));
                            successful_engines.push(ScriptEngine::Lua);
                        },
                        Err(error) => {
                            failed_engines.push(EngineFailure {
                                engine: ScriptEngine::Lua,
                                error: error.to_string(),
                            });
                        }
                    }
                },
                CrossEngineHandle::JavaScript(js_handle) => {
                    match coordinator.js_context.await_completion(js_handle).await {
                        Ok(result) => {
                            results.push(EngineResult::JavaScript(result));
                            successful_engines.push(ScriptEngine::JavaScript);
                        },
                        Err(error) => {
                            failed_engines.push(EngineFailure {
                                engine: ScriptEngine::JavaScript,
                                error: error.to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(CoordinationExecutionResult {
            results,
            successful_engines,
            failed_engines,
        })
    }
}

// Parallel coordination executor
pub struct ParallelCoordinationExecutor;

#[async_trait]
impl CoordinationExecutor for ParallelCoordinationExecutor {
    async fn execute_coordination(
        &self,
        coordination_id: CoordinationId,
        handles: Vec<CrossEngineHandle>,
        coordinator: &CrossEngineAsyncCoordinator
    ) -> Result<CoordinationExecutionResult, CoordinationError> {
        let mut futures = Vec::new();
        
        // Create futures for all handles
        for handle in handles {
            match handle {
                CrossEngineHandle::Rust(rust_handle) => {
                    let context = Arc::clone(&coordinator.rust_context);
                    let future = async move {
                        match context.await_completion(rust_handle).await {
                            Ok(result) => Ok(EngineResult::Rust(result)),
                            Err(error) => Err(EngineFailure {
                                engine: ScriptEngine::Rust,
                                error: error.to_string(),
                            })
                        }
                    };
                    futures.push(Box::pin(future));
                },
                CrossEngineHandle::Lua(lua_handle) => {
                    let context = Arc::clone(&coordinator.lua_context);
                    let future = async move {
                        match context.await_completion(lua_handle).await {
                            Ok(result) => {
                                let json_result = lua_value_to_json(result)?;
                                Ok(EngineResult::Lua(json_result))
                            },
                            Err(error) => Err(EngineFailure {
                                engine: ScriptEngine::Lua,
                                error: error.to_string(),
                            })
                        }
                    };
                    futures.push(Box::pin(future));
                },
                CrossEngineHandle::JavaScript(js_handle) => {
                    let context = Arc::clone(&coordinator.js_context);
                    let future = async move {
                        match context.await_completion(js_handle).await {
                            Ok(result) => Ok(EngineResult::JavaScript(result)),
                            Err(error) => Err(EngineFailure {
                                engine: ScriptEngine::JavaScript,
                                error: error.to_string(),
                            })
                        }
                    };
                    futures.push(Box::pin(future));
                }
            }
        }
        
        // Wait for all futures to complete
        let future_results = futures::future::join_all(futures).await;
        
        let mut results = Vec::new();
        let mut successful_engines = Vec::new();
        let mut failed_engines = Vec::new();
        
        for future_result in future_results {
            match future_result {
                Ok(engine_result) => {
                    match &engine_result {
                        EngineResult::Rust(_) => successful_engines.push(ScriptEngine::Rust),
                        EngineResult::Lua(_) => successful_engines.push(ScriptEngine::Lua),
                        EngineResult::JavaScript(_) => successful_engines.push(ScriptEngine::JavaScript),
                    }
                    results.push(engine_result);
                },
                Err(engine_failure) => {
                    failed_engines.push(engine_failure);
                }
            }
        }
        
        Ok(CoordinationExecutionResult {
            results,
            successful_engines,
            failed_engines,
        })
    }
}
```

### Agent Async Execution Integration

```rust
// Async-enabled agent executor
pub struct AsyncAgentExecutor {
    agents: HashMap<String, Box<dyn Agent>>,
    async_coordinator: CrossEngineAsyncCoordinator,
    execution_planner: AsyncExecutionPlanner,
    performance_monitor: AsyncPerformanceMonitor,
}

impl AsyncAgentExecutor {
    pub async fn execute_agent_async(
        &mut self,
        agent_id: &str,
        input: AgentInput,
        execution_config: AsyncExecutionConfig
    ) -> Result<AsyncAgentExecutionResult, AsyncExecutionError> {
        let execution_id = ExecutionId::new();
        let start_time = Instant::now();
        
        // Get agent
        let agent = self.agents.get_mut(agent_id)
            .ok_or_else(|| AsyncExecutionError::AgentNotFound(agent_id.to_string()))?;
        
        // Plan async execution
        let execution_plan = self.execution_planner.plan_execution(
            agent,
            &input,
            &execution_config
        ).await?;
        
        // Execute based on plan
        let execution_result = match execution_plan.strategy {
            AsyncExecutionStrategy::SingleThreaded => {
                self.execute_single_threaded(agent, input, execution_plan).await?
            },
            AsyncExecutionStrategy::MultiThreaded => {
                self.execute_multi_threaded(agent, input, execution_plan).await?
            },
            AsyncExecutionStrategy::Distributed => {
                self.execute_distributed(agent, input, execution_plan).await?
            },
            AsyncExecutionStrategy::Streaming => {
                self.execute_streaming(agent, input, execution_plan).await?
            },
        };
        
        let total_duration = start_time.elapsed();
        
        // Record performance metrics
        self.performance_monitor.record_execution(
            execution_id,
            agent_id,
            &execution_result,
            total_duration
        ).await;
        
        Ok(AsyncAgentExecutionResult {
            execution_id,
            agent_id: agent_id.to_string(),
            input,
            output: execution_result.output,
            execution_plan,
            total_duration,
            async_operations: execution_result.async_operations,
            resource_usage: execution_result.resource_usage,
        })
    }
    
    async fn execute_streaming(
        &mut self,
        agent: &mut Box<dyn Agent>,
        input: AgentInput,
        execution_plan: AsyncExecutionPlan
    ) -> Result<ExecutionResult, AsyncExecutionError> {
        // Create streaming context
        let streaming_context = StreamingContext::new(execution_plan.buffer_size);
        
        // Start agent execution in streaming mode
        let mut agent_stream = agent.execute_streaming(input, streaming_context).await?;
        
        let mut accumulated_output = String::new();
        let mut async_operations = Vec::new();
        let mut resource_usage = ResourceUsage::new();
        
        // Process stream chunks
        while let Some(chunk_result) = agent_stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    match chunk.chunk_type {
                        StreamChunkType::Content(content) => {
                            accumulated_output.push_str(&content);
                        },
                        StreamChunkType::ToolCall(tool_call) => {
                            // Execute tool call asynchronously
                            let tool_task = AsyncTask {
                                id: TaskId::new(),
                                kind: TaskKind::ToolInvocation,
                                payload: TaskPayload::ToolParameters(tool_call.parameters),
                                priority: TaskPriority::Normal,
                                timeout: Some(Duration::from_secs(30)),
                                dependencies: vec![],
                                metadata: HashMap::new(),
                            };
                            
                            let tool_handle = self.async_coordinator
                                .rust_context
                                .execute_async(tool_task)
                                .await?;
                            
                            async_operations.push(AsyncOperation {
                                operation_type: AsyncOperationType::ToolCall,
                                handle: CrossEngineHandle::Rust(tool_handle),
                                started_at: Instant::now(),
                            });
                        },
                        StreamChunkType::Metadata(metadata) => {
                            // Update resource usage
                            if let Some(memory_usage) = metadata.get("memory_usage") {
                                resource_usage.memory_bytes = memory_usage.as_u64().unwrap_or(0);
                            }
                        },
                    }
                },
                Err(stream_error) => {
                    return Err(AsyncExecutionError::StreamingError(stream_error.to_string()));
                }
            }
        }
        
        Ok(ExecutionResult {
            output: AgentOutput {
                content: accumulated_output,
                metadata: HashMap::new(),
            },
            async_operations,
            resource_usage,
        })
    }
    
    pub async fn coordinate_multi_agent_async(
        &mut self,
        coordination_request: MultiAgentCoordinationRequest
    ) -> Result<MultiAgentCoordinationResult, AsyncExecutionError> {
        let coordination_id = CoordinationId::new();
        let start_time = Instant::now();
        
        // Start all agent executions
        let mut agent_handles = Vec::new();
        
        for agent_request in coordination_request.agent_requests {
            let task = AsyncTask {
                id: TaskId::new(),
                kind: TaskKind::AgentExecution,
                payload: TaskPayload::AgentInput(agent_request.input),
                priority: agent_request.priority,
                timeout: agent_request.timeout,
                dependencies: agent_request.dependencies,
                metadata: agent_request.metadata,
            };
            
            let handle = self.async_coordinator
                .rust_context
                .execute_async(task)
                .await?;
            
            agent_handles.push(AgentHandle {
                agent_id: agent_request.agent_id,
                handle: CrossEngineHandle::Rust(handle),
            });
        }
        
        // Coordinate execution
        let coordination_result = self.async_coordinator.coordinate_execution(
            coordination_id,
            agent_handles.into_iter().map(|ah| ah.handle).collect(),
            coordination_request.coordination_strategy
        ).await?;
        
        let total_duration = start_time.elapsed();
        
        Ok(MultiAgentCoordinationResult {
            coordination_id,
            coordination_result,
            total_duration,
            agents_involved: coordination_request.agent_requests.len(),
        })
    }
}

// Async execution configuration
#[derive(Debug, Clone)]
pub struct AsyncExecutionConfig {
    pub strategy: AsyncExecutionStrategy,
    pub timeout: Option<Duration>,
    pub max_concurrent_operations: Option<u32>,
    pub buffer_size: Option<usize>,
    pub resource_limits: Option<ResourceLimits>,
    pub coordination_strategy: Option<CoordinationStrategy>,
}

#[derive(Debug, Clone)]
pub enum AsyncExecutionStrategy {
    SingleThreaded,  // Traditional single-threaded execution
    MultiThreaded,   // Parallel execution with thread pool
    Distributed,     // Execution across multiple nodes
    Streaming,       // Streaming execution with backpressure
}

// Resource management for async operations
pub struct AsyncResourceManager {
    memory_tracker: Arc<MemoryTracker>,
    cpu_tracker: Arc<CpuTracker>,
    network_tracker: Arc<NetworkTracker>,
    resource_limits: Arc<ResourceLimits>,
    enforcement_strategy: ResourceEnforcementStrategy,
}

impl AsyncResourceManager {
    pub async fn allocate_resources(
        &self,
        task_id: TaskId,
        requirements: ResourceRequirements
    ) -> Result<ResourceAllocation, ResourceError> {
        // Check if resources are available
        if !self.check_resource_availability(&requirements).await? {
            return Err(ResourceError::InsufficientResources(requirements));
        }
        
        // Allocate memory
        let memory_allocation = self.memory_tracker.allocate(
            task_id,
            requirements.memory_bytes
        ).await?;
        
        // Allocate CPU quota
        let cpu_allocation = self.cpu_tracker.allocate(
            task_id,
            requirements.cpu_percentage
        ).await?;
        
        // Allocate network bandwidth
        let network_allocation = self.network_tracker.allocate(
            task_id,
            requirements.network_bandwidth_bps
        ).await?;
        
        Ok(ResourceAllocation {
            task_id,
            memory_allocation,
            cpu_allocation,
            network_allocation,
            allocated_at: Instant::now(),
        })
    }
    
    pub async fn monitor_resource_usage(&self, task_id: TaskId) -> Result<ResourceUsage, ResourceError> {
        let memory_usage = self.memory_tracker.get_usage(task_id).await?;
        let cpu_usage = self.cpu_tracker.get_usage(task_id).await?;
        let network_usage = self.network_tracker.get_usage(task_id).await?;
        
        Ok(ResourceUsage {
            memory_bytes: memory_usage.current_bytes,
            cpu_percentage: cpu_usage.current_percentage,
            network_bytes_sent: network_usage.bytes_sent,
            network_bytes_received: network_usage.bytes_received,
            duration: memory_usage.duration,
        })
    }
    
    pub async fn enforce_resource_limits(&self, task_id: TaskId) -> Result<(), ResourceError> {
        let current_usage = self.monitor_resource_usage(task_id).await?;
        
        // Check memory limit
        if current_usage.memory_bytes > self.resource_limits.max_memory_bytes {
            match self.enforcement_strategy {
                ResourceEnforcementStrategy::Kill => {
                    return Err(ResourceError::MemoryLimitExceeded {
                        current: current_usage.memory_bytes,
                        limit: self.resource_limits.max_memory_bytes,
                    });
                },
                ResourceEnforcementStrategy::Throttle => {
                    self.throttle_task(task_id, ThrottleReason::MemoryLimit).await?;
                },
                ResourceEnforcementStrategy::Warn => {
                    log::warn!("Task {} exceeding memory limit: {} > {}", 
                        task_id, current_usage.memory_bytes, self.resource_limits.max_memory_bytes);
                }
            }
        }
        
        // Check CPU limit  
        if current_usage.cpu_percentage > self.resource_limits.max_cpu_percentage {
            match self.enforcement_strategy {
                ResourceEnforcementStrategy::Kill => {
                    return Err(ResourceError::CpuLimitExceeded {
                        current: current_usage.cpu_percentage,
                        limit: self.resource_limits.max_cpu_percentage,
                    });
                },
                ResourceEnforcementStrategy::Throttle => {
                    self.throttle_task(task_id, ThrottleReason::CpuLimit).await?;
                },
                ResourceEnforcementStrategy::Warn => {
                    log::warn!("Task {} exceeding CPU limit: {}% > {}%",
                        task_id, current_usage.cpu_percentage, self.resource_limits.max_cpu_percentage);
                }
            }
        }
        
        Ok(())
    }
}
```

This complete async patterns integration provides production-ready async execution across all engines with unified interfaces, resource management, and sophisticated coordination strategies.