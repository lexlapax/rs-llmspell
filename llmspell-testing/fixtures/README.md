# Test Fixtures

This directory contains test fixtures and data used across the LLMSpell test suite.

## Directory Structure

```
fixtures/
├── data/                    # Test data files
│   └── migration_test_cases/  # Migration test scenarios
│       ├── complex_nested_migration.json
│       ├── error_scenarios.json
│       └── v1_to_v2_user_schema.json
├── lua/                     # Lua test scripts
│   ├── basic_hooks.lua      # Basic hook functionality tests
│   ├── cross_language.lua   # Cross-language integration tests
│   ├── performance.lua      # Performance benchmark scripts
│   ├── lua_tool_integration.lua  # Tool integration tests
│   └── ...                  # Additional test scripts
└── temp/                    # Temporary test files (auto-cleaned)
```

## Using Fixtures in Tests

The `llmspell-testing` crate provides utilities for loading fixtures:

```rust
use llmspell_testing::fixtures::{
    fixture_path,
    load_fixture_text,
    load_fixture_json,
    load_fixture_lua,
    load_test_data,
    create_temp_fixture,
    cleanup_temp_fixtures,
};

// Get path to a fixture
let path = fixture_path("data/test.json");

// Load text content
let content = load_fixture_text("data/sample.txt")?;

// Load JSON data
let json_data = load_fixture_json("data/config.json")?;

// Load Lua script
let script = load_fixture_lua("basic_hooks.lua")?;

// Load binary test data
let bytes = load_test_data("image.png")?;

// Create temporary test file
let temp_path = create_temp_fixture("test.tmp", "test content")?;
// ... use temp file ...
cleanup_temp_fixtures()?;
```

## Fixture Categories

### Data Fixtures (`data/`)
- **Migration Test Cases**: JSON files testing state migration scenarios
- **Test Data**: Sample data files for various tools and components

### Lua Fixtures (`lua/`)
- **Hook Tests**: Scripts testing hook registration and execution
- **Integration Tests**: Full integration test scenarios
- **Performance Tests**: Benchmarking and performance validation
- **Tool Tests**: Individual tool functionality validation

## Adding New Fixtures

1. Place files in the appropriate subdirectory
2. Use descriptive names that indicate the test purpose
3. Include comments in scripts explaining the test scenario
4. Update this README if adding new categories

## Path Resolution

The fixture loading utilities automatically handle path resolution across different test environments:
- When running from crate root
- When running from workspace root
- When running individual tests
- When using `CARGO_MANIFEST_DIR`

## Best Practices

1. **Reuse fixtures** across tests to maintain consistency
2. **Version fixtures** if they represent different schema versions
3. **Document complex fixtures** with comments or accompanying docs
4. **Clean up temp files** using the provided cleanup utilities
5. **Use generators** for property-based testing data

## Test Data Generators

For dynamic test data, use the generators in `llmspell_testing::generators`:
- `sample_json_data_strategy()` - Generate random JSON data
- `test_file_path_strategy()` - Generate test file paths
- `mock_api_response_strategy()` - Generate API response mocks
- `file_content_strategy()` - Generate file content
- And many more...

See `src/generators.rs` for the full list of available generators.