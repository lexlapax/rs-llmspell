# llmspell-repl

**REPL session management with kernel integration** ⭐ **NEW Phase 9**

[← Back to Rust API](README.md) | [Crates.io](https://crates.io/crates/llmspell-repl) | [API Docs](https://docs.rs/llmspell-repl)

---

## Overview

The `llmspell-repl` crate provides core REPL (Read-Eval-Print Loop) functionality, including command parsing, kernel communication, session state management, and debug command interfaces. This crate contains all REPL business logic while the CLI layer (`llmspell-cli`) provides only terminal I/O.

**Key Features:**
- **Kernel Connection**: Seamless integration with `llmspell-kernel`
- **Debug Commands**: Interactive debugging with `.break`, `.step`, `.locals`, `.stack`
- **Session Management**: Maintain execution state and history
- **Performance Monitoring**: Track execution times and workload classification
- **Command System**: Extensible command handling architecture
- **Variable Tracking**: Inspect local, global, and upvalue variables
- **Breakpoint Management**: Set and manage breakpoints interactively

## Architecture

### Core Components

```rust
pub struct ReplSession {
    kernel: Box<dyn KernelConnection>,
    config: ReplConfig,
    execution_count: u32,
    command_history: Vec<String>,
    variables: HashMap<String, Value>,
}

pub struct ReplConfig {
    pub enable_performance_monitoring: bool,
    pub enable_debug_commands: bool,
}
```

### KernelConnection Trait

The REPL interacts with kernels through the `KernelConnection` trait:

```rust
#[async_trait]
pub trait KernelConnection: Send + Sync {
    /// Connect to kernel or start new one
    async fn connect_or_start(&mut self) -> Result<()>;
    
    /// Execute code on kernel
    async fn execute(&mut self, code: &str) -> Result<String>;
    
    /// Send debug command (DAP protocol)
    async fn send_debug_command(&mut self, command: Value) -> Result<Value>;
    
    /// Disconnect from kernel
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check connection status
    fn is_connected(&self) -> bool;
    
    /// Classify workload for performance monitoring
    fn classify_workload(&self, operation: &str) -> WorkloadType;
    
    /// Get execution manager for debug operations
    fn execution_manager(&self) -> Option<&dyn std::any::Any>;
}
```

## REPL Commands

### Basic Commands

| Command | Description | Example |
|---------|-------------|---------|
| `.help` | Show help text | `.help` |
| `.exit` / `.quit` | Exit REPL | `.exit` |
| `.vars` | Show current variables | `.vars` |
| `.clear` | Clear screen/variables | `.clear` |
| `.history` | Show command history | `.history` |
| `.info` | Show session information | `.info` |

### Debug Commands

| Command | Description | Example |
|---------|-------------|---------|
| `.break` | Set breakpoint | `.break script.lua 10` |
| `.step` | Step into next line | `.step` |
| `.continue` | Continue execution | `.continue` |
| `.locals` | Show local variables | `.locals` |
| `.globals` | Show global variables | `.globals` |
| `.upvalues` | Show upvalues | `.upvalues` |
| `.stack` | Show call stack | `.stack` |
| `.watch` | Add watch expression | `.watch x > 5` |

## Usage Examples

### Basic REPL Session

```rust
use llmspell_repl::{ReplSession, ReplConfig, KernelConnection};
use llmspell_kernel::JupyterKernel;

#[tokio::main]
async fn main() -> Result<()> {
    // Create kernel connection
    let kernel = JupyterKernel::spawn_embedded().await?;
    let connection = Box::new(kernel) as Box<dyn KernelConnection>;
    
    // Create REPL session
    let config = ReplConfig::default();
    let mut session = ReplSession::new(connection, config).await?;
    
    // Interactive loop
    loop {
        print!("> ");
        let input = read_line()?;
        
        match session.handle_input(&input).await? {
            ReplResponse::Exit => break,
            ReplResponse::ExecutionResult { output, .. } => {
                println!("{}", output);
            }
            ReplResponse::Error(msg) => {
                eprintln!("Error: {}", msg);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### Debug Session Example

```rust
use llmspell_repl::{ReplSession, ReplConfig};

async fn debug_session_example(session: &mut ReplSession) -> Result<()> {
    // Set breakpoint
    let response = session.handle_input(".break main.lua 15").await?;
    println!("{:?}", response);
    
    // Execute code that hits breakpoint
    session.handle_input(r#"
        function test()
            local x = 10
            print(x)  -- Line 15: breakpoint here
            return x * 2
        end
        test()
    "#).await?;
    
    // When paused at breakpoint, inspect locals
    let locals = session.handle_input(".locals").await?;
    println!("Local variables: {:?}", locals);
    
    // Step to next line
    session.handle_input(".step").await?;
    
    // Continue execution
    session.handle_input(".continue").await?;
    
    Ok(())
}
```

### Custom KernelConnection Implementation

```rust
use llmspell_repl::{KernelConnection, WorkloadType};
use async_trait::async_trait;

pub struct CustomKernel {
    // Custom kernel implementation
}

#[async_trait]
impl KernelConnection for CustomKernel {
    async fn connect_or_start(&mut self) -> Result<()> {
        // Connect to custom kernel
        Ok(())
    }
    
    async fn execute(&mut self, code: &str) -> Result<String> {
        // Execute on custom kernel
        Ok(format!("Executed: {}", code))
    }
    
    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        // Handle debug commands
        Ok(json!({
            "success": true,
            "body": {}
        }))
    }
    
    fn classify_workload(&self, operation: &str) -> WorkloadType {
        match operation {
            "execute_line" => WorkloadType::Micro,
            "execute_block" => WorkloadType::Light,
            _ => WorkloadType::Medium,
        }
    }
    
    // ... other methods
}
```

## Response Types

The REPL returns different response types based on the operation:

```rust
pub enum ReplResponse {
    /// Empty input
    Empty,
    
    /// Exit REPL
    Exit,
    
    /// Execution result
    ExecutionResult {
        output: String,
        execution_count: u32,
    },
    
    /// Debug command response
    DebugResponse(Value),
    
    /// Help text
    Help(String),
    
    /// Information message
    Info(String),
    
    /// Error message
    Error(String),
    
    /// Performance warning
    PerformanceWarning {
        expected: Duration,
        actual: Duration,
        workload: WorkloadType,
    },
}
```

## Performance Monitoring

The REPL includes built-in performance monitoring:

```rust
pub enum WorkloadType {
    Micro,  // <10ms expected
    Light,  // <100ms expected
    Medium, // <1s expected
    Heavy,  // >1s expected
}

impl ReplSession {
    fn check_performance(workload: WorkloadType, actual: Duration) {
        let threshold = match workload {
            WorkloadType::Micro => Duration::from_millis(10),
            WorkloadType::Light => Duration::from_millis(100),
            WorkloadType::Medium => Duration::from_secs(1),
            WorkloadType::Heavy => Duration::from_secs(10),
        };
        
        if actual > threshold * 2 {
            eprintln!(
                "⚠️  Performance warning: {:?} operation took {:?} (expected <{:?})",
                workload, actual, threshold
            );
        }
    }
}
```

## Debug Integration

### Setting Breakpoints

```rust
impl ReplSession {
    async fn handle_breakpoint_command(&mut self, parts: &[&str]) -> Result<ReplResponse> {
        let file = parts[1];
        let line: u32 = parts[2].parse()?;
        
        let request = json!({
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    "name": file,
                    "path": file
                },
                "lines": [line]
            }
        });
        
        let response = self.kernel.send_debug_command(request).await?;
        Ok(ReplResponse::DebugResponse(response))
    }
}
```

### Variable Inspection

```rust
impl ReplSession {
    async fn handle_locals_command(&mut self) -> Result<ReplResponse> {
        let dap_request = json!({
            "command": "variables",
            "arguments": {
                "variablesReference": 1000,  // 1000 = locals
            }
        });
        
        let response = self.kernel.send_debug_command(dap_request).await?;
        
        // Format variables for display
        let variables = response["body"]["variables"].as_array()?;
        let mut output = String::from("Local variables:\n");
        
        for var in variables {
            let name = var["name"].as_str().unwrap_or("?");
            let value = var["value"].as_str().unwrap_or("?");
            let var_type = var["type"].as_str().unwrap_or("unknown");
            writeln!(output, "  {} = {} ({})", name, value, var_type)?;
        }
        
        Ok(ReplResponse::Info(output))
    }
}
```

### Stack Trace

```rust
impl ReplSession {
    async fn handle_stack_command(&mut self) -> Result<ReplResponse> {
        let dap_request = json!({
            "command": "stackTrace",
            "arguments": {
                "threadId": 1,
                "startFrame": 0,
                "levels": 20
            }
        });
        
        let response = self.kernel.send_debug_command(dap_request).await?;
        
        // Format stack frames
        let frames = response["body"]["stackFrames"].as_array()?;
        let mut output = String::from("Call stack:\n");
        
        for (i, frame) in frames.iter().enumerate() {
            let name = frame["name"].as_str().unwrap_or("?");
            let source = frame["source"]["name"].as_str().unwrap_or("?");
            let line = frame["line"].as_u64().unwrap_or(0);
            writeln!(output, "  #{}: {} at {}:{}", i, name, source, line)?;
        }
        
        Ok(ReplResponse::Info(output))
    }
}
```

## Session State Management

The REPL maintains session state including:

```rust
impl ReplSession {
    /// Save session state
    pub async fn save_state(&self) -> Result<()> {
        let state = SessionState {
            execution_count: self.execution_count,
            command_history: self.command_history.clone(),
            variables: self.variables.clone(),
        };
        
        // Persist through kernel's state manager
        self.kernel.save_session_state(state).await?;
        Ok(())
    }
    
    /// Restore session state
    pub async fn restore_state(&mut self) -> Result<()> {
        if let Some(state) = self.kernel.load_session_state().await? {
            self.execution_count = state.execution_count;
            self.command_history = state.command_history;
            self.variables = state.variables;
        }
        Ok(())
    }
}
```

## Command History

The REPL maintains command history with navigation:

```rust
impl ReplSession {
    /// Add command to history
    fn add_to_history(&mut self, command: &str) {
        // Skip duplicates and empty commands
        if !command.is_empty() && 
           self.command_history.last() != Some(&command.to_string()) {
            self.command_history.push(command.to_string());
            
            // Limit history size
            if self.command_history.len() > 1000 {
                self.command_history.remove(0);
            }
        }
    }
    
    /// Get previous command
    pub fn previous_command(&self, index: usize) -> Option<&String> {
        self.command_history.iter().rev().nth(index)
    }
    
    /// Search history
    pub fn search_history(&self, query: &str) -> Vec<&String> {
        self.command_history
            .iter()
            .filter(|cmd| cmd.contains(query))
            .collect()
    }
}
```

## Configuration

### REPL Configuration

```rust
pub struct ReplConfig {
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    
    /// Enable debug commands
    pub enable_debug_commands: bool,
    
    /// Maximum history size
    pub max_history_size: usize,
    
    /// Auto-save interval (seconds)
    pub auto_save_interval: u64,
    
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_debug_commands: true,
            max_history_size: 1000,
            auto_save_interval: 300, // 5 minutes
            performance_thresholds: PerformanceThresholds::default(),
        }
    }
}
```

### Performance Thresholds

```rust
pub struct PerformanceThresholds {
    pub micro_ms: u64,   // Default: 10ms
    pub light_ms: u64,   // Default: 100ms
    pub medium_ms: u64,  // Default: 1000ms
    pub heavy_ms: u64,   // Default: 10000ms
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
    async fn test_repl_session() {
        let kernel = MockKernel::new();
        let session = ReplSession::new(
            Box::new(kernel),
            ReplConfig::default()
        ).await.unwrap();
        
        // Test code execution
        let response = session.handle_input("print('test')").await.unwrap();
        assert!(matches!(response, ReplResponse::ExecutionResult { .. }));
        
        // Test command
        let response = session.handle_input(".help").await.unwrap();
        assert!(matches!(response, ReplResponse::Help(_)));
    }
    
    #[tokio::test]
    async fn test_debug_commands() {
        let mut session = create_debug_session().await;
        
        // Set breakpoint
        let response = session.handle_input(".break test.lua 10").await.unwrap();
        assert!(matches!(response, ReplResponse::DebugResponse(_)));
        
        // Check locals
        let response = session.handle_input(".locals").await.unwrap();
        assert!(matches!(response, ReplResponse::Info(_)));
    }
    
    #[tokio::test]
    async fn test_performance_monitoring() {
        let mut session = create_session_with_monitoring().await;
        
        // Execute heavy workload
        session.kernel.set_delay(Duration::from_secs(2));
        let response = session.handle_input("heavy_computation()").await.unwrap();
        
        // Should get performance warning
        assert!(matches!(
            response,
            ReplResponse::PerformanceWarning { .. }
        ));
    }
}
```

## Integration with CLI

The `llmspell-cli` crate provides terminal I/O for the REPL:

```rust
// In llmspell-cli
use llmspell_repl::{ReplSession, ReplConfig};
use rustyline::Editor;

pub async fn run_repl(kernel: Box<dyn KernelConnection>) -> Result<()> {
    let mut session = ReplSession::new(kernel, ReplConfig::default()).await?;
    let mut rl = Editor::<()>::new()?;
    
    loop {
        let readline = rl.readline("llmspell> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                match session.handle_input(&line).await? {
                    ReplResponse::Exit => break,
                    response => display_response(response),
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        }
    }
    
    Ok(())
}
```

## Error Handling

The REPL provides comprehensive error handling:

```rust
impl ReplSession {
    /// Handle errors gracefully
    async fn handle_error(&mut self, error: anyhow::Error) -> ReplResponse {
        // Log error for debugging
        tracing::error!("REPL error: {:?}", error);
        
        // Provide user-friendly message
        let message = match error.downcast_ref::<KernelError>() {
            Some(KernelError::NotConnected) => {
                "Kernel not connected. Use .connect to reconnect.".to_string()
            }
            Some(KernelError::ExecutionTimeout) => {
                "Execution timed out. Use .interrupt to stop.".to_string()
            }
            _ => format!("Error: {}", error),
        };
        
        ReplResponse::Error(message)
    }
}
```

## Future Enhancements

Planned features for future releases:

- **Tab Completion**: Context-aware code completion
- **Syntax Highlighting**: Colored output for code and results
- **Multi-line Editing**: Better support for blocks of code
- **Remote REPL**: Connect to remote kernels
- **Jupyter Integration**: Use as Jupyter console backend
- **Custom Prompts**: Configurable prompt formatting
- **Plugin System**: Extensible command system

## Related Documentation

- [llmspell-kernel](llmspell-kernel.md) - Kernel that REPL connects to
- [llmspell-debug](llmspell-debug.md) - Debug infrastructure for REPL commands
- [llmspell-cli](llmspell-cli.md) - CLI layer providing terminal I/O
- [Kernel Architecture](../../../technical/kernel-protocol-architecture.md) - Kernel design
- [Debug Architecture](../../../technical/debug-dap-architecture.md) - Debug system

---

**Version**: 0.9.0 | **Phase**: 9 | **Status**: Complete