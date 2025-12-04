# Phase 14: Web Interface - Design Document

**Version**: 1.2
**Date**: December 2025
**Status**: DESIGN COMPLETE - Ready for Implementation
**Phase**: 14 (Web Interface)
**Timeline**: Weeks 55-58 (4 weeks)
**Priority**: HIGH (User Experience & Accessibility)
**Dependencies**: Phase 13c (Storage Consolidation) ✅, Phase 10 (Service Integration)

> **📋 Web Interface**: This phase implements a comprehensive HTTP API and browser-based web interface as a single binary, enabling visual management of agents, real-time streaming, and broader accessibility.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Strategic Context](#strategic-context)
3. [Architecture Overview](#architecture-overview)
4. [Configuration Schema](#configuration-schema)
5. [Dependencies](#dependencies)
6. [Detailed Architecture](#detailed-architecture)
7. [Daemon Integration](#daemon-integration)
8. [Authentication & Security](#authentication--security)
9. [API Schema Definitions](#api-schema-definitions)
10. [Example Usage](#example-usage)
11. [llmspell-web Crate Structure](#llmspell-web-crate-structure)
12. [Testing Strategy](#testing-strategy)
13. [Documentation Requirements](#documentation-requirements)
14. [Risk Assessment](#risk-assessment)
15. [Alternatives Considered](#alternatives-considered)
16. [Phase 15+ Implications](#phase-15-implications)
17. [Implementation Plan](#implementation-plan)

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
│   ├── middleware/         # Auth, Logging, CORS
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── logging.rs
│   ├── handlers/           # Route handlers
│   │   ├── mod.rs
│   │   ├── scripts.rs
│   │   ├── agents.rs
│   │   ├── sessions.rs
│   │   ├── ws.rs
│   │   └── static_files.rs
│   └── models/             # API Request/Response structs
│       ├── mod.rs
│       └── ...
└── frontend/               # Source for frontend
    ├── package.json
    ├── vite.config.ts
    └── src/
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

### Phase 14a: HTTP Backend (Weeks 55-56)
-   [ ] Create `llmspell-web` crate.
-   [ ] Implement Axum server with basic health check.
-   [ ] Implement API endpoints for Scripts, Agents, and Tools.
-   [ ] Implement WebSocket streaming for kernel events.
-   [ ] Integrate with `llmspell-cli` (`llmspell web start`).

### Phase 14b: Web Frontend (Weeks 57-58)
-   [ ] Initialize React/Vite project in `llmspell-web/frontend`.
-   [ ] Implement "Mission Control" dashboard.
-   [ ] Implement Script Editor (Monaco).
-   [ ] Integrate `rust-embed` to bundle `dist/` into binary.
-   [ ] Polish UI/UX and verify streaming performance.
