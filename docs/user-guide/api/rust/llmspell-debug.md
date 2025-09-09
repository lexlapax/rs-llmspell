# llmspell-debug

**Debug infrastructure with interactive debugging and DAP support** ⭐ **NEW Phase 9**

[← Back to Rust API](README.md) | [Crates.io](https://crates.io/crates/llmspell-debug) | [API Docs](https://docs.rs/llmspell-debug)

---

## Overview

The `llmspell-debug` crate provides enhanced interactive debugging capabilities following the three-layer architecture established in Phase 9. It integrates with the existing `ExecutionBridge` from `llmspell-bridge` and provides high-level debugging interfaces, session management, and condition evaluation.

**Key Features:**
- **Interactive Debugging**: Step through code with breakpoints and watches
- **Session Management**: Multi-client debug sessions with persistence
- **Condition Evaluation**: Evaluate breakpoint conditions and watch expressions
- **DAP Integration**: Debug Adapter Protocol support through kernel
- **Performance Optimized**: <3% overhead when no breakpoints active
- **Language Agnostic**: Works with Lua, JavaScript, Python through bridges

## Architecture

### Three-Layer Design

```
Interactive Layer (llmspell-debug)
    ↓
Bridge Layer (llmspell-bridge ExecutionBridge)
    ↓
Language Layer (Lua/JS/Python debug hooks)
```

### Core Components

```rust
/// Interactive debugger coordinating with ExecutionBridge
pub struct InteractiveDebugger {
    execution_manager: Arc<ExecutionManager>,
    shared_context: Arc<RwLock<SharedExecutionContext>>,
    session_manager: Arc<DebugSessionManager>,
}

/// Debug session for multi-client debugging
pub struct DebugSession {
    pub session_id: String,
    pub client_id: String,
    pub script_path: Option<PathBuf>,
    pub debug_state: DebugState,
    pub current_frame: usize,
    pub breakpoints: Vec<Breakpoint>,
    pub shared_context: SharedExecutionContext,
    pub watch_expressions: Vec<String>,
}
```

## Unified Types from ExecutionBridge

The crate re-exports unified types from `llmspell-bridge::execution_bridge`:

```rust
// Breakpoint with enhanced features
pub struct Breakpoint {
    pub id: String,
    pub source: String,
    pub line: u32,
    pub condition: Option<String>,
    pub hit_count: u32,
    pub ignore_count: u32,
    pub enabled: bool,
}

// Debug state enumeration
pub enum DebugState {
    Running,
    Paused(PauseReason),
    Stepping,
    Terminated,
}

// Pause reasons
pub enum PauseReason {
    Breakpoint { id: String },
    Step,
    Exception { message: String },
    Entry,
    Exit,
}

// Stack frame information
pub struct StackFrame {
    pub id: usize,
    pub name: String,
    pub source: String,
    pub line: u32,
    pub column: u32,
}

// Variable information
pub struct Variable {
    pub name: String,
    pub value: String,
    pub var_type: String,
    pub reference: Option<usize>,
}

// Debug commands
pub enum DebugCommand {
    Continue,
    StepInto,
    StepOver,
    StepOut,
    Pause,
    Terminate,
}
```

## Usage Examples

### Basic Interactive Debugging

```rust
use llmspell_debug::{InteractiveDebugger, ExecutionManager};
use llmspell_bridge::SharedExecutionContext;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    // Create execution manager
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    
    // Create interactive debugger
    let debugger = InteractiveDebugger::new(
        execution_manager,
        shared_context,
    );
    
    // Set a breakpoint
    let bp_id = debugger.set_breakpoint(
        "script.lua".to_string(),
        10
    ).await?;
    
    // Set conditional breakpoint
    let cond_bp = debugger.set_conditional_breakpoint(
        "script.lua".to_string(),
        15,
        "x > 5".to_string()
    ).await?;
    
    // Continue execution
    debugger.continue_execution().await?;
    
    // When paused, get debug state
    let state = debugger.get_debug_state().await;
    println!("Debug state: {:?}", state);
    
    // Get stack trace
    let stack = debugger.get_stack_trace().await;
    for frame in stack {
        println!("  {} at {}:{}", frame.name, frame.source, frame.line);
    }
    
    Ok(())
}
```

### Session Management

```rust
use llmspell_debug::DebugSessionManager;

async fn multi_client_debugging(manager: &DebugSessionManager) -> Result<()> {
    // Create session for client
    let session_id = manager.create_session("client-123".to_string()).await?;
    
    // Start debugging a script
    manager.start_debugging(
        &session_id,
        PathBuf::from("app.lua")
    ).await?;
    
    // Set session-specific breakpoints
    manager.set_session_breakpoint(
        &session_id,
        "app.lua".to_string(),
        25
    ).await?;
    
    // Get session state
    let session = manager.get_session(&session_id).await?;
    println!("Session {} debugging: {:?}", session_id, session.script_path);
    
    // Handle disconnection/reconnection
    manager.disconnect_session(&session_id).await?;
    
    // Client can reconnect to same session
    let reconnected = manager.reconnect_session("client-123").await?;
    assert_eq!(reconnected, session_id);
    
    Ok(())
}
```

### Condition Evaluation

```rust
use llmspell_debug::{ConditionEvaluator, ConditionTemplates};

async fn evaluate_conditions() -> Result<()> {
    let evaluator = ConditionEvaluator::new();
    
    // Validate condition syntax
    let is_valid = evaluator.validate_condition("x > 5 and y < 10")?;
    
    // Evaluate with context
    let context = HashMap::from([
        ("x".to_string(), json!(7)),
        ("y".to_string(), json!(3)),
    ]);
    
    let result = evaluator.evaluate("x > 5 and y < 10", &context).await?;
    assert!(result);
    
    // Use templates for common conditions
    let templates = ConditionTemplates::default();
    let condition = templates.value_range("count", 10, 20);
    println!("Template condition: {}", condition);
    
    Ok(())
}
```

### Installing Debug Hooks

```rust
use llmspell_debug::InteractiveDebugger;
use mlua::Lua;

fn setup_lua_debugging(debugger: &InteractiveDebugger) -> Result<()> {
    let lua = Lua::new();
    
    // Install debug hooks into Lua VM
    debugger.install_lua_hooks(&lua)?;
    
    // Now Lua execution will respect breakpoints
    lua.load(r#"
        function test()
            local x = 5
            print(x)  -- Breakpoint can be set here
            return x * 2
        end
        test()
    "#).exec()?;
    
    Ok(())
}
```

## REPL Debug Commands

The debugger integrates with REPL commands:

```rust
impl InteractiveDebugger {
    /// Handle REPL debug command
    pub async fn handle_repl_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts[0] {
            ".break" => {
                let source = parts[1];
                let line = parts[2].parse()?;
                let id = self.set_breakpoint(source.to_string(), line).await?;
                Ok(format!("Breakpoint set: {}", id))
            }
            ".step" => {
                self.step_into().await?;
                Ok("Stepping into next statement".to_string())
            }
            ".continue" => {
                self.continue_execution().await?;
                Ok("Continuing execution".to_string())
            }
            ".locals" => {
                let vars = self.get_local_variables().await;
                Ok(format_variables(vars))
            }
            ".stack" => {
                let stack = self.get_stack_trace().await;
                Ok(format_stack_trace(stack))
            }
            _ => Err(anyhow!("Unknown debug command")),
        }
    }
}
```

## Variable Inspection

### Getting Variables

```rust
impl InteractiveDebugger {
    /// Get local variables for current frame
    pub async fn get_local_variables(&self) -> Vec<Variable> {
        self.execution_manager.get_local_variables().await
    }
    
    /// Get global variables
    pub async fn get_global_variables(&self) -> Vec<Variable> {
        self.execution_manager.get_global_variables().await
    }
    
    /// Get upvalues (closures)
    pub async fn get_upvalues(&self) -> Vec<Variable> {
        self.execution_manager.get_upvalues().await
    }
    
    /// Evaluate expression in current context
    pub async fn evaluate_expression(&self, expr: &str) -> Result<Variable> {
        self.execution_manager.evaluate_expression(expr).await
    }
}
```

### Variable References

Variables with complex types have references for lazy expansion:

```rust
async fn inspect_complex_variable(debugger: &InteractiveDebugger) -> Result<()> {
    let vars = debugger.get_local_variables().await;
    
    for var in vars {
        if let Some(ref_id) = var.reference {
            // Expand complex variable
            let children = debugger.get_variable_children(ref_id).await?;
            println!("{} has {} children", var.name, children.len());
            
            for child in children {
                println!("  {}: {} = {}", child.name, child.var_type, child.value);
            }
        }
    }
    
    Ok(())
}
```

## Breakpoint Management

### Breakpoint Features

```rust
impl Breakpoint {
    /// Create basic breakpoint
    pub fn new(source: String, line: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            line,
            condition: None,
            hit_count: 0,
            ignore_count: 0,
            enabled: true,
        }
    }
    
    /// Add condition
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }
    
    /// Set hit count (break after N hits)
    pub fn with_hit_count(mut self, count: u32) -> Self {
        self.hit_count = count;
        self
    }
    
    /// Set ignore count (ignore first N hits)
    pub fn with_ignore_count(mut self, count: u32) -> Self {
        self.ignore_count = count;
        self
    }
}
```

### Managing Breakpoints

```rust
async fn manage_breakpoints(debugger: &InteractiveDebugger) -> Result<()> {
    // Set multiple breakpoints
    let bp1 = debugger.set_breakpoint("main.lua".into(), 10).await?;
    let bp2 = debugger.set_breakpoint("utils.lua".into(), 25).await?;
    
    // Set conditional breakpoint
    let bp3 = debugger.set_conditional_breakpoint(
        "main.lua".into(),
        30,
        "counter >= 10".into()
    ).await?;
    
    // List all breakpoints
    let breakpoints = debugger.list_breakpoints().await;
    for bp in breakpoints {
        println!("Breakpoint {} at {}:{}", bp.id, bp.source, bp.line);
    }
    
    // Enable/disable breakpoints
    debugger.enable_breakpoint(&bp1, false).await?;
    
    // Remove breakpoint
    debugger.remove_breakpoint(&bp2).await?;
    
    // Clear all breakpoints
    debugger.clear_breakpoints().await?;
    
    Ok(())
}
```

## Step Operations

The debugger supports various step operations:

```rust
impl InteractiveDebugger {
    /// Step into function calls
    pub async fn step_into(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::StepInto)
            .await;
        Ok(())
    }
    
    /// Step over function calls
    pub async fn step_over(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::StepOver)
            .await;
        Ok(())
    }
    
    /// Step out of current function
    pub async fn step_out(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::StepOut)
            .await;
        Ok(())
    }
    
    /// Run to specific line
    pub async fn run_to_line(&self, source: &str, line: u32) -> Result<()> {
        // Set temporary breakpoint
        let temp_bp = self.set_breakpoint(source.to_string(), line).await?;
        
        // Continue execution
        self.continue_execution().await?;
        
        // Remove temporary breakpoint when reached
        self.remove_breakpoint(&temp_bp).await?;
        
        Ok(())
    }
}
```

## Watch Expressions

Monitor expressions during debugging:

```rust
impl DebugSession {
    /// Add watch expression
    pub fn add_watch(&mut self, expression: String) {
        self.watch_expressions.push(expression);
    }
    
    /// Evaluate all watches
    pub async fn evaluate_watches(&self) -> Vec<(String, Result<Variable>)> {
        let mut results = Vec::new();
        
        for expr in &self.watch_expressions {
            let result = self.evaluate_expression(expr).await;
            results.push((expr.clone(), result));
        }
        
        results
    }
    
    /// Remove watch
    pub fn remove_watch(&mut self, expression: &str) -> bool {
        if let Some(pos) = self.watch_expressions.iter().position(|x| x == expression) {
            self.watch_expressions.remove(pos);
            true
        } else {
            false
        }
    }
}
```

## Performance Monitoring

The debugger tracks performance metrics:

```rust
pub struct DebugPerformanceMetrics {
    /// Time spent in debug hooks (microseconds)
    pub hook_overhead_us: u64,
    /// Number of breakpoint checks
    pub breakpoint_checks: u64,
    /// Number of condition evaluations
    pub condition_evals: u64,
    /// Cache hit rate for debug state
    pub cache_hit_rate: f64,
}

impl InteractiveDebugger {
    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> DebugPerformanceMetrics {
        let stats = self.execution_manager.get_statistics().await;
        
        DebugPerformanceMetrics {
            hook_overhead_us: stats.total_hook_time_us,
            breakpoint_checks: stats.breakpoint_checks,
            condition_evals: stats.condition_evaluations,
            cache_hit_rate: stats.cache_hits as f64 / stats.cache_accesses as f64,
        }
    }
    
    /// Reset performance counters
    pub async fn reset_metrics(&self) {
        self.execution_manager.reset_statistics().await;
    }
}
```

## Script Locking

Prevent conflicting debug sessions:

```rust
impl DebugSessionManager {
    /// Start debugging with script lock
    pub async fn start_debugging(
        &self,
        session_id: &str,
        script_path: PathBuf,
    ) -> Result<()> {
        // Check if script is already being debugged
        let locks = self.script_locks.read().await;
        if let Some(existing_session) = locks.get(&script_path) {
            return Err(anyhow!(
                "Script already being debugged by session {}",
                existing_session
            ));
        }
        drop(locks);
        
        // Acquire lock
        self.script_locks.write().await.insert(
            script_path.clone(),
            session_id.to_string()
        );
        
        // Update session
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.script_path = Some(script_path);
        }
        
        Ok(())
    }
    
    /// Stop debugging and release lock
    pub async fn stop_debugging(&self, session_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.read().await.get(session_id) {
            if let Some(ref path) = session.script_path {
                self.script_locks.write().await.remove(path);
            }
        }
        Ok(())
    }
}
```

## Testing

The crate includes comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::prelude::*;
    
    #[tokio::test]
    async fn test_breakpoint_hit() {
        let debugger = create_test_debugger().await;
        
        // Set breakpoint
        let bp_id = debugger.set_breakpoint("test.lua".into(), 5).await.unwrap();
        
        // Execute script
        execute_test_script(&debugger).await;
        
        // Verify paused at breakpoint
        let state = debugger.get_debug_state().await;
        assert!(matches!(state, DebugState::Paused(PauseReason::Breakpoint { .. })));
        
        // Verify location
        let stack = debugger.get_stack_trace().await;
        assert_eq!(stack[0].line, 5);
    }
    
    #[tokio::test]
    async fn test_conditional_breakpoint() {
        let debugger = create_test_debugger().await;
        
        // Set conditional breakpoint
        debugger.set_conditional_breakpoint(
            "test.lua".into(),
            10,
            "i == 5".into()
        ).await.unwrap();
        
        // Run loop that increments i
        execute_loop_script(&debugger).await;
        
        // Should pause when i == 5
        let state = debugger.get_debug_state().await;
        assert!(matches!(state, DebugState::Paused(_)));
        
        // Verify i value
        let vars = debugger.get_local_variables().await;
        let i_var = vars.iter().find(|v| v.name == "i").unwrap();
        assert_eq!(i_var.value, "5");
    }
    
    #[tokio::test]
    async fn test_performance_overhead() {
        let debugger = create_test_debugger().await;
        
        // Run without breakpoints
        let start = Instant::now();
        execute_benchmark_script(&debugger).await;
        let no_debug_time = start.elapsed();
        
        // Add breakpoints
        for i in 1..=10 {
            debugger.set_breakpoint("bench.lua".into(), i * 10).await.unwrap();
        }
        
        // Run with breakpoints
        let start = Instant::now();
        execute_benchmark_script(&debugger).await;
        let debug_time = start.elapsed();
        
        // Verify <3% overhead
        let overhead = (debug_time.as_millis() - no_debug_time.as_millis()) as f64
            / no_debug_time.as_millis() as f64;
        assert!(overhead < 0.03, "Debug overhead {}% exceeds 3%", overhead * 100.0);
    }
}
```

## Configuration

### Debug Configuration

```toml
[debug]
# Enable interactive debugging
enabled = true

# Maximum breakpoints per session
max_breakpoints = 100

# Session timeout (seconds)
session_timeout = 3600

# Enable condition evaluation
enable_conditions = true

# Cache size for debug state
cache_size = 1000

# Performance monitoring
track_performance = true
```

## Integration with DAP

The debugger integrates with Debug Adapter Protocol through the kernel:

```rust
impl InteractiveDebugger {
    /// Convert to DAP response
    pub fn to_dap_response(&self, request: &str) -> Result<serde_json::Value> {
        match request {
            "stackTrace" => {
                let stack = self.get_stack_trace().await;
                Ok(json!({
                    "stackFrames": stack.iter().map(|f| json!({
                        "id": f.id,
                        "name": f.name,
                        "source": { "path": f.source },
                        "line": f.line,
                        "column": f.column
                    })).collect::<Vec<_>>()
                }))
            }
            "variables" => {
                let vars = self.get_local_variables().await;
                Ok(json!({
                    "variables": vars.iter().map(|v| json!({
                        "name": v.name,
                        "value": v.value,
                        "type": v.var_type,
                        "variablesReference": v.reference.unwrap_or(0)
                    })).collect::<Vec<_>>()
                }))
            }
            _ => Err(anyhow!("Unknown DAP request"))
        }
    }
}
```

## Related Documentation

- [llmspell-bridge](llmspell-bridge.md) - ExecutionBridge and ExecutionManager
- [llmspell-kernel](llmspell-kernel.md) - Kernel with DAP bridge
- [llmspell-repl](llmspell-repl.md) - REPL with debug commands
- [Debug Architecture](../../../technical/debug-dap-architecture.md) - Detailed design
- [DAP Bridge](../../../technical/debug-dap-architecture.md#dap-bridge) - DAP integration

---

**Version**: 0.9.0 | **Phase**: 9 | **Status**: Complete