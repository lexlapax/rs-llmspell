# Phase 14: Web Interface - Design Document

**Version**: 1.3
**Date**: December 2025
**Status**: IMPLEMENTATION IN PROGRESS
**Phase**: 14 (Web Interface)
**Timeline**: Weeks 55-58 (4 weeks)
**Priority**: HIGH (User Experience & Accessibility)
**Dependencies**: Phase 13c (Storage Consolidation) ✅, Phase 10 (Service Integration) ✅

> **📋 Web Interface**: This phase implements a comprehensive HTTP API and browser-based web interface as a single binary, enabling visual management of agents, real-time streaming, and broader accessibility.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Strategic Context](#strategic-context)
3. [Architecture Overview](#architecture-overview)
4. [UX Architecture](#ux-architecture)
5. [Configuration Schema](#configuration-schema)
6. [Dependencies](#dependencies)
7. [Detailed Architecture](#detailed-architecture)
8. [Daemon Integration](#daemon-integration)
9. [Authentication & Security](#authentication--security)
10. [API Schema Definitions](#api-schema-definitions)
11. [Example Usage](#example-usage)
12. [llmspell-web Crate Structure](#llmspell-web-crate-structure)
13. [Testing Strategy](#testing-strategy)
14. [Documentation Requirements](#documentation-requirements)
15. [Risk Assessment](#risk-assessment)
16. [Alternatives Considered](#alternatives-considered)
17. [Phase 15+ Implications](#phase-15-implications)
18. [Implementation Plan](#implementation-plan)

---

## Executive Summary

### The Accessibility Gap

**Problem Statement**: `rs-llmspell` has matured into a powerful AI experimentation platform, but its CLI-only nature limits accessibility and visualization. Users cannot easily visualize memory graphs, monitor real-time agent interactions, or develop scripts in a rich integrated environment without external tools.

**Phase 14 Solution**: A unified, single-binary web interface providing a "Mission Control" for AI agents. This includes a high-performance HTTP/WebSocket API and an embedded React/WASM frontend, enabling visual interaction with zero additional deployment complexity.

### Quantitative Targets

| Metric | Current (CLI) | Target (Web) | Improvement |
|--------|---------------|--------------|-------------|
| **Script Execution** | Terminal Output | Real-time Streaming UI | Visual Feedback |
| **Agent Monitoring** | Log Files | Live Dashboard | Real-time Observability |
| **Memory Inspection** | JSON Dumps | Interactive Graph | Visual Exploration |
| **Session Replay** | Text Replay | Interactive Timeline | Rich Context |
| **API Latency** | N/A | <100ms (P95) | High Responsiveness |
| **Binary Size** | ~35MB | <50MB | Minimal Bloat (+15MB) |
| **Setup Time** | <1 min | <1 min | Zero Config Overhead |

### Key Deliverables

1.  **`llmspell-web` Crate**: A new core crate hosting the Axum web server and embedded frontend assets.
2.  **Unified API**: RESTful endpoints for resource management and WebSockets for real-time streaming.
3.  **Embedded Frontend**: A modern web UI bundled directly into the `llmspell` binary.
4.  **Daemon Management**: Robust background process management for long-running server instances.
5.  **Library/Spells Page** (NEW): Turn-key template catalog exposing Phase 12 templates with Launch action.
6.  **Settings/Configuration** (NEW): UI for Phase 13c 18-layer profile system and provider management.
7.  **Workflow Visualizer** (NEW): Graph/tree visualization of workflow execution steps.

---

## Strategic Context

### Why Now?

Phase 13 completed the "backend" revolution (Memory, Context, Storage). The platform is now capable of complex, stateful interactions that are difficult to debug or appreciate via text logs alone. Phase 14 bridges this gap, exposing the platform's power to the user visually.

### Architecture Principles

-   **Single Binary Distribution**: Frontend assets embedded via `rust-embed`, no separate Node.js process required.
-   **Leverage Daemon Infrastructure**: Reuse existing signal handling, PID management, graceful shutdown from `llmspell-kernel/src/daemon/`.
-   **CLI-Managed Lifecycle**: Web server started/stopped via `llmspell web` subcommands.
-   **Zero External Dependencies**: Self-contained binary with embedded static files.

---

## Architecture Overview

### System Diagram

```mermaid
graph TD
    User[User / Browser] <-->|HTTP/WS| WebServer[Axum Web Server]
    
    subgraph "llmspell Binary"
        WebServer -->|Embeds| Assets[Static Assets (rust-embed)]
        WebServer -->|Calls| Kernel[Script Kernel]
        WebServer -->|Queries| Storage[Storage Layer]
        
        Kernel -->|Events| EventBus[Event Bus]
        EventBus -->|Stream| WebServer
    end
```

### Component Stack

-   **Web Framework**: `axum` (Tokio ecosystem, high performance, ergonomic).
-   **Async Runtime**: `tokio` (Standard for Rust async).
-   **Serialization**: `serde` + `serde_json` (JSON handling).
-   **Streaming**: `tokio-stream` + `axum::extract::ws` (WebSockets).
-   **Asset Embedding**: `rust-embed` (Compile-time asset inclusion).

---

## UX Architecture

### Design Philosophy

The web interface addresses two critical user problems:

1. **"0-day Retention Problem"**: New users need immediate, actionable entry points (solved by Library/Spells)
2. **"Configuration Blindness"**: Users can't use features without setting up providers (solved by Settings)

### Navigation Model

```
┌─────────────────────────────────────────────────────────────────┐
│  Dashboard     │ Metrics, system health, quick actions         │
├─────────────────────────────────────────────────────────────────┤
│  Library       │ Template/Spell catalog with Launch action     │
│  (Spells)      │ → Cards for 6+ templates (Phase 12)           │
│                │ → Config modal → Creates Session               │
├─────────────────────────────────────────────────────────────────┤
│  Sessions      │ Execution history + Timeline replay           │
│                │ → What's running, what ran, replay events     │
├─────────────────────────────────────────────────────────────────┤
│  Agents        │ Active/Sleeping instances (process view)      │
│                │ → Runtime management, not catalog             │
├─────────────────────────────────────────────────────────────────┤
│  Memory        │ [Graph] [Sources] [Explorer] (tabs)           │
│                │ → Graph: Semantic visualization (Phase 13)    │
│                │ → Sources: Document upload/management (RAG)   │
│                │ → Explorer: Vector search debug               │
├─────────────────────────────────────────────────────────────────┤
│  Tools         │ Capability reference + Playground             │
│                │ → 40+ tools catalog                           │
│                │ → Test in isolation (REPL-like)               │
├─────────────────────────────────────────────────────────────────┤
│  Settings      │ Configuration management                      │
│                │ → Providers (API keys, models)                │
│                │ → Limits (memory, steps, timeouts)            │
│                │ → Sandbox (tool permissions)                  │
│                │ → Profiles (active layers visualization)      │
└─────────────────────────────────────────────────────────────────┘
```

### User Journey Mapping

**Casual User (Task-Focused)**:
```
Dashboard → Library → Browse spells → Configure → Launch → Sessions → Results
```

**Power User (Building)**:
```
Tools → Playground → Test tool → Memory (Sources) → Upload docs → Library → Launch
```

**Developer (Creating)**:
```
Tools → Editor → Write custom workflow → Test → Sessions → Debug with Timeline
```

**Admin (Configuring)**:
```
Settings → Providers → Add Ollama → Limits → Set memory cap → Dashboard → Verify
```

### Key UX Components

#### 1. Library/Spells (P0 - Critical)
The primary entry point for new users. Exposes Phase 12 templates:
- **Research Assistant**: Multi-agent research with synthesis
- **Interactive Chat**: Session-based conversation
- **Data Analysis**: Stats + visualization agents
- **Code Generator**: Spec → impl → test chain
- **Document Processor**: PDF/OCR + transformation
- **Workflow Orchestrator**: Custom patterns

Each template has:
- Card view with description and category
- Configuration modal for input parameters
- "Launch" action that creates a Session

#### 2. Settings/Configuration (P0 - Critical)
Exposes the Phase 13c 18-layer profile system:
- **Provider Management**: API keys, model selection, health status
- **System Limits**: Memory caps, step limits, timeouts
- **Tool Sandbox**: Permission levels (Safe/Restricted/Privileged)
- **Profile Visualization**: Active layers, override chain

#### 3. Agents Instance View (P1)
Clarifies the runtime vs. catalog distinction:
- Shows **Active/Sleeping/Terminated** agent instances
- Instance controls (Stop, Restart)
- Links instances to their source Session
- Not just a list of agent types

#### 4. Workflow Visualizer (P1)
Visualizes Phase 3.3 workflow patterns:
- Graph/Tree view of workflow steps
- Step status indication (Pending, Running, Completed, Failed)
- Click-to-inspect step details
- Integrates with Sessions Timeline

#### 5. Knowledge Base Manager (P2/P3)
Expands Memory page with tabs:
- **Graph Tab**: Existing semantic visualization
- **Sources Tab**: Document upload, collection management
- **Explorer Tab**: Vector search testing, debug queries

---

## Configuration Schema

The web interface is configured via the standard `llmspell.toml` file under a new `[web]` section.

```toml
# llmspell.toml

[web]
enabled = true
port = 8080
host = "127.0.0.1"
cors_allowed_origins = ["http://localhost:3000"] # For dev
jwt_secret = "change_me_in_production"          # For session tokens
static_dir = "frontend/dist"                    # Optional override for dev

[web.limits]
max_payload_size = "10MB"
max_concurrent_connections = 100
request_timeout_seconds = 30
rate_limit_per_minute = 600

[web.logging]
level = "info"
format = "json"
```

---

## Dependencies

The `llmspell-web` crate will require the following dependencies:

```toml
[dependencies]
# Web Server
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.4", features = ["util", "timeout", "limit"] }
tower-http = { version = "0.5", features = ["cors", "trace", "fs", "compression-br"] }
tokio = { version = "1.36", features = ["full"] }
tokio-stream = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Asset Embedding
rust-embed = "8.2"
mime_guess = "2.0"

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5" # For API key hashing if needed

# Observability
tracing = "0.1"
metrics = "0.22"
```

---

## Detailed Architecture

### 1. Web Server Entry Point (`llmspell-web/src/lib.rs`)

```rust
use axum::{Router, routing::{get, post}};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub struct WebServer {
    config: WebConfig,
    kernel: Arc<Kernel>,
}

impl WebServer {
    pub async fn run(&self) -> Result<()> {
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/api/scripts/execute", post(execute_script))
            .route("/ws", get(ws_handler))
            .fallback(static_handler)
            .layer(CorsLayer::permissive())
            .with_state(AppState {
                kernel: self.kernel.clone(),
            });

        let addr = SocketAddr::from((self.config.host, self.config.port));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
        
        Ok(())
    }
}
```

### 2. Application State (`llmspell-web/src/state.rs`)

```rust
#[derive(Clone)]
pub struct AppState {
    pub kernel: Arc<Kernel>,
    pub event_bus: Arc<EventBus>,
    pub session_manager: Arc<SessionManager>,
}
```

### 3. WebSocket Handler (`llmspell-web/src/handlers/ws.rs`)

```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Subscribe to kernel events
    let mut rx = state.event_bus.subscribe();
    
    while let Ok(event) = rx.recv().await {
        let msg = serde_json::to_string(&event).unwrap();
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
```

### 4. Static Asset Serving (`llmspell-web/src/handlers/static.rs`)

```rust
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            // SPA Fallback
            Assets::get("index.html")
                .map(|c| ([(header::CONTENT_TYPE, "text/html")], c.data))
                .into_response()
        }
    }
}
```

---

## Daemon Integration

The web server integrates with the existing `llmspell-kernel` daemon infrastructure to ensure consistent lifecycle management.

### 1. Lifecycle Hooks
The `llmspell web start --daemon` command will:
1.  Check for existing PID file (`~/.llmspell/run/web.pid`).
2.  Fork the process (or start background task).
3.  Write new PID.
4.  Register signal handlers (`SIGTERM`, `SIGINT`).

### 2. Signal Handling
We leverage `tokio::signal` to handle graceful shutdowns.

```rust
// llmspell-web/src/server.rs

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    info!("Signal received, starting graceful shutdown...");
}
```

### 3. Kernel Mode
When running in web mode, the `Kernel` is initialized in **Server Mode**, which may differ from CLI mode (e.g., persistent session storage enabled by default, different logging outputs).

---

## Authentication & Security

### 1. API Key Management
-   **Storage**: API keys are stored in the `llmspell-config` (hashed in `llmspell.toml` or `secrets.toml`) or an SQLite `api_keys` table for multi-user setups.
-   **Header**: Requests must include `X-API-Key: <key>`.
-   **Rate Limiting**: Rate limits are enforced per API key using `tower-governor` or similar middleware.

### 2. JWT for Sessions
-   **Login**: A `POST /api/auth/login` endpoint exchanges an API Key for a short-lived JWT (JSON Web Token).
-   **Usage**: The JWT is sent in the `Authorization: Bearer <token>` header for subsequent requests.
-   **Refresh**: A refresh token mechanism allows keeping sessions alive without re-sending the API key.

### 3. Security Best Practices
-   **CORS**: Strict origin validation (configurable).
-   **Binding**: Default bind to `127.0.0.1`.
-   **Payload Limits**: Enforced max body size (10MB) to prevent DoS.

---

## API Schema Definitions

### 1. Error Response Schema
Standardized error format for all non-2xx responses.

```json
{
  "error": {
    "code": "SCRIPT_EXECUTION_FAILED",
    "message": "Lua syntax error on line 5",
    "details": {
      "line": 5,
      "column": 12,
      "snippet": "local x = "
    },
    "request_id": "req_892374"
  }
}
```

**HTTP Status Mapping**:
-   `LLMSpellError::Configuration` -> `400 Bad Request`
-   `LLMSpellError::Authentication` -> `401 Unauthorized`
-   `LLMSpellError::NotFound` -> `404 Not Found`
-   `LLMSpellError::ScriptExecution` -> `422 Unprocessable Entity`
-   `LLMSpellError::Internal` -> `500 Internal Server Error`

### 2. Execute Script (`POST /api/scripts/execute`)

**Request:**
```json
{
  "script": "local agent = Agent.new('researcher'); return agent:run('Analyze Rust async');",
  "language": "lua",
  "timeout_seconds": 60
}
```

**Response:**
```json
{
  "execution_id": "exec_12345",
  "status": "queued",
  "stream_url": "/ws/stream?exec_id=exec_12345"
}
```

### 3. List Sessions (`GET /api/sessions`)

**Response:**
```json
{
  "sessions": [
    {
      "id": "sess_abc123",
      "created_at": "2025-12-04T10:00:00Z",
      "agent_count": 2,
      "status": "active"
    }
  ],
  "pagination": {
    "page": 1,
    "total": 15
  }
}
```

### 4. Template APIs (NEW - Task 14.2.7)

#### List Templates (`GET /api/templates`)

**Response:**
```json
{
  "templates": [
    {
      "id": "research-assistant",
      "name": "Research Assistant",
      "description": "Multi-agent research with synthesis and validation",
      "category": "research",
      "version": "1.0.0",
      "tags": ["research", "multi-agent", "rag"]
    },
    {
      "id": "interactive-chat",
      "name": "Interactive Chat",
      "description": "Session-based conversation with tool integration",
      "category": "chat",
      "version": "1.0.0",
      "tags": ["chat", "conversation", "tools"]
    }
  ]
}
```

#### Get Template Schema (`GET /api/templates/:id`)

**Response:**
```json
{
  "id": "research-assistant",
  "name": "Research Assistant",
  "description": "Multi-agent research with synthesis and validation",
  "category": "research",
  "schema": {
    "type": "object",
    "properties": {
      "topic": {
        "type": "string",
        "description": "Research topic to investigate"
      },
      "max_sources": {
        "type": "integer",
        "default": 10,
        "description": "Maximum number of sources to gather"
      },
      "output_format": {
        "type": "string",
        "enum": ["markdown", "json", "html"],
        "default": "markdown"
      }
    },
    "required": ["topic"]
  }
}
```

#### Launch Template (`POST /api/templates/:id/launch`)

**Request:**
```json
{
  "params": {
    "topic": "Impact of AI regulation on startup ecosystems",
    "max_sources": 15,
    "output_format": "markdown"
  }
}
```

**Response:**
```json
{
  "session_id": "sess_xyz789",
  "template_id": "research-assistant",
  "status": "running",
  "stream_url": "/ws/stream?session_id=sess_xyz789"
}
```

### 5. Configuration APIs (NEW - Task 14.2.7)

#### Get Current Configuration (`GET /api/config`)

**Response:**
```json
{
  "active_profile": "development",
  "layers": [
    {"source": "builtin:minimal", "priority": 1},
    {"source": "builtin:development", "priority": 2},
    {"source": "user:~/.llmspell/profile.toml", "priority": 3},
    {"source": "env", "priority": 4}
  ],
  "providers": {
    "openai": {"status": "configured", "models": ["gpt-4", "gpt-3.5-turbo"]},
    "anthropic": {"status": "not_configured"},
    "ollama": {"status": "available", "models": ["llama3.1:8b", "mistral:7b"]}
  },
  "limits": {
    "max_memory_mb": 2048,
    "max_steps": 100,
    "timeout_seconds": 300
  }
}
```

#### Update Configuration (`PUT /api/config`)

**Request:**
```json
{
  "providers": {
    "openai": {
      "api_key": "sk-...",
      "default_model": "gpt-4"
    }
  },
  "limits": {
    "max_memory_mb": 4096
  }
}
```

**Response:**
```json
{
  "status": "updated",
  "restart_required": false
}
```

### 6. Document/Knowledge APIs (NEW - Task 14.2.7)

#### List Documents (`GET /api/documents`)

**Response:**
```json
{
  "documents": [
    {
      "id": "doc_abc123",
      "filename": "rust-async-book.pdf",
      "type": "pdf",
      "size_bytes": 1048576,
      "chunk_count": 42,
      "uploaded_at": "2025-12-04T10:00:00Z"
    }
  ],
  "total_chunks": 150,
  "total_size_bytes": 5242880
}
```

#### Upload Document (`POST /api/documents/upload`)

**Request:** `multipart/form-data` with file

**Response:**
```json
{
  "id": "doc_def456",
  "filename": "research-paper.pdf",
  "status": "processing",
  "chunks_created": 0
}
```

#### Search Documents (`POST /api/documents/search`)

**Request:**
```json
{
  "query": "async await rust concurrency",
  "top_k": 5,
  "filters": {
    "document_ids": ["doc_abc123"]
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "chunk_id": "chunk_001",
      "document_id": "doc_abc123",
      "content": "Async/await in Rust provides...",
      "score": 0.92,
      "metadata": {"page": 15, "section": "Chapter 3"}
    }
  ]
}
```

---

## Example Usage

### cURL Example
Execute a simple Lua script via the API.

```bash
curl -X POST http://localhost:8080/api/scripts/execute \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your_secret_key" \
  -d '{
    "script": "return 1 + 1", 
    "language": "lua"
  }'
```

### WebSocket Example (JavaScript)
Connect to the streaming endpoint to receive real-time updates.

```javascript
const ws = new WebSocket('ws://localhost:8080/ws/stream?exec_id=exec_12345');

ws.onopen = () => {
    console.log('Connected to stream');
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    if (data.type === 'token') {
        process.stdout.write(data.content);
    } else if (data.type === 'log') {
        console.log(`[LOG] ${data.message}`);
    }
};

ws.onerror = (error) => {
    console.error('WebSocket Error:', error);
};
```

---

## llmspell-web Crate Structure

```
llmspell-web/
├── Cargo.toml              # Dependencies (axum, tokio, serde, rust-embed)
├── build.rs                # Build script (optional frontend build trigger)
├── src/
│   ├── lib.rs              # Library entry point
│   ├── server.rs           # Server startup logic & signal handling
│   ├── state.rs            # AppState definition
│   ├── error.rs            # WebError types & IntoResponse impls
│   ├── middleware/         # Auth, Logging, CORS, Metrics
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── logging.rs
│   │   └── metrics.rs
│   ├── handlers/           # Route handlers
│   │   ├── mod.rs
│   │   ├── scripts.rs
│   │   ├── agents.rs
│   │   ├── sessions.rs
│   │   ├── memory.rs
│   │   ├── tools.rs
│   │   ├── templates.rs    # NEW - Task 14.2.7
│   │   ├── config.rs       # NEW - Task 14.2.7
│   │   ├── documents.rs    # NEW - Task 14.2.7
│   │   ├── ws.rs
│   │   ├── metrics.rs
│   │   └── assets.rs       # static file serving
│   └── models/             # API Request/Response structs
│       ├── mod.rs
│       ├── script.rs
│       ├── template.rs     # NEW
│       ├── config.rs       # NEW
│       ├── document.rs     # NEW
│       └── ...
└── frontend/               # Source for frontend
    ├── package.json
    ├── vite.config.ts
    └── src/
        ├── App.tsx
        ├── api/
        │   ├── client.ts
        │   └── types.ts
        ├── components/
        │   ├── Layout.tsx
        │   ├── editor/
        │   │   ├── CodeEditor.tsx
        │   │   └── Console.tsx
        │   ├── memory/
        │   │   ├── MemoryGraph.tsx
        │   │   └── DocumentList.tsx    # NEW - Task 14.3.11
        │   ├── session/
        │   │   └── Timeline.tsx
        │   ├── templates/              # NEW - Task 14.3.8
        │   │   ├── TemplateCard.tsx
        │   │   └── ConfigModal.tsx
        │   ├── workflow/               # NEW - Task 14.3.9
        │   │   └── WorkflowGraph.tsx
        │   └── widgets/
        │       ├── SystemStatus.tsx
        │       ├── RecentActivity.tsx
        │       ├── QuickActions.tsx
        │       └── ProviderStatus.tsx  # NEW - Task 14.3.10
        ├── hooks/
        │   ├── useWebSocket.ts
        │   └── useSystemStatus.ts
        └── pages/
            ├── Dashboard.tsx
            ├── Sessions.tsx
            ├── Memory.tsx
            ├── Agents.tsx
            ├── Tools.tsx
            ├── Library.tsx             # NEW - Task 14.3.8
            └── Settings.tsx            # NEW - Task 14.3.7
```

---

## Testing Strategy

### 1. Unit Tests
Test individual handlers and logic in isolation.

```rust
// llmspell-web/src/handlers/scripts.rs

#[tokio::test]
async fn test_execute_script_handler() {
    let state = AppState::mock(); // Mock kernel/event bus
    let payload = ExecuteScriptRequest {
        script: "return 1".to_string(),
        language: "lua".to_string(),
        ..Default::default()
    };
    
    let response = execute_script(State(state), Json(payload)).await;
    assert!(response.is_ok());
    // Assert response body contains execution_id
}
```

### 2. Integration Tests
Test the full API stack with a real (but test-configured) kernel.

```rust
// tests/api_integration.rs

#[tokio::test]
async fn test_full_execution_flow() {
    // Start test server
    let server = TestServer::new().await;
    let client = server.client();
    
    // 1. Submit script
    let resp = client.post("/api/scripts/execute")
        .json(&json!({"script": "return 'hello'"}))
        .send()
        .await;
    assert_eq!(resp.status(), 200);
    
    let exec_id = resp.json::<ExecuteResponse>().await.execution_id;
    
    // 2. Connect WS
    let mut ws = client.ws_connect(format!("/ws/stream?exec_id={}", exec_id)).await;
    
    // 3. Verify stream
    let msg = ws.recv_json().await;
    assert_eq!(msg.type, "result");
    assert_eq!(msg.content, "hello");
}
```

### 3. E2E Browser Tests
Use Playwright to verify the frontend integration.
-   **Tools**: Playwright (Node.js or Python).
-   **Scenarios**:
    -   Load dashboard.
    -   Type script in editor -> Run -> Verify output in console.
    -   Navigate to Memory Graph -> Click node -> Verify details pane.

### 4. Load Testing
Use `k6` or `criterion` to benchmark API throughput.
-   **Target**: 100 concurrent script executions.
-   **Metric**: P95 latency < 100ms for API overhead.

---

## Documentation Requirements

### 1. User Guide (`docs/user-guide/web-interface.md`)
-   **Installation**: How to enable the web feature (if behind a flag) or download the binary.
-   **Quick Start**: `llmspell web start` and navigating the UI.
-   **Configuration**: Detailed explanation of `[web]` toml settings.
-   **Security**: Best practices for exposing the interface (tunneling, auth).

### 2. API Reference
-   **Format**: OpenAPI 3.0 (Swagger).
-   **Generation**: Use `utoipa` crate to generate OpenAPI spec from Rust structs/handlers automatically.
-   **Location**: Hosted at `/api/docs` (Swagger UI) and `docs/api/openapi.json`.

---

## Risk Assessment

### 1. Security Risks
-   **Risk**: Exposing the API to the public internet could allow arbitrary code execution.
-   **Mitigation**: Bind to `127.0.0.1` by default. Require API Key. Strict CORS.
-   **Warning**: Documentation must clearly state this is a developer tool.

### 2. Performance Risks
-   **Risk**: Large static assets bloating the binary size.
-   **Mitigation**: Use aggressive compression (brotli/gzip). Optimize frontend build.
-   **Risk**: WebSocket connection limits.
-   **Mitigation**: Use `ulimit` checks and configurable connection caps.

### 3. Stability Risks
-   **Risk**: Web server panic taking down the entire daemon.
-   **Mitigation**: Run web server in a separate tokio task with `CatchUnwind`.

---

## Alternatives Considered

### 1. Web Framework: Axum vs. Actix-web vs. Warp
-   **Decision**: **Axum**.
-   **Rationale**: Axum is part of the official Tokio ecosystem, shares the `Service` trait with Tower (middleware), and has excellent ergonomics. Actix-web is faster but has a separate runtime model (Actors). Warp is functional but can have complex type signatures.

### 2. Asset Embedding: rust-embed vs. include_dir
-   **Decision**: **rust-embed**.
-   **Rationale**: Proven track record, simple API, supports compression, and file metadata (MIME types) easily.

### 3. Frontend: React vs. Leptos (Rust)
-   **Decision**: **React** (initially).
-   **Rationale**: The ecosystem for React (Monaco Editor, Visualization libraries like React Flow) is vastly more mature than Leptos. We need a rich "Mission Control" UI, and React allows us to leverage existing high-quality components. We can migrate to WASM later if needed.

---

## Phase 15+ Implications

### Enabling MCP (Phase 15)
The HTTP/WebSocket infrastructure is a **direct prerequisite** for the Model Context Protocol (MCP).
-   **Transport**: MCP uses JSON-RPC over WebSocket/SSE, which `llmspell-web` implements.
-   **Tool Exposure**: The API endpoints for tool calling (`POST /api/tools/...`) will map directly to MCP tool exposure.

### Enabling Agent-to-Agent (A2A) (Phase 16)
-   **Remote Agents**: The web server allows `llmspell` instances to communicate over HTTP, forming the backbone of a distributed agent mesh.

---

## Implementation Plan

### Phase 14.1: Foundation & Crate Setup ✅ COMPLETE
-   [x] Create `llmspell-web` crate (Task 14.1.1)
-   [x] Implement core web server with health check (Task 14.1.2)
-   [x] Create web configuration profiles (Task 14.1.3)

### Phase 14.2: HTTP Backend ✅ COMPLETE + IN PROGRESS
-   [x] Implement Script Execution API (Task 14.2.1)
-   [x] Implement WebSocket Streaming (Task 14.2.2)
-   [x] Implement Resource APIs - Sessions, Memory (Task 14.2.3)
-   [x] Implement Agent & Tool APIs (Task 14.2.4)
-   [x] Error Handling & Response Types (Task 14.2.5)
-   [x] Metrics Endpoint (Task 14.2.6)
-   [ ] **NEW: Template, Config, Document APIs** (Task 14.2.7) - P0

### Phase 14.3: Frontend Integration - IN PROGRESS

#### Completed Tasks ✅
-   [x] Initialize Frontend Project (Task 14.3.1)
-   [x] Dashboard Layout & Navigation (Task 14.3.2a)
-   [x] Dashboard Widgets (Task 14.3.2b)
-   [x] Dashboard API Integration (Task 14.3.2c)
-   [x] Monaco Editor Integration (Task 14.3.3a)
-   [x] WebSocket Hook & State (Task 14.3.3b)
-   [x] Console Component (Task 14.3.3c)
-   [x] Embed Frontend Assets (Task 14.3.4)
-   [x] Memory Graph Visualization (Task 14.3.5)
-   [x] Session Timeline (Task 14.3.6)

#### New UX Tasks (from Analysis)

**P0 - Critical (Retention/Usability)**:
-   [ ] **Configuration Manager** (Task 14.3.7) - Settings page with provider management, profile visualization
-   [ ] **Template Library (Spells)** (Task 14.3.8) - Gallery of Phase 12 templates with Launch action

**P1 - High (Core Features)**:
-   [ ] **Workflow Visualizer** (Task 14.3.9) - Graph/tree visualization of workflow steps
-   [ ] **Navigation Enhancement** (Task 14.3.12) - Update sidebar for new pages (Library, Settings)
-   [ ] **Agents Instance View** (Task 14.3.13) - Clarify runtime instances vs. catalog

**P2 - Medium (Enhancements)**:
-   [ ] **Provider Status Widget** (Task 14.3.10) - LLM provider health indicator
-   [ ] **Knowledge Base Manager** (Task 14.3.11) - Memory page tabs for document management

### Phase 14.4: Security & Daemon Integration
-   [ ] Implement Authentication (Task 14.4.1)
-   [ ] Daemon Lifecycle Integration (Task 14.4.2)
-   [ ] CLI Web Subcommand (Task 14.4.3)

### Phase 14.5: Testing & Documentation
-   [ ] Comprehensive Testing (Task 14.5.1)
-   [ ] Documentation & Polish (Task 14.5.2)
-   [ ] OpenAPI Generation (Task 14.5.3)

### Implementation Priority Order

```
1. Task 14.2.7  Backend APIs (P0)         ← Enables frontend features
2. Task 14.3.7  Configuration Manager (P0) ← Users need this first
3. Task 14.3.8  Template Library (P0)      ← Main entry point
4. Task 14.3.12 Navigation Enhancement (P1)← Wires it together
5. Task 14.3.13 Agents Instance View (P1)  ← Clarifies UX
6. Task 14.3.9  Workflow Visualizer (P1)   ← Enhances Sessions
7. Task 14.3.10 Provider Status Widget (P2)← Nice-to-have
8. Task 14.3.11 Knowledge Base Manager (P2)← Completes Memory
9. Task 14.4.*  Security & Daemon          ← Production readiness
10. Task 14.5.* Testing & Docs             ← Release quality
```
