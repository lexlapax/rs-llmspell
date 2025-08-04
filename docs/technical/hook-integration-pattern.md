# Hook Integration Pattern

This document describes the standardized pattern for integrating hooks into executors across the llmspell codebase.

## Executor Structure

All executors that support hooks should follow this pattern:

```rust
#[derive(Clone)]
pub struct MyExecutor {
    /// Hook executor for running hooks
    hook_executor: Option<Arc<HookExecutor>>,
    /// Hook registry for retrieving hooks
    hook_registry: Option<Arc<HookRegistry>>,
    /// Circuit breaker for performance protection (optional)
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    /// Configuration
    config: MyLifecycleConfig,
    /// Component ID for this executor
    component_id: ComponentId,
}
```

## Constructor Pattern

```rust
impl MyExecutor {
    pub fn new(
        config: MyLifecycleConfig,
        hook_executor: Option<Arc<HookExecutor>>,
        hook_registry: Option<Arc<HookRegistry>>,
    ) -> Self {
        let component_id = ComponentId::new(ComponentType::MyType, "my_executor".to_string());
        
        // Circuit breaker is optional
        let circuit_breaker = if config.enable_circuit_breaker {
            hook_registry.as_ref().map(|_registry| {
                Arc::new(CircuitBreaker::new(format!(
                    "my_executor_{}",
                    component_id.name
                )))
            })
        } else {
            None
        };
        
        Self {
            hook_executor,
            hook_registry,
            circuit_breaker,
            config,
            component_id,
        }
    }
}
```

## Hook Execution Pattern

```rust
async fn execute_hook_phase(&self, context: &MyHookContext) -> Result<()> {
    // Check if hooks are enabled
    if !self.config.enable_hooks {
        return Ok(());
    }
    
    // Ensure both executor and registry are available
    let (Some(hook_executor), Some(hook_registry)) = (&self.hook_executor, &self.hook_registry) else {
        return Ok(());
    };
    
    // Get hook point from context
    let hook_point = context.get_hook_point();
    
    // Track execution time
    let start_time = Instant::now();
    
    // Get hooks from registry
    let hooks = hook_registry.get_hooks(&hook_point);
    
    if !hooks.is_empty() {
        // Convert domain-specific context to HookContext
        let mut hook_context = context.base_context.clone();
        
        // Add domain-specific metadata
        hook_context.metadata.insert("key".to_string(), "value".to_string());
        
        // Execute hooks
        let results = hook_executor.execute_hooks(&hooks, &mut hook_context).await;
        
        match results {
            Ok(hook_results) => {
                // Check for cancellation
                for result in hook_results {
                    if let HookResult::Cancel(reason) = result {
                        return Err(LLMSpellError::MyErrorType {
                            message: format!("Hook cancelled execution: {}", reason),
                            // other fields...
                        });
                    }
                }
            }
            Err(e) => {
                // Log warning but continue execution
                warn!("Hook execution failed: {}", e);
                // Hooks should not break core functionality
            }
        }
    }
    
    let duration = start_time.elapsed();
    debug!("Hooks executed in {:?}", duration);
    
    // Record success with circuit breaker if available
    if let Some(circuit_breaker) = &self.circuit_breaker {
        circuit_breaker.record_success(duration);
    }
    
    Ok(())
}
```

## Key Principles

1. **Always store both hook_executor and hook_registry** - Both are needed for proper hook execution
2. **Use `.as_ref()` when passing Option<Arc<T>>** - Avoid moving the Arc when not needed
3. **Check for HookResult::Cancel** - This is the standard way hooks can block execution
4. **Continue on hook errors** - Hook failures should log warnings but not break core functionality
5. **Track execution time** - Important for performance monitoring and circuit breaker
6. **Use domain-specific error types** - Return appropriate error variants (Tool, Workflow, etc.)

## Examples

- **Tools**: See `llmspell-tools/src/lifecycle/hook_integration.rs`
- **Workflows**: See `llmspell-workflows/src/hooks/integration.rs`
- **Agents**: See `llmspell-agents/src/lifecycle/state_machine.rs`

## Migration Guide

If you find code with stubbed hook execution (e.g., `tokio::time::sleep(Duration::from_millis(1)).await`), replace it with the pattern above, ensuring:

1. Add `hook_registry` field to the executor struct
2. Store `hook_registry` in the constructor
3. Replace stubbed code with actual hook execution following the pattern
4. Add appropriate error handling for HookResult::Cancel
5. Ensure hooks don't break core functionality on failure