# llmspell-kernel

Jupyter kernel implementation for rs-llmspell with Debug Adapter Protocol (DAP) support.

## Testing

### Unit Tests
```bash
cargo test -p llmspell-kernel
```

### Python Integration Tests
The Python integration tests validate DAP functionality through real Jupyter protocol interactions. They are **ignored by default** as they require special setup.

**Requirements:**
- Built llmspell binary (`cargo build`)
- Python 3 with pytest and jupyter_client
- ~30 seconds to complete

**Running:**
```bash
# Run ignored Python tests
cargo test -p llmspell-kernel --test python_integration -- --ignored

# Skip Python tests permanently
cargo test --features skip-python-tests
```

The tests automatically start a kernel daemon if needed. See `tests/python/` for test implementation details.