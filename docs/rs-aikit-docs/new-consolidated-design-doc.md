# Aikit: A Holistic Greenfield Architecture Design

**Version**: 1.0  
**Status**: Proposed Design  
**Contact**: AI Agent

## 1. Executive Summary

This document presents the complete architectural design for `rs-aikit`, a new from-scratch implementation built upon the lessons learned from its predecessor. The design addresses critical architectural flaws—notably async runtime conflicts and fragmented functionality—by introducing a robust, kernel-centric architecture.

The new system is built on a clean, 5-crate workspace that prioritizes separation of concerns, maintainability, and performance. At its heart is the `aikit-kernel`, a central application that manages a language-agnostic script runtime and communicates with clients via the full 5-channel Jupyter protocol. All script-callable components, including Tools, Agents, Workflows, RAG, State, and Session management, are consolidated into a single, unified `aikit-components` library.

This design establishes a stable, observable, and scalable foundation for all current and future phases of the project.

## 2. Motivation & Core Problems Addressed

Analysis of the previous implementation revealed three fundamental architectural problems that this new design solves:

1.  **Runtime Context Mismatch:** The primary issue was the "dispatch task is gone" error, caused by components (like HTTP clients in the `rig` crate) being created in short-lived `tokio::spawn` tasks. When the task ended, the runtime context was dropped, making the clients invalid. This design solves this by establishing a **single, global, long-lived I/O runtime** for the entire application.
2.  **Incomplete Jupyter Protocol:** The previous implementation used only a single channel, making it incompatible with standard Jupyter clients (Lab, VS Code) and limiting its capabilities. This design implements the **full 5-channel Jupyter protocol** (Shell, IOPub, Control, Stdin, Heartbeat) for robust, standard-compliant communication.
3.  **Fragmented Functionality:** The 20+ crates in the old workspace led to duplicated logic, circular dependencies, and high cognitive overhead. This design consolidates the entire project into a **logical 5-crate structure**, dramatically improving clarity and maintainability.

## 3. Guiding Architectural Principles

1.  **Kernel-Centric:** The `aikit-kernel` is the application. All other crates are libraries that serve it.
2.  **Unified Global Runtime:** A single, static `tokio` runtime for all I/O operations ensures context stability.
3.  **Protocol First:** The Jupyter protocol is the primary, formal interface to the kernel's capabilities.
4.  **Consolidated Components:** All script-callable functionality is unified into a single `aikit-components` library for clarity and ease of management.
5.  **Pervasive Tracing:** Deep observability using the `tracing` crate is a non-negotiable, first-class feature built into every layer.
6.  **Reference, Don't Migrate:** The old codebase is a library of proven solutions, not a structure to be preserved. We will adapt logic, not migrate crate structures.

## 4. The Greenfield Crate Architecture

The new architecture is composed of five core crates within a single workspace.

---

### **4.1. Crate: `aikit-core`**

*   **Purpose:** Defines the fundamental, shared traits and data types for the entire workspace. It has zero dependencies on other workspace crates, preventing circular dependencies.
*   **Directory Structure:**
    ```
    aikit-core/
    └── src/
        ├── lib.rs
        ├── traits.rs      # Defines BaseAgent, Tool, Workflow, ScriptEngineBridge
        ├── types.rs       # Defines AgentInput, AgentOutput, ComponentMetadata
        └── error.rs       # Defines the primary AikitError enum and variants
    ```
*   **External Dependencies:** `serde`, `tokio`, `async-trait`, `thiserror`.

---

### **4.2. Crate: `aikit-kernel`**

*   **Purpose:** The main application binary and the central nervous system. It owns the core application loop, manages communication protocols, and hosts essential infrastructure like the event bus, hook system, and debug coordinator.
*   **Directory Structure:**
    ```
    aikit-kernel/
    └── src/
        ├── main.rs        # Entry point for the kernel binary
        ├── lib.rs
        ├── kernel.rs      # The IntegratedKernel struct and main message loop
        ├── runtime.rs     # The critical GlobalIoRuntime implementation
        ├── transport.rs   # Jupyter protocol (5-channel ZeroMQ) implementation
        ├── protocol.rs    # Message structs (Jupyter, DAP)
        ├── events.rs      # The Event Bus implementation
        ├── hooks.rs       # The Hook Registry and execution logic
        └── debug/
            ├── mod.rs
            ├── coordinator.rs # The DebugCoordinator for managing debug state
            └── dap.rs         # The Debug Adapter Protocol (DAP) bridge
    ```
*   **Key Components:**
    *   **`GlobalIoRuntime`:** A `OnceLock`-wrapped `tokio::runtime::Runtime` that provides a stable execution context for all I/O operations application-wide.
    *   **`JupyterTransport`:** Manages the 5 ZeroMQ sockets required for full Jupyter protocol compliance.
    *   **`IntegratedKernel`:** The primary struct that owns all other components and runs the main message-handling loop.
    *   **`EventBus` & `HookRegistry`:** The core infrastructure for observability and extensibility, enabling decoupled communication between components.
    *   **`DebugCoordinator` & `DAPBridge`:** The complete debugging system, allowing IDEs to connect and control the execution of scripts within the kernel.
*   **External Dependencies:** `tokio`, `tracing`, `zeromq`, `uuid`, `serde`, `serde_json`, `aikit-core`, `aikit-runtime`, `aikit-components`.

---

### **4.3. Crate: `aikit-runtime`**

*   **Purpose:** To abstract away the specifics of different scripting languages, providing a single, consistent interface for the kernel to execute code.
*   **Directory Structure:**
    ```
    aikit-runtime/
    └── src/
        ├── lib.rs
        ├── runtime.rs     # The ScriptRuntime struct, handles API injection
        ├── bridge.rs      # The ScriptEngineBridge trait from aikit-core
        └── lua/
            ├── mod.rs
            └── engine.rs  # The LuaEngine implementation using mlua
    ```
*   **Key Components:**
    *   **`ScriptRuntime`:** Manages the lifecycle of a script engine instance and is responsible for injecting the global APIs (`Tool`, `Agent`, etc.) into the script's environment.
    *   **`LuaEngine`:** The first concrete implementation of the `ScriptEngineBridge` trait, using the `mlua` crate.
*   **External Dependencies:** `mlua`, `tokio`, `tracing`, `serde`, `aikit-core`, `aikit-components`.

---

### **4.4. Crate: `aikit-components`**

*   **Purpose:** A single, unified library for **all** script-callable functionality and their supporting infrastructure. This crate consolidates over a dozen crates from the old project into a cohesive whole.
*   **Directory Structure:**
    ```
    aikit-components/
    └── src/
        ├── lib.rs
        ├── registry.rs    # ComponentRegistry to discover all components
        ├── config.rs      # All configuration structs for aikit.toml
        ├── security.rs    # Sandboxing, policies, and multi-tenancy logic
        ├── storage/
        │   ├── mod.rs
        │   ├── engine.rs    # StorageEngine trait and backends (memory, sled)
        │   └── vector.rs    # Vector storage backend (HNSW)
        ├── state.rs       # StateManager implementation
        ├── sessions.rs    # SessionManager and artifact management
        ├── tools/         # Module containing all 37+ tools
        ├── agents/        # Agent templates and logic
        ├── workflows/     # Workflow patterns (sequential, etc.)
        └── rag/
            ├── mod.rs
            ├── pipeline.rs  # RAG pipeline orchestrator
            ├── embedding.rs # Embedding provider integration (rig)
            └── chunking.rs  # Document chunking strategies
    ```
*   **Key Components:**
    *   **`config.rs`:** Defines all `serde`-compatible structs for parsing the `aikit.toml` file.
    *   **`security.rs`:** Implements the script execution sandbox, access control policies, and the logic for **Multi-Tenancy**, such as `StateScope` and tenant isolation rules.
    *   **`storage/`:** Provides the **Storage Engine and Persistence** layer. `engine.rs` defines a key-value store trait with `memory` and `sled` backends. `vector.rs` provides the HNSW implementation for vector search.
    *   **`state.rs` & `sessions.rs`:** Implement the high-level **State and Session** management logic, which uses the storage engines for persistence.
    *   **`rag/`:** Contains the full **RAG** system, including the embedding provider integration (using the `rig` crate), document chunking, and the retrieval pipeline.
    *   **`tools/`, `agents/`, `workflows/`:** These modules contain the concrete implementations for all user-facing **Tools, Agents, and Workflows**.
*   **External Dependencies:** `rig`, `sled`, `hnsw`, `rmp-serde`, `tiktoken`, `serde`, `tokio`, `tracing`, `aikit-core`.

---

### **4.5. Crate: `aikit-cli`**

*   **Purpose:** The user-facing client application for starting and interacting with the kernel.
*   **Directory Structure:**
    ```
    aikit-cli/
    └── src/
        ├── main.rs
        └── commands/
            ├── mod.rs
            ├── serve.rs     # Logic for `aikit serve` (daemon mode)
            ├── run.rs       # Logic for `aikit run` (direct mode)
            └── repl.rs      # Logic for `aikit repl` (interactive client)
    ```
*   **External Dependencies:** `clap`, `tokio`, `aikit-kernel`.

## 5. Core Architectural Patterns & Data Flows

### 5.1. The Global I/O Runtime

To solve the "dispatch task is gone" error, all asynchronous I/O (especially HTTP requests) must be executed on a single, long-lived `tokio` runtime. This is achieved with a static, `OnceLock`-guarded runtime in `aikit-kernel`. Any component needing to perform I/O will use this global runtime, ensuring its context is never dropped prematurely.

```rust
// aikit-kernel/src/runtime.rs
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;

static GLOBAL_IO_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

pub fn global_io_runtime() -> &'static Arc<Runtime> {
    GLOBAL_IO_RUNTIME.get_or_init(|| {
        Arc::new(Runtime::new().expect("Failed to create global I/O runtime"))
    })
}
```

### 5.2. Kernel-Jupyter Data Flow

A user's request follows a clear, traceable path:

1.  **Request:** The `aikit-cli` (or another Jupyter client) sends an `execute_request` message to the kernel's **Shell** socket.
2.  **Transport:** The `JupyterTransport` in `aikit-kernel` receives the message. A `tracing` span is created, correlated by the message ID.
3.  **Kernel:** The `IntegratedKernel`'s main loop receives the message and dispatches it to the `aikit-runtime`.
4.  **Runtime:** The `ScriptRuntime` executes the code using the `LuaEngine`.
5.  **Components:** The script calls globals like `Tool.invoke()`. The runtime calls the corresponding component in `aikit-components`.
6.  **I/O:** If a tool needs to make an HTTP call, it uses the `GlobalIoRuntime`.
7.  **Output Stream:** As the script prints output (`print()`), the `LuaEngine` captures it and sends it to the kernel's I/O manager, which broadcasts it as a `stream` message on the **IOPub** socket for all clients to see in real-time.
8.  **Reply:** Once execution is complete, the final result is sent back to the kernel, which wraps it in an `execute_reply` message and sends it back on the **Shell** socket.

### 5.3. Pervasive Tracing & Observability

Tracing is a first-class citizen. The `#[instrument]` macro and `tracing::span!` will be used extensively:
*   **Kernel:** Spans for the kernel session, message handling, and execution requests.
*   **Components:** Spans for every agent, tool, and workflow execution, capturing parameters and duration.
*   **Providers:** Spans for every external API call, capturing the model, latency, and token counts.
*   **RAG:** Nested spans for each stage of the RAG pipeline (chunk, embed, search).
*   **Correlation:** Message IDs and session IDs will be attached to spans, allowing a request to be traced end-to-end across all components.

## 6. Performance & Success Metrics

The new architecture will be validated against the following performance targets:

*   **Runtime Stability:** Zero "dispatch task is gone" errors during extended operation.
*   **Protocol Compliance:** Full compatibility with Jupyter Lab and VS Code Jupyter extension.
*   **Tool Initialization:** < 10ms.
*   **Agent Creation:** < 50ms.
*   **RAG Search Latency (100k vectors):** < 10ms P95.
*   **Tracing Overhead (`RUST_LOG=info`):** < 2% performance impact.

## 7. Validation Strategy

The architecture will be validated using the comprehensive application test suite from the previous implementation. This suite tests the system at increasing layers of complexity, from simple 2-agent scripts to a 21-agent web application creator. The new implementation must pass these tests while meeting the performance and stability metrics defined above.
