# Phase 14: Web Interface - TODO List

**Version**: 1.1
**Date**: December 2025
**Status**: Implementation Ready
**Phase**: 14 (Web Interface)
**Timeline**: Weeks 55-58 (20 working days / 4 weeks)
**Priority**: HIGH (User Experience & Accessibility)
**Dependencies**:
- Phase 13c: Storage Consolidation ✅
- Phase 10: Service Integration ✅

**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-14-design-doc.md
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE14-TODO.md)

> **📋 Actionable Task List**: This document breaks down Phase 14 implementation into specific, measurable tasks for building a unified, single-binary web interface with HTTP/WebSocket API and embedded React frontend.

---

## Overview

**Goal**: Create a unified, single-binary web interface providing a "Mission Control" for AI agents, including a high-performance HTTP/WebSocket API and an embedded React/WASM frontend.

**Strategic Context**:
- **Problem**: CLI-only nature limits accessibility, visualization, and real-time monitoring of complex agent interactions.
- **Solution**: `llmspell-web` crate with Axum backend + React frontend embedded via `rust-embed`.
- **Approach**: Single binary distribution, daemon integration, and zero external dependencies.

**Architecture Summary**:
- **New Crate**: `llmspell-web` (Axum, Tokio, Serde, RustEmbed)
- **API**: RESTful endpoints + WebSocket streaming
- **Frontend**: React + Vite + Monaco Editor (Embedded)
- **Security**: API Key/JWT Auth, Localhost binding, Rate limiting

**Success Criteria Summary**:
- [ ] `llmspell-web` crate compiles with all dependencies
- [ ] `llmspell web start` launches server (foreground & daemon)
- [ ] REST API endpoints functional (<100ms latency)
- [ ] WebSocket streaming operational for script execution
- [ ] Frontend assets embedded and served from binary
- [ ] "Mission Control" UI functional (Editor, Graph, Dashboard)
- [ ] Security measures validated (Auth, CORS, Rate Limits)
- [ ] >90% Test Coverage (Unit, Integration, E2E)

---

## Dependency Analysis

**Critical Path**:
1.  **Foundation (Days 1-2)**: Crate setup + Core Server → Hello World
2.  **Backend Core (Days 3-7)**: API Endpoints + WebSocket → Script Execution
3.  **Frontend Core (Days 8-12)**: React Setup + Embedding → UI Serving
4.  **Integration (Days 13-15)**: Daemon + Auth + CLI → Full Lifecycle
5.  **Validation (Days 16-20)**: Testing + Docs + Polish → Release

**Parallel Tracks**:
-   **Backend Track**: Days 3-7 (API/WS) → Days 13-15 (Auth/Daemon)
-   **Frontend Track**: Days 8-12 (UI Implementation) → Days 16-17 (E2E Tests)

**Hard Dependencies**:
- Phase 14.3 (Frontend) depends on Phase 14.2 (Backend API) for endpoints
- Phase 14.4.2 (Daemon) depends on Phase 14.1.2 (Core Server) for WebServer struct
- Phase 14.5.1 (E2E Testing) depends on Phases 14.2 + 14.3 (Full stack)

---

## Phase 14.1: Foundation & Crate Setup (Days 1-2)

### Task 14.1.1: Create llmspell-web Crate Structure
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Backend Lead

**Description**: Initialize the `llmspell-web` crate with necessary dependencies and module structure.

**Acceptance Criteria**:
- [x] Crate created at `llmspell-web/`
- [x] `Cargo.toml` configured with Axum, Tokio, Serde, RustEmbed
- [x] Basic module structure (`lib.rs`, `server.rs`, `error.rs`)
- [x] Added to workspace
- [x] Compiles successfully

**Files to Create/Modify**:
- `llmspell-web/Cargo.toml` (NEW)
- `llmspell-web/src/lib.rs` (NEW)
- `llmspell-web/src/server/mod.rs` (NEW)
- `llmspell-web/src/error/mod.rs` (NEW)
- `Cargo.toml` (MODIFY - add workspace member)

**Implementation Steps**:
1.  `cargo new llmspell-web --lib`
2.  Update `Cargo.toml` with dependencies:
    ```toml
    [dependencies]
    axum = { version = "0.7", features = ["ws", "macros"] }
    tokio = { version = "1.0", features = ["full"] }
    serde = { version = "1.0", features = ["derive"] }
    rust-embed = "8.0"
    tower = "0.4"
    tower-http = { version = "0.5", features = ["cors", "trace", "fs"] }
    ```
3.  Create module files.
4.  Register in root `Cargo.toml`.

**Definition of Done**:
- [x] Crate compiles
- [x] Dependencies resolve
- [x] Module structure matches design

**Implementation Insights**:
- ✅ Initialized `llmspell-web` with Axum 0.7 and Tokio 1.0.
- ✅ Configured workspace members in root `Cargo.toml`.
- ✅ Verified compilation with `cargo check`.
- ✅ Added `anyhow` and `serde_json` for upcoming handlers.

### Task 14.1.2: Implement Core Web Server
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Backend Developer

**Description**: Implement the basic Axum server setup with configuration and graceful shutdown.

**Acceptance Criteria**:
- [x] `WebServer` struct implemented
- [x] Configuration loading (`[web]` section)
- [x] Basic `/health` endpoint
- [x] Graceful shutdown signal handling
- [x] Bind to port from config

**Files to Create/Modify**:
- `llmspell-web/src/server/mod.rs` (MODIFY)
- `llmspell-web/src/config.rs` (NEW)
- `llmspell-web/src/handlers/health.rs` (NEW)

**Implementation Steps**:
1.  Define `WebConfig` struct:
    ```rust
    #[derive(Deserialize)]
    pub struct WebConfig {
        pub port: u16,
        pub host: String,
        pub cors_origins: Vec<String>,
    }
    ```
2.  Implement `WebServer::run`:
    ```rust
    pub async fn run(config: WebConfig) -> Result<()> {
        let app = Router::new().route("/health", get(health_check));
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
        Ok(())
    }
    ```
3.  Add `/health` handler.
4.  Implement `shutdown_signal` handler.

**Definition of Done**:
- [x] Server starts and listens on port
- [x] `/health` returns 200 OK
- [x] Ctrl+C triggers graceful shutdown

**Implementation Insights**:
- ✅ Implemented `WebServer` with graceful shutdown using `tokio::signal`.
- ✅ Created `WebConfig` with default values.
- ✅ Added `/health` endpoint returning version info.
- ✅ Verified health check with unit tests.

### Task 14.1.3: Create Web Profile
**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: DevOps Engineer

**Description**: Create default configuration profiles for the web interface.

**Acceptance Criteria**:
- [x] `web.toml` created with default settings
- [x] `web-development.toml` created with dev settings (CORS, verbose logging)

**Files to Create/Modify**:
- `llmspell-config/presets/web.toml` (NEW)
- `llmspell-config/presets/web-development.toml` (NEW)

**Implementation Steps**:
1.  Create `web.toml`:
    ```toml
    [web]
    port = 3000
    host = "127.0.0.1"
    cors_origins = ["http://localhost:3000"]
    ```
2.  Create `web-development.toml` with wider permissions.

**Definition of Done**:
- [x] Config files exist
- [x] Can be loaded by `llmspell-config`

**Implementation Insights**:
- ✅ Created `web.toml` and `web-development.toml` in `llmspell-config/presets/`.
- ✅ Configured default ports and CORS settings.

---

## Phase 14.2: HTTP Backend Implementation (Days 3-7)

### Task 14.2.1: Implement Script Execution API
**Priority**: CRITICAL
**Estimated Time**: 8 hours
**Assignee**: Backend Developer

**Description**: Implement `POST /api/scripts/execute` and related state management.

**Acceptance Criteria**:
- [ ] Request/Response structs defined
- [ ] Handler calls Kernel execution
- [ ] Returns `execution_id`
- [ ] Error handling for invalid scripts

**Files to Create/Modify**:
- `llmspell-web/src/handlers/scripts.rs` (NEW)
- `llmspell-web/src/models/script.rs` (NEW)
- `llmspell-kernel/src/execution/integrated.rs` (MODIFY - ensure public API)

**Implementation Steps**:
1.  Define `ExecuteScriptRequest`:
    ```rust
    #[derive(Deserialize)]
    pub struct ExecuteScriptRequest {
        pub script: String,
        pub language: ScriptLanguage,
    }
    ```
2.  Implement handler in `src/handlers/scripts.rs`.
3.  Integrate with `Kernel`.

**Definition of Done**:
- [ ] Endpoint accepts JSON
- [ ] Returns valid Execution ID
- [ ] Tests pass

### Task 14.2.2: Implement WebSocket Streaming
**Priority**: CRITICAL
**Estimated Time**: 12 hours
**Assignee**: Backend Developer

**Description**: Implement `/ws/stream` for real-time kernel events.

**Acceptance Criteria**:
- [ ] WebSocket upgrade handler
- [ ] Event bus subscription
- [ ] JSON serialization of events
- [ ] Connection management (ping/pong)

**Files to Create/Modify**:
- `llmspell-web/src/handlers/ws.rs` (NEW)
- `llmspell-web/src/models/events.rs` (NEW)

**Implementation Steps**:
1.  Implement `ws_handler`:
    ```rust
    pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
        ws.on_upgrade(|socket| handle_socket(socket, state))
    }
    ```
2.  Connect to `EventBus`.
3.  Loop: Receive Event -> Serialize -> Send WS Message.

**Definition of Done**:
- [ ] WS connection establishes
- [ ] Events flow from Kernel to Client
- [ ] Connection closes cleanly

### Task 14.2.3: Implement Resource APIs (Sessions, Memory)
**Priority**: HIGH
**Estimated Time**: 8 hours
**Assignee**: Backend Developer

**Description**: Implement read-only APIs for Sessions and Memory inspection.

**Acceptance Criteria**:
- [ ] `GET /api/sessions`
- [ ] `GET /api/memory/search`
- [ ] Pagination support
- [ ] Filtering support

**Files to Create/Modify**:
- `llmspell-web/src/handlers/sessions.rs` (NEW)
- `llmspell-web/src/handlers/memory.rs` (NEW)

**Implementation Steps**:
1.  Implement `src/handlers/sessions.rs`.
2.  Implement `src/handlers/memory.rs`.
3.  Connect to `SessionManager` and `MemoryManager`.

**Definition of Done**:
- [ ] Endpoints return correct JSON data
- [ ] Query params work for filtering

### Task 14.2.4: Implement Agent & Tool APIs
**Priority**: HIGH
**Estimated Time**: 8 hours
**Assignee**: Backend Developer

**Description**: Implement APIs for invoking Agents and calling Tools directly.

**Acceptance Criteria**:
- [ ] `POST /api/agents/{name}/invoke`
- [ ] `POST /api/tools/{name}/call`
- [ ] Parameter validation
- [ ] Streaming response support for agents

**Files to Create/Modify**:
- `llmspell-web/src/handlers/agents.rs` (NEW)
- `llmspell-web/src/handlers/tools.rs` (NEW)

**Implementation Steps**:
1.  Implement `src/handlers/agents.rs`.
2.  Implement `src/handlers/tools.rs`.
3.  Connect to `AgentRegistry` and `ToolRegistry`.

**Definition of Done**:
- [ ] Can invoke agent via API
- [ ] Can call tool via API
- [ ] Tests pass

### Task 14.2.5: Error Handling & Response Types
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Backend Developer

**Description**: Implement WebError types with HTTP status mapping.

**Acceptance Criteria**:
- [ ] `WebError` enum defined
- [ ] `IntoResponse` implemented for `WebError`
- [ ] Standardized JSON error format

**Files to Create/Modify**:
- `llmspell-web/src/error/mod.rs` (MODIFY)

**Implementation Steps**:
1.  Create error types:
    ```rust
    pub enum WebError {
        ScriptExecution { message: String, details: serde_json::Value },
        Authentication { message: String },
        NotFound { resource: String },
        Internal(anyhow::Error),
    }
    ```
2.  Implement `IntoResponse` to map errors to status codes and JSON bodies.

**Definition of Done**:
- [ ] All handlers use `Result<Json<T>, WebError>`
- [ ] Errors return correct HTTP codes

### Task 14.2.6: Metrics Endpoint
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Assignee**: Backend Developer

**Description**: Implement `/metrics` with Prometheus format.

**Acceptance Criteria**:
- [ ] `/metrics` endpoint returns text/plain
- [ ] Request duration histogram
- [ ] Active connection gauge

**Files to Create/Modify**:
- `llmspell-web/src/handlers/metrics.rs` (NEW)
- `llmspell-web/src/middleware/metrics.rs` (NEW)

**Implementation Steps**:
1.  Add `metrics-exporter-prometheus` dependency.
2.  Register metrics middleware.
3.  Expose handler.

**Definition of Done**:
- [ ] `/metrics` returns valid Prometheus data

---

## Phase 14.3: Frontend Integration (Days 8-12)

### Task 14.3.1: Initialize Frontend Project
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Frontend Developer

**Description**: Set up React/Vite project within `llmspell-web/frontend`.

**Acceptance Criteria**:
- [ ] Vite project created
- [ ] TypeScript configured
- [ ] Tailwind/CSS setup
- [ ] Build script generates `dist/`

**Files to Create/Modify**:
- `llmspell-web/frontend/package.json` (NEW)
- `llmspell-web/frontend/vite.config.ts` (NEW)
- `llmspell-web/frontend/tsconfig.json` (NEW)

**Implementation Steps**:
1.  `npm create vite@latest frontend -- --template react-ts`
2.  Install dependencies (React Router, Lucide, etc.)
3.  Configure `vite.config.ts` for proxying API.

**Definition of Done**:
- [ ] Dev server runs
- [ ] Build produces static assets

### Task 14.3.2a: Dashboard Layout & Navigation
**Priority**: HIGH
**Estimated Time**: 6 hours
**Assignee**: Frontend Developer

**Description**: Build the main UI layout and navigation structure.

**Acceptance Criteria**:
- [ ] Sidebar with links
- [ ] Header with status
- [ ] Responsive container

**Files to Create/Modify**:
- `llmspell-web/frontend/src/components/Layout.tsx` (NEW)
- `llmspell-web/frontend/src/App.tsx` (MODIFY)

**Implementation Steps**:
1.  Create `Layout` component using Tailwind.
2.  Setup React Router.

**Definition of Done**:
- [ ] Navigation works between routes

### Task 14.3.2b: Dashboard Widgets
**Priority**: HIGH
**Estimated Time**: 6 hours
**Assignee**: Frontend Developer

**Description**: Implement widgets for the main dashboard view.

**Acceptance Criteria**:
- [ ] System Status widget
- [ ] Recent Activity widget
- [ ] Quick Actions widget

**Files to Create/Modify**:
- `llmspell-web/frontend/src/pages/Dashboard.tsx` (NEW)
- `llmspell-web/frontend/src/components/widgets/` (NEW)

**Implementation Steps**:
1.  Create widget components.
2.  Compose them in `Dashboard.tsx`.

**Definition of Done**:
- [ ] Dashboard displays widgets correctly

### Task 14.3.2c: Dashboard API Integration
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Frontend Developer

**Description**: Connect dashboard widgets to backend APIs.

**Acceptance Criteria**:
- [ ] Fetch status from `/health`
- [ ] Fetch activity from `/api/sessions`

**Files to Create/Modify**:
- `llmspell-web/frontend/src/api/client.ts` (NEW)
- `llmspell-web/frontend/src/hooks/useSystemStatus.ts` (NEW)

**Implementation Steps**:
1.  Create API client using `fetch` or `axios`.
2.  Create React hooks for data fetching.

**Definition of Done**:
- [ ] Real data shown in dashboard

### Task 14.3.3a: Monaco Editor Integration
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Frontend Developer

**Description**: Integrate Monaco Editor for script editing.

**Acceptance Criteria**:
- [ ] Editor component renders
- [ ] Language selection works
- [ ] Theme support

**Files to Create/Modify**:
- `llmspell-web/frontend/src/components/editor/CodeEditor.tsx` (NEW)

**Implementation Steps**:
1.  Install `@monaco-editor/react`.
2.  Configure editor options.

**Definition of Done**:
- [ ] Editor usable for typing code

### Task 14.3.3b: WebSocket Hook & State
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Frontend Developer

**Description**: Implement WebSocket hook for real-time communication.

**Acceptance Criteria**:
- [ ] Auto-reconnect logic
- [ ] Message parsing
- [ ] State updates on events

**Files to Create/Modify**:
- `llmspell-web/frontend/src/hooks/useWebSocket.ts` (NEW)

**Implementation Steps**:
1.  Create custom hook managing `WebSocket` instance.
2.  Handle `onmessage` and dispatch updates.

**Definition of Done**:
- [ ] Hook reliably receives messages

### Task 14.3.3c: Console Component
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Frontend Developer

**Description**: Implement a console component to display execution logs.

**Acceptance Criteria**:
- [ ] Log lines rendering
- [ ] ANSI color support
- [ ] Auto-scroll

**Files to Create/Modify**:
- `llmspell-web/frontend/src/components/editor/Console.tsx` (NEW)

**Implementation Steps**:
1.  Create Console component.
2.  Use a library for ANSI to HTML conversion if needed.

**Definition of Done**:
- [ ] Logs display correctly with colors

### Task 14.3.4: Embed Frontend Assets
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Backend Developer

**Description**: Configure `rust-embed` to bundle the built frontend.

**Acceptance Criteria**:
- [ ] `Assets` struct with `#[derive(RustEmbed)]`
- [ ] Fallback handler for SPA routing
- [ ] Binary contains assets

**Files to Create/Modify**:
- `llmspell-web/src/handlers/static.rs` (NEW)

**Implementation Steps**:
1.  Configure `rust-embed`:
    ```rust
    #[derive(RustEmbed)]
    #[folder = "frontend/dist"]
    struct Assets;
    ```
2.  Implement `static_handler`.

**Definition of Done**:
- [ ] `llmspell web start` serves UI
- [ ] Refreshing on sub-routes works

### Task 14.3.5: Memory Graph Visualization
**Priority**: HIGH
**Estimated Time**: 8 hours
**Assignee**: Frontend Developer

**Description**: Interactive graph component for memory exploration.

**Acceptance Criteria**:
- [ ] Force-directed graph rendering
- [ ] Node inspection on click
- [ ] Filtering by entity type

**Files to Create/Modify**:
- `llmspell-web/frontend/src/components/memory/MemoryGraph.tsx` (NEW)

**Implementation Steps**:
1.  Use `react-force-graph` or similar.
2.  Fetch nodes/edges from `/api/memory/graph`.

**Definition of Done**:
- [ ] Graph visualizes memory connections

### Task 14.3.6: Session Timeline
**Priority**: MEDIUM
**Estimated Time**: 6 hours
**Assignee**: Frontend Developer

**Description**: Interactive session replay with timeline scrubbing.

**Acceptance Criteria**:
- [ ] Timeline visualization of events
- [ ] Click to jump to event
- [ ] Play/Pause replay

**Files to Create/Modify**:
- `llmspell-web/frontend/src/components/session/Timeline.tsx` (NEW)

**Implementation Steps**:
1.  Create timeline component.
2.  Map session events to timeline points.

**Definition of Done**:
- [ ] Can scrub through session history

---

## Phase 14.4: Security & Daemon Integration (Days 13-15)

### Task 14.4.1: Implement Authentication
**Priority**: CRITICAL
**Estimated Time**: 8 hours
**Assignee**: Security Developer

**Description**: Add API Key and JWT authentication middleware.

**Acceptance Criteria**:
- [ ] `X-API-Key` validation
- [ ] JWT generation/validation
- [ ] Middleware applied to protected routes

**Files to Create/Modify**:
- `llmspell-web/src/middleware/auth.rs` (NEW)
- `llmspell-web/src/handlers/auth.rs` (NEW)

**Implementation Steps**:
1.  Implement middleware checking headers.
2.  Add login endpoint.

**Definition of Done**:
- [ ] Unauthorized requests rejected (401)
- [ ] Valid keys accepted

### Task 14.4.2: Daemon Lifecycle Integration
**Priority**: HIGH
**Estimated Time**: 8 hours
**Assignee**: Systems Developer

**Description**: Integrate with `llmspell-kernel` daemon infrastructure.

**Acceptance Criteria**:
- [ ] `llmspell web start --daemon` works
- [ ] PID file management
- [ ] `llmspell web stop` works

**Files to Create/Modify**:
- `llmspell-kernel/src/daemon/mod.rs` (MODIFY - expose helpers)
- `llmspell-web/src/daemon.rs` (NEW)

**Implementation Steps**:
1.  Use `daemonize` or similar crate (or existing kernel utils).
2.  Manage PID files.

**Definition of Done**:
- [ ] Background process starts/stops reliably
- [ ] PID file correct

### Task 14.4.3: CLI Web Subcommand
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: CLI Developer

**Description**: Implement `llmspell web start/stop/status/open` commands.

**Acceptance Criteria**:
- [ ] `web` subcommand registered
- [ ] Arguments parsed correctly
- [ ] Commands execute backend logic

**Files to Create/Modify**:
- `llmspell-cli/src/commands/web.rs` (NEW)
- `llmspell-cli/src/cli.rs` (MODIFY)

**Implementation Steps**:
1.  Define `WebCommands` enum.
2.  Implement command handlers calling `llmspell-web`.

**Definition of Done**:
- [ ] CLI commands control the web server

---

## Phase 14.5: Testing & Documentation (Days 16-20)

### Task 14.5.1: Comprehensive Testing
**Priority**: CRITICAL
**Estimated Time**: 16 hours
**Assignee**: QA Engineer

**Description**: Implement Unit, Integration, and E2E tests.

**Acceptance Criteria**:
- [ ] Unit tests for all handlers
- [ ] Integration tests for full flow
- [ ] E2E tests (Playwright) for UI
- [ ] Load tests (k6)

**Files to Create/Modify**:
- `llmspell-web/tests/api_integration.rs` (NEW)
- `llmspell-web/e2e/` (NEW)

**Implementation Steps**:
1.  Write handler unit tests.
2.  Create `tests/api_integration.rs`.
3.  Setup Playwright.

**Quality Gates**:
- [ ] `./scripts/quality/quality-check-minimal.sh` passes
- [ ] `./scripts/quality/quality-check-fast.sh` passes
- [ ] Zero clippy warnings

**Definition of Done**:
- [ ] All tests pass
- [ ] Coverage >90%

### Task 14.5.2: Documentation & Polish
**Priority**: HIGH
**Estimated Time**: 8 hours
**Assignee**: Tech Writer

**Description**: Create user guides and API docs.

**Acceptance Criteria**:
- [ ] `docs/user-guide/web-interface.md`
- [ ] CLI help text updated

**Files to Create/Modify**:
- `docs/user-guide/web-interface.md` (NEW)

**Implementation Steps**:
1.  Write User Guide.
2.  Update READMEs.

**Definition of Done**:
- [ ] Docs complete and reviewed

### Task 14.5.3: OpenAPI Generation
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Assignee**: Backend Developer

**Description**: Integrate utoipa for automatic OpenAPI spec generation.

**Acceptance Criteria**:
- [ ] OpenAPI JSON endpoint
- [ ] Swagger UI (optional)

**Files to Create/Modify**:
- `llmspell-web/src/api_docs.rs` (NEW)

**Implementation Steps**:
1.  Add `utoipa` dependency.
2.  Annotate handlers.
3.  Serve spec at `/api/openapi.json`.

**Definition of Done**:
- [ ] Valid OpenAPI spec generated
