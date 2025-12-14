# Project Phoenix: A Holistic Greenfield Architecture and Implementation Plan for Aikit

This is a complete plan for building `rs-aikit` from scratch. We will not migrate old crates; we will build a new, clean architecture and reference the logic from the old codebase (`~/projects/lexlapax/rs-llmspell`) as a library of proven solutions.

#### **Guiding Principles**

1.  **Greenfield Crate Structure:** We will create a minimal set of new crates designed specifically for the kernel-centric architecture.
2.  **Reference, Don't Migrate:** The old codebase is a library of proven solutions, not a structure to be preserved. We will copy functions, logic, and data structures, but not entire files or modules without scrutiny.
3.  **Kernel-Centric Design:** The `aikit-kernel` is the application. All other components are libraries that serve it.
4.  **Unified Component Model:** All script-callable components (Tools, Agents, Workflows, etc.) will live in a single, unified crate for simplicity and discoverability.
5.  **Pervasive Tracing:** Observability using the `tracing` crate will be designed into every layer from the beginning.

---

### **Part 1: The New Greenfield Architecture**

The new architecture is designed for clarity, separation of concerns, and maintainability. It consolidates 20+ old crates into a logical set of five.

**The 5-Crate Structure:**

1.  **`aikit-core` (The Common Language):** A foundational library with no workspace dependencies.
2.  **`aikit-kernel` (The Central Nervous System):** The main application. It runs the show.
3.  **`aikit-runtime` (The Execution Engine):** A library for abstracting script language execution.
4.  **`aikit-components` (The Toolbox):** A unified library for all script-callable functionality.
5.  **`aikit-cli` (The User Interface):** The client application for interacting with the kernel.

---

#### **Architectural Deep Dive: Crate by Crate**

Here is the detailed breakdown of each crate's purpose, internal structure, and responsibilities.

##### **1. Crate: `aikit-core`**

*   **Purpose:** To define the fundamental, shared traits and data types that the entire workspace depends on. This prevents circular dependencies and creates a common language for all other crates.
*   **Directory Structure:**
    ```
    aikit-core/
    └── src/
        ├── lib.rs
        ├── traits.rs      # Defines BaseAgent, Tool, Workflow, ScriptEngineBridge
        ├── types.rs       # Defines AgentInput, AgentOutput, ComponentMetadata, etc.
        └── error.rs       # Defines the primary AikitError enum and variants
    ```
*   **Logic to Reference:** The contents of `~/projects/lexlapax/rs-llmspell/llmspell-core/` can be copied almost verbatim, as these interfaces are proven.

##### **2. Crate: `aikit-kernel`**

*   **Purpose:** The main application binary and the heart of the system. It manages the primary event loop, communication protocols, and core infrastructure like events, hooks, and debugging.
*   **Directory Structure:**
    ```
    aikit-kernel/
    └── src/
        ├── main.rs        # Entry point for the kernel binary
        ├── lib.rs
        ├── kernel.rs      # The IntegratedKernel struct, main message loop
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
*   **Key Responsibilities & Referenced Logic:**
    *   **`kernel.rs`:** Contains the main `async` loop that listens for Jupyter messages. This is the orchestrator.
    *   **`runtime.rs`:** Implements the `GlobalIoRuntime` to solve the async context problem. (Logic from `phase-09-design-doc.md`).
    *   **`transport.rs`:** Implements the full 5-channel Jupyter protocol. (Logic from `~/projects/lexlapax/rs-llmspell/llmspell-kernel/src/transport/zeromq.rs`).
    *   **`events.rs` & `hooks.rs`:** The **Hook and Event systems** live here. They are core kernel functions, not optional components. (Logic from `~/projects/lexlapax/rs-llmspell/llmspell-hooks/` and `llmspell-events/`).
    *   **`debug/`:** The entire **Debugging infrastructure** is a core part of the kernel, responsible for pausing, inspecting, and controlling execution. (Logic from `~/projects/lexlapax/rs-llmspell/llmspell-bridge/src/debug/` and `llmspell-kernel/src/dap_bridge.rs`).

##### **3. Crate: `aikit-runtime`**

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
            └── engine.rs  # The LuaEngine implementation
    ```
*   **Key Responsibilities & Referenced Logic:**
    *   **`runtime.rs`:** The `ScriptRuntime` is responsible for taking code as a string, handing it to the correct engine, and injecting the necessary global APIs. (Logic from `~/projects/lexlapax/rs-llmspell/llmspell-bridge/src/script_runtime.rs`).
    *   **`lua/engine.rs`:** The concrete implementation for Lua. (Logic from `~/projects/lexlapax/rs-llmspell/llmspell-bridge/src/lua/`).

##### **4. Crate: `aikit-components`**

*   **Purpose:** A single, unified library for **all** script-callable functionality. This is the biggest consolidation, replacing over a dozen old crates.
*   **Directory Structure:**
    ```
    aikit-components/
    └── src/
        ├── lib.rs
        ├── registry.rs    # ComponentRegistry to discover all components in this crate
        ├── config.rs      # All configuration structs (replaces llmspell-config)
        ├── security.rs    # Sandboxing, policies, and multi-tenancy logic
        ├── storage/
        │   ├── mod.rs
        │   ├── engine.rs    # The StorageEngine trait and backends (memory, sled)
        │   └── vector.rs    # Vector storage backend (HNSW)
        ├── state.rs       # The StateManager implementation
        ├── sessions.rs    # The SessionManager and artifact management
        ├── tools/         # Module containing all 37+ tools
        │   ├── mod.rs
        │   ├── file_ops.rs
        │   └── web_search.rs # ... etc
        ├── agents/
        │   ├── mod.rs
        │   └── chat.rs      # Example: ChatAgent
        ├── workflows/
        │   ├── mod.rs
        │   └── sequential.rs # Example: SequentialWorkflow
        └── rag/
            ├── mod.rs
            ├── pipeline.rs  # The RAG pipeline orchestrator
            ├── embedding.rs # Embedding provider integration
            └── chunking.rs  # Document chunking strategies
    ```
*   **Key Responsibilities & Referenced Logic:**
    *   This crate is the new home for the logic from `llmspell-config`, `llmspell-security`, `llmspell-tenancy`, `llmspell-storage`, `llmspell-state-persistence`, `llmspell-sessions`, `llmspell-tools`, `llmspell-agents`, `llmspell-workflows`, and `llmspell-rag`.
    *   **`config.rs`:** Defines all `struct`s for `aikit.toml` parsing.
    *   **`security.rs`:** Contains the logic for sandboxing script execution and the **Multi-Tenancy and Security** policies.
    *   **`storage/`:** Implements the **Storage Engine and Persistence**, including both key-value stores (`sled`) and vector stores (`HNSW`).
    *   **`state.rs` & `sessions.rs`:** Implement the **State and Session** management logic, using the storage engines.
    *   **`rag/`:** Implements the complete **RAG** system.
    *   **`tools/`, `agents/`, `workflows/`:** Contain the implementations for all **Tools, Agents, and Workflows**.

##### **5. Crate: `aikit-cli`**

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

---

### **Part 2: The New Phased Implementation Plan**

This plan builds the architecture defined above, step-by-step.

#### **Phase 0: Scaffolding & Core Interfaces (2 Days)**

*   **Task 0.1:** Create the 5-crate Cargo workspace with the exact directory structures laid out above.
*   **Task 0.2:** Populate `aikit-core` by copying the proven traits (`BaseAgent`, `Tool`, etc.) and types from the old codebase.

#### **Phase 1: The Kernel's Skeleton (3 Days)**

*   **Task 1.1:** In `aikit-kernel`, implement the `GlobalIoRuntime` and the basic `IntegratedKernel` struct.
*   **Task 1.2:** In `aikit-kernel`, implement the 5-channel Jupyter `transport.rs`.
*   **Task 1.3:** In `aikit-cli`, implement the `serve` command to start the kernel, which will listen for messages but do nothing with them yet.

#### **Phase 2: The Scripting Heartbeat (3 Days)**

*   **Task 2.1:** In `aikit-runtime`, implement the `ScriptRuntime` and the `LuaEngine`.
*   **Task 2.2:** In `aikit-kernel`, integrate the `ScriptRuntime`. The kernel's main loop will now pass any `execute_request` code to the runtime.
*   **Outcome:** The kernel can now execute a "hello world" Lua script.

#### **Phase 3: Building the Component Library - Part 1 (Config, Tools, Providers) (4 Days)**

*   **Task 3.1:** In `aikit-components`, implement `config.rs` to load and parse `aikit.toml`.
*   **Task 3.2:** In `aikit-components`, implement the `rig` provider integration, ensuring it uses the kernel's `GlobalIoRuntime`.
*   **Task 3.3:** In `aikit-components/tools/`, implement a representative set of tools (e.g., `file_ops`, `web_search`).
*   **Task 3.4:** In `aikit-runtime`, implement the API injection logic so the `Tool` and `Config` globals are available in Lua scripts.

#### **Phase 4: Building the Component Library - Part 2 (Agents & Workflows) (3 Days)**

*   **Task 4.1:** In `aikit-components`, implement the `agents` and `workflows` modules.
*   **Task 4.2:** In `aikit-runtime`, enhance the API injection to make the `Agent` and `Workflow` globals available in scripts.
*   **Task 4.3:** In `aikit-components`, implement the `registry.rs` to allow components to be discovered and managed.

#### **Phase 5: Implementing Core Infrastructure (Hooks, Events, State, Sessions) (5 Days)**

*   **Task 5.1:** In `aikit-kernel`, implement the `events.rs` (Event Bus) and `hooks.rs` (Hook System). Instrument the kernel's execution loop to fire `PreExecute`/`PostExecute` hooks.
*   **Task 5.2:** In `aikit-components/storage/`, implement the `engine.rs` with `memory` and `sled` backends.
*   **Task 5.3:** In `aikit-components`, implement `state.rs` (StateManager) and `sessions.rs` (SessionManager), which use the new storage engine.
*   **Task 5.4:** In `aikit-runtime`, inject the `State` and `Session` globals into scripts.

#### **Phase 6: Implementing Advanced Features (RAG, Security, Tenancy) (4 Days)**

*   **Task 6.1:** In `aikit-components/storage/`, implement `vector.rs` with the HNSW backend.
*   **Task 6.2:** In `aikit-components/rag/`, implement the full RAG pipeline (chunking, embedding, retrieval).
*   **Task 6.3:** In `aikit-components/security.rs`, implement the sandboxing, access control, and **multi-tenancy** logic. The `StateScope` and tenant isolation policies will live here.
*   **Task 6.4:** In `aikit-runtime`, inject the `RAG` global and ensure the `ExecutionContext` passed to all components contains the security/tenant context.

#### **Phase 7: The Developer Experience & Final Validation (3 Days)**

*   **Task 7.1:** In `aikit-kernel/debug/`, implement the `DebugCoordinator` and DAP bridge.
*   **Task 7.2:** In `aikit-cli`, build out the full `repl` and `run` commands.
*   **Task 7.3:** Write comprehensive end-to-end integration tests and update all project documentation (`README.md`, `GEMINI.md`) to reflect the new, clean architecture.
