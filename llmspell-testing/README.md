# llmspell-testing

Testing utilities for Rs-LLMSpell development.

## Features
- Mock implementations for all core traits
- Property-based testing helpers with proptest
- Benchmark utilities using criterion

## Usage
```rust
use llmspell_testing::{MockAgent, MockProvider, test_utils};

let mock_agent = MockAgent::new()
    .expect_execute()
    .returning(|_| Ok(json!({"result": "test"})));

test_utils::assert_agent_behavior(&mock_agent).await?;
```

## Dependencies
- `llmspell-core` - Traits to mock
- External: `mockall`, `proptest`, `criterion`
- All other llmspell crates for integration testing