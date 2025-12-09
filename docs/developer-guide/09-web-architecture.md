# LLMSpell Web Architecture

Developer guide for understanding and extending the LLMSpell web interface.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Backend Components](#backend-components)
3. [Frontend Architecture](#frontend-architecture)
4. [API Design Patterns](#api-design-patterns)
5. [Adding New Features](#adding-new-features)
6. [Security Considerations](#security-considerations)
7. [Build and Deployment](#build-and-deployment)

## Architecture Overview

The LLMSpell web interface is a full-stack application combining a Rust backend with a React frontend, deployed as a single binary.

### Technology Stack

**Backend**:
- **Framework**: Axum 0.7 (async HTTP server)
- **Runtime**: Tokio (async runtime)
- **Serialization**: Serde (JSON)
- **OpenAPI**: utoipa + utoipa-swagger-ui
- **WebSocket**: Axum WebSocket support

**Frontend**:
- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite
- **Routing**: React Router
- **HTTP Client**: Axios
- **Styling**: CSS Modules

**Deployment**:
- **Asset Embedding**: rust-embed
- **Single Binary**: Frontend assets embedded in Rust binary
- **Distribution**: No separate frontend server needed

### Crate Structure

```
llmspell-web/
├── src/
│   ├── lib.rs              # Crate entry point
│   ├── server/             # HTTP server
│   │   └── mod.rs          # WebServer implementation
│   ├── handlers/           # Request handlers
│   │   ├── scripts.rs      # Script execution
│   │   ├── sessions.rs     # Session management
│   │   ├── memory.rs       # Memory operations
│   │   ├── agents.rs       # Agent management
│   │   ├── tools.rs        # Tool execution
│   │   ├── templates.rs    # Template workflows
│   │   ├── config.rs       # Runtime configuration
│   │   ├── static_config.rs# Static configuration
│   │   ├── providers.rs    # Provider status
│   │   ├── ws.rs           # WebSocket streaming
│   │   ├── assets.rs       # Static asset serving
│   │   ├── auth.rs         # Authentication
│   │   └── metrics.rs      # Prometheus metrics
│   ├── middleware/         # HTTP middleware
│   │   ├── auth.rs         # API key authentication
│   │   ├── cors.rs         # CORS configuration
│   │   └── metrics.rs      # Request metrics
│   ├── api_docs.rs         # OpenAPI documentation
│   ├── config.rs           # Server configuration
│   ├── error.rs            # Error types
│   └── state.rs            # Application state
├── frontend/               # React application
│   ├── src/
│   │   ├── api/            # API client
│   │   ├── components/     # React components
│   │   ├── pages/          # Page components
│   │   └── App.tsx         # Root component
│   └── dist/               # Build output (embedded)
└── Cargo.toml
```

### Request Flow

```
Browser → Axum Router → Middleware → Handler → Kernel → Response
                ↓
         WebSocket → EventBus → Stream Events
```

1. **HTTP Request**: Browser sends request to Axum server
2. **Middleware**: Authentication, CORS, metrics
3. **Handler**: Process request, interact with kernel
4. **Kernel**: Execute operations via KernelHandle
5. **Response**: Serialize and return JSON
6. **WebSocket**: Real-time events streamed to connected clients

## Backend Components

### WebServer Initialization

The `WebServer` struct manages the HTTP server lifecycle:

```rust
pub struct WebServer {
    config: WebConfig,
    state: AppState,
}

impl WebServer {
    pub async fn new(config: WebConfig, kernel: KernelHandle) -> Result<Self> {
        let state = AppState {
            kernel: Arc::new(Mutex::new(kernel)),
            metrics_recorder,
            config: config.clone(),
            runtime_config,
            static_config_path,
            config_store,
        };
        
        Ok(Self { config, state })
    }
    
    pub async fn run(self) -> Result<()> {
        let app = Self::build_app(self.state);
        let listener = TcpListener::bind(&self.config.bind_address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
```

### Handler Modules

Each handler module follows a consistent pattern:

**File**: `src/handlers/scripts.rs`
```rust
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ExecuteScriptRequest {
    pub script: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct ExecuteScriptResponse {
    pub execution_id: String,
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/api/scripts/execute",
    request_body = ExecuteScriptRequest,
    responses(
        (status = 200, description = "Script executed", body = ExecuteScriptResponse)
    )
)]
pub async fn execute_script(
    State(state): State<AppState>,
    Json(req): Json<ExecuteScriptRequest>,
) -> Result<Json<ExecuteScriptResponse>, WebError> {
    // Implementation
}
```

### AppState and Dependency Injection

`AppState` provides shared access to system components:

```rust
#[derive(Clone)]
pub struct AppState {
    pub kernel: Arc<Mutex<KernelHandle>>,
    pub metrics_recorder: PrometheusHandle,
    pub config: WebConfig,
    pub runtime_config: Arc<RwLock<EnvRegistry>>,
    pub static_config_path: Option<PathBuf>,
    pub config_store: Option<Arc<RwLock<ConfigStore>>>,
}
```

Handlers access state via Axum's `State` extractor:
```rust
async fn handler(State(state): State<AppState>) -> Result<Json<Response>> {
    let kernel = state.kernel.lock().await;
    // Use kernel...
}
```

### Error Handling

Centralized error handling with `WebError`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("Kernel error: {0}")]
    Kernel(#[from] KernelError),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebError::Kernel(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            WebError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            WebError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### OpenAPI Integration

OpenAPI documentation is generated using `utoipa`:

**File**: `src/api_docs.rs`
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::scripts::execute_script,
        handlers::sessions::list_sessions,
        // ... all endpoints
    ),
    components(schemas(
        ExecuteScriptRequest,
        ExecuteScriptResponse,
        // ... all schemas
    )),
    tags(
        (name = "scripts", description = "Script execution"),
        (name = "sessions", description = "Session management"),
        // ... all tags
    )
)]
pub struct ApiDoc;
```

Served via Swagger UI:
```rust
.merge(
    SwaggerUi::new("/swagger-ui")
        .url("/api/openapi.json", ApiDoc::openapi())
)
```

## Frontend Architecture

### Component Hierarchy

```
App
├── Layout
│   ├── Sidebar (navigation)
│   └── Content
│       ├── Dashboard
│       ├── Tools
│       ├── Sessions
│       ├── Agents
│       ├── Memory
│       ├── Library (templates)
│       └── Configuration
└── Components
    ├── Editor (script editor)
    ├── Console (output display)
    ├── TemplateCard
    ├── LaunchModal
    └── ...
```

### API Client

**File**: `frontend/src/api/client.ts`
```typescript
import axios from 'axios';

const API_BASE = '/api';

export const apiClient = axios.create({
  baseURL: API_BASE,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Scripts API
export const executeScript = async (script: string, language: string) => {
  const response = await apiClient.post('/scripts/execute', {
    script,
    language,
  });
  return response.data;
};

// Sessions API
export const listSessions = async () => {
  const response = await apiClient.get('/sessions');
  return response.data;
};
```

### Type Definitions

**File**: `frontend/src/api/types.ts`
```typescript
export interface ScriptExecutionRequest {
  script: string;
  language: 'lua' | 'javascript' | 'python';
}

export interface ScriptExecutionResponse {
  execution_id: string;
  status: string;
}

export interface Session {
  id: string;
  name?: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}
```

### State Management

React hooks for local state:
```typescript
const [sessions, setSessions] = useState<Session[]>([]);
const [loading, setLoading] = useState(false);
const [error, setError] = useState<string | null>(null);

useEffect(() => {
  const fetchSessions = async () => {
    setLoading(true);
    try {
      const data = await listSessions();
      setSessions(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };
  
  fetchSessions();
}, []);
```

### Routing

**File**: `frontend/src/App.tsx`
```typescript
import { BrowserRouter, Routes, Route } from 'react-router-dom';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="tools" element={<Tools />} />
          <Route path="sessions" element={<Sessions />} />
          <Route path="agents" element={<Agents />} />
          <Route path="memory" element={<Memory />} />
          <Route path="library" element={<Library />} />
          <Route path="settings" element={<Configuration />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
```

## API Design Patterns

### RESTful Conventions

- **GET**: Retrieve resources (list or single)
- **POST**: Create resources or execute actions
- **PUT**: Update resources (full replacement)
- **PATCH**: Partial updates (not currently used)
- **DELETE**: Remove resources

### Endpoint Naming

- **Collections**: `/api/sessions` (plural)
- **Single Resource**: `/api/sessions/:id`
- **Actions**: `/api/scripts/execute`, `/api/agents/:id/stop`
- **Nested Resources**: `/api/sessions/:id/artifacts`

### Request/Response Format

**Request**:
```json
{
  "script": "print('hello')",
  "language": "lua"
}
```

**Success Response** (200):
```json
{
  "execution_id": "exec-123",
  "status": "completed"
}
```

**Error Response** (4xx/5xx):
```json
{
  "error": "Invalid script syntax"
}
```

### Pagination

For list endpoints:
```
GET /api/sessions?limit=20&offset=0
```

Response includes metadata:
```json
{
  "items": [...],
  "total": 100,
  "limit": 20,
  "offset": 0
}
```

### WebSocket Message Protocol

**Connection**: `ws://localhost:3000/ws/stream`

**Message Format**:
```json
{
  "type": "script_execution",
  "data": {
    "execution_id": "exec-123",
    "status": "running",
    "output": "Hello, World!"
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

**Event Types**:
- `script_execution`: Script execution events
- `session_update`: Session changes
- `memory_change`: Memory updates
- `agent_lifecycle`: Agent state transitions

## Adding New Features

### Creating a New API Endpoint

1. **Define Request/Response Types**:
   ```rust
   // src/handlers/my_feature.rs
   #[derive(Deserialize, ToSchema)]
   pub struct MyRequest {
       pub param: String,
   }
   
   #[derive(Serialize, ToSchema)]
   pub struct MyResponse {
       pub result: String,
   }
   ```

2. **Implement Handler**:
   ```rust
   #[utoipa::path(
       post,
       path = "/api/my-feature",
       request_body = MyRequest,
       responses(
           (status = 200, body = MyResponse)
       )
   )]
   pub async fn my_handler(
       State(state): State<AppState>,
       Json(req): Json<MyRequest>,
   ) -> Result<Json<MyResponse>, WebError> {
       // Implementation
       Ok(Json(MyResponse { result: "success".to_string() }))
   }
   ```

3. **Register Route**:
   ```rust
   // src/server/mod.rs
   let api_routes = Router::new()
       .route("/my-feature", post(handlers::my_feature::my_handler))
       // ... other routes
   ```

4. **Add to OpenAPI**:
   ```rust
   // src/api_docs.rs
   #[openapi(
       paths(
           handlers::my_feature::my_handler,
           // ... other paths
       )
   )]
   ```

### Adding a Frontend Page

1. **Create Page Component**:
   ```typescript
   // frontend/src/pages/MyFeature.tsx
   export function MyFeature() {
       return (
           <div>
               <h1>My Feature</h1>
               {/* Implementation */}
           </div>
       );
   }
   ```

2. **Add API Client Method**:
   ```typescript
   // frontend/src/api/client.ts
   export const callMyFeature = async (param: string) => {
       const response = await apiClient.post('/my-feature', { param });
       return response.data;
   };
   ```

3. **Add Route**:
   ```typescript
   // frontend/src/App.tsx
   <Route path="my-feature" element={<MyFeature />} />
   ```

4. **Add Navigation**:
   ```typescript
   // frontend/src/components/Layout.tsx
   <Link to="/my-feature">My Feature</Link>
   ```

### Integrating with Kernel

Access kernel functionality via `KernelHandle`:

```rust
async fn handler(State(state): State<AppState>) -> Result<Json<Response>> {
    let kernel = state.kernel.lock().await;
    
    // Execute script
    let result = kernel.execute_script(script, language).await?;
    
    // Access session manager
    let session_manager = kernel.session_manager();
    let sessions = session_manager.list_sessions().await?;
    
    Ok(Json(Response { /* ... */ }))
}
```

## Security Considerations

### CORS Configuration

CORS is configured in middleware:

```rust
// src/middleware/cors.rs
use tower_http::cors::{CorsLayer, Any};

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
```

**Production**: Restrict origins to specific domains.

### API Key Handling

API keys are stored encrypted:
- Never log API keys
- Use environment variables for configuration
- Encrypt at rest in database
- Mask in UI (show only last 4 characters)

### Session Management

Sessions are isolated per tenant:
- Row-Level Security (RLS) in PostgreSQL
- Session IDs are UUIDs (non-guessable)
- Session data is scoped to authenticated user

### Input Validation

All inputs are validated:
```rust
fn validate_script(script: &str) -> Result<(), WebError> {
    if script.is_empty() {
        return Err(WebError::InvalidRequest("Script cannot be empty".into()));
    }
    if script.len() > MAX_SCRIPT_LENGTH {
        return Err(WebError::InvalidRequest("Script too long".into()));
    }
    Ok(())
}
```

## Build and Deployment

### Frontend Build Process

Build frontend assets:
```bash
cd llmspell-web/frontend
npm install
npm run build
```

Output: `frontend/dist/` (embedded by rust-embed)

### Asset Embedding

**File**: `src/handlers/assets.rs`
```rust
#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            // SPA fallback: serve index.html for unknown routes
            match Assets::get("index.html") {
                Some(content) => {
                    ([(header::CONTENT_TYPE, "text/html")], content.data).into_response()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}
```

### Single-Binary Distribution

Build complete binary:
```bash
# Build frontend
cd llmspell-web/frontend && npm run build && cd ../..

# Build Rust binary (assets embedded automatically)
cargo build --release -p llmspell-cli

# Binary includes:
# - Rust backend
# - Embedded frontend assets
# - All dependencies
```

### Docker Deployment

**Dockerfile**:
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

### Reverse Proxy Configuration

**Nginx**:
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

**Apache**:
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

## See Also

- [User Guide](../user-guide/12-web-interface.md) - End-user documentation
- [API Reference](../technical/web-api-reference.md) - Complete API documentation
- [CLI Reference](../user-guide/05-cli-reference.md) - Command-line interface
- [Configuration Guide](../user-guide/03-configuration.md) - Configuration options
