# llmspell-utils

Shared utility functions and helpers for the LLMSpell framework.

## Features

- **Async Utilities**: Timeout management, cancellation tokens, and concurrency helpers
- **File Utilities**: Cross-platform file operations and path manipulation
- **String Utilities**: String manipulation, truncation, and sanitization
- **System Info**: System information gathering and environment utilities
- **Error Builders**: Convenient error construction helpers
- **ID Generator**: UUID-based component ID generation
- **Serialization**: Common serialization/deserialization utilities

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
llmspell-utils = { path = "../llmspell-utils" }
```

Then use the utilities in your code:

```rust
use llmspell_utils::{
    generate_component_id,
    ensure_dir,
    truncate,
    SystemInfo,
};

// Generate a unique component ID
let id = generate_component_id("agent");

// Ensure a directory exists
ensure_dir(&config_path)?;

// Truncate long strings
let summary = truncate(&long_text, 100);
```

## Module Structure

- `async_utils` - Async operation helpers
- `file_utils` - File system operations
- `string_utils` - String manipulation
- `system_info` - System information
- `error_builders` - Error construction
- `id_generator` - ID generation
- `serialization` - Serialization helpers