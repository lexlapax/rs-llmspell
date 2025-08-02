# Script Engine Design Document Change Analysis

**Date**: 2025-06-26  
**Context**: Comprehensive analysis of phase-01-design-doc.md changes needed  
**Scope**: Holistic updates to align with ScriptEngineBridge architecture from implementation-phases.md and master-architecture-vision.md

---

## Executive Summary

The `phase-01-design-doc.md` requires **fundamental architectural restructuring** to align with the ScriptEngineBridge abstraction pattern mandated in the updated implementation phases. The current design shows direct Lua coupling that violates the multi-language vision and would create technical debt in Phase 5.

**Key Finding**: Tasks up to 1.1.4 have been implemented correctly, but Phase 1.2+ design must be completely restructured around the ScriptEngineBridge abstraction pattern.

---

## Critical Architecture Misalignment

### **Current Problem (Lines 172-177)**
```rust
// WRONG - Direct coupling shown in current design
pub struct ScriptRuntime {
    lua: Arc<Mutex<Lua>>,  // ❌ Violates multi-language architecture
    registry: Arc<ComponentRegistry>,
    provider_manager: Arc<ProviderManager>,
    execution_context: Arc<RwLock<ExecutionContext>>,
}
```

### **Required Architecture (Bridge Pattern)**
```rust
// CORRECT - Language-agnostic design
pub struct ScriptRuntime {
    engine: Box<dyn ScriptEngineBridge>,  // ✅ Multi-language ready
    registry: Arc<ComponentRegistry>,
    provider_manager: Arc<ProviderManager>,
    execution_context: Arc<RwLock<ExecutionContext>>,
}
```

---

## **Section 1: Script Runtime Architecture (Complete Restructure)**

### **Current Issues:**
- ScriptRuntime shows direct Lua coupling (`lua: Arc<Mutex<Lua>>`)
- No abstraction layer for multi-language support
- Factory pattern not implemented
- Engine selection not supported

### **Required Changes:**

**1.1 ScriptRuntime Struct Definition**
```rust
// Replace all instances with bridge pattern
pub struct ScriptRuntime {
    engine: Box<dyn ScriptEngineBridge>,  // Language-agnostic engine
    registry: Arc<ComponentRegistry>,
    provider_manager: Arc<ProviderManager>,
    execution_context: Arc<RwLock<ExecutionContext>>,
    config: RuntimeConfig,
}
```

**1.2 Factory Pattern Implementation**
```rust
impl ScriptRuntime {
    // Primary factory method for Lua (Phase 1.2)
    pub async fn new_with_lua(config: RuntimeConfig) -> Result<Self> {
        let engine = Box::new(LuaEngine::new(&config.lua_config)?);
        Self::new_with_engine(engine, config).await
    }
    
    // Future factory method for JavaScript (Phase 5)
    pub async fn new_with_javascript(config: RuntimeConfig) -> Result<Self> {
        let engine = Box::new(JSEngine::new(&config.js_config)?);
        Self::new_with_engine(engine, config).await
    }
    
    // Core initialization logic (language-agnostic)
    async fn new_with_engine(
        mut engine: Box<dyn ScriptEngineBridge>, 
        config: RuntimeConfig
    ) -> Result<Self> {
        let registry = Arc::new(ComponentRegistry::new());
        let provider_manager = Arc::new(ProviderManager::new(config.providers)?);
        
        // Language-agnostic API injection
        engine.inject_apis(&registry, &provider_manager)?;
        
        Ok(Self {
            engine,
            registry,
            provider_manager,
            execution_context: Arc::new(RwLock::new(ExecutionContext::new())),
            config,
        })
    }
}
```

**1.3 Core Methods Update**
```rust
impl ScriptRuntime {
    // Language-agnostic execution
    pub async fn execute_script(&self, script: &str) -> Result<ScriptOutput> {
        self.engine.execute_script(script).await
    }
    
    // Language-agnostic streaming
    pub async fn execute_script_streaming(&self, script: &str) -> Result<ScriptStream> {
        self.engine.execute_script_streaming(script).await
    }
    
    // Engine capability detection
    pub fn supports_streaming(&self) -> bool {
        self.engine.supports_streaming()
    }
    
    pub fn get_engine_name(&self) -> &'static str {
        self.engine.get_engine_name()
    }
}
```

---

## **Section 2: Directory Structure (Complete Reorganization)**

### **Current Issues:**
- Directory structure doesn't reflect multi-language design
- No separation between bridge abstraction and engine implementations
- API injection not organized by language

### **Required Changes:**

**2.1 New Directory Structure**
```
llmspell-bridge/
├── src/
│   ├── lib.rs
│   ├── runtime.rs                    # Language-agnostic ScriptRuntime
│   ├── config.rs                     # Configuration structures
│   ├── engine/                       # Language abstraction layer
│   │   ├── mod.rs
│   │   ├── bridge.rs                 # ScriptEngineBridge trait
│   │   ├── factory.rs                # Engine factory pattern
│   │   ├── types.rs                  # Common script types
│   │   └── executor.rs               # Common execution patterns
│   ├── lua/                          # Lua-specific implementation
│   │   ├── mod.rs
│   │   ├── engine.rs                 # LuaEngine: ScriptEngineBridge
│   │   ├── api/                      # Lua-specific API injection
│   │   │   ├── mod.rs
│   │   │   ├── agent.rs              # Agent.create(), agent:execute()
│   │   │   ├── tool.rs               # Tool.get(), tool:execute()
│   │   │   ├── workflow.rs           # Workflow patterns
│   │   │   └── streaming.rs          # Coroutine-based streaming
│   │   └── types.rs                  # Lua ↔ Rust type conversions
│   ├── javascript/                   # Future JS implementation (Phase 5)
│   │   ├── mod.rs
│   │   ├── engine.rs                 # JSEngine: ScriptEngineBridge
│   │   ├── api/                      # Same API structure as Lua
│   │   │   ├── mod.rs
│   │   │   ├── agent.rs              # Promise-based agents
│   │   │   ├── tool.rs
│   │   │   ├── workflow.rs
│   │   │   └── streaming.rs          # Async generator streaming
│   │   └── types.rs                  # JS ↔ Rust type conversions
│   └── python/                       # Future Python (via pyo3)
```

**2.2 Module Organization Principles**
- `engine/` contains language-agnostic abstractions
- `lua/`, `javascript/`, `python/` contain language-specific implementations
- Each language follows identical API structure
- Type conversions isolated to language-specific modules

---

## **Section 3: Factory Pattern Implementation (New Section)**

### **Add Comprehensive Factory Section:**

**3.1 Engine Factory Pattern**
```rust
// llmspell-bridge/src/engine/factory.rs
pub struct EngineFactory;

impl EngineFactory {
    pub fn create_lua_engine(config: &LuaConfig) -> Result<Box<dyn ScriptEngineBridge>> {
        Ok(Box::new(LuaEngine::new(config)?))
    }
    
    pub fn create_js_engine(config: &JSConfig) -> Result<Box<dyn ScriptEngineBridge>> {
        Ok(Box::new(JSEngine::new(config)?))
    }
    
    pub fn create_from_name(name: &str, config: &Value) -> Result<Box<dyn ScriptEngineBridge>> {
        match name {
            "lua" => Self::create_lua_engine(&LuaConfig::try_from(config)?),
            "javascript" => Self::create_js_engine(&JSConfig::try_from(config)?),
            _ => Err(LLMSpellError::UnsupportedEngine(name.to_string())),
        }
    }
}
```

**3.2 Plugin Architecture for Third-Party Engines**
```rust
// llmspell-bridge/src/engine/plugin.rs
pub trait ScriptEnginePlugin: Send + Sync {
    fn engine_name() -> &'static str;
    fn create_engine(config: Value) -> Result<Box<dyn ScriptEngineBridge>>;
    fn supported_features() -> EngineFeatures;
}

// Registry for third-party engines
pub struct EngineRegistry {
    plugins: HashMap<String, Box<dyn ScriptEnginePlugin>>,
}

impl EngineRegistry {
    pub fn register_plugin<P: ScriptEnginePlugin + 'static>(&mut self) {
        self.plugins.insert(P::engine_name().to_string(), Box::new(P));
    }
    
    pub fn create_engine(&self, name: &str, config: Value) -> Result<Box<dyn ScriptEngineBridge>> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.create_engine(config)
        } else {
            EngineFactory::create_from_name(name, &config)
        }
    }
}
```

---

## **Section 4: API Injection Architecture (Language-Agnostic Approach)**

### **Current Issues:**
- API injection shown as Lua-specific
- No abstraction for cross-language consistency
- Type conversions mixed with API logic

### **Required Changes:**

**4.1 ScriptEngineBridge Trait Definition**
```rust
// llmspell-bridge/src/engine/bridge.rs
#[async_trait]
pub trait ScriptEngineBridge: Send + Sync {
    // Core execution methods
    async fn execute_script(&self, script: &str) -> Result<ScriptOutput>;
    async fn execute_script_streaming(&self, script: &str) -> Result<ScriptStream>;
    
    // API injection (language-agnostic interface)
    fn inject_apis(&mut self, registry: &ComponentRegistry, providers: &ProviderManager) -> Result<()>;
    
    // Capability detection
    fn get_engine_name(&self) -> &'static str;
    fn supports_streaming(&self) -> bool;
    fn supports_multimodal(&self) -> bool;
    fn supported_features(&self) -> EngineFeatures;
    
    // State management
    fn get_execution_context(&self) -> Result<ExecutionContext>;
    fn set_execution_context(&mut self, context: ExecutionContext) -> Result<()>;
}
```

**4.2 Language-Agnostic API Surface**
```rust
// llmspell-bridge/src/engine/types.rs
#[derive(Debug, Clone)]
pub struct ApiSurface {
    pub agent_api: AgentApiDefinition,
    pub tool_api: ToolApiDefinition,
    pub workflow_api: WorkflowApiDefinition,
    pub streaming_api: StreamingApiDefinition,
}

impl ApiSurface {
    pub fn standard() -> Self {
        Self {
            agent_api: AgentApiDefinition::standard(),
            tool_api: ToolApiDefinition::standard(),
            workflow_api: WorkflowApiDefinition::standard(),
            streaming_api: StreamingApiDefinition::standard(),
        }
    }
}
```

**4.3 LuaEngine Implementation Example**
```rust
// llmspell-bridge/src/lua/engine.rs
pub struct LuaEngine {
    lua: Arc<Mutex<Lua>>,
    api_surface: ApiSurface,
}

#[async_trait]
impl ScriptEngineBridge for LuaEngine {
    fn inject_apis(&mut self, registry: &ComponentRegistry, providers: &ProviderManager) -> Result<()> {
        let lua = self.lua.lock().unwrap();
        
        // Use language-specific injection modules
        lua::api::agent::inject_agent_api(&lua, registry, &self.api_surface.agent_api)?;
        lua::api::tool::inject_tool_api(&lua, registry, &self.api_surface.tool_api)?;
        lua::api::workflow::inject_workflow_api(&lua, &self.api_surface.workflow_api)?;
        lua::api::streaming::inject_streaming_api(&lua, &self.api_surface.streaming_api)?;
        
        Ok(())
    }
    
    fn get_engine_name(&self) -> &'static str {
        "lua"
    }
    
    fn supports_streaming(&self) -> bool {
        true // Lua supports coroutine-based streaming
    }
}
```

---

## **Section 5: CLI Updates (Engine Selection Support)**

### **Current Issues:**
- CLI doesn't support engine selection
- No multi-language configuration
- Streaming output tied to Lua implementation

### **Required Changes:**

**5.1 CLI Command Structure**
```rust
// llmspell-cli/src/cli.rs
#[derive(Parser)]
#[command(name = "llmspell")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
        /// Script file to execute
        script: PathBuf,
        
        /// Script engine to use (lua, javascript)
        #[arg(long, default_value = "lua")]
        engine: String,
        
        /// Enable streaming output
        #[arg(long)]
        stream: bool,
        
        /// Configuration file
        #[arg(long)]
        config: Option<PathBuf>,
    },
}
```

**5.2 Engine-Agnostic Execution**
```rust
// llmspell-cli/src/commands/run.rs
pub async fn execute_run(
    script_path: PathBuf,
    engine_name: String,
    streaming: bool,
    config_path: Option<PathBuf>,
) -> Result<()> {
    // Load configuration
    let config = RuntimeConfig::load(config_path).await?;
    
    // Create runtime with specified engine
    let runtime = match engine_name.as_str() {
        "lua" => ScriptRuntime::new_with_lua(config).await?,
        "javascript" => ScriptRuntime::new_with_javascript(config).await?,
        _ => return Err(LLMSpellError::UnsupportedEngine(engine_name)),
    };
    
    // Read script
    let script_content = tokio::fs::read_to_string(script_path).await?;
    
    // Execute with appropriate output handling
    if streaming && runtime.supports_streaming() {
        execute_streaming(&runtime, &script_content).await
    } else {
        execute_batch(&runtime, &script_content).await
    }
}
```

**5.3 Streaming Output (Engine-Agnostic)**
```rust
async fn execute_streaming(runtime: &ScriptRuntime, script: &str) -> Result<()> {
    let mut stream = runtime.execute_script_streaming(script).await?;
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner());
    
    while let Some(chunk) = stream.next().await {
        match chunk? {
            AgentChunk { content: ChunkContent::Text(text), .. } => {
                pb.suspend(|| println!("{}", text));
            }
            AgentChunk { content: ChunkContent::Delta(delta), .. } => {
                pb.suspend(|| print!("{}", delta));
                io::stdout().flush()?;
            }
            _ => {
                // Handle other chunk types (media, metadata, etc.)
            }
        }
    }
    
    pb.finish_and_clear();
    Ok(())
}
```

---

## **Section 6: Testing Strategy Updates (Bridge Abstraction Tests)**

### **Current Issues:**
- Tests focus on Lua-specific implementation
- No cross-engine compatibility testing
- Bridge abstraction not tested

### **Required Changes:**

**6.1 Bridge Abstraction Test Suite**
```rust
// llmspell-bridge/tests/bridge_tests.rs
#[cfg(test)]
mod bridge_tests {
    use super::*;
    
    // Test that ScriptEngineBridge trait works consistently
    #[tokio::test]
    async fn test_bridge_consistency_lua() {
        let engine = LuaEngine::new(&LuaConfig::default()).unwrap();
        test_engine_compliance(Box::new(engine)).await;
    }
    
    #[tokio::test]
    async fn test_bridge_consistency_javascript() {
        let engine = JSEngine::new(&JSConfig::default()).unwrap();
        test_engine_compliance(Box::new(engine)).await;
    }
    
    // Generic test function for any ScriptEngineBridge implementation
    async fn test_engine_compliance(mut engine: Box<dyn ScriptEngineBridge>) {
        // Test basic execution
        let result = engine.execute_script("return 'hello'").await.unwrap();
        assert_eq!(result.output, "hello");
        
        // Test streaming capability detection
        if engine.supports_streaming() {
            let _stream = engine.execute_script_streaming("return 'hello'").await.unwrap();
        }
        
        // Test API injection
        let registry = ComponentRegistry::new();
        let providers = ProviderManager::new(vec![]).unwrap();
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Test engine metadata
        assert!(!engine.get_engine_name().is_empty());
    }
}
```

**6.2 Cross-Engine API Compatibility Tests**
```rust
// llmspell-bridge/tests/api_compatibility_tests.rs
#[cfg(test)]
mod api_compatibility_tests {
    // Test that same API calls work across engines
    #[tokio::test]
    async fn test_agent_api_cross_engine() {
        let lua_result = test_agent_api_with_engine("lua").await.unwrap();
        let js_result = test_agent_api_with_engine("javascript").await.unwrap();
        
        // Results should be equivalent (content may differ, but structure same)
        assert_eq!(lua_result.status, js_result.status);
        assert_eq!(lua_result.agent_id, js_result.agent_id);
    }
    
    async fn test_agent_api_with_engine(engine_name: &str) -> Result<AgentResult> {
        let config = RuntimeConfig::default();
        let runtime = match engine_name {
            "lua" => ScriptRuntime::new_with_lua(config).await?,
            "javascript" => ScriptRuntime::new_with_javascript(config).await?,
            _ => panic!("Unsupported engine: {}", engine_name),
        };
        
        let script = get_agent_test_script(engine_name);
        let result = runtime.execute_script(&script).await?;
        
        // Parse result into common format for comparison
        Ok(AgentResult::from_script_output(result))
    }
}
```

**6.3 Engine Implementation Compliance Tests**
```rust
// llmspell-bridge/tests/engine_compliance_tests.rs
#[cfg(test)]
mod engine_compliance_tests {
    // Test that all engine implementations follow the contract
    #[test]
    fn test_lua_engine_implements_bridge() {
        // Compile-time check that LuaEngine implements ScriptEngineBridge
        fn assert_bridge_impl<T: ScriptEngineBridge>() {}
        assert_bridge_impl::<LuaEngine>();
    }
    
    #[test]
    fn test_javascript_engine_implements_bridge() {
        fn assert_bridge_impl<T: ScriptEngineBridge>() {}
        assert_bridge_impl::<JSEngine>();
    }
    
    // Test engine factory pattern
    #[test]
    fn test_engine_factory() {
        let lua_config = serde_json::json!({"stdlib": "safe"});
        let lua_engine = EngineFactory::create_from_name("lua", &lua_config).unwrap();
        assert_eq!(lua_engine.get_engine_name(), "lua");
        
        let js_config = serde_json::json!({"strict_mode": true});
        let js_engine = EngineFactory::create_from_name("javascript", &js_config).unwrap();
        assert_eq!(js_engine.get_engine_name(), "javascript");
    }
}
```

---

## **Section 7: Success Criteria Updates (Validate Bridge Abstraction)**

### **Current Issues:**
- Success criteria focus on Lua-specific functionality
- No validation of bridge pattern implementation
- Engine selection not tested

### **Required Changes:**

**7.1 Updated Success Criteria**
```diff
**Phase 1.2 Success Criteria:**
- [ ] ScriptRuntime struct with all fields
+ [ ] ScriptEngineBridge abstraction works (not just Lua integration)
+ [ ] Engine factory pattern functional
+ [ ] Directory structure supports multi-language from day one
+ [ ] API injection is language-agnostic (ready for Phase 5)
- [ ] Can execute simple Lua scripts with Agent/Tool APIs
+ [ ] Can execute simple Lua scripts through ScriptEngineBridge abstraction
+ [ ] Runtime can switch between engines (even with only Lua implemented)
+ [ ] Third-party engine plugin interface defined
- [ ] LLM provider calling via `rig`
+ [ ] LLM provider calling via bridge-abstracted engines
```

**7.2 Bridge Pattern Validation Checklist**
```rust
// Validation test suite for bridge pattern
#[cfg(test)]
mod bridge_validation_tests {
    #[tokio::test]
    async fn validate_bridge_abstraction() {
        // ✅ ScriptRuntime uses Box<dyn ScriptEngineBridge>, not direct Lua
        let config = RuntimeConfig::default();
        let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();
        
        // ✅ Engine can be swapped without changing ScriptRuntime
        assert_eq!(runtime.get_engine_name(), "lua");
        
        // ✅ Factory pattern works
        let runtime2 = ScriptRuntime::new_with_javascript(config).await.unwrap();
        assert_eq!(runtime2.get_engine_name(), "javascript");
        
        // ✅ API surface consistent across engines
        test_api_consistency(&runtime).await;
        test_api_consistency(&runtime2).await;
    }
    
    async fn test_api_consistency(runtime: &ScriptRuntime) {
        // Same API calls should work regardless of engine
        let agent_script = runtime.get_engine_name() match {
            "lua" => "local agent = Agent.create('test'); return agent:execute('hello')",
            "javascript" => "const agent = Agent.create('test'); return agent.execute('hello');",
            _ => panic!("Unsupported engine"),
        };
        
        let result = runtime.execute_script(agent_script).await.unwrap();
        assert!(!result.output.is_empty());
    }
}
```

---

## **Section 8: Performance Validation Updates (Measure Bridge Overhead)**

### **Current Issues:**
- Performance tests don't account for bridge abstraction overhead
- No comparison between direct implementation and bridge pattern
- Memory usage not measured with abstraction layer

### **Required Changes:**

**8.1 Bridge Overhead Benchmarks**
```rust
// benches/bridge_overhead.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_bridge_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("lua_through_bridge", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RuntimeConfig::default();
            let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();
            let result = runtime.execute_script(black_box("return 2 + 2")).await.unwrap();
            black_box(result);
        });
    });
    
    c.bench_function("javascript_through_bridge", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RuntimeConfig::default();
            let runtime = ScriptRuntime::new_with_javascript(config).await.unwrap();
            let result = runtime.execute_script(black_box("2 + 2")).await.unwrap();
            black_box(result);
        });
    });
}

criterion_group!(benches, benchmark_bridge_overhead);
criterion_main!(benches);
```

**8.2 Memory Usage Validation**
```rust
// tests/memory_tests.rs
#[cfg(test)]
mod memory_tests {
    #[tokio::test]
    async fn test_bridge_memory_overhead() {
        let initial_memory = get_memory_usage();
        
        let config = RuntimeConfig::default();
        let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();
        
        let memory_after_creation = get_memory_usage();
        let creation_overhead = memory_after_creation - initial_memory;
        
        // Bridge pattern should add minimal overhead
        assert!(creation_overhead < 5_000_000); // <5MB overhead
        
        // Execute simple script
        let _result = runtime.execute_script("return 'hello'").await.unwrap();
        
        let memory_after_execution = get_memory_usage();
        let execution_overhead = memory_after_execution - memory_after_creation;
        
        // Execution should be efficient through bridge
        assert!(execution_overhead < 1_000_000); // <1MB execution overhead
    }
    
    #[tokio::test]
    async fn test_engine_switching_memory() {
        let config = RuntimeConfig::default();
        
        // Test Lua engine
        let lua_runtime = ScriptRuntime::new_with_lua(config.clone()).await.unwrap();
        let lua_memory = get_memory_usage();
        
        // Switch to JavaScript engine
        let js_runtime = ScriptRuntime::new_with_javascript(config).await.unwrap();
        let js_memory = get_memory_usage();
        
        // Memory usage should be comparable
        let memory_diff = (js_memory as i64 - lua_memory as i64).abs();
        assert!(memory_diff < 10_000_000); // <10MB difference
    }
}
```

---

## **Section 9: Error Handling Updates (Abstracted Error Types)**

### **Current Issues:**
- Error types are Lua-specific
- No abstraction for engine-agnostic error handling
- Error context doesn't include engine information

### **Required Changes:**

**9.1 Engine-Agnostic Error Types**
```rust
// llmspell-core/src/error.rs - Updated error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ScriptEngineError {
    #[error("Script execution failed in {engine}: {details}")]
    ExecutionError { engine: String, details: String },
    
    #[error("Script syntax error in {engine}: {message} at line {line}")]
    SyntaxError { engine: String, message: String, line: u32 },
    
    #[error("API injection failed in {engine}: {api_name}")]
    ApiInjectionError { engine: String, api_name: String },
    
    #[error("Engine {engine} does not support feature: {feature}")]
    UnsupportedFeature { engine: String, feature: String },
    
    #[error("Type conversion failed in {engine}: {details}")]
    TypeConversionError { engine: String, details: String },
    
    #[error("Engine not found: {engine_name}")]
    EngineNotFound { engine_name: String },
    
    #[error("Engine configuration invalid for {engine}: {details}")]
    ConfigurationError { engine: String, details: String },
}
```

**9.2 Error Context Propagation**
```rust
// llmspell-bridge/src/lua/engine.rs - Lua error conversion
impl From<mlua::Error> for ScriptEngineError {
    fn from(err: mlua::Error) -> Self {
        match err {
            mlua::Error::SyntaxError { message, incomplete_input: _ } => {
                ScriptEngineError::SyntaxError {
                    engine: "lua".to_string(),
                    message,
                    line: 0, // Extract line number if available
                }
            }
            mlua::Error::RuntimeError(msg) => {
                ScriptEngineError::ExecutionError {
                    engine: "lua".to_string(),
                    details: msg,
                }
            }
            _ => {
                ScriptEngineError::ExecutionError {
                    engine: "lua".to_string(),
                    details: err.to_string(),
                }
            }
        }
    }
}
```

**9.3 Error Recovery Strategies**
```rust
// llmspell-bridge/src/engine/executor.rs
impl ScriptRuntime {
    pub async fn execute_script_with_fallback(&self, script: &str) -> Result<ScriptOutput> {
        match self.execute_script(script).await {
            Ok(output) => Ok(output),
            Err(ScriptEngineError::UnsupportedFeature { feature, .. }) => {
                tracing::warn!("Feature '{}' not supported by {}, using fallback", 
                    feature, self.get_engine_name());
                self.execute_with_feature_fallback(script, &feature).await
            }
            Err(ScriptEngineError::SyntaxError { line, message, .. }) => {
                tracing::error!("Syntax error at line {}: {}", line, message);
                Err(ScriptEngineError::SyntaxError {
                    engine: self.get_engine_name().to_string(),
                    message: format!("Syntax error at line {}: {}", line, message),
                    line,
                })
            }
            Err(e) => Err(e),
        }
    }
}
```

---

## **Section 10: Configuration Updates (Engine Selection Config)**

### **Current Issues:**
- Configuration tied to Lua implementation
- No engine selection configuration
- Configuration doesn't support multi-engine setups

### **Required Changes:**

**10.1 Multi-Engine Configuration Structure**
```rust
// llmspell-bridge/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Default script engine to use
    pub default_engine: String,
    
    /// Engine-specific configurations
    pub engines: EngineConfigs,
    
    /// Provider configurations
    pub providers: ProviderConfigs,
    
    /// Global runtime settings
    pub runtime: GlobalRuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfigs {
    pub lua: LuaConfig,
    pub javascript: JSConfig,
    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaConfig {
    /// Lua standard library access level
    pub stdlib: StdlibLevel,
    /// Maximum memory usage in bytes
    pub max_memory: Option<usize>,
    /// Enable debug features
    pub debug: bool,
    /// Custom package paths
    pub package_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSConfig {
    /// Enable strict mode
    pub strict_mode: bool,
    /// Maximum heap size in bytes
    pub max_heap_size: Option<usize>,
    /// Enable console API
    pub enable_console: bool,
    /// Module resolution strategy
    pub module_resolution: ModuleResolution,
}
```

**10.2 Configuration Loading and Engine Selection**
```rust
impl RuntimeConfig {
    pub async fn load(config_path: Option<PathBuf>) -> Result<Self> {
        let config = if let Some(path) = config_path {
            Self::load_from_file(path).await?
        } else {
            Self::discover_and_load().await?
        };
        
        // Validate engine configurations
        config.validate()?;
        
        Ok(config)
    }
    
    pub fn get_engine_config(&self, engine_name: &str) -> Result<Value> {
        match engine_name {
            "lua" => Ok(serde_json::to_value(&self.engines.lua)?),
            "javascript" => Ok(serde_json::to_value(&self.engines.javascript)?),
            custom => {
                self.engines.custom.get(custom)
                    .cloned()
                    .ok_or_else(|| LLMSpellError::EngineNotFound { 
                        engine_name: custom.to_string() 
                    })
            }
        }
    }
    
    pub fn supports_engine(&self, engine_name: &str) -> bool {
        match engine_name {
            "lua" | "javascript" => true,
            custom => self.engines.custom.contains_key(custom),
        }
    }
}
```

**10.3 Example Configuration File**
```toml
# llmspell.toml
default_engine = "lua"

[engines.lua]
stdlib = "safe"
max_memory = 50_000_000  # 50MB
debug = false
package_paths = ["./lua_modules"]

[engines.javascript]
strict_mode = true
max_heap_size = 100_000_000  # 100MB
enable_console = true
module_resolution = "node"

[providers]
default = "openai"

[providers.openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"
max_tokens = 2048

[runtime]
max_concurrent_scripts = 10
script_timeout_seconds = 300
enable_streaming = true
```

---

## **Implementation Priority and Critical Path**

### **Phase 1.2 Implementation Order (CRITICAL)**
1. **Day 1**: Implement ScriptEngineBridge trait and factory pattern
2. **Day 2**: Create LuaEngine as first bridge implementation  
3. **Day 3**: Update ScriptRuntime to use bridge pattern
4. **Day 4**: Implement language-agnostic API injection
5. **Day 5**: Add CLI engine selection and configuration support

### **Validation Requirements**
- [ ] ScriptRuntime uses Box<dyn ScriptEngineBridge> (NO direct Lua coupling)
- [ ] Factory pattern works for engine creation
- [ ] Directory structure supports multi-language from day one
- [ ] API injection is language-agnostic
- [ ] Engine selection works through CLI
- [ ] Configuration supports multiple engines
- [ ] Bridge abstraction tested comprehensively

---

## **Architectural Consistency Check**

This design change analysis ensures consistency with:
- ✅ **master-architecture-vision.md**: Bridge-first design principles
- ✅ **implementation-phases.md**: Updated Phase 1.2 requirements
- ✅ **SCRIPT-ENGINE-ANALYSIS.md**: Abstraction layer recommendations
- ✅ **SCRIPT-ENGINE-ARCH-CHANGE.md**: Architecture document fixes
- ✅ **SCRIPT-ENGINE-IMPL-PHASE-CHANGE.md**: Implementation phase corrections

---

## **Summary**

The phase-01-design-doc.md requires **comprehensive restructuring** around the ScriptEngineBridge abstraction pattern. The key insight is that **Phase 1.2 must implement proper abstraction from day one**, not defer it to Phase 5.

**Critical Changes:**
1. **Complete ScriptRuntime restructure** - Remove direct Lua coupling
2. **Directory reorganization** - Support multi-language from start
3. **Factory pattern implementation** - Enable engine selection
4. **Language-agnostic API injection** - Prepare for Phase 5
5. **CLI engine selection** - Support runtime engine switching
6. **Bridge abstraction testing** - Validate architecture early
7. **Performance monitoring** - Ensure bridge overhead acceptable
8. **Error handling abstraction** - Engine-agnostic error types
9. **Multi-engine configuration** - Support engine selection

This transforms Phase 1.2 from "implement Lua integration" to "implement proper multi-language foundation with Lua as first engine" - a **fundamental architectural shift** that prevents technical debt in Phase 5.