# llmspell-web

Web interface for LLMSpell - Browser-based AI workflow development and monitoring.

## Overview

`llmspell-web` provides a full-featured web interface for LLMSpell, combining a Rust backend (Axum) with a React frontend, deployed as a single binary with embedded assets.

## Features

- **Script Editor**: Write and execute Lua/JavaScript/Python scripts with syntax highlighting
- **Session Management**: Visual session browser with history and artifacts
- **Template Library**: Browse and launch workflow templates with parameter forms
- **Memory Browser**: Explore episodic memory and knowledge graph
- **Agent Monitor**: Real-time agent lifecycle and workflow tracking
- **Tool Catalog**: Interactive tool execution with parameter forms
- **Configuration UI**: Edit configuration, manage profiles, restart server
- **WebSocket Streaming**: Real-time event updates
- **OpenAPI Documentation**: Interactive Swagger UI
- **Single Binary Deployment**: Frontend assets embedded via rust-embed

## Quick Start

```bash
# Start web server
llmspell web start

# Access at http://localhost:3000
# Or open automatically:
llmspell web open
```

## Building

### Backend Only

```bash
cargo build --release -p llmspell-web
```

### Frontend + Backend

```bash
# Build frontend
cd llmspell-web/frontend
npm install
npm run build

# Build Rust binary (embeds frontend assets)
cd ../..
cargo build --release -p llmspell-cli
```

## Configuration

### Server Options

```bash
# Custom port
llmspell web start --port 8080

# Custom host
llmspell web start --host 0.0.0.0

# Background daemon
llmspell web start --daemon

# Debug logging
llmspell web start --log-level debug
```

### Environment Variables

- `LLMSPELL_WEB_PORT`: Default port (default: 3000)
- `LLMSPELL_WEB_HOST`: Bind address (default: 127.0.0.1)
- `LLMSPELL_LOG_LEVEL`: Logging level (error, warn, info, debug, trace)

## Development

### Backend Development

```bash
# Run with hot reload (requires cargo-watch)
cargo watch -x 'run -p llmspell-cli -- web start'

# Run tests
cargo test -p llmspell-web

# Check for errors
cargo clippy -p llmspell-web
```

### Frontend Development

```bash
cd llmspell-web/frontend

# Install dependencies
npm install

# Development server (with hot reload)
npm run dev

# Build for production
npm run build

# Type checking
npm run type-check

# Linting
npm run lint
```

### Full Stack Development

1. Start backend in one terminal:
   ```bash
   cargo run -p llmspell-cli -- web start
   ```

2. Start frontend dev server in another:
   ```bash
   cd llmspell-web/frontend
   npm run dev
   ```

Frontend dev server proxies API requests to backend at `http://localhost:3000`.

## Deployment

### Single Binary

The recommended deployment method is the single binary with embedded assets:

```bash
# Build frontend
cd llmspell-web/frontend && npm run build && cd ../..

# Build binary (assets embedded automatically)
cargo build --release -p llmspell-cli

# Deploy binary
cp target/release/llmspell /usr/local/bin/

# Run
llmspell web start --host 0.0.0.0 --port 3000
```

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cd llmspell-web/frontend && npm install && npm run build
RUN cargo build --release -p llmspell-cli

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llmspell /usr/local/bin/
EXPOSE 3000
CMD ["llmspell", "web", "start", "--host", "0.0.0.0"]
```

### Reverse Proxy

#### Nginx

```nginx
server {
    listen 80;
    server_name llmspell.example.com;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

#### Apache

```apache
<VirtualHost *:80>
    ServerName llmspell.example.com
    
    ProxyPreserveHost On
    ProxyPass / http://localhost:3000/
    ProxyPassReverse / http://localhost:3000/
    
    RewriteEngine on
    RewriteCond %{HTTP:Upgrade} websocket [NC]
    RewriteCond %{HTTP:Connection} upgrade [NC]
    RewriteRule ^/?(.*) "ws://localhost:3000/$1" [P,L]
</VirtualHost>
```

## API Documentation

### OpenAPI Specification

Access interactive API documentation:
- **Swagger UI**: http://localhost:3000/swagger-ui/
- **OpenAPI JSON**: http://localhost:3000/api/openapi.json

### HTTP Endpoints

See [Web API Reference](../docs/technical/web-api-reference.md) for complete endpoint documentation.

**Key Endpoints**:
- `POST /api/scripts/execute` - Execute script
- `GET /api/sessions` - List sessions
- `GET /api/templates` - List templates
- `POST /api/templates/:id/launch` - Launch template
- `GET /api/memory/search` - Search memory
- `GET /api/agents` - List agents
- `GET /api/tools` - List tools
- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration
- `POST /api/config/restart` - Restart server

### WebSocket

**Endpoint**: `ws://localhost:3000/ws/stream`

**Event Types**:
- `script_execution` - Script execution events
- `session_update` - Session changes
- `memory_change` - Memory updates
- `agent_lifecycle` - Agent state transitions

## Architecture

### Backend

- **Framework**: Axum 0.7
- **Runtime**: Tokio
- **OpenAPI**: utoipa + utoipa-swagger-ui
- **Asset Embedding**: rust-embed

**Structure**:
```
src/
├── lib.rs              # Crate entry
├── server/             # HTTP server
├── handlers/           # Request handlers
├── middleware/         # HTTP middleware
├── api_docs.rs         # OpenAPI docs
├── config.rs           # Server config
├── error.rs            # Error types
└── state.rs            # App state
```

### Frontend

- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite
- **Routing**: React Router
- **HTTP Client**: Axios

**Structure**:
```
frontend/
├── src/
│   ├── api/            # API client
│   ├── components/     # React components
│   ├── pages/          # Page components
│   └── App.tsx         # Root component
└── dist/               # Build output
```

## Testing

```bash
# Backend tests
cargo test -p llmspell-web

# Frontend tests
cd llmspell-web/frontend
npm test

# Integration tests
cargo test -p llmspell-web --test api_integration

# E2E tests (requires running server)
cd llmspell-web/frontend
npm run test:e2e
```

## Troubleshooting

### Server Won't Start

- Check if port is in use: `lsof -i :3000`
- Try different port: `llmspell web start --port 8080`
- Check logs: `llmspell web start --log-level debug`

### Frontend Build Fails

- Clear node_modules: `rm -rf node_modules && npm install`
- Clear build cache: `rm -rf dist`
- Check Node version: `node --version` (requires 18+)

### WebSocket Connection Issues

- Verify server is running: `llmspell web status`
- Check browser console for errors
- Ensure firewall allows WebSocket connections
- Check reverse proxy WebSocket configuration

## Contributing

See [Developer Guide](../docs/developer-guide/09-web-architecture.md) for:
- Adding new API endpoints
- Creating frontend pages
- Integrating with kernel
- Testing strategies

## License

Apache License, Version 2.0. See [LICENSE-APACHE](../LICENSE-APACHE) for details.

## See Also

- [User Guide](../docs/user-guide/12-web-interface.md) - End-user documentation
- [Developer Guide](../docs/developer-guide/09-web-architecture.md) - Architecture and extension
- [API Reference](../docs/technical/web-api-reference.md) - Complete API documentation
- [CLI Reference](../docs/user-guide/05-cli-reference.md) - Command-line interface
