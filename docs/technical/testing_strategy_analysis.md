# Testing Strategy Analysis

## Overview

This document analyzes comprehensive testing strategies for rs-llmspell's advanced features including hook systems, event systems, tool-wrapped agents, built-in components, async script execution, and cross-engine compatibility. The testing approach ensures reliability, performance, and correctness across all architectural components.

## Hook System Testing Patterns

### 1. Hook Registration and Execution Testing

**Challenge**: Hooks are dynamically registered and executed at various points, making testing complex.

**Strategy**: Comprehensive test coverage for hook lifecycle, ordering, and isolation.

```rust
// Hook system test utilities
#[cfg(test)]
mod hook_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::collections::VecDeque;

    // Test hook that records execution
    #[derive(Clone)]
    struct TestHook {
        name: String,
        execution_log: Arc<Mutex<VecDeque<HookExecution>>>,
        should_fail: bool,
        execution_delay: Duration,
    }

    #[derive(Debug, Clone)]
    struct HookExecution {
        hook_name: String,
        hook_point: HookPoint,
        timestamp: Instant,
        context_snapshot: String,
        thread_id: std::thread::ThreadId,
    }

    impl Hook for TestHook {
        async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
            let start_time = Instant::now();
            
            // Simulate processing time
            if self.execution_delay > Duration::ZERO {
                tokio::time::sleep(self.execution_delay).await;
            }
            
            // Record execution
            {
                let mut log = self.execution_log.lock().unwrap();
                log.push_back(HookExecution {
                    hook_name: self.name.clone(),
                    hook_point: context.hook_point,
                    timestamp: start_time,
                    context_snapshot: format!("{:?}", context),
                    thread_id: std::thread::current().id(),
                });
            }
            
            // Simulate failure if configured
            if self.should_fail {
                return Err(anyhow!("Test hook {} configured to fail", self.name));
            }
            
            Ok(HookResult {
                success: true,
                modifications: HashMap::new(),
                metadata: HashMap::from([
                    ("execution_time".to_string(), Value::Number(start_time.elapsed().as_millis().into())),
                ]),
            })
        }
    }

    // Hook system test harness
    struct HookSystemTestHarness {
        hook_manager: OptimizedHookManager,
        execution_log: Arc<Mutex<VecDeque<HookExecution>>>,
        registered_hooks: Vec<String>,
    }

    impl HookSystemTestHarness {
        fn new() -> Self {
            Self {
                hook_manager: OptimizedHookManager::new(HookExecutionStrategy::Adaptive),
                execution_log: Arc::new(Mutex::new(VecDeque::new())),
                registered_hooks: Vec::new(),
            }
        }
        
        fn register_test_hook(&mut self, name: &str, point: HookPoint, priority: i32, should_fail: bool) {
            let hook = TestHook {
                name: name.to_string(),
                execution_log: Arc::clone(&self.execution_log),
                should_fail,
                execution_delay: Duration::from_millis(10),
            };
            
            self.hook_manager.register(point, RegisteredHook {
                id: name.to_string(),
                handler: Box::new(hook),
                priority,
                execution_mode: ExecutionMode::Synchronous,
                dependencies: Vec::new(),
                performance_profile: HookPerformanceProfile::default(),
                cache_strategy: CacheStrategy::None,
            }).unwrap();
            
            self.registered_hooks.push(name.to_string());
        }
        
        fn get_execution_order(&self) -> Vec<String> {
            let log = self.execution_log.lock().unwrap();
            log.iter().map(|e| e.hook_name.clone()).collect()
        }
        
        fn clear_log(&self) {
            let mut log = self.execution_log.lock().unwrap();
            log.clear();
        }
    }

    #[tokio::test]
    async fn test_hook_execution_order() {
        let mut harness = HookSystemTestHarness::new();
        
        // Register hooks with different priorities
        harness.register_test_hook("high_priority", HookPoint::BeforeLLMCall, 100, false);
        harness.register_test_hook("medium_priority", HookPoint::BeforeLLMCall, 50, false);
        harness.register_test_hook("low_priority", HookPoint::BeforeLLMCall, 10, false);
        
        // Execute hooks
        let mut context = HookContext {
            hook_point: HookPoint::BeforeLLMCall,
            agent_id: "test_agent".to_string(),
            input: json!({"message": "test"}),
            ..Default::default()
        };
        
        let result = harness.hook_manager.execute_hooks(HookPoint::BeforeLLMCall, &mut context).await;
        assert!(result.is_ok());
        
        // Verify execution order (highest priority first)
        let execution_order = harness.get_execution_order();
        assert_eq!(execution_order, vec![
            "high_priority".to_string(),
            "medium_priority".to_string(), 
            "low_priority".to_string()
        ]);
    }

    #[tokio::test]
    async fn test_hook_failure_isolation() {
        let mut harness = HookSystemTestHarness::new();
        
        // Register hooks, one configured to fail
        harness.register_test_hook("before_fail", HookPoint::BeforeLLMCall, 100, false);
        harness.register_test_hook("failing_hook", HookPoint::BeforeLLMCall, 50, true);
        harness.register_test_hook("after_fail", HookPoint::BeforeLLMCall, 10, false);
        
        let mut context = HookContext {
            hook_point: HookPoint::BeforeLLMCall,
            agent_id: "test_agent".to_string(),
            input: json!({"message": "test"}),
            ..Default::default()
        };
        
        let result = harness.hook_manager.execute_hooks(HookPoint::BeforeLLMCall, &mut context).await;
        
        // Should handle failure gracefully
        assert!(result.is_ok());
        
        // Verify other hooks still executed
        let execution_order = harness.get_execution_order();
        assert!(execution_order.contains(&"before_fail".to_string()));
        assert!(execution_order.contains(&"failing_hook".to_string()));
        assert!(execution_order.contains(&"after_fail".to_string()));
        
        // Verify failure was recorded
        let result = result.unwrap();
        assert_eq!(result.failed_hooks.len(), 1);
        assert_eq!(result.failed_hooks[0].hook_id, "failing_hook");
    }

    #[tokio::test]
    async fn test_parallel_hook_execution() {
        let mut harness = HookSystemTestHarness::new();
        
        // Register hooks with parallel execution mode
        for i in 0..5 {
            let hook = TestHook {
                name: format!("parallel_hook_{}", i),
                execution_log: Arc::clone(&harness.execution_log),
                should_fail: false,
                execution_delay: Duration::from_millis(50), // Longer delay to test parallelism
            };
            
            harness.hook_manager.register(HookPoint::BeforeLLMCall, RegisteredHook {
                id: format!("parallel_hook_{}", i),
                handler: Box::new(hook),
                priority: 50, // Same priority
                execution_mode: ExecutionMode::Parallel { group: "test_group".to_string() },
                dependencies: Vec::new(),
                performance_profile: HookPerformanceProfile::default(),
                cache_strategy: CacheStrategy::None,
            }).unwrap();
        }
        
        let start_time = Instant::now();
        
        let mut context = HookContext {
            hook_point: HookPoint::BeforeLLMCall,
            agent_id: "test_agent".to_string(),
            input: json!({"message": "test"}),
            ..Default::default()
        };
        
        let result = harness.hook_manager.execute_hooks(HookPoint::BeforeLLMCall, &mut context).await;
        let total_time = start_time.elapsed();
        
        assert!(result.is_ok());
        
        // Parallel execution should be faster than sequential
        // 5 hooks * 50ms = 250ms sequential, but parallel should be ~50ms
        assert!(total_time < Duration::from_millis(150), "Parallel execution took too long: {:?}", total_time);
        
        // All hooks should have executed
        let execution_order = harness.get_execution_order();
        assert_eq!(execution_order.len(), 5);
    }

    #[tokio::test]
    async fn test_hook_context_isolation() {
        let mut harness = HookSystemTestHarness::new();
        
        // Register hooks that modify context
        let modifying_hook = TestHook {
            name: "modifying_hook".to_string(),
            execution_log: Arc::clone(&harness.execution_log),
            should_fail: false,
            execution_delay: Duration::ZERO,
        };
        
        // Override execute to modify context
        struct ContextModifyingHook {
            log: Arc<Mutex<VecDeque<HookExecution>>>,
        }
        
        #[async_trait]
        impl Hook for ContextModifyingHook {
            async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
                // Modify context
                context.metadata.insert("modified_by".to_string(), Value::String("test_hook".to_string()));
                
                let mut log = self.log.lock().unwrap();
                log.push_back(HookExecution {
                    hook_name: "modifying_hook".to_string(),
                    hook_point: context.hook_point,
                    timestamp: Instant::now(),
                    context_snapshot: format!("{:?}", context.metadata),
                    thread_id: std::thread::current().id(),
                });
                
                Ok(HookResult {
                    success: true,
                    modifications: HashMap::from([
                        ("test_modification".to_string(), Value::String("applied".to_string()))
                    ]),
                    metadata: HashMap::new(),
                })
            }
        }
        
        harness.hook_manager.register(HookPoint::BeforeLLMCall, RegisteredHook {
            id: "modifying_hook".to_string(),
            handler: Box::new(ContextModifyingHook {
                log: Arc::clone(&harness.execution_log),
            }),
            priority: 50,
            execution_mode: ExecutionMode::Synchronous,
            dependencies: Vec::new(),
            performance_profile: HookPerformanceProfile::default(),
            cache_strategy: CacheStrategy::None,
        }).unwrap();
        
        let mut context = HookContext {
            hook_point: HookPoint::BeforeLLMCall,
            agent_id: "test_agent".to_string(),
            input: json!({"message": "test"}),
            metadata: HashMap::new(),
            ..Default::default()
        };
        
        let result = harness.hook_manager.execute_hooks(HookPoint::BeforeLLMCall, &mut context).await;
        
        assert!(result.is_ok());
        
        // Verify context was modified
        assert_eq!(context.metadata.get("modified_by"), Some(&Value::String("test_hook".to_string())));
        
        // Verify modifications were applied
        let result = result.unwrap();
        assert_eq!(result.applied_modifications.get("test_modification"), Some(&Value::String("applied".to_string())));
    }
}
```

### 2. Hook Performance Testing

```rust
#[cfg(test)]
mod hook_performance_tests {
    use super::*;
    use criterion::{Criterion, black_box};
    
    // Performance benchmarks for hook system
    pub fn bench_hook_execution(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        c.bench_function("hook_execution_sequential", |b| {
            b.to_async(&rt).iter(|| async {
                let mut harness = HookSystemTestHarness::new();
                
                // Register 10 lightweight hooks
                for i in 0..10 {
                    harness.register_test_hook(&format!("hook_{}", i), HookPoint::BeforeLLMCall, i * 10, false);
                }
                
                let mut context = HookContext {
                    hook_point: HookPoint::BeforeLLMCall,
                    agent_id: "test_agent".to_string(),
                    input: json!({"message": "test"}),
                    ..Default::default()
                };
                
                black_box(harness.hook_manager.execute_hooks(HookPoint::BeforeLLMCall, &mut context).await)
            });
        });
        
        c.bench_function("hook_execution_parallel", |b| {
            b.to_async(&rt).iter(|| async {
                let mut harness = HookSystemTestHarness::new();
                
                // Register 10 parallel hooks
                for i in 0..10 {
                    let hook = TestHook {
                        name: format!("parallel_hook_{}", i),
                        execution_log: Arc::clone(&harness.execution_log),
                        should_fail: false,
                        execution_delay: Duration::from_millis(1),
                    };
                    
                    harness.hook_manager.register(HookPoint::BeforeLLMCall, RegisteredHook {
                        id: format!("parallel_hook_{}", i),
                        handler: Box::new(hook),
                        priority: 50,
                        execution_mode: ExecutionMode::Parallel { group: "bench_group".to_string() },
                        dependencies: Vec::new(),
                        performance_profile: HookPerformanceProfile::default(),
                        cache_strategy: CacheStrategy::None,
                    }).unwrap();
                }
                
                let mut context = HookContext {
                    hook_point: HookPoint::BeforeLLMCall,
                    agent_id: "test_agent".to_string(),
                    input: json!({"message": "test"}),
                    ..Default::default()
                };
                
                black_box(harness.hook_manager.execute_hooks(HookPoint::BeforeLLMCall, &mut context).await)
            });
        });
    }
    
    // Memory usage benchmarks
    pub fn bench_hook_memory_usage(c: &mut Criterion) {
        c.bench_function("hook_registration_memory", |b| {
            b.iter(|| {
                let mut hook_manager = OptimizedHookManager::new(HookExecutionStrategy::Sequential);
                
                // Register many hooks to test memory usage
                for i in 0..1000 {
                    let hook = TestHook {
                        name: format!("hook_{}", i),
                        execution_log: Arc::new(Mutex::new(VecDeque::new())),
                        should_fail: false,
                        execution_delay: Duration::ZERO,
                    };
                    
                    hook_manager.register(HookPoint::BeforeLLMCall, RegisteredHook {
                        id: format!("hook_{}", i),
                        handler: Box::new(hook),
                        priority: i as i32,
                        execution_mode: ExecutionMode::Synchronous,
                        dependencies: Vec::new(),
                        performance_profile: HookPerformanceProfile::default(),
                        cache_strategy: CacheStrategy::None,
                    }).unwrap();
                }
                
                black_box(hook_manager)
            });
        });
    }
}
```

## Event System Testing

### 1. Event Bus Testing Patterns

```rust
#[cfg(test)]
mod event_system_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    // Test event subscriber that tracks received events
    #[derive(Clone)]
    struct TestEventSubscriber {
        id: String,
        received_events: Arc<Mutex<Vec<Event>>>,
        processing_delay: Duration,
        should_fail: bool,
        failure_rate: f64,
    }
    
    impl TestEventSubscriber {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                received_events: Arc::new(Mutex::new(Vec::new())),
                processing_delay: Duration::from_millis(1),
                should_fail: false,
                failure_rate: 0.0,
            }
        }
        
        fn with_failure_rate(mut self, rate: f64) -> Self {
            self.failure_rate = rate;
            self
        }
        
        fn get_received_events(&self) -> Vec<Event> {
            self.received_events.lock().unwrap().clone()
        }
    }
    
    #[async_trait]
    impl EventSubscriber for TestEventSubscriber {
        fn id(&self) -> &str {
            &self.id
        }
        
        async fn handle_event(&self, event: &Event) -> Result<()> {
            // Simulate processing time
            if self.processing_delay > Duration::ZERO {
                tokio::time::sleep(self.processing_delay).await;
            }
            
            // Simulate random failures
            if self.failure_rate > 0.0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen::<f64>() < self.failure_rate {
                    return Err(anyhow!("Simulated failure in subscriber {}", self.id));
                }
            }
            
            // Record event
            {
                let mut events = self.received_events.lock().unwrap();
                events.push(event.clone());
            }
            
            Ok(())
        }
    }
    
    // Event bus test harness
    struct EventBusTestHarness {
        event_bus: HighPerformanceEventBus,
        subscribers: HashMap<String, TestEventSubscriber>,
        emitted_events: Arc<Mutex<Vec<Event>>>,
    }
    
    impl EventBusTestHarness {
        async fn new() -> Self {
            let event_bus = HighPerformanceEventBus::new(EventBusConfig {
                default_channel_capacity: 1000,
                batch_processing_size: 100,
                batch_flush_interval: Duration::from_millis(10),
                enable_circuit_breaker: true,
            }).await.unwrap();
            
            Self {
                event_bus,
                subscribers: HashMap::new(),
                emitted_events: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        async fn add_subscriber(&mut self, channel: &str, subscriber: TestEventSubscriber) {
            let subscriber_id = subscriber.id().to_string();
            self.event_bus.subscribe(channel, Box::new(subscriber.clone())).await.unwrap();
            self.subscribers.insert(subscriber_id, subscriber);
        }
        
        async fn emit_event(&self, event: Event) -> Result<EmissionResult> {
            // Track emitted events
            {
                let mut events = self.emitted_events.lock().unwrap();
                events.push(event.clone());
            }
            
            self.event_bus.emit_event(event).await
        }
        
        fn get_subscriber_events(&self, subscriber_id: &str) -> Vec<Event> {
            self.subscribers.get(subscriber_id)
                .map(|s| s.get_received_events())
                .unwrap_or_default()
        }
    }
    
    #[tokio::test]
    async fn test_event_delivery() {
        let mut harness = EventBusTestHarness::new().await;
        
        // Add subscribers to different channels
        harness.add_subscriber("user_events", TestEventSubscriber::new("user_subscriber")).await;
        harness.add_subscriber("system_events", TestEventSubscriber::new("system_subscriber")).await;
        harness.add_subscriber("user_events", TestEventSubscriber::new("analytics_subscriber")).await;
        
        // Emit events
        let user_event = Event {
            event_type: "user_login".to_string(),
            data: json!({"user_id": "user123"}),
            timestamp: Utc::now(),
            sequence: 1,
            source: "auth_service".to_string(),
        };
        
        let system_event = Event {
            event_type: "system_startup".to_string(),
            data: json!({"service": "api_server"}),
            timestamp: Utc::now(),
            sequence: 2,
            source: "system".to_string(),
        };
        
        harness.emit_event(user_event.clone()).await.unwrap();
        harness.emit_event(system_event.clone()).await.unwrap();
        
        // Wait for event processing
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Verify delivery
        let user_subscriber_events = harness.get_subscriber_events("user_subscriber");
        let system_subscriber_events = harness.get_subscriber_events("system_subscriber");
        let analytics_subscriber_events = harness.get_subscriber_events("analytics_subscriber");
        
        // User subscriber should receive user events only
        assert_eq!(user_subscriber_events.len(), 1);
        assert_eq!(user_subscriber_events[0].event_type, "user_login");
        
        // System subscriber should receive system events only
        assert_eq!(system_subscriber_events.len(), 1);
        assert_eq!(system_subscriber_events[0].event_type, "system_startup");
        
        // Analytics subscriber should also receive user events
        assert_eq!(analytics_subscriber_events.len(), 1);
        assert_eq!(analytics_subscriber_events[0].event_type, "user_login");
    }
    
    #[tokio::test]
    async fn test_event_bus_throughput() {
        let mut harness = EventBusTestHarness::new().await;
        
        // Add high-performance subscriber
        harness.add_subscriber("high_throughput", TestEventSubscriber::new("throughput_subscriber")).await;
        
        let num_events = 10000;
        let start_time = Instant::now();
        
        // Emit many events rapidly
        let mut emission_futures = Vec::new();
        for i in 0..num_events {
            let event = Event {
                event_type: "throughput_test".to_string(),
                data: json!({"index": i}),
                timestamp: Utc::now(),
                sequence: i as u64,
                source: "test".to_string(),
            };
            
            emission_futures.push(harness.emit_event(event));
        }
        
        // Wait for all emissions
        let results = futures::future::try_join_all(emission_futures).await.unwrap();
        let emission_time = start_time.elapsed();
        
        // Wait for processing to complete
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let processing_time = start_time.elapsed();
        
        // Verify all events were emitted successfully
        let successful_emissions = results.iter().filter(|r| matches!(r, EmissionResult::Success { .. })).count();
        assert_eq!(successful_emissions, num_events);
        
        // Verify throughput metrics
        let events_per_second = num_events as f64 / emission_time.as_secs_f64();
        println!("Emission throughput: {:.2} events/sec", events_per_second);
        
        let total_events_per_second = num_events as f64 / processing_time.as_secs_f64();
        println!("Total throughput: {:.2} events/sec", total_events_per_second);
        
        // Should handle at least 1000 events/sec
        assert!(events_per_second > 1000.0, "Emission throughput too low: {:.2}", events_per_second);
        
        // Verify all events were delivered
        let received_events = harness.get_subscriber_events("throughput_subscriber");
        assert_eq!(received_events.len(), num_events);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_behavior() {
        let mut harness = EventBusTestHarness::new().await;
        
        // Add failing subscriber
        harness.add_subscriber("failing_channel", 
            TestEventSubscriber::new("failing_subscriber").with_failure_rate(0.8)).await;
        
        // Add reliable subscriber to same channel
        harness.add_subscriber("failing_channel", 
            TestEventSubscriber::new("reliable_subscriber").with_failure_rate(0.0)).await;
        
        let num_events = 100;
        
        // Emit events that should trigger circuit breaker
        for i in 0..num_events {
            let event = Event {
                event_type: "test_event".to_string(),
                data: json!({"index": i}),
                timestamp: Utc::now(),
                sequence: i as u64,
                source: "test".to_string(),
            };
            
            harness.emit_event(event).await.unwrap();
        }
        
        // Wait for processing
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let failing_subscriber_events = harness.get_subscriber_events("failing_subscriber");
        let reliable_subscriber_events = harness.get_subscriber_events("reliable_subscriber");
        
        // Failing subscriber should receive fewer events due to circuit breaker
        assert!(failing_subscriber_events.len() < num_events);
        
        // Reliable subscriber should receive all events
        assert_eq!(reliable_subscriber_events.len(), num_events);
        
        println!("Failing subscriber received: {} events", failing_subscriber_events.len());
        println!("Reliable subscriber received: {} events", reliable_subscriber_events.len());
    }
}
```

## Tool-Wrapped Agent Testing

### 1. Tool-Agent Integration Testing

```rust
#[cfg(test)]
mod tool_wrapped_agent_tests {
    use super::*;
    
    // Mock agent for testing
    #[derive(Clone)]
    struct MockAgent {
        id: String,
        responses: Arc<Mutex<VecDeque<String>>>,
        execution_delay: Duration,
        should_fail: bool,
    }
    
    impl MockAgent {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                responses: Arc::new(Mutex::new(VecDeque::new())),
                execution_delay: Duration::from_millis(10),
                should_fail: false,
            }
        }
        
        fn add_response(&self, response: &str) {
            let mut responses = self.responses.lock().unwrap();
            responses.push_back(response.to_string());
        }
        
        fn set_failure_mode(&mut self, should_fail: bool) {
            self.should_fail = should_fail;
        }
    }
    
    #[async_trait]
    impl Agent for MockAgent {
        fn id(&self) -> &str {
            &self.id
        }
        
        fn name(&self) -> &str {
            &self.id
        }
        
        async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput> {
            if self.execution_delay > Duration::ZERO {
                tokio::time::sleep(self.execution_delay).await;
            }
            
            if self.should_fail {
                return Err(anyhow!("Mock agent configured to fail"));
            }
            
            let response = {
                let mut responses = self.responses.lock().unwrap();
                responses.pop_front().unwrap_or_else(|| "Default response".to_string())
            };
            
            Ok(AgentOutput {
                content: response,
                metadata: HashMap::from([
                    ("agent_id".to_string(), Value::String(self.id.clone())),
                    ("execution_time_ms".to_string(), Value::Number(self.execution_delay.as_millis().into())),
                ]),
            })
        }
        
        async fn chat(&mut self, message: &str) -> Result<String> {
            let input = AgentInput {
                message: message.to_string(),
                context: HashMap::new(),
            };
            let output = self.execute(input).await?;
            Ok(output.content)
        }
    }
    
    #[tokio::test]
    async fn test_agent_as_tool_basic_functionality() {
        let mut mock_agent = MockAgent::new("test_agent");
        mock_agent.add_response("I analyzed your code and found 3 issues");
        
        let agent_tool = AgentAsTool::new(
            Box::new(mock_agent),
            "code_analyzer".to_string(),
            "Analyzes code for issues and improvements".to_string(),
        );
        
        // Test tool interface
        assert_eq!(agent_tool.name(), "code_analyzer");
        assert_eq!(agent_tool.description(), "Analyzes code for issues and improvements");
        
        // Test schema
        let schema = agent_tool.parameters_schema();
        assert!(schema.get("properties").is_some());
        assert!(schema["properties"].get("message").is_some());
        
        // Test execution
        let params = json!({
            "message": "Please analyze this code: fn main() { println!(\"Hello\"); }"
        });
        
        let result = agent_tool.execute(params).await.unwrap();
        assert_eq!(result.content["response"], "I analyzed your code and found 3 issues");
        
        // Verify metadata
        assert_eq!(result.metadata["agent_id"], Value::String("test_agent".to_string()));
        assert_eq!(result.metadata["agent_type"], Value::String("wrapped_agent".to_string()));
    }
    
    #[tokio::test]
    async fn test_agent_tool_error_handling() {
        let mut mock_agent = MockAgent::new("failing_agent");
        mock_agent.set_failure_mode(true);
        
        let agent_tool = AgentAsTool::new(
            Box::new(mock_agent),
            "failing_tool".to_string(),
            "Tool that fails for testing".to_string(),
        );
        
        let params = json!({
            "message": "This should fail"
        });
        
        let result = agent_tool.execute(params).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Mock agent configured to fail"));
    }
    
    #[tokio::test]
    async fn test_agent_tool_parameter_validation() {
        let mock_agent = MockAgent::new("validation_agent");
        
        let agent_tool = AgentAsTool::new(
            Box::new(mock_agent),
            "validator_tool".to_string(),
            "Tests parameter validation".to_string(),
        );
        
        // Test missing required parameter
        let params = json!({
            "context": {"some": "data"}
        });
        
        let result = agent_tool.execute(params).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Missing message parameter"));
    }
    
    #[tokio::test]
    async fn test_agent_tool_composition() {
        // Create multiple agent tools
        let mut code_analyzer = MockAgent::new("code_analyzer");
        code_analyzer.add_response("Code analysis: 2 warnings, 1 error");
        
        let mut security_scanner = MockAgent::new("security_scanner");
        security_scanner.add_response("Security scan: No vulnerabilities found");
        
        let mut performance_profiler = MockAgent::new("performance_profiler");
        performance_profiler.add_response("Performance analysis: 95% efficiency");
        
        // Wrap as tools
        let tools = vec![
            Box::new(AgentAsTool::new(
                Box::new(code_analyzer),
                "code_analyzer".to_string(),
                "Analyzes code quality".to_string(),
            )) as Box<dyn Tool>,
            Box::new(AgentAsTool::new(
                Box::new(security_scanner),
                "security_scanner".to_string(),
                "Scans for security issues".to_string(),
            )) as Box<dyn Tool>,
            Box::new(AgentAsTool::new(
                Box::new(performance_profiler),
                "performance_profiler".to_string(),
                "Profiles performance".to_string(),
            )) as Box<dyn Tool>,
        ];
        
        // Create orchestrator agent with tool-wrapped agents
        let mut orchestrator = MockAgent::new("orchestrator");
        orchestrator.add_response("Running comprehensive code review...");
        
        // Simulate orchestrator using all tools
        let code_to_analyze = "fn fibonacci(n: u32) -> u32 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }";
        
        let mut results = Vec::new();
        
        for tool in tools {
            let params = json!({
                "message": format!("Analyze this code: {}", code_to_analyze)
            });
            
            let result = tool.execute(params).await.unwrap();
            results.push(result);
        }
        
        // Verify all tools executed successfully
        assert_eq!(results.len(), 3);
        
        assert!(results[0].content["response"].as_str().unwrap().contains("Code analysis"));
        assert!(results[1].content["response"].as_str().unwrap().contains("Security scan"));
        assert!(results[2].content["response"].as_str().unwrap().contains("Performance analysis"));
        
        // Verify different agent IDs
        assert_eq!(results[0].metadata["agent_id"], Value::String("code_analyzer".to_string()));
        assert_eq!(results[1].metadata["agent_id"], Value::String("security_scanner".to_string()));
        assert_eq!(results[2].metadata["agent_id"], Value::String("performance_profiler".to_string()));
    }
    
    #[tokio::test]
    async fn test_recursive_agent_tool_prevention() {
        // Test to ensure agent tools don't create infinite recursion
        let mut recursive_agent = MockAgent::new("recursive_agent");
        recursive_agent.add_response("I'm calling myself!");
        
        let agent_tool = AgentAsTool::new(
            Box::new(recursive_agent),
            "recursive_tool".to_string(),
            "Tool that could cause recursion".to_string(),
        );
        
        // Create agent that uses the tool
        let mut orchestrator = Agent::new(AgentConfig {
            system_prompt: Some("You have access to tools but should not create infinite loops".to_string()),
            tools: vec![Box::new(agent_tool)],
            ..Default::default()
        });
        
        // This should complete without infinite recursion
        let result = orchestrator.chat("Use the recursive_tool to analyze something").await;
        
        // Should either complete successfully or fail gracefully (not hang)
        assert!(result.is_ok() || result.is_err());
        
        // If successful, should have executed in reasonable time
        // (This test mainly ensures no infinite loops occur)
    }
}
```

## Built-in Component Testing

### 1. Component Integration Testing

```rust
#[cfg(test)]
mod builtin_component_tests {
    use super::*;
    
    // Test harness for built-in components
    struct ComponentTestHarness {
        agent: Agent,
        tool_registry: HashMap<String, Box<dyn Tool>>,
        execution_log: Arc<Mutex<Vec<ComponentExecution>>>,
    }
    
    #[derive(Debug, Clone)]
    struct ComponentExecution {
        component_type: String,
        component_name: String,
        input: serde_json::Value,
        output: Option<serde_json::Value>,
        error: Option<String>,
        duration: Duration,
        timestamp: Instant,
    }
    
    impl ComponentTestHarness {
        fn new() -> Self {
            let execution_log = Arc::new(Mutex::new(Vec::new()));
            
            // Create agent with comprehensive built-in tools
            let tools = vec![
                // Data tools
                Box::new(CsvTool::new()) as Box<dyn Tool>,
                Box::new(JsonTool::new()) as Box<dyn Tool>,
                Box::new(XmlTool::new()) as Box<dyn Tool>,
                
                // Web tools
                Box::new(WebSearchTool::new(WebSearchConfig {
                    api_key: "test_key".to_string(),
                    max_results: 10,
                    rate_limit: RateLimit::new(100, Duration::from_secs(60)),
                })) as Box<dyn Tool>,
                Box::new(WebScrapingTool::new()) as Box<dyn Tool>,
                
                // File tools
                Box::new(FileSystemTool::new(FileSystemConfig {
                    allowed_paths: vec!["./test_data".to_string(), "/tmp".to_string()],
                    read_only: false,
                    max_file_size: 10 * 1024 * 1024, // 10MB
                })) as Box<dyn Tool>,
                
                // Communication tools
                Box::new(EmailTool::new(EmailConfig {
                    smtp_server: "smtp.test.com".to_string(),
                    username: "test@test.com".to_string(),
                    password: "test_password".to_string(),
                })) as Box<dyn Tool>,
                
                // AI tools
                Box::new(EmbeddingTool::new(EmbeddingConfig {
                    model: "text-embedding-ada-002".to_string(),
                    api_key: "test_key".to_string(),
                })) as Box<dyn Tool>,
            ];
            
            let agent = Agent::new(AgentConfig {
                system_prompt: Some("You are a test agent with access to various tools".to_string()),
                tools,
                ..Default::default()
            });
            
            Self {
                agent,
                tool_registry: HashMap::new(),
                execution_log,
            }
        }
        
        async fn test_tool_execution(&mut self, tool_name: &str, input: serde_json::Value) -> Result<ComponentExecution> {
            let start_time = Instant::now();
            
            let result = self.agent.execute_tool(tool_name, input.clone()).await;
            let duration = start_time.elapsed();
            
            let execution = ComponentExecution {
                component_type: "tool".to_string(),
                component_name: tool_name.to_string(),
                input,
                output: result.as_ref().ok().map(|r| r.content.clone()),
                error: result.as_ref().err().map(|e| e.to_string()),
                duration,
                timestamp: start_time,
            };
            
            {
                let mut log = self.execution_log.lock().unwrap();
                log.push(execution.clone());
            }
            
            Ok(execution)
        }
        
        fn get_execution_log(&self) -> Vec<ComponentExecution> {
            self.execution_log.lock().unwrap().clone()
        }
    }
    
    #[tokio::test]
    async fn test_data_tools_integration() {
        let mut harness = ComponentTestHarness::new();
        
        // Test CSV tool
        let csv_data = "name,age,city\nJohn,30,NYC\nJane,25,LA\nBob,35,Chicago";
        
        let csv_result = harness.test_tool_execution("csv_tool", json!({
            "action": "parse",
            "data": csv_data
        })).await.unwrap();
        
        assert!(csv_result.error.is_none());
        assert!(csv_result.output.is_some());
        
        let output = csv_result.output.unwrap();
        assert!(output["rows"].is_array());
        assert_eq!(output["rows"].as_array().unwrap().len(), 3);
        
        // Test JSON tool
        let json_data = r#"{"users": [{"name": "Alice", "age": 28}, {"name": "Bob", "age": 32}]}"#;
        
        let json_result = harness.test_tool_execution("json_tool", json!({
            "action": "parse",
            "data": json_data
        })).await.unwrap();
        
        assert!(json_result.error.is_none());
        assert!(json_result.output.is_some());
        
        let output = json_result.output.unwrap();
        assert!(output["parsed"].is_object());
        assert_eq!(output["parsed"]["users"].as_array().unwrap().len(), 2);
    }
    
    #[tokio::test]
    async fn test_file_system_tool_security() {
        let mut harness = ComponentTestHarness::new();
        
        // Test allowed path access
        let allowed_result = harness.test_tool_execution("filesystem_tool", json!({
            "action": "write",
            "path": "/tmp/test_file.txt",
            "content": "Test content"
        })).await.unwrap();
        
        assert!(allowed_result.error.is_none());
        
        // Test disallowed path access
        let disallowed_result = harness.test_tool_execution("filesystem_tool", json!({
            "action": "write", 
            "path": "/etc/passwd",
            "content": "malicious content"
        })).await.unwrap();
        
        assert!(disallowed_result.error.is_some());
        assert!(disallowed_result.error.unwrap().contains("Path not allowed"));
        
        // Test path traversal attack
        let traversal_result = harness.test_tool_execution("filesystem_tool", json!({
            "action": "read",
            "path": "../../../etc/passwd"
        })).await.unwrap();
        
        assert!(traversal_result.error.is_some());
        assert!(traversal_result.error.unwrap().contains("Path not allowed"));
    }
    
    #[tokio::test]
    async fn test_web_tools_rate_limiting() {
        let mut harness = ComponentTestHarness::new();
        
        // Make rapid requests to test rate limiting
        let mut requests = Vec::new();
        
        for i in 0..150 { // Exceed rate limit of 100/minute
            let request = harness.test_tool_execution("web_search_tool", json!({
                "query": format!("test query {}", i),
                "max_results": 5
            }));
            
            requests.push(request);
        }
        
        let results = futures::future::join_all(requests).await;
        
        let successful_requests = results.iter()
            .filter(|r| r.as_ref().unwrap().error.is_none())
            .count();
        
        let rate_limited_requests = results.iter()
            .filter(|r| r.as_ref().unwrap().error.as_ref()
                .map(|e| e.contains("rate limit"))
                .unwrap_or(false))
            .count();
        
        // Should have some successful requests and some rate-limited
        assert!(successful_requests > 0);
        assert!(rate_limited_requests > 0);
        assert_eq!(successful_requests + rate_limited_requests, 150);
        
        println!("Successful requests: {}", successful_requests);
        println!("Rate limited requests: {}", rate_limited_requests);
    }
    
    #[tokio::test]
    async fn test_tool_chain_execution() {
        let mut harness = ComponentTestHarness::new();
        
        // Test chaining multiple tools together
        
        // Step 1: Create test data file
        let file_creation = harness.test_tool_execution("filesystem_tool", json!({
            "action": "write",
            "path": "/tmp/test_data.csv",
            "content": "product,price,category\nLaptop,999.99,Electronics\nBook,29.99,Education\nShirt,19.99,Clothing"
        })).await.unwrap();
        
        assert!(file_creation.error.is_none());
        
        // Step 2: Read and parse the CSV file
        let file_read = harness.test_tool_execution("filesystem_tool", json!({
            "action": "read",
            "path": "/tmp/test_data.csv"
        })).await.unwrap();
        
        assert!(file_read.error.is_none());
        let file_content = file_read.output.unwrap()["content"].as_str().unwrap();
        
        // Step 3: Parse CSV data
        let csv_parse = harness.test_tool_execution("csv_tool", json!({
            "action": "parse",
            "data": file_content
        })).await.unwrap();
        
        assert!(csv_parse.error.is_none());
        let parsed_data = csv_parse.output.unwrap();
        
        // Step 4: Convert to JSON
        let json_conversion = harness.test_tool_execution("json_tool", json!({
            "action": "format",
            "data": parsed_data["rows"]
        })).await.unwrap();
        
        assert!(json_conversion.error.is_none());
        
        // Verify the complete chain executed successfully
        let execution_log = harness.get_execution_log();
        assert_eq!(execution_log.len(), 4);
        
        // Verify all steps were successful
        for execution in execution_log {
            assert!(execution.error.is_none(), "Tool {} failed: {:?}", execution.component_name, execution.error);
        }
        
        // Clean up
        let cleanup = harness.test_tool_execution("filesystem_tool", json!({
            "action": "delete",
            "path": "/tmp/test_data.csv"
        })).await.unwrap();
        
        assert!(cleanup.error.is_none());
    }
}
```

## Async Script Execution Testing

### 1. Lua Coroutine Testing

```lua
-- test_lua_async.lua - Lua async pattern testing
local test_utils = require("test_utils")

-- Test cooperative scheduling
function test_cooperative_scheduling()
    local scheduler = AsyncScheduler.new()
    local execution_order = {}
    
    -- Task 1: Short running
    scheduler:add_task("short_task", function()
        table.insert(execution_order, "short_start")
        coroutine.yield() -- Cooperative yield
        table.insert(execution_order, "short_end")
        return "short_result"
    end)
    
    -- Task 2: Long running with yields
    scheduler:add_task("long_task", function()
        table.insert(execution_order, "long_start")
        for i = 1, 3 do
            table.insert(execution_order, "long_step_" .. i)
            coroutine.yield() -- Yield after each step
        end
        table.insert(execution_order, "long_end")
        return "long_result"
    end)
    
    -- Task 3: Another short task
    scheduler:add_task("short_task_2", function()
        table.insert(execution_order, "short2_start")
        coroutine.yield()
        table.insert(execution_order, "short2_end")
        return "short2_result"
    end)
    
    -- Run scheduler
    scheduler:run()
    
    -- Verify interleaved execution
    local expected_order = {
        "short_start", "long_start", "short2_start",
        "short_end", "long_step_1", "short2_end",
        "long_step_2", "long_step_3", "long_end"
    }
    
    test_utils.assert_array_equal(execution_order, expected_order)
    print("✓ Cooperative scheduling test passed")
end

-- Test error handling in coroutines
function test_coroutine_error_handling()
    local scheduler = AsyncScheduler.new()
    local results = {}
    
    -- Successful task
    scheduler:add_task("success_task", function()
        return "success"
    end)
    
    -- Failing task
    scheduler:add_task("failing_task", function()
        error("Intentional failure")
    end)
    
    -- Task that runs after failure
    scheduler:add_task("after_failure_task", function()
        return "continued"
    end)
    
    scheduler:run()
    
    -- Verify failure isolation
    local task_results = scheduler:get_results()
    
    assert(task_results["success_task"].status == "completed")
    assert(task_results["success_task"].result == "success")
    
    assert(task_results["failing_task"].status == "error")
    assert(string.find(task_results["failing_task"].error, "Intentional failure"))
    
    assert(task_results["after_failure_task"].status == "completed")
    assert(task_results["after_failure_task"].result == "continued")
    
    print("✓ Coroutine error handling test passed")
end

-- Test agent execution with coroutines
function test_agent_coroutine_integration()
    local agent = Agent.new({
        system_prompt = "You are a test agent",
        tools = {TestTool.new()}
    })
    
    -- Create async agent wrapper
    local async_agent = CoroutineAgentWrapper.new(agent)
    
    local tasks = {}
    
    -- Multiple concurrent agent calls
    for i = 1, 5 do
        local task = coroutine.create(function()
            local result = async_agent:chat_async("Process request " .. i)
            return result
        end)
        table.insert(tasks, task)
    end
    
    -- Execute all tasks cooperatively
    local results = {}
    local completed = 0
    
    while completed < #tasks do
        for i, task in ipairs(tasks) do
            if coroutine.status(task) ~= "dead" then
                local success, result = coroutine.resume(task)
                if success and coroutine.status(task) == "dead" then
                    results[i] = result
                    completed = completed + 1
                elseif not success then
                    error("Task " .. i .. " failed: " .. tostring(result))
                end
            end
        end
        
        -- Small yield to prevent tight loop
        os.execute("sleep 0.001")
    end
    
    -- Verify all tasks completed
    assert(#results == 5)
    for i, result in ipairs(results) do
        assert(result ~= nil, "Task " .. i .. " returned nil")
        assert(string.find(result, "Process request " .. i), "Unexpected result for task " .. i)
    end
    
    print("✓ Agent coroutine integration test passed")
end

-- Run all tests
function run_lua_async_tests()
    print("Running Lua async pattern tests...")
    
    test_cooperative_scheduling()
    test_coroutine_error_handling()
    test_agent_coroutine_integration()
    
    print("All Lua async tests passed! ✓")
end

return {
    run_tests = run_lua_async_tests
}
```

### 2. JavaScript Promise Testing

```javascript
// test_js_async.js - JavaScript async pattern testing
const assert = require('assert');

class AsyncTestHarness {
    constructor() {
        this.executionLog = [];
        this.startTime = Date.now();
    }
    
    log(event) {
        this.executionLog.push({
            ...event,
            timestamp: Date.now() - this.startTime
        });
    }
    
    getExecutionOrder() {
        return this.executionLog.map(e => e.event);
    }
}

// Test Promise-based agent coordination
async function testPromiseCoordination() {
    const harness = new AsyncTestHarness();
    
    // Mock agents with different execution times
    const createMockAgent = (name, delay) => ({
        async execute(input) {
            harness.log({ event: `${name}_start`, input });
            await new Promise(resolve => setTimeout(resolve, delay));
            harness.log({ event: `${name}_end`, input });
            return `${name}_result_${input}`;
        }
    });
    
    const fastAgent = createMockAgent('fast', 10);
    const mediumAgent = createMockAgent('medium', 50);
    const slowAgent = createMockAgent('slow', 100);
    
    // Test parallel execution
    console.log('Testing parallel execution...');
    const startTime = Date.now();
    
    const parallelPromises = [
        fastAgent.execute('input1'),
        mediumAgent.execute('input2'),
        slowAgent.execute('input3')
    ];
    
    const parallelResults = await Promise.all(parallelPromises);
    const parallelTime = Date.now() - startTime;
    
    // Should complete in ~100ms (slowest agent time), not 160ms (sum of all)
    assert(parallelTime < 150, `Parallel execution took too long: ${parallelTime}ms`);
    assert.equal(parallelResults.length, 3);
    assert(parallelResults[0].includes('fast_result'));
    assert(parallelResults[1].includes('medium_result'));
    assert(parallelResults[2].includes('slow_result'));
    
    console.log('✓ Promise coordination test passed');
}

// Test error handling in async contexts
async function testAsyncErrorHandling() {
    const harness = new AsyncTestHarness();
    
    const createMockAgent = (name, shouldFail) => ({
        async execute(input) {
            harness.log({ event: `${name}_start`, input });
            await new Promise(resolve => setTimeout(resolve, 10));
            
            if (shouldFail) {
                harness.log({ event: `${name}_error`, input });
                throw new Error(`${name} intentional failure`);
            }
            
            harness.log({ event: `${name}_success`, input });
            return `${name}_result`;
        }
    });
    
    const successAgent = createMockAgent('success', false);
    const failingAgent = createMockAgent('failing', true);
    const recoverAgent = createMockAgent('recover', false);
    
    // Test Promise.allSettled for error isolation
    console.log('Testing async error handling...');
    
    const promises = [
        successAgent.execute('input1'),
        failingAgent.execute('input2'),
        recoverAgent.execute('input3')
    ];
    
    const results = await Promise.allSettled(promises);
    
    // Verify results
    assert.equal(results[0].status, 'fulfilled');
    assert.equal(results[0].value, 'success_result');
    
    assert.equal(results[1].status, 'rejected');
    assert(results[1].reason.message.includes('failing intentional failure'));
    
    assert.equal(results[2].status, 'fulfilled');
    assert.equal(results[2].value, 'recover_result');
    
    // Verify execution order
    const executionOrder = harness.getExecutionOrder();
    assert(executionOrder.includes('success_start'));
    assert(executionOrder.includes('failing_start'));
    assert(executionOrder.includes('recover_start'));
    assert(executionOrder.includes('success_success'));
    assert(executionOrder.includes('failing_error'));
    assert(executionOrder.includes('recover_success'));
    
    console.log('✓ Async error handling test passed');
}

// Test stream processing with backpressure
async function testStreamProcessing() {
    console.log('Testing stream processing...');
    
    class AsyncAgentStream {
        constructor(concurrency = 3) {
            this.concurrency = concurrency;
            this.processing = new Set();
            this.queue = [];
            this.results = [];
        }
        
        async process(items, processor) {
            // Add all items to queue
            this.queue.push(...items.map((item, index) => ({ item, index })));
            
            return new Promise((resolve) => {
                const processNext = async () => {
                    while (this.processing.size < this.concurrency && this.queue.length > 0) {
                        const { item, index } = this.queue.shift();
                        
                        const promise = processor(item, index).then(result => {
                            this.results[index] = result;
                            this.processing.delete(promise);
                            
                            if (this.queue.length > 0) {
                                processNext();
                            } else if (this.processing.size === 0) {
                                resolve(this.results);
                            }
                        }).catch(error => {
                            this.results[index] = { error: error.message };
                            this.processing.delete(promise);
                            processNext();
                        });
                        
                        this.processing.add(promise);
                    }
                    
                    if (this.queue.length === 0 && this.processing.size === 0) {
                        resolve(this.results);
                    }
                };
                
                processNext();
            });
        }
    }
    
    const stream = new AsyncAgentStream(3);
    
    // Create test data
    const testItems = Array.from({ length: 10 }, (_, i) => `item_${i}`);
    
    // Mock processor with variable processing time
    const processor = async (item, index) => {
        const delay = Math.random() * 50 + 10; // 10-60ms
        await new Promise(resolve => setTimeout(resolve, delay));
        return `processed_${item}_${index}`;
    };
    
    const startTime = Date.now();
    const results = await stream.process(testItems, processor);
    const totalTime = Date.now() - startTime;
    
    // Verify results
    assert.equal(results.length, 10);
    results.forEach((result, index) => {
        if (!result.error) {
            assert(result.includes(`processed_item_${index}_${index}`));
        }
    });
    
    // Should be faster than sequential processing
    // Sequential would be ~10 * average_delay = ~350ms
    // Concurrent with 3 workers should be ~4 * average_delay = ~140ms
    assert(totalTime < 250, `Stream processing took too long: ${totalTime}ms`);
    
    console.log(`✓ Stream processing test passed (${totalTime}ms)`);
}

// Test resource cleanup in interrupted operations
async function testResourceCleanup() {
    console.log('Testing resource cleanup...');
    
    class ResourceTracker {
        constructor() {
            this.resources = new Set();
            this.cleaned = new Set();
        }
        
        allocate(id) {
            this.resources.add(id);
            return {
                id,
                cleanup: () => {
                    this.resources.delete(id);
                    this.cleaned.add(id);
                }
            };
        }
        
        getStats() {
            return {
                allocated: this.resources.size,
                cleaned: this.cleaned.size
            };
        }
    }
    
    const tracker = new ResourceTracker();
    
    // Create agent that allocates resources
    const createResourceAgent = (name, shouldFail = false, failAfter = 50) => ({
        async execute(input) {
            const resource = tracker.allocate(`${name}_${input}`);
            
            try {
                await new Promise(resolve => setTimeout(resolve, 30));
                
                if (shouldFail) {
                    await new Promise(resolve => setTimeout(resolve, failAfter));
                    throw new Error(`${name} failed after allocating resource`);
                }
                
                await new Promise(resolve => setTimeout(resolve, 20));
                return `${name}_success`;
            } finally {
                // Always cleanup resources
                resource.cleanup();
            }
        }
    });
    
    const successAgent = createResourceAgent('success', false);
    const failingAgent = createResourceAgent('failing', true);
    const timeoutAgent = createResourceAgent('timeout', false);
    
    // Test normal execution
    await successAgent.execute('test1');
    
    // Test with failure
    try {
        await failingAgent.execute('test2');
    } catch (error) {
        // Expected failure
    }
    
    // Test with timeout/cancellation
    const controller = new AbortController();
    const timeoutPromise = timeoutAgent.execute('test3');
    
    setTimeout(() => controller.abort(), 25); // Cancel early
    
    try {
        await Promise.race([
            timeoutPromise,
            new Promise((_, reject) => {
                controller.signal.addEventListener('abort', () => {
                    reject(new Error('Operation cancelled'));
                });
            })
        ]);
    } catch (error) {
        // Expected cancellation
    }
    
    // Allow some time for cleanup
    await new Promise(resolve => setTimeout(resolve, 100));
    
    const stats = tracker.getStats();
    
    // All resources should be cleaned up
    assert.equal(stats.allocated, 0, `${stats.allocated} resources still allocated`);
    assert(stats.cleaned >= 2, `Only ${stats.cleaned} resources were cleaned`);
    
    console.log('✓ Resource cleanup test passed');
}

// Main test runner
async function runJavaScriptAsyncTests() {
    console.log('Running JavaScript async pattern tests...');
    
    await testPromiseCoordination();
    await testAsyncErrorHandling();
    await testStreamProcessing();
    await testResourceCleanup();
    
    console.log('All JavaScript async tests passed! ✓');
}

module.exports = {
    runTests: runJavaScriptAsyncTests
};
```

This comprehensive testing strategy ensures reliability and correctness across all advanced rs-llmspell features, providing robust validation for production deployment.