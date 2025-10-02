# GEMINI.md

## Project Overview

**Project Identity:** rs-llmspell is for **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems.

This project, `rs-llmspell`, is a production-ready AI workflow orchestration platform built in Rust. It is designed for script-driven LLM coordination with RAG (Retrieval-Augmented Generation) at scale.

The platform is highly modular and can be built with different feature sets to control the binary size. It supports multiple LLM providers and offers enterprise-grade features like multi-tenancy and security.

## Key Documents

*   **Architecture**: `/docs/technical/master-architecture-vision.md`
*   **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (16-phase roadmap)
*   **Current Work**: Always read `/TODO.md` for active tasks.

## Development Philosophy

*   **Less code is better**: REPLACE code, don't add (breaking changes are acceptable until version 1.0).
*   **Less files are better**: Edit existing files; do not create new ones unless critical.
*   **NO backward compatibility until 1.0**: Prioritize correctness and simplicity.
*   **NO SHORTCUTS**: Holistic completion is required; no TODOs for later.
*   Do not leave TODO comments in the code.
*   Do not use `unwrap()` in production code.

### Architecture Mandates

*   **Traits over dependencies**: Use the Rust trait system for modularity.
*   **State-First Design**: Components communicate through shared state.
*   **Bridge-First**: Leverage existing crates versus reimplementing.
*   **Script API Consistency**: The same surface across Lua/JS/Python.

## Crate Structure

*   `llmspell-core`: Core traits and types
*   `llmspell-tools`: Tool implementations
*   `llmspell-agents`: Agent infrastructure
*   `llmspell-workflows`: Workflow patterns
*   `llmspell-bridge`: Script language integration
*   `llmspell-utils`: Shared utilities (use for ALL shared code)
*   `llmspell-state-persistence`: State with persistence
*   `llmspell-hooks`: Hook system with replay
*   `llmspell-events`: Event system with correlation
*   `llmspell-sessions`: Session management with artifacts

## Building and Running

### Building the Project

The project can be built using `cargo`. There are three main build configurations:

*   **Minimal (19MB)**: Includes core functionality with Lua scripting and essential tools.
    ```bash
    cargo build --release
    ```
*   **Common (25MB)**: Adds template engines and PDF processing.
    ```bash
    cargo build --release --features common
    ```
*   **Full (35MB)**: Includes all optional components, such as CSV/Parquet, Excel, archives, email, and database support.
    ```bash
    cargo build --release --features full
    ```

### Running Applications

The project includes a script for easily running applications, which is the recommended way for new users to get started.

```bash
./scripts/utilities/llmspell-easy.sh <application-name>
```

For example, to run the `file-organizer` application:

```bash
./scripts/utilities/llmspell-easy.sh file-organizer
```

**Advanced Users:**

You can also run applications directly with the `llmspell` binary:

```bash
llmspell app run <application-name>
```

## Development Workflow

1.  Read the relevant phase documentation and `TODO.md`.
2.  Write a failing test using `llmspell-testing` helpers.
3.  Implement the minimal solution.
4.  Run `./scripts/quality/quality-check-fast.sh`.
5.  Update sub-tasks in `TODO.md` as they are completed.

## Testing and Quality

### Quality Gates

The following scripts are mandatory to run before committing:

*   `./scripts/quality/quality-check-minimal.sh`: Runs in seconds (format, clippy, compile).
*   `./scripts/quality/quality-check-fast.sh`: Takes about 1 minute (adds unit tests & docs).
*   `./scripts/quality/quality-check.sh`: Takes 5+ minutes (full validation).

### Testing System

*   Use helpers from the **`llmspell-testing`** crate ONLY.
*   Testing is feature-based via Cargo features.
*   Target **>90% test coverage** and **>95% API documentation coverage**.

### Performance Targets

*   **Tool initialization**: <10ms
*   **Agent creation**: <50ms
*   **Hook overhead**: <1%
*   **State operations**: <5ms write, <1ms read
*   **Zero warnings policy**: `cargo clippy --workspace --all-target --all-features`