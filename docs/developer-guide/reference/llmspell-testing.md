# llmspell-testing

## Purpose

Comprehensive testing framework and utilities for LLMSpell, providing test organization, categorization, mocks, fixtures, property-based testing generators, benchmarking helpers, and test runners. This crate enables both unit and integration testing with feature-based test execution and realistic test data generation.

## Core Concepts

- **Test Categorization**: Tests organized by type (unit, integration, agents, scenarios, lua, performance)
- **Feature-Based Execution**: Run specific test categories using Cargo features
- **Mock Implementations**: Pre-built mocks for all core traits (BaseAgent, Tool, Workflow)
- **Property Testing**: Generators for creating random but valid test data
- **Test Fixtures**: Reusable test data and environment setup
- **Benchmark Framework**: Performance testing and regression detection
- **Test Attributes**: Macros for test categorization and conditional execution

## Primary Components

### Test Categories

**Purpose**: Organize tests by type for selective execution using Cargo features.

Tests are organized into categories activated by feature flags in Cargo.toml:
- `unit-tests` - Fast, isolated unit tests for individual components
- `integration-tests` - Cross-crate integration tests
- `agent-tests` - Agent-specific functionality tests
- `scenario-tests` - End-to-end scenario tests simulating real usage
- `lua-tests` - Lua scripting bridge tests
- `performance-tests` - Performance benchmarks

Use cargo test with the appropriate feature flag to run specific test categories:
```bash
cargo test -p llmspell-testing --features unit-tests
cargo test -p llmspell-testing --features integration-tests
```

### Test Macros

**Purpose**: Categorize and conditionally run tests based on requirements.

```rust
/// Mark test as belonging to a category
#[macro_export]
macro_rules! test_category {
    ($category:ident, $test_fn:item) => {
        #[cfg(feature = concat!("", stringify!($category), "-tests"))]
        #[test]
        $test_fn
    };
}

/// Mark test as slow (>1s execution time)
#[macro_export]
macro_rules! slow_test {
    ($test_fn:item) => {
        #[cfg(not(feature = "skip-slow-tests"))]
        #[test]
        $test_fn
    };
}

/// Mark test as requiring network access
#[macro_export]
macro_rules! requires_network {
    ($test_fn:item) => {
        #[cfg(all(
            feature = "network-tests",
            not(feature = "offline-tests")
        ))]
        #[test]
        $test_fn
    };
}

/// Mark test as requiring LLM provider
#[macro_export]
macro_rules! requires_llm {
    ($provider:literal, $test_fn:item) => {
        #[cfg(all(
            feature = "llm-tests",
            env = concat!($provider, "_API_KEY")
        ))]
        #[test]
        $test_fn
    };
}

/// Mark test as flaky (may fail intermittently)
#[macro_export]
macro_rules! flaky_test {
    ($max_retries:literal, $test_fn:item) => {
        #[test]
        #[retry($max_retries)]
        $test_fn
    };
}
```

**Usage Example**:
```rust
use llmspell_testing::{test_category, slow_test, requires_network};

test_category!(unit, {
    fn test_component_creation() {
        let component = MyComponent::new();
        assert_eq!(component.name(), "test");
    }
});

slow_test!({
    async fn test_large_dataset_processing() {
        let data = generate_large_dataset(1_000_000);
        let result = process_dataset(data).await;
        assert!(result.is_ok());
    }
});

requires_network!({
    async fn test_external_api_call() {
        let client = ApiClient::new();
        let response = client.call("https://api.example.com").await?;
        assert_eq!(response.status(), 200);
    }
});
```

### Mock Implementations

**Purpose**: Pre-built mocks for testing components that depend on LLMSpell traits.

```rust
use llmspell_testing::mocks::{
    MockBaseAgent, MockAgent, MockTool, MockWorkflow,
    MockStateAccess, MockEventEmitter
};
use mockall::predicate::*;

/// Mock BaseAgent implementation
pub struct MockBaseAgent {
    metadata: ComponentMetadata,
    execute_impl: Box<dyn Fn(AgentInput, ExecutionContext) -> Result<AgentOutput>>,
}

impl MockBaseAgent {
    pub fn new() -> Self {
        Self::with_name("mock-agent")
    }
    
    pub fn with_name(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                "Mock agent for testing".to_string()
            ),
            execute_impl: Box::new(|input, _| {
                Ok(AgentOutput::text(format!("Mock: {}", input.text)))
            }),
        }
    }
    
    pub fn with_response(response: String) -> Self {
        let mut mock = Self::new();
        mock.execute_impl = Box::new(move |_, _| {
            Ok(AgentOutput::text(response.clone()))
        });
        mock
    }
    
    pub fn with_error(error: LLMSpellError) -> Self {
        let mut mock = Self::new();
        mock.execute_impl = Box::new(move |_, _| {
            Err(error.clone())
        });
        mock
    }
}

/// Mock state access
pub struct MockStateAccess {
    storage: Arc<RwLock<HashMap<String, Value>>>,
}

impl MockStateAccess {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_data(data: HashMap<String, Value>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(data)),
        }
    }
}

#[async_trait]
impl StateAccess for MockStateAccess {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        Ok(self.storage.read().await.get(key).cloned())
    }
    
    async fn set(&self, key: &str, value: Value) -> Result<()> {
        self.storage.write().await.insert(key.to_string(), value);
        Ok(())
    }
}
```

### Property-Based Test Generators

**Purpose**: Generate random but valid test data for property testing.

```rust
use proptest::prelude::*;
use llmspell_testing::generators;

/// Generate valid component IDs
pub fn component_id_strategy() -> impl Strategy<Value = ComponentId> {
    "[a-z][a-z0-9-]{2,30}".prop_map(|s| ComponentId::new(s))
}

/// Generate valid agent inputs
pub fn agent_input_strategy() -> impl Strategy<Value = AgentInput> {
    (
        ".*",  // Any text
        prop::option::of(any::<Value>()),  // Optional context
        prop::collection::hash_map(
            "[a-z]+",
            any::<Value>(),
            0..5
        ),  // Parameters
    ).prop_map(|(text, context, params)| {
        let mut input = AgentInput::text(text);
        if let Some(ctx) = context {
            input = input.with_context(ctx);
        }
        for (k, v) in params {
            input = input.with_param(k, v);
        }
        input
    })
}

/// Generate execution contexts
pub fn execution_context_strategy() -> impl Strategy<Value = ExecutionContext> {
    (
        prop::option::of("[a-z0-9-]{8,12}"),  // Session ID
        prop::option::of("[a-z0-9-]{8,12}"),  // Conversation ID
        prop::bool::ANY,  // Has state
        prop::bool::ANY,  // Has events
    ).prop_map(|(session, conversation, has_state, has_events)| {
        let mut ctx = ExecutionContext::new();
        ctx.session_id = session;
        ctx.conversation_id = conversation;
        
        if has_state {
            ctx.state = Some(Arc::new(MockStateAccess::new()));
        }
        if has_events {
            ctx.events = Some(Arc::new(MockEventEmitter::new()));
        }
        
        ctx
    })
}

/// Generate error cases
pub fn error_strategy() -> impl Strategy<Value = LLMSpellError> {
    prop_oneof![
        Just(LLMSpellError::Validation {
            message: "Test validation error".to_string(),
            field: Some("test_field".to_string()),
        }),
        Just(LLMSpellError::Component {
            message: "Test component error".to_string(),
            source: None,
        }),
        Just(LLMSpellError::Network {
            message: "Test network error".to_string(),
            retryable: true,
            source: None,
        }),
    ]
}
```

### Test Fixtures

**Purpose**: Reusable test data and environment setup.

```rust
use llmspell_testing::fixtures::{
    AgentFixture, ToolFixture, WorkflowFixture,
    load_fixture, create_test_environment
};

/// Pre-configured agent fixtures
pub struct AgentFixture {
    pub simple_agent: Arc<dyn Agent>,
    pub llm_agent: Arc<dyn Agent>,
    pub tool_capable_agent: Arc<dyn Agent>,
}

impl AgentFixture {
    pub fn new() -> Self {
        Self {
            simple_agent: Arc::new(create_simple_agent()),
            llm_agent: Arc::new(create_llm_agent()),
            tool_capable_agent: Arc::new(create_tool_capable_agent()),
        }
    }
}

/// Load fixture from file
pub fn load_fixture<T: DeserializeOwned>(name: &str) -> Result<T> {
    let path = format!("fixtures/{}.json", name);
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Create isolated test environment
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub state_manager: Arc<StateManager>,
    pub event_bus: Arc<EventBus>,
    pub registry: Arc<ComponentRegistry>,
}

pub fn create_test_environment() -> TestEnvironment {
    let temp_dir = TempDir::new().unwrap();
    
    TestEnvironment {
        temp_dir: temp_dir.clone(),
        state_manager: Arc::new(StateManager::memory()),
        event_bus: Arc::new(EventBus::new()),
        registry: Arc::new(ComponentRegistry::new()),
    }
}
```

### Benchmark Helpers

**Purpose**: Performance testing and regression detection.

```rust
use llmspell_testing::benchmarks::{
    BenchmarkRunner, BenchmarkResult, benchmark_async
};
use std::time::Duration;

/// Benchmark runner with statistics
pub struct BenchmarkRunner {
    warmup_iterations: usize,
    benchmark_iterations: usize,
    timeout: Duration,
}

impl BenchmarkRunner {
    pub async fn run<F, Fut>(&self, name: &str, f: F) -> BenchmarkResult
    where
        F: Fn() -> Fut,
        Fut: Future<Output = ()>,
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            timeout(self.timeout, f()).await.unwrap();
        }
        
        // Benchmark
        let mut durations = Vec::new();
        for _ in 0..self.benchmark_iterations {
            let start = Instant::now();
            timeout(self.timeout, f()).await.unwrap();
            durations.push(start.elapsed());
        }
        
        BenchmarkResult {
            name: name.to_string(),
            min: *durations.iter().min().unwrap(),
            max: *durations.iter().max().unwrap(),
            mean: calculate_mean(&durations),
            median: calculate_median(&mut durations),
            std_dev: calculate_std_dev(&durations),
            iterations: self.benchmark_iterations,
        }
    }
}

/// Macro for easy benchmarking
#[macro_export]
macro_rules! benchmark_async {
    ($name:expr, $iterations:expr, $body:block) => {{
        let runner = BenchmarkRunner::default()
            .with_iterations($iterations);
        
        runner.run($name, || async move $body).await
    }};
}
```

## Usage Patterns

### Testing a Custom Component

**When to use**: Testing components that implement BaseAgent or other traits.

**Benefits**: Isolated testing with controlled dependencies.

**Example**:
```rust
use llmspell_testing::{
    mocks::{MockStateAccess, MockEventEmitter},
    fixtures::create_test_environment,
    test_category,
};

test_category!(unit, {
    async fn test_my_component() {
        // Setup
        let env = create_test_environment();
        let component = MyComponent::new();
        
        // Create mock context
        let context = ExecutionContext::new()
            .with_state(Arc::new(MockStateAccess::new()))
            .with_events(Arc::new(MockEventEmitter::new()));
        
        // Test execution
        let input = AgentInput::text("test input");
        let output = component.execute(input, context).await.unwrap();
        
        assert_eq!(output.text, "Expected output");
    }
});
```

### Property-Based Testing

**When to use**: Testing components with many possible inputs.

**Benefits**: Finds edge cases automatically.

**Example**:
```rust
use proptest::prelude::*;
use llmspell_testing::generators::{
    agent_input_strategy,
    execution_context_strategy
};

proptest! {
    #[test]
    fn test_component_never_panics(
        input in agent_input_strategy(),
        context in execution_context_strategy()
    ) {
        let component = MyComponent::new();
        
        // Should not panic for any valid input
        let _ = component.execute(input, context).await;
    }
    
    #[test]
    fn test_validation_catches_invalid_input(
        text in "[^a-zA-Z0-9]{1,100}"  // Invalid characters
    ) {
        let component = MyComponent::new();
        let input = AgentInput::text(text);
        
        let result = component.validate_input(&input).await;
        prop_assert!(result.is_err());
    }
}
```

### Benchmark Testing

**When to use**: Performance testing and regression detection.

**Benefits**: Tracks performance over time.

**Example**:
```rust
use llmspell_testing::benchmarks::{BenchmarkRunner, benchmark_async};

#[tokio::test]
async fn benchmark_component_execution() {
    let component = MyComponent::new();
    
    let result = benchmark_async!("component_execution", 100, {
        let input = AgentInput::text("benchmark input");
        let context = ExecutionContext::new();
        component.execute(input, context).await.unwrap();
    });
    
    println!("Execution time: mean={:?}, std_dev={:?}", 
             result.mean, result.std_dev);
    
    // Assert performance requirements
    assert!(result.mean < Duration::from_millis(100));
    assert!(result.max < Duration::from_millis(200));
}
```

## Integration Examples

### Testing with Multiple Mocks

```rust
use llmspell_testing::mocks::{MockAgent, MockTool, MockWorkflow};

#[test]
async fn test_agent_tool_integration() {
    // Create mocked components
    let mock_tool = MockTool::new()
        .with_name("calculator")
        .with_handler(|params| {
            let a = params["a"].as_f64().unwrap();
            let b = params["b"].as_f64().unwrap();
            Ok(json!({"result": a + b}))
        });
    
    let mock_agent = MockAgent::new()
        .with_tool(mock_tool.clone())
        .with_handler(|input, tools| {
            // Agent calls tool
            let result = tools.invoke("calculator", json!({
                "a": 5,
                "b": 3
            })).await?;
            
            Ok(AgentOutput::text(format!("Result: {}", result["result"])))
        });
    
    // Test the integration
    let input = AgentInput::text("Add 5 and 3");
    let output = mock_agent.execute(input, ExecutionContext::new()).await?;
    
    assert_eq!(output.text, "Result: 8");
}
```

### Testing Error Scenarios

```rust
use llmspell_testing::{
    mocks::MockBaseAgent,
    generators::error_strategy,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_error_handling(error in error_strategy()) {
        let component = MyComponent::new();
        let mock = MockBaseAgent::with_error(error.clone());
        
        // Component should handle various errors gracefully
        let result = component.handle_dependency_error(mock, error).await;
        
        prop_assert!(result.is_ok() || result.is_err());
        // No panics
    }
}
```

## Configuration

```toml
# Test configuration in Cargo.toml
[features]
default = ["test-utilities"]

# Test categories
unit-tests = []
integration-tests = []
agent-tests = []
scenario-tests = []
lua-tests = []
performance-tests = []
all-tests = [
    "unit-tests",
    "integration-tests", 
    "agent-tests",
    "scenario-tests",
    "lua-tests",
    "performance-tests"
]

# Test conditions
skip-slow-tests = []
network-tests = []
llm-tests = []
offline-tests = []

# Test utilities
test-utilities = ["mockall", "proptest", "criterion"]

[dev-dependencies]
mockall = "0.11"
proptest = "1.0"
criterion = "0.5"
tempfile = "3.0"
```

## Performance Considerations

- **Mock Overhead**: Mocks add overhead - use lightweight stubs for performance tests
- **Property Test Iterations**: Default 256 iterations - reduce for slow tests
- **Benchmark Warmup**: Always include warmup iterations to stabilize CPU/cache
- **Test Isolation**: Each test should create its own environment - avoid shared state
- **Async Runtime**: Tests use multi-threaded runtime by default - use single-threaded for determinism
- **Feature Flags**: Compile only needed test categories to reduce binary size

## Security Considerations

- **Test Data**: Never commit real API keys or sensitive data in fixtures
- **Network Tests**: Mock external services when possible to avoid dependencies
- **File System**: Always use temp directories for file operations
- **Resource Limits**: Set timeouts on all async operations to prevent hangs
- **Determinism**: Use fixed seeds for random generators in CI

## Migration Guide

### From v0.5.x to v0.6.x

Breaking changes:
- Test categories now use features instead of runtime flags
- Mock traits updated to match new BaseAgent interface
- Fixture format changed to JSON from YAML

Migration steps:
1. Update `Cargo.toml` to use feature flags
2. Replace runtime category checks with compile-time features
3. Update mock implementations to new trait signatures
4. Convert YAML fixtures to JSON format

### From v0.6.x to v0.8.x (Phase 8)

New features:
- RAG-specific test helpers and fixtures
- Multi-tenant testing utilities
- Session replay testing
- Vector storage mocks

Migration steps:
1. Add RAG test fixtures for vector operations
2. Use multi-tenant test environments
3. Add session replay to integration tests
4. Mock vector storage for unit tests