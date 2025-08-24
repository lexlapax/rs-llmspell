# CLI Embedded Resources

This directory contains resources that are embedded directly into the `llmspell` binary at compile time using Rust's `include_str!` macro.

## Structure

```
resources/
└── applications/       # Example applications embedded in binary
    ├── file-organizer/
    ├── research-collector/
    ├── content-creator/
    ├── communication-manager/
    ├── process-orchestrator/
    ├── code-review-assistant/
    └── webapp-creator/
```

## Why Here?

These applications are embedded in the CLI binary for single-binary distribution. By keeping them within the `llmspell-cli` crate:

1. **Self-contained**: The CLI crate doesn't depend on external files
2. **True embedding**: Resources are part of the crate that uses them
3. **Clean distribution**: Single binary contains everything needed
4. **No path confusion**: Users don't need to configure paths

## Usage

Users can run embedded applications with simple commands:
```bash
llmspell apps list                # List all available apps
llmspell apps file-organizer      # Run the file organizer
llmspell apps research-collector  # Run the research collector
```

## Implementation

The `embedded_resources.rs` module uses `include_str!` to embed these files at compile time:
```rust
include_str!("../resources/applications/file-organizer/main.lua")
```

At runtime, these are extracted to a temporary directory and executed.