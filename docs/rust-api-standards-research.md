# Rust API Design Standards Research

**Date**: August 1, 2025
**Purpose**: Research industry standards and best practices for Rust API design

## Popular Crate Analysis

### 1. Tokio (Async Runtime)
```rust
// Constructor patterns
Runtime::new()                    // Simple default
Runtime::with_config(config)      // With configuration
Builder::new_multi_thread()       // Builder pattern

// Method patterns
runtime.spawn(future)             // Action verbs
runtime.block_on(future)          // Descriptive names
handle.clone()                    // Standard trait impls
```

### 2. Serde (Serialization)
```rust
// Derive macros for ergonomics
#[derive(Serialize, Deserialize)]

// Method patterns
serde_json::to_string(value)     // to_* for conversions
serde_json::from_str(s)          // from_* for parsing
serializer.serialize_struct()     // Verb-first naming
```

### 3. Reqwest (HTTP Client)
```rust
// Builder pattern
Client::builder()
    .timeout(Duration::from_secs(10))
    .build()

// Method chaining
client.get(url)
    .header("User-Agent", "rust")
    .send()
    .await?
```

### 4. Clap (CLI Parsing)
```rust
// Builder pattern with method chaining
Command::new("app")
    .version("1.0")
    .author("Name")
    .about("Description")
    .arg(Arg::new("input"))
```

### 5. Diesel (ORM)
```rust
// Query builder pattern
users.filter(name.eq("John"))
     .order(id.desc())
     .limit(5)
     .load::<User>(&mut conn)
```

---

## Rust API Guidelines (Official)

### Naming Conventions

#### Casing (C-CASE)
- **Types**: `UpperCamelCase`
- **Functions/Methods**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Type parameters**: Single letter (`T`) or `UpperCamelCase`

#### Word Order (C-WORD-ORDER)
```rust
// Verb-object-preposition
// GOOD
client.send_request_to(server)

// NOT
client.request_send_to(server)
```

#### Getters (C-GETTER)
```rust
// Getters don't use get_ prefix for simple access
struct Person {
    name: String,
}

impl Person {
    // GOOD
    pub fn name(&self) -> &str { &self.name }
    
    // AVOID (unless computing/searching)
    pub fn get_name(&self) -> &str { &self.name }
}

// Use get_ when it's not simple field access
impl Database {
    // GOOD - implies lookup/computation
    pub async fn get_user(&self, id: u64) -> Result<User> { ... }
}
```

#### Methods with Prepositions (C-PREPOSITION)
```rust
// Use established patterns
impl Path {
    fn join(&self, other: &Path) -> PathBuf  // Not join_with
}

// Common prepositions
from_*    // Conversion from another type
to_*      // Conversion to another type  
as_*      // Cheap reference conversion
into_*    // Consuming conversion
with_*    // Builder-style configuration
```

### Constructor Conventions

#### new() Method (C-NEW)
```rust
impl MyType {
    // Simple construction should be new()
    pub fn new() -> Self { ... }
    
    // Complex construction can use descriptive names
    pub fn with_capacity(cap: usize) -> Self { ... }
    pub fn from_config(config: Config) -> Result<Self> { ... }
}
```

#### Builder Pattern (C-BUILDER)
```rust
// For 3+ configuration parameters
impl MyTypeBuilder {
    pub fn new() -> Self { ... }
    pub fn option1(mut self, val: T) -> Self { ... }
    pub fn option2(mut self, val: U) -> Self { ... }
    pub fn build(self) -> Result<MyType> { ... }
}
```

### Type Conversions

#### Conversion Traits (C-CONV)
```rust
// Implement standard traits
impl From<OtherType> for MyType { ... }
impl TryFrom<OtherType> for MyType { ... }
impl FromStr for MyType { ... }

// Provide convenience methods
impl OtherType {
    pub fn into_my_type(self) -> MyType { ... }
    pub fn to_my_type(&self) -> MyType { ... }
}
```

### Error Handling

#### Result Types (C-RESULT)
```rust
// Use Result for fallible operations
pub fn parse(input: &str) -> Result<MyType, ParseError> { ... }

// Provide context in errors
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Failed to parse {input}: {reason}")]
    ParseError { input: String, reason: String },
}
```

### Ownership and Borrowing

#### Method Receivers (C-RECEIVER)
```rust
// Immutable access
fn get(&self) -> &T

// Mutable access
fn get_mut(&mut self) -> &mut T

// Consuming
fn into_inner(self) -> T

// Cloning
fn to_owned(&self) -> T where T: Clone
```

### Async Patterns

#### Async Methods (C-ASYNC)
```rust
// Async trait methods need Send + Sync bounds
#[async_trait]
pub trait MyTrait: Send + Sync {
    async fn do_work(&self) -> Result<()>;
}

// Provide sync wrappers when needed
impl Client {
    pub async fn fetch(&self) -> Result<Data> { ... }
    
    // For use in sync contexts
    pub fn fetch_blocking(&self) -> Result<Data> {
        tokio::runtime::Runtime::new()?.block_on(self.fetch())
    }
}
```

---

## Best Practices Summary

### 1. Naming
- ✅ Use `get_` only for computed/lookup operations
- ✅ Simple field access should omit `get_` prefix
- ✅ Use established preposition patterns
- ✅ Verb-first for actions: `send_request()` not `request_send()`

### 2. Constructors
- ✅ `new()` for simple/default construction
- ✅ `with_*()` for parameterized variants
- ✅ `from_*()` for conversions
- ✅ Builder pattern for 3+ parameters

### 3. Type Design
- ✅ Implement standard traits (Clone, Debug, Send, Sync)
- ✅ Use newtype pattern for type safety
- ✅ Provide conversion traits (From, TryFrom)

### 4. Error Handling
- ✅ All fallible operations return `Result<T, E>`
- ✅ Custom error types with context
- ✅ Use `thiserror` for error definitions

### 5. API Evolution
- ✅ Use `#[non_exhaustive]` for enums that may grow
- ✅ Deprecate before removing
- ✅ Sealed traits for internal flexibility

---

## Recommendations for rs-llmspell

### High Priority Changes

1. **Remove Service Suffix**
   ```rust
   // Change
   HookExecutorService → HookExecutor
   EventBusService → EventBus
   ```

2. **Standardize Getters**
   ```rust
   // Change retrieve_session to get_session
   // Keep get_ prefix since it's a lookup operation
   pub async fn get_session(&self, id: &SessionId) -> Result<Session>
   ```

3. **Add Builder Patterns**
   ```rust
   SessionManagerConfig::builder()
       .max_sessions(100)
       .retention_days(30)
       .build()
   ```

### Medium Priority

1. **Consistent Async Patterns**
   - Ensure all async traits have `Send + Sync` bounds
   - Consider sync wrappers for script contexts

2. **Improve Error Context**
   - Add more context to error variants
   - Use error chains effectively

### Low Priority

1. **Documentation Patterns**
   - Add examples to all public APIs
   - Include performance characteristics
   - Document panic conditions

2. **API Stability**
   - Mark experimental APIs
   - Use feature flags for unstable features