# Release Notes: v0.14.1 - Web Interface & Mission Control

**Phase 14 Complete** - Added a unified, single-binary web interface providing a "Mission Control" for AI agents, featuring a high-performance HTTP/WebSocket API and an embedded React frontend.

## Highlights

### Unified Web Interface (`llmspell-web`)
- **Single Binary**: No separate frontend server required. React assets are embedded directly into the Rust binary.
- **Mission Control UI**: A comprehensive dashboard for monitoring system status, recent activity, and quick actions.
- **Embedded Monaco Editor**: Full-featured code editor for Lua/JavaScript scripts with syntax highlighting and theme support.
- **Real-time Console**: Interactive console streaming script output (stdout/stderr) via WebSockets.

### Powerful API Backend
- **RESTful Endpoints**: Complete API for managing Sessions, Memory, Agents, Tools, and Configuration.
- **WebSocket Streaming**: Real-time event bus subscription for live updates of kernel events.
- **Metrics & Health**: Built-in Prometheus metrics (`/metrics`) and health checks (`/health`).

### Frontend Experience
- **Interactive Visualizations**: Force-directed graph for exploring memory nodes and relationships.
- **Session Replay**: Timeline visualization with playback controls for reviewing session history.
- **Configuration Manager**: UI for managing the 18-layer profile system with secure editing simulation.

## Key Features

- **Dashboard**: System status, activity logs, and quick launch widgets.
- **Script Editor**: Integrated development environment for creating and running agent scripts.
- **Memory Explorer**: Visual graph tool to navigate semantic memory.
- **Session Inspector**: Detailed view of past sessions with timeline scrubbing.

## Phase 14 Tasks Completed

- **14.1**: Foundation & Crate Setup (Axum + Tokio + RustEmbed) ✅
- **14.2**: HTTP Backend Implementation (API + WebSocket + Metrics) ✅
- **14.3**: Frontend Integration (React + Vite + Tailwind + Monaco) ✅
- **14.4**: Daemon Integration (Lifecycle Management) ✅
- **14.5**: API Documentation (OpenAPI/Swagger) ✅
- **14.6**: Quality & Refinement (Database Locking Fixes, Clippy Cleanup) ✅

## Getting Started

Start the web server with the new `web` command:

```bash
# Start the web interface (defaults to http://127.0.0.1:3000)
./target/debug/llmspell web start

# Start with a specific profile
./target/debug/llmspell web start -p openai-prod

# specific host/port
./target/debug/llmspell web start --host 0.0.0.0 --port 8080
```

Open your browser to the displayed URL to access the Mission Control interface.

## What's Next

**Phase 15**: Advanced Integrations & MCP Tools
**Phase 16**: Distributed Agent Mesh

See [implementation-phases.md](docs/in-progress/implementation-phases.md) for roadmap details.

## Upgrade Notes

- **New Command**: `llmspell web` subcommand added.
- **New Dependency**: `llmspell-web` crate added to the workspace.
- **Database**: Ensure your `llmspell.db` is accessible if upgrading from an older dev version (startup checks now enforce WAL mode).
