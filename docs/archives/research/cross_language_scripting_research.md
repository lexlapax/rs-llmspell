# Cross-Language Scripting Research: Unified Value Conversion and Memory Management

## Overview

This document researches patterns for handling unified value conversion, shared memory management, and consistent error handling between Lua (mlua) and JavaScript engines (boa, rquickjs, deno_core) when embedded in Rust. Critical for rs-llmspell's scripting bridge layer to provide consistent behavior across both languages.

## 1. Common Value Conversion Patterns

### 1.1 Type Mapping Strategy

#### Basic Types Mapping
```rust
// Unified value representation
enum ScriptValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
    Function(Box<dyn ScriptFunction>),
    Bytes(Vec<u8>),
    // Special types
    Promise(Box<dyn Future<Output = Result<ScriptValue, ScriptError>>>),
    Agent(Arc<dyn Agent>),
    Tool(Arc<dyn Tool>),
}
```

#### Lua Type Conversion (mlua)
```rust
// From Lua to Rust
impl<'lua> FromLua<'lua> for ScriptValue {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            Value::Nil => Ok(ScriptValue::Nil),
            Value::Boolean(b) => Ok(ScriptValue::Boolean(b)),
            Value::Integer(i) => Ok(ScriptValue::Number(i as f64)),
            Value::Number(n) => Ok(ScriptValue::Number(n)),
            Value::String(s) => Ok(ScriptValue::String(s.to_str()?.to_owned())),
            Value::Table(t) => {
                // Check if array or object
                if t.raw_len() > 0 {
                    // Array
                    let mut array = Vec::new();
                    for i in 1..=t.raw_len() {
                        array.push(ScriptValue::from_lua(t.get(i)?, lua)?);
                    }
                    Ok(ScriptValue::Array(array))
                } else {
                    // Object
                    let mut object = HashMap::new();
                    for pair in t.pairs::<String, Value>() {
                        let (k, v) = pair?;
                        object.insert(k, ScriptValue::from_lua(v, lua)?);
                    }
                    Ok(ScriptValue::Object(object))
                }
            }
            Value::Function(f) => {
                // Wrap Lua function
                Ok(ScriptValue::Function(Box::new(LuaFunction::new(f))))
            }
            Value::UserData(ud) => {
                // Check for known types
                if ud.is::<AgentWrapper>() {
                    let agent = ud.borrow::<AgentWrapper>()?;
                    Ok(ScriptValue::Agent(agent.0.clone()))
                } else if ud.is::<ToolWrapper>() {
                    let tool = ud.borrow::<ToolWrapper>()?;
                    Ok(ScriptValue::Tool(tool.0.clone()))
                } else {
                    Err(LuaError::FromLuaConversionError { ... })
                }
            }
            _ => Err(LuaError::FromLuaConversionError { ... }),
        }
    }
}
```

#### JavaScript Type Conversion (rquickjs)
```rust
// From JavaScript to Rust
impl<'js> FromJs<'js> for ScriptValue {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        match value.type_of() {
            Type::Undefined | Type::Null => Ok(ScriptValue::Nil),
            Type::Bool => Ok(ScriptValue::Boolean(value.as_bool().unwrap())),
            Type::Number => Ok(ScriptValue::Number(value.as_number().unwrap())),
            Type::String => Ok(ScriptValue::String(value.as_string().unwrap().to_string()?)),
            Type::Array => {
                let array = value.as_array().unwrap();
                let mut vec = Vec::new();
                for i in 0..array.len() {
                    vec.push(ScriptValue::from_js(ctx, array.get(i)?)?);
                }
                Ok(ScriptValue::Array(vec))
            }
            Type::Object => {
                if let Some(promise) = value.as_promise() {
                    // Handle Promise specially
                    Ok(ScriptValue::Promise(Box::new(JsPromise::new(promise))))
                } else if value.is_instance_of::<AgentClass>() {
                    // Extract Agent
                    let agent = value.get::<_, Object>("__agent")?;
                    Ok(ScriptValue::Agent(agent.opaque::<Arc<dyn Agent>>().clone()))
                } else {
                    // Regular object
                    let obj = value.as_object().unwrap();
                    let mut map = HashMap::new();
                    for prop in obj.props::<String, Value>() {
                        let (k, v) = prop?;
                        map.insert(k, ScriptValue::from_js(ctx, v)?);
                    }
                    Ok(ScriptValue::Object(map))
                }
            }
            Type::Function => {
                Ok(ScriptValue::Function(Box::new(JsFunction::new(value))))
            }
            _ => Err(Error::new_from_js("type", "unsupported type")),
        }
    }
}
```

### 1.2 Function Conversion Patterns

#### Unified Function Trait
```rust
trait ScriptFunction: Send + Sync {
    fn call(&self, args: Vec<ScriptValue>) -> BoxFuture<'_, Result<ScriptValue, ScriptError>>;
    fn call_method(&self, this: ScriptValue, args: Vec<ScriptValue>) -> BoxFuture<'_, Result<ScriptValue, ScriptError>>;
}

// Lua function wrapper
struct LuaFunction {
    func: Arc<Mutex<RegistryKey>>,
    runtime: Arc<LuaRuntime>,
}

impl ScriptFunction for LuaFunction {
    fn call(&self, args: Vec<ScriptValue>) -> BoxFuture<'_, Result<ScriptValue, ScriptError>> {
        Box::pin(async move {
            self.runtime.with_lua(|lua| {
                let func: Function = lua.registry_value(&self.func)?;
                let lua_args: Vec<Value> = args.into_iter()
                    .map(|arg| arg.to_lua(lua))
                    .collect::<Result<_, _>>()?;
                let result: Value = func.call_async(lua_args).await?;
                ScriptValue::from_lua(result, lua)
            }).await
        })
    }
}

// JavaScript function wrapper
struct JsFunction {
    func: Persistent<Function>,
    runtime: Arc<JsRuntime>,
}

impl ScriptFunction for JsFunction {
    fn call(&self, args: Vec<ScriptValue>) -> BoxFuture<'_, Result<ScriptValue, ScriptError>> {
        Box::pin(async move {
            self.runtime.with_context(|ctx| {
                let func = self.func.clone().restore(ctx)?;
                let js_args: Vec<Value> = args.into_iter()
                    .map(|arg| arg.to_js(ctx))
                    .collect::<Result<_, _>>()?;
                let result = func.call::<_, Value>((This(ctx.globals()), js_args))?;
                ScriptValue::from_js(ctx, result)
            }).await
        })
    }
}
```

## 2. Memory Management Strategies

### 2.1 Reference Counting and Ownership

#### Shared State Pattern
```rust
// Shared state accessible from both Lua and JavaScript
struct SharedState {
    values: Arc<RwLock<HashMap<String, ScriptValue>>>,
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

// Reference wrapper for cross-language objects
struct ScriptReference {
    id: Uuid,
    kind: ReferenceKind,
    refcount: Arc<AtomicUsize>,
}

enum ReferenceKind {
    Agent(Weak<dyn Agent>),
    Tool(Weak<dyn Tool>),
    Value(Weak<RwLock<ScriptValue>>),
}

// Garbage collection coordinator
struct GcCoordinator {
    lua_gc: Arc<Mutex<LuaGcState>>,
    js_gc: Arc<Mutex<JsGcState>>,
    references: Arc<RwLock<HashMap<Uuid, ScriptReference>>>,
}

impl GcCoordinator {
    async fn collect(&self) {
        // Coordinate GC across both engines
        let mut dead_refs = Vec::new();
        
        // Check reference counts
        {
            let refs = self.references.read().await;
            for (id, reference) in refs.iter() {
                if reference.refcount.load(Ordering::Relaxed) == 0 {
                    dead_refs.push(*id);
                }
            }
        }
        
        // Remove dead references
        if !dead_refs.is_empty() {
            let mut refs = self.references.write().await;
            for id in dead_refs {
                refs.remove(&id);
            }
        }
        
        // Trigger engine-specific GC
        self.lua_gc.lock().await.step();
        self.js_gc.lock().await.step();
    }
}
```

### 2.2 Cross-Language Object Lifecycle

#### Object Wrapper Pattern
```rust
// Wrapper for objects that can cross language boundaries
#[derive(Clone)]
struct CrossLangObject {
    inner: Arc<dyn Any + Send + Sync>,
    vtable: Arc<ObjectVTable>,
    gc_handle: Arc<GcHandle>,
}

struct ObjectVTable {
    get_property: fn(&dyn Any, &str) -> Result<ScriptValue, ScriptError>,
    set_property: fn(&mut dyn Any, &str, ScriptValue) -> Result<(), ScriptError>,
    call_method: fn(&dyn Any, &str, Vec<ScriptValue>) -> BoxFuture<'static, Result<ScriptValue, ScriptError>>,
    to_string: fn(&dyn Any) -> String,
}

// GC handle for tracking object lifecycle
struct GcHandle {
    id: Uuid,
    coordinator: Weak<GcCoordinator>,
    refcount: Arc<AtomicUsize>,
}

impl Drop for GcHandle {
    fn drop(&mut self) {
        // Decrement reference count
        self.refcount.fetch_sub(1, Ordering::Release);
        
        // Schedule GC if needed
        if let Some(coordinator) = self.coordinator.upgrade() {
            tokio::spawn(async move {
                coordinator.collect().await;
            });
        }
    }
}

// Lua UserData implementation
impl UserData for CrossLangObject {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Index, |_, this, key: String| {
            let value = (this.vtable.get_property)(this.inner.as_ref(), &key)?;
            Ok(value)
        });
        
        methods.add_meta_method(MetaMethod::NewIndex, |_, this, (key, value): (String, ScriptValue)| {
            (this.vtable.set_property)(this.inner.as_mut(), &key, value)?;
            Ok(())
        });
    }
}
```

## 3. Error Handling Patterns

### 3.1 Unified Error Type

```rust
#[derive(Debug, Clone)]
enum ScriptError {
    // Language-specific errors
    LuaError(String),
    JavaScriptError(String),
    
    // Common errors
    TypeError { expected: String, got: String },
    RuntimeError(String),
    MemoryError(String),
    AsyncError(String),
    
    // Agent/Tool errors
    AgentError(String),
    ToolError(String),
    
    // Conversion errors
    ConversionError { from: String, to: String, reason: String },
}

// Error conversion traits
impl From<mlua::Error> for ScriptError {
    fn from(err: mlua::Error) -> Self {
        ScriptError::LuaError(err.to_string())
    }
}

impl From<rquickjs::Error> for ScriptError {
    fn from(err: rquickjs::Error) -> Self {
        ScriptError::JavaScriptError(err.to_string())
    }
}

// Error propagation helpers
trait ScriptResult<T> {
    fn map_script_err(self) -> Result<T, ScriptError>;
}

impl<T> ScriptResult<T> for mlua::Result<T> {
    fn map_script_err(self) -> Result<T, ScriptError> {
        self.map_err(ScriptError::from)
    }
}
```

### 3.2 Cross-Language Error Propagation

```rust
// Error context preservation
struct ErrorContext {
    language: ScriptLanguage,
    stack_trace: Vec<StackFrame>,
    cause_chain: Vec<ScriptError>,
}

struct StackFrame {
    function: String,
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
}

// Enhanced error with context
struct ContextualError {
    error: ScriptError,
    context: ErrorContext,
}

// Error propagation across language boundaries
impl CrossLangObject {
    async fn call_with_error_handling(
        &self,
        method: &str,
        args: Vec<ScriptValue>,
    ) -> Result<ScriptValue, ContextualError> {
        match (self.vtable.call_method)(self.inner.as_ref(), method, args).await {
            Ok(value) => Ok(value),
            Err(err) => {
                // Capture context
                let context = self.capture_error_context(&err).await;
                Err(ContextualError {
                    error: err,
                    context,
                })
            }
        }
    }
    
    async fn capture_error_context(&self, error: &ScriptError) -> ErrorContext {
        // Implementation would capture stack traces from both engines
        ErrorContext {
            language: self.detect_language(),
            stack_trace: self.get_stack_trace().await,
            cause_chain: self.get_cause_chain(error),
        }
    }
}
```

## 4. Practical Implementation Strategies

### 4.1 Unified Runtime Manager

```rust
// Manages both Lua and JavaScript runtimes
struct ScriptRuntimeManager {
    lua_runtime: Arc<LuaRuntime>,
    js_runtime: Arc<JsRuntime>,
    shared_state: Arc<SharedState>,
    gc_coordinator: Arc<GcCoordinator>,
    value_converter: Arc<ValueConverter>,
}

impl ScriptRuntimeManager {
    async fn new() -> Result<Self, ScriptError> {
        let shared_state = Arc::new(SharedState::default());
        let gc_coordinator = Arc::new(GcCoordinator::new());
        
        let lua_runtime = Arc::new(LuaRuntime::new(
            shared_state.clone(),
            gc_coordinator.clone(),
        ).await?);
        
        let js_runtime = Arc::new(JsRuntime::new(
            shared_state.clone(),
            gc_coordinator.clone(),
        ).await?);
        
        let value_converter = Arc::new(ValueConverter::new(
            lua_runtime.clone(),
            js_runtime.clone(),
        ));
        
        Ok(Self {
            lua_runtime,
            js_runtime,
            shared_state,
            gc_coordinator,
            value_converter,
        })
    }
    
    // Execute script in appropriate runtime
    async fn execute(
        &self,
        language: ScriptLanguage,
        code: &str,
        context: HashMap<String, ScriptValue>,
    ) -> Result<ScriptValue, ContextualError> {
        // Set up shared context
        for (key, value) in context {
            self.shared_state.values.write().await.insert(key, value);
        }
        
        match language {
            ScriptLanguage::Lua => {
                self.lua_runtime.execute(code).await
            }
            ScriptLanguage::JavaScript => {
                self.js_runtime.execute(code).await
            }
        }
    }
}
```

### 4.2 Value Converter Implementation

```rust
struct ValueConverter {
    lua_runtime: Arc<LuaRuntime>,
    js_runtime: Arc<JsRuntime>,
    conversion_cache: Arc<RwLock<HashMap<TypeId, ConversionStrategy>>>,
}

enum ConversionStrategy {
    Direct,
    Serialize,
    Reference,
    Custom(Box<dyn Fn(ScriptValue) -> Result<ScriptValue, ScriptError> + Send + Sync>),
}

impl ValueConverter {
    // Convert value between languages
    async fn convert(
        &self,
        value: ScriptValue,
        from: ScriptLanguage,
        to: ScriptLanguage,
    ) -> Result<ScriptValue, ScriptError> {
        if from == to {
            return Ok(value);
        }
        
        match (&value, from, to) {
            // Simple types - direct conversion
            (ScriptValue::Nil, _, _) |
            (ScriptValue::Boolean(_), _, _) |
            (ScriptValue::Number(_), _, _) |
            (ScriptValue::String(_), _, _) => Ok(value),
            
            // Arrays - recursive conversion
            (ScriptValue::Array(arr), _, _) => {
                let mut converted = Vec::new();
                for item in arr {
                    converted.push(self.convert(item.clone(), from, to).await?);
                }
                Ok(ScriptValue::Array(converted))
            }
            
            // Objects - recursive conversion with cycle detection
            (ScriptValue::Object(obj), _, _) => {
                let mut converted = HashMap::new();
                for (key, val) in obj {
                    converted.insert(
                        key.clone(),
                        self.convert(val.clone(), from, to).await?,
                    );
                }
                Ok(ScriptValue::Object(converted))
            }
            
            // Functions - create proxy
            (ScriptValue::Function(f), _, _) => {
                self.create_function_proxy(f.clone(), from, to).await
            }
            
            // Special types - use reference
            (ScriptValue::Agent(_), _, _) |
            (ScriptValue::Tool(_), _, _) => Ok(value),
            
            // Promises - special handling
            (ScriptValue::Promise(p), ScriptLanguage::JavaScript, ScriptLanguage::Lua) => {
                // Convert JS Promise to Lua coroutine
                self.promise_to_coroutine(p.clone()).await
            }
            (ScriptValue::Promise(p), ScriptLanguage::Lua, ScriptLanguage::JavaScript) => {
                // Lua coroutines are already wrapped as promises
                Ok(value)
            }
            
            _ => Err(ScriptError::ConversionError {
                from: format!("{:?}", from),
                to: format!("{:?}", to),
                reason: "Unsupported conversion".to_string(),
            }),
        }
    }
}
```

## 5. Examples from Existing Projects

### 5.1 Deno's Unified Value System

Deno uses a unified approach for handling values between V8 and Rust:

```rust
// Simplified from deno_core
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(v8::Global<v8::Object>),
    // ... other types
}

// Conversion traits
impl<'a> TryFrom<v8::Local<'a, v8::Value>> for JsValue {
    type Error = Error;
    
    fn try_from(value: v8::Local<'a, v8::Value>) -> Result<Self, Self::Error> {
        // ... conversion logic
    }
}
```

### 5.2 Tauri's Multi-Language Approach

Tauri handles JavaScript and native code interaction:

```rust
// Command registration that works across languages
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Unified event system
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### 5.3 WASM-based Unity Pattern

Projects like Wasmer provide unified memory management:

```rust
// Shared linear memory accessible from multiple languages
let memory = Memory::new(&store, MemoryType::new(1, None, false))?;
let memory_view = memory.view(&store);

// Read/write operations work identically from any language
memory_view.write(0, b"Hello from any language!")?;
```

## 6. Recommendations for rs-llmspell

### 6.1 Architecture Recommendations

1. **Unified Value System**: Implement `ScriptValue` enum as the common type
2. **Reference Counting**: Use Arc/Weak for shared objects across languages
3. **GC Coordination**: Implement cooperative GC between Lua and JavaScript
4. **Error Context**: Preserve stack traces and error context across boundaries

### 6.2 Implementation Priority

1. **Phase 1**: Basic type conversion (primitives, arrays, objects)
2. **Phase 2**: Function proxies and callbacks
3. **Phase 3**: Agent/Tool reference sharing
4. **Phase 4**: Promise/Coroutine interop
5. **Phase 5**: Advanced GC coordination

### 6.3 Performance Considerations

1. **Conversion Caching**: Cache conversion strategies for known types
2. **Lazy Conversion**: Only convert values when crossing boundaries
3. **Zero-Copy**: Use references for large objects when possible
4. **Batch Operations**: Group conversions to reduce overhead

### 6.4 Testing Strategy

1. **Type Conversion Tests**: Round-trip testing for all types
2. **Memory Leak Tests**: Long-running tests with object creation/destruction
3. **Error Propagation Tests**: Ensure errors maintain context
4. **Performance Benchmarks**: Measure conversion overhead
5. **Concurrency Tests**: Test parallel script execution

## 7. Conclusion

Implementing unified value conversion and memory management between Lua and JavaScript requires:

1. A common value representation (`ScriptValue`)
2. Bidirectional conversion traits for each language
3. Shared memory management with reference counting
4. Coordinated garbage collection
5. Consistent error handling with context preservation

The recommended approach balances performance, safety, and developer ergonomics while maintaining the ability to leverage each language's strengths. The phased implementation allows for incremental development and testing of each component.