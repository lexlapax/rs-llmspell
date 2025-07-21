# Provider Enhancement Test Fixes

**Date**: 2025-07-21  
**Issue**: Provider enhancement tests failing due to async/coroutine errors

## Problem
The provider enhancement tests were failing with:
```
attempt to yield from outside a coroutine
```

## Solution
Updated all `Agent.create()` calls to `Agent.createAsync()` in the test file:
- `/llmspell-bridge/tests/provider_enhancement_test.rs`

## Changes Made
Updated 8 instances of `Agent.create` to `Agent.createAsync`:
1. test_agent_create_with_provider_model_syntax - 3 instances
2. test_base_url_override - 1 instance  
3. test_backward_compatibility - 1 instance
4. test_invalid_provider_handling - 2 instances
5. test_provider_fallback - 1 instance
6. test_provider_model_parsing - 1 instance
7. test_multiple_providers_same_script - 1 instance

## Result
All Agent creation calls in tests now use the async-safe version, preventing coroutine errors during test execution.

## Note
These tests validate provider configuration and model parsing functionality. The async change maintains the same test logic while ensuring compatibility with the Lua coroutine requirements.