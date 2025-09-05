#### Task 9.8.10: Complete CLI Migration to In-Process Kernel Architecture
**Priority**: CRITICAL  
**Estimated Time**: 16 hours (REVISED - this is a major architecture rewrite)
**Assignee**: Architecture Team

**Description**: Complete the architectural migration from direct ScriptRuntime usage to in-process kernel-based execution. The CLI is currently half-migrated and broken - it tries to use kernel connections but the implementations don't exist.

**ARCHITECTURAL INSIGHT**: 
```
OLD: CLI → Direct ScriptRuntime → Execute
NEW: CLI → In-Process JupyterKernel → ScriptRuntime → Execute
```

**🔍 CRITICAL DISCOVERY**:
The CLI code is **already trying to use kernel connections** but they're not implemented:
- `run.rs` calls `kernel.execute()` but it returns "not implemented"  
- `repl.rs` calls `kernel.connect_or_start()` but method doesn't exist
- All `KernelConnectionBuilder` methods missing or broken
- Test infrastructure expects methods that don't exist

This isn't just removing old protocols - it's **building a complete in-process kernel client**.

**Implementation Steps:**

**PHASE 1: Fix Compilation (Critical Blocker)**

1. **Fix KernelConnectionBuilder methods** ❌ CRITICAL:
   ```rust
   // BROKEN CODE:
   .diagnostics(DiagnosticsBridge::builder().build()) // ← METHOD DOESN'T EXIST
   .build() // ← RETURNS ERROR
   
   // IMPLEMENTATION NEEDED:
   impl KernelConnectionBuilder {
       pub fn diagnostics(mut self, diag: DiagnosticsBridge) -> Self { ... }
       pub async fn build(self) -> Result<Box<dyn KernelConnectionTrait>> {
           // Create in-process JupyterKernel instance
           let kernel_id = uuid::Uuid::new_v4().to_string();
           let config = self.config.unwrap_or_default();
           
           // Create actual kernel, not stub
           let kernel = JupyterKernel::from_config(kernel_id, config).await?;
           Ok(Box::new(InProcessKernelConnection::new(kernel)))
       }
   }
   ```

2. **Implement missing KernelConnectionTrait methods** ❌ CRITICAL:
   ```rust
   // BROKEN CODE:
   kernel.connect_or_start().await?; // ← METHOD DOESN'T EXIST
   kernel.is_connected() // ← METHOD DOESN'T EXIST  
   kernel.disconnect().await? // ← METHOD DOESN'T EXIST
   
   // TRAIT NEEDS THESE METHODS:
   #[async_trait]
   pub trait KernelConnectionTrait: Send + Sync {
       async fn connect_or_start(&mut self) -> Result<()>;
       fn is_connected(&self) -> bool;
       async fn disconnect(&mut self) -> Result<()>;
       // ... existing methods
   }
   ```

3. **Fix trait bound issues** ❌ CRITICAL:
   ```rust
   // BROKEN CODE:
   .circuit_breaker(Box::new(ExponentialBackoffBreaker::default())) 
   // ← ExponentialBackoffBreaker doesn't implement CliCircuitBreakerTrait
   
   // IMPLEMENTATION NEEDED:
   impl CliCircuitBreakerTrait for ExponentialBackoffBreaker { ... }
   ```

4. **Create missing test infrastructure** ❌ CRITICAL:
   ```rust
   // BROKEN CODE:
   use crate::kernel::{NullKernelConnection, NullKernelDiscovery}; // ← DOESN'T EXIST
   
   // IMPLEMENTATION NEEDED:
   pub struct NullKernelConnection { ... }
   impl KernelConnectionTrait for NullKernelConnection { ... }
   ```

**PHASE 2: In-Process Kernel Creation**

5. **Implement InProcessKernelConnection** ❌ NEW:
   ```rust
   pub struct InProcessKernelConnection {
       kernel: JupyterKernel,
       connected: bool,
   }
   
   impl KernelConnectionTrait for InProcessKernelConnection {
       async fn execute(&mut self, code: &str) -> Result<String> {
           // Direct call to in-process kernel
           let execute_request = ExecuteRequest {
               code: code.to_string(),
               silent: false,
               store_history: true,
               user_expressions: None,
               allow_stdin: false,
               stop_on_error: false,
           };
           
           let reply = self.kernel.handle_execute_request(execute_request).await?;
           Ok(format!("{:?}", reply)) // TODO: Proper formatting
       }
       
       async fn connect_or_start(&mut self) -> Result<()> {
           // For in-process kernel, just mark as connected
           self.connected = true;
           Ok(())
       }
       
       fn is_connected(&self) -> bool {
           self.connected
       }
       
       // ... other methods
   }
   ```

6. **Update kernel creation in run.rs** ❌ FIXING:
   ```rust
   // CURRENT BROKEN CODE:
   let mut kernel = super::create_kernel_connection(runtime_config).await?; // ← RETURNS ERROR
   let result = kernel.execute(&script_content).await?; // ← RETURNS "NOT IMPLEMENTED"
   
   // WORKING IMPLEMENTATION:
   pub async fn create_kernel_connection(config: LLMSpellConfig) -> Result<Box<dyn KernelConnectionTrait>> {
       let mut builder = KernelConnectionBuilder::new()
           .config(config)
           .discovery(Box::new(CliKernelDiscovery::new()));
           
       let mut connection = builder.build().await?;
       connection.connect_or_start().await?;
       Ok(connection)
   }
   ```

**PHASE 3: REPL Integration**

7. **Fix REPL kernel integration** ❌ BROKEN:
   ```rust
   // CURRENT BROKEN CODE in repl.rs:
   let mut kernel = KernelConnectionBuilder::new()
       .diagnostics(DiagnosticsBridge::builder().build()) // ← BROKEN
       .build(); // ← BROKEN
   
   // WORKING IMPLEMENTATION:
   let mut kernel = KernelConnectionBuilder::new()
       .config(runtime_config.clone())
       .build().await?;
       
   kernel.connect_or_start().await?;
   
   let mut cli_client = CLIReplInterface::builder()
       .kernel(kernel)
       .config(runtime_config)
       .history_file(history_file)
       .build()?;
   ```

8. **Implement REPL session management** ❌ NEW:
   ```rust
   // Need to maintain REPL state through kernel
   impl CLIReplInterface {
       pub async fn run_interactive_loop(&mut self) -> Result<()> {
           loop {
               let input = self.read_input().await?;
               match input.trim() {
                   ".exit" => break,
                   line if line.starts_with('.') => {
                       self.handle_debug_command(line).await?;
                   }
                   code => {
                       let result = self.kernel.execute(code).await?;
                       println!("{}", result);
                   }
               }
           }
           self.kernel.disconnect().await?;
           Ok(())
       }
   }
   ```

**PHASE 4: Standalone Kernel Mode**

9. **Add --kernel CLI option for standalone mode** ❌ NEW:
   ```rust
   // In llmspell-cli/src/cli.rs:
   #[derive(Parser, Debug)]
   #[command(name = "llmspell")]
   pub struct Cli {
       /// Start standalone kernel server (don't run commands)
       #[arg(long)]
       pub kernel: bool,
       
       /// Port for standalone kernel (default: 9555)
       #[arg(long, default_value = "9555")]
       pub kernel_port: u16,
       
       /// Kernel ID for standalone mode (auto-generated if not provided)
       #[arg(long)]
       pub kernel_id: Option<String>,
       
       // ... existing fields
   }
   ```

10. **Implement standalone kernel startup** ❌ NEW:
    ```rust
    // In llmspell-cli/src/commands/mod.rs:
    pub async fn start_standalone_kernel(
        port: u16,
        kernel_id: Option<String>,
        config: LLMSpellConfig,
    ) -> Result<()> {
        let kernel_id = kernel_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        println!("Starting LLMSpell kernel...");
        println!("  Kernel ID: {}", kernel_id);
        println!("  Port: {}", port);
        println!("  Press Ctrl+C to stop");
        
        // Create connection info for clients
        let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);
        
        // Start kernel in server mode
        let mut kernel = JupyterKernel::from_config_with_connection(
            kernel_id,
            Arc::new(config),
            connection_info,
        ).await?;
        
        // Serve until interrupted
        kernel.serve().await?;
        Ok(())
    }
    ```

11. **Update main CLI dispatch** ❌ MODIFY:
    ```rust
    // In llmspell-cli/src/main.rs or commands/mod.rs:
    pub async fn run_cli_commands(cli: Cli) -> Result<()> {
        // Check for standalone kernel mode FIRST
        if cli.kernel {
            return start_standalone_kernel(
                cli.kernel_port,
                cli.kernel_id,
                load_config(cli.config.as_deref()).await?,
            ).await;
        }
        
        // Normal command processing...
        match cli.command {
            Commands::Run { ... } => { ... }
            Commands::Repl { ... } => { ... }
            // ... existing commands
        }
    }
    ```

**Usage Examples:**
```bash
# Start standalone kernel (blocks until Ctrl+C)
llmspell --kernel
# Starting LLMSpell kernel...
#   Kernel ID: abc-123-def
#   Port: 9555  
#   Press Ctrl+C to stop

# Start kernel on specific port with custom ID
llmspell --kernel --kernel-port 8888 --kernel-id my-kernel

# Normal CLI usage (in-process kernel)  
llmspell run script.lua
llmspell repl

# Connect to existing standalone kernel (future feature)
llmspell run script.lua --connect-to-kernel abc-123-def
```

**ARCHITECTURAL BENEFIT**: With `--kernel` option in CLI, we can **remove the separate llmspell-kernel binary entirely**. The CLI becomes the unified entry point for all functionality.

**PHASE 5: Debug Integration**

12. **Implement debug commands through kernel** ❌ NEW:
   ```rust
   // Current debug commands return errors
   // Need to implement through kernel comm channels
   
   impl CLIReplInterface {
       async fn handle_debug_command(&mut self, command: &str) -> Result<()> {
           let debug_request = match command {
               ".break" => serde_json::json!({
                   "command": "setBreakpoints",
                   "arguments": { "source": {"name": "repl"}, "lines": [1] }
               }),
               ".step" => serde_json::json!({
                   "command": "stepIn", 
                   "arguments": { "threadId": 1 }
               }),
               // ... other debug commands
           };
           
           let response = self.kernel.send_debug_command(debug_request).await?;
           println!("Debug response: {:?}", response);
           Ok(())
       }
   }
   ```

13. **Update debug run command** ❌ BROKEN:
    ```rust
    // Current run_debug.rs just returns error
    // Need actual implementation:
    
    pub async fn execute_script_debug(
        script_content: String,
        script_path: PathBuf,
        runtime_config: LLMSpellConfig,
        args: Vec<String>,
        output_format: OutputFormat,
    ) -> Result<()> {
        // Enable debug in config
        let mut config = runtime_config;
        config.debug.enabled = true;
        
        // Create kernel connection with debug enabled  
        let mut kernel = create_kernel_connection(config).await?;
        
        // Execute with debug support
        let result = kernel.execute(&script_content).await?;
        println!("{}", format_output(&parse_kernel_result(result), output_format)?);
        
        kernel.disconnect().await?;
        Ok(())
    }
    ```

**CLEANUP PHASE: Remove Redundant Binary**

14. **Remove llmspell-kernel binary** ❌ NEW:
    ```bash
    # Since CLI now has --kernel option, remove separate binary
    rm -rf llmspell-kernel/src/bin/llmspell-kernel.rs
    
    # Update Cargo.toml to remove binary target:
    # DELETE from llmspell-kernel/Cargo.toml:
    # [[bin]]
    # name = "llmspell-kernel"  
    # path = "src/bin/llmspell-kernel.rs"
    ```

15. **Update documentation and scripts** ❌ NEW:
    ```bash
    # Update any references from llmspell-kernel to llmspell --kernel
    # Update build scripts, documentation, examples
    
    # OLD: 
    ./target/debug/llmspell-kernel --port 9555
    
    # NEW:
    ./target/debug/llmspell --kernel --kernel-port 9555
    ```

**Acceptance Criteria:**
- [ ] **Compilation**: Full workspace builds without errors
- [ ] **Run Command**: `llmspell run script.lua` executes through in-process kernel  
- [ ] **REPL Command**: `llmspell repl` starts interactive session through kernel
- [ ] **Standalone Kernel**: `llmspell --kernel` starts server mode (blocks until Ctrl+C)
- [ ] **Debug Commands**: `.break`, `.step`, `.continue` work in REPL
- [ ] **Debug Run**: `llmspell run --debug script.lua` enables debugging
- [ ] **Binary Removal**: llmspell-kernel binary removed, CLI is unified entry point
- [ ] **Error Handling**: Graceful error messages for all failure modes
- [ ] **Tests**: All CLI tests pass with new architecture
- [ ] **Performance**: No significant performance regression vs direct execution

**Definition of Done:**
All CLI functionality (run, repl, debug) works through in-process kernel with same user experience as before, but using Jupyter protocol internally.

---

#### Task 9.8.11: End-to-End CLI Functionality Verification
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Comprehensive verification that the CLI works end-to-end through the in-process kernel architecture with full functionality restored.

**Test Scenarios:**

**BASIC EXECUTION TESTS**
1. **Script Execution**: 
   ```bash
   echo 'print("hello")' > test.lua
   llmspell run test.lua
   # Should output: hello
   ```

2. **Script Arguments**:
   ```bash  
   llmspell run test.lua arg1 --flag value
   # Script should receive arguments properly
   ```

3. **Error Handling**:
   ```bash
   echo 'error("test error")' > error.lua
   llmspell run error.lua
   # Should show formatted error, not crash
   ```

**REPL FUNCTIONALITY TESTS**
4. **Basic REPL**:
   ```bash
   llmspell repl
   > print("hello")
   hello
   > x = 42
   > print(x)
   42
   > .exit
   ```

5. **REPL History**:
   ```bash
   llmspell repl
   > print("test")
   > <UP_ARROW> # Should recall previous command
   ```

6. **REPL Debug Commands**:
   ```bash
   llmspell repl  
   > .break 5
   Breakpoint set at line 5
   > .step
   Step command acknowledged  
   > .continue
   Continue command acknowledged
   ```

**DEBUG FUNCTIONALITY TESTS**  
7. **Debug Mode Execution**:
   ```bash
   echo 'for i=1,3 do print(i) end' > loop.lua
   llmspell run --debug loop.lua
   # Should show debug output/capability
   ```

8. **Interactive Debug Session**:
   ```bash
   llmspell debug script.lua
   # Should start debug session with breakpoint capability
   ```

**ERROR RECOVERY TESTS**
9. **Kernel Recovery**:
   - Simulate kernel error during execution
   - Verify CLI shows meaningful error
   - Verify subsequent commands work

10. **Connection Recovery**:
    - Test REPL session interrupted
    - Verify graceful restart capability

**PERFORMANCE TESTS**
11. **Execution Speed**:
    - Large script execution time
    - Should be comparable to direct execution
    - No significant overhead from kernel layer

12. **REPL Responsiveness**:
    - Interactive command response time  
    - Should feel instant for simple commands

**STANDALONE KERNEL TESTS**
13. **Kernel Startup**:
    ```bash
    # Start standalone kernel in background
    llmspell --kernel &
    KERNEL_PID=$!
    
    # Verify it's running
    sleep 2
    ps -p $KERNEL_PID || { echo "FAIL: Kernel not running"; exit 1; }
    
    # Clean shutdown
    kill $KERNEL_PID
    wait $KERNEL_PID
    ```

14. **Kernel with Custom Options**:
    ```bash
    llmspell --kernel --kernel-port 8888 --kernel-id test-kernel &
    # Should start on port 8888 with ID test-kernel
    ```

**INTEGRATION TESTS**
15. **Output Formatting**:
    ```bash
    llmspell run script.lua --format json
    llmspell run script.lua --format table  
    llmspell run script.lua --format plain
    # All formats should work correctly
    ```

16. **Engine Selection**:
    ```bash
    llmspell run script.lua --engine lua
    llmspell run script.js --engine js
    # Engine routing through kernel should work
    ```

17. **Configuration Loading**:
    ```bash
    llmspell run script.lua --config custom.toml
    # Custom config should be passed to kernel
    ```

**Acceptance Criteria:**
- [ ] **All 17 test scenarios pass** without manual intervention
- [ ] **Zero regression** in functionality from pre-kernel CLI  
- [ ] **Error messages** are user-friendly and actionable
- [ ] **Performance** within 10% of baseline (pre-kernel)
- [ ] **Memory usage** stable across long REPL sessions
- [ ] **Documentation** updated with new architecture notes

**Verification Script:**
```bash
#!/bin/bash
# run_cli_verification.sh

set -e
echo "=== CLI Functionality Verification ==="

# Test 1: Basic execution
echo 'print("hello world")' > test_basic.lua
OUTPUT=$(llmspell run test_basic.lua)
[[ "$OUTPUT" == "hello world" ]] || { echo "FAIL: Basic execution"; exit 1; }
echo "✅ Basic execution"

# Test 2: REPL automation  
echo -e 'print("repl test")\n.exit' | llmspell repl | grep -q "repl test" || { echo "FAIL: REPL"; exit 1; }
echo "✅ REPL functionality"

# Test 3: Debug mode
echo 'for i=1,2 do print(i) end' > test_debug.lua  
llmspell run --debug test_debug.lua >/dev/null || { echo "FAIL: Debug mode"; exit 1; }
echo "✅ Debug mode"

# Test 4: Error handling
echo 'error("test error")' > test_error.lua
llmspell run test_error.lua 2>&1 | grep -q "test error" || { echo "FAIL: Error handling"; exit 1; }
echo "✅ Error handling"

# Test 5: Output formats
for fmt in json table plain; do
    llmspell run test_basic.lua --format $fmt >/dev/null || { echo "FAIL: Format $fmt"; exit 1; }
done
echo "✅ Output formats"

# Test 6: Standalone kernel mode
llmspell --kernel --kernel-port 9999 &
KERNEL_PID=$!
sleep 2
ps -p $KERNEL_PID >/dev/null || { echo "FAIL: Standalone kernel"; exit 1; }
kill $KERNEL_PID && wait $KERNEL_PID
echo "✅ Standalone kernel mode"

# Test 7: Verify binary removal
[[ ! -f ./target/debug/llmspell-kernel ]] || { echo "FAIL: llmspell-kernel binary still exists"; exit 1; }
echo "✅ Binary removal verification"

# Cleanup
rm -f test_*.lua

echo "🎉 All CLI functionality tests passed!"
echo "CLI successfully migrated to unified in-process kernel architecture."
```

**Definition of Done:**
The CLI provides the same user experience as before the migration, but now runs entirely through the in-process kernel architecture. All functionality works reliably with proper error handling and performance characteristics.