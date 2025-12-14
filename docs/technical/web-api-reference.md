# LLMSpell Web API Reference

Complete technical reference for the LLMSpell HTTP API and WebSocket protocol.

## Table of Contents

1. [HTTP API Endpoints](#http-api-endpoints)
2. [Request/Response Schemas](#requestresponse-schemas)
3. [WebSocket Protocol](#websocket-protocol)
4. [OpenAPI Specification](#openapi-specification)
5. [Error Codes and Handling](#error-codes-and-handling)

## HTTP API Endpoints

Base URL: `http://localhost:3000/api`

### Scripts

#### Execute Script

Execute a script in the specified language.

**Endpoint**: `POST /api/scripts/execute`

**Request Body**:
```json
{
  "script": "print('Hello, World!')",
  "language": "lua"
}
```

**Response** (200 OK):
```json
{
  "execution_id": "exec-abc123",
  "status": "completed",
  "output": "Hello, World!\n"
}
```

**Errors**:
- `400`: Invalid script or language
- `500`: Execution error

### Sessions

#### List Sessions

Retrieve all sessions.

**Endpoint**: `GET /api/sessions`

**Query Parameters**:
- `limit` (optional): Maximum number of results (default: 50)
- `offset` (optional): Pagination offset (default: 0)

**Response** (200 OK):
```json
[
  {
    "id": "session-123",
    "name": "Research Session",
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T01:00:00Z",
    "message_count": 42
  }
]
```

#### Get Session

Retrieve a specific session by ID.

**Endpoint**: `GET /api/sessions/:id`

**Response** (200 OK):
```json
{
  "id": "session-123",
  "name": "Research Session",
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T01:00:00Z",
  "message_count": 42,
  "messages": [
    {
      "role": "user",
      "content": "Hello",
      "timestamp": "2025-01-01T00:00:00Z"
    }
  ]
}
```

**Errors**:
- `404`: Session not found

#### Create Session

Create a new session.

**Endpoint**: `POST /api/sessions`

**Request Body**:
```json
{
  "name": "New Session",
  "description": "Optional description"
}
```

**Response** (201 Created):
```json
{
  "id": "session-456",
  "name": "New Session",
  "created_at": "2025-01-01T02:00:00Z"
}
```

#### Delete Session

Delete a session.

**Endpoint**: `DELETE /api/sessions/:id`

**Response** (204 No Content)

**Errors**:
- `404`: Session not found

### Memory

#### Search Memory

Search episodic memory.

**Endpoint**: `GET /api/memory/search`

**Query Parameters**:
- `query`: Search query string
- `limit` (optional): Maximum results (default: 10)

**Response** (200 OK):
```json
{
  "results": [
    {
      "id": "mem-123",
      "content": "Relevant memory content",
      "score": 0.95,
      "timestamp": "2025-01-01T00:00:00Z"
    }
  ]
}
```

#### Add Memory

Add entry to episodic memory.

**Endpoint**: `POST /api/memory/add`

**Request Body**:
```json
{
  "content": "Memory content",
  "metadata": {
    "source": "user",
    "session_id": "session-123"
  }
}
```

**Response** (201 Created):
```json
{
  "id": "mem-456",
  "status": "added"
}
```

#### Memory Statistics

Get memory system statistics.

**Endpoint**: `GET /api/memory/stats`

**Response** (200 OK):
```json
{
  "episodic_count": 1000,
  "semantic_count": 50,
  "total_size_bytes": 1048576
}
```

### Agents

#### List Agents

List all agent instances.

**Endpoint**: `GET /api/agents`

**Response** (200 OK):
```json
[
  {
    "id": "agent-123",
    "name": "Research Agent",
    "status": "active",
    "session_id": "session-123",
    "created_at": "2025-01-01T00:00:00Z"
  }
]
```

#### Get Agent

Get specific agent details.

**Endpoint**: `GET /api/agents/:id`

**Response** (200 OK):
```json
{
  "id": "agent-123",
  "name": "Research Agent",
  "status": "active",
  "session_id": "session-123",
  "created_at": "2025-01-01T00:00:00Z",
  "metadata": {}
}
```

**Errors**:
- `404`: Agent not found

#### Stop Agent

Stop a running agent.

**Endpoint**: `POST /api/agents/:id/stop`

**Response** (200 OK):
```json
{
  "id": "agent-123",
  "status": "terminated"
}
```

### Tools

#### List Tools

List available tools.

**Endpoint**: `GET /api/tools`

**Response** (200 OK):
```json
[
  {
    "name": "calculator",
    "description": "Perform calculations",
    "parameters": {
      "operation": "string",
      "input": "string"
    }
  }
]
```

#### Execute Tool

Execute a tool.

**Endpoint**: `POST /api/tools/:name/execute`

**Request Body**:
```json
{
  "parameters": {
    "operation": "evaluate",
    "input": "2 + 2"
  }
}
```

**Response** (200 OK):
```json
{
  "output": "{\"result\": 4}",
  "status": "success"
}
```

### Templates

#### List Templates

List available workflow templates.

**Endpoint**: `GET /api/templates`

**Response** (200 OK):
```json
[
  {
    "id": "research-assistant",
    "name": "Research Assistant",
    "description": "AI-powered research workflow",
    "category": "research",
    "parameters": ["topic", "max_sources"]
  }
]
```

#### Get Template

Get template details.

**Endpoint**: `GET /api/templates/:id`

**Response** (200 OK):
```json
{
  "id": "research-assistant",
  "name": "Research Assistant",
  "description": "AI-powered research workflow",
  "category": "research",
  "parameters": {
    "topic": {
      "type": "string",
      "required": true
    },
    "max_sources": {
      "type": "integer",
      "default": 10
    }
  }
}
```

#### Launch Template

Execute a template workflow.

**Endpoint**: `POST /api/templates/:id/launch`

**Request Body**:
```json
{
  "params": {
    "topic": "Rust async programming",
    "max_sources": 5
  },
  "session_id": "session-123"
}
```

**Response** (200 OK):
```json
{
  "session_id": "session-123",
  "status": "created",
  "workflow_id": "workflow-456"
}
```

### Configuration

#### Get Runtime Configuration

Get current runtime configuration.

**Endpoint**: `GET /api/config`

**Response** (200 OK):
```json
[
  {
    "name": "DEFAULT_ENGINE",
    "value": "lua",
    "category": "runtime"
  }
]
```

#### Update Runtime Configuration

Update runtime configuration.

**Endpoint**: `PUT /api/config`

**Request Body**:
```json
{
  "overrides": {
    "DEFAULT_ENGINE": "javascript"
  }
}
```

**Response** (200 OK):
```json
{
  "updated": ["DEFAULT_ENGINE"]
}
```

#### Get Static Configuration

Get static configuration source.

**Endpoint**: `GET /api/config/source`

**Response** (200 OK):
```json
{
  "source": "default_engine = \"lua\"\n..."
}
```

#### Update Static Configuration

Update static configuration file.

**Endpoint**: `PUT /api/config/source`

**Request Body**:
```json
{
  "source": "default_engine = \"javascript\"\n..."
}
```

**Response** (200 OK):
```json
{
  "status": "updated",
  "restart_required": true
}
```

#### Get Configuration Schema

Get configuration schema.

**Endpoint**: `GET /api/config/schema`

**Response** (200 OK):
```json
{
  "properties": {
    "default_engine": {
      "type": "string",
      "enum": ["lua", "javascript", "python"]
    }
  }
}
```

#### Get Profiles

List available configuration profiles.

**Endpoint**: `GET /api/config/profiles`

**Response** (200 OK):
```json
[
  {
    "name": "minimal",
    "description": "Minimal configuration"
  },
  {
    "name": "rag-prod",
    "description": "Production RAG setup"
  }
]
```

#### Restart Server

Restart server to apply configuration changes.

**Endpoint**: `POST /api/config/restart`

**Response** (202 Accepted):
```json
{
  "status": "restarting"
}
```

### Providers

#### List Providers

Get status of LLM providers.

**Endpoint**: `GET /api/providers`

**Response** (200 OK):
```json
[
  {
    "name": "openai",
    "status": "online",
    "models": ["gpt-4", "gpt-3.5-turbo"]
  },
  {
    "name": "anthropic",
    "status": "offline",
    "error": "API key not configured"
  }
]
```

## Request/Response Schemas

### Common Types

**Session**:
```typescript
{
  id: string;
  name?: string;
  description?: string;
  created_at: string;  // ISO 8601
  updated_at: string;  // ISO 8601
  message_count: number;
}
```

**Agent**:
```typescript
{
  id: string;
  name: string;
  status: "active" | "sleeping" | "terminated";
  session_id: string;
  created_at: string;
  metadata: Record<string, any>;
}
```

**Tool**:
```typescript
{
  name: string;
  description: string;
  parameters: Record<string, ParameterSchema>;
}
```

**Template**:
```typescript
{
  id: string;
  name: string;
  description: string;
  category: string;
  parameters: Record<string, ParameterSchema>;
}
```

## WebSocket Protocol

### Connection

**Endpoint**: `ws://localhost:3000/ws/stream`

**Connection**: Standard WebSocket upgrade

### Message Format

All messages are JSON:

```json
{
  "type": "event_type",
  "data": { /* event-specific data */ },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

### Event Types

#### Script Execution

```json
{
  "type": "script_execution",
  "data": {
    "execution_id": "exec-123",
    "status": "running" | "completed" | "failed",
    "output": "script output",
    "error": "error message (if failed)"
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

#### Session Update

```json
{
  "type": "session_update",
  "data": {
    "session_id": "session-123",
    "action": "created" | "updated" | "deleted",
    "session": { /* Session object */ }
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

#### Memory Change

```json
{
  "type": "memory_change",
  "data": {
    "action": "added" | "updated" | "deleted",
    "memory_id": "mem-123",
    "content": "memory content"
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

#### Agent Lifecycle

```json
{
  "type": "agent_lifecycle",
  "data": {
    "agent_id": "agent-123",
    "status": "active" | "sleeping" | "terminated",
    "session_id": "session-123"
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

### Connection Lifecycle

1. **Connect**: Client initiates WebSocket connection
2. **Subscribe**: Automatically subscribed to all events
3. **Receive**: Server streams events as they occur
4. **Disconnect**: Client closes connection or server shuts down
5. **Reconnect**: Client should implement automatic reconnection with exponential backoff

### Error Handling

WebSocket errors are sent as messages:

```json
{
  "type": "error",
  "data": {
    "code": "INTERNAL_ERROR",
    "message": "Error description"
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

## OpenAPI Specification

### Accessing Swagger UI

Interactive API documentation is available at:
- **URL**: http://localhost:3000/swagger-ui/
- **Features**: Try API calls, view schemas, test authentication

### Downloading OpenAPI JSON

Download the complete OpenAPI specification:
- **URL**: http://localhost:3000/api/openapi.json
- **Format**: OpenAPI 3.0
- **Use Cases**: Code generation, API testing, documentation

### API Versioning

Current version: **v1** (implicit in `/api/` prefix)

Future versions will use explicit versioning:
- `/api/v2/...` for breaking changes
- `/api/v1/...` remains available for backward compatibility

## Error Codes and Handling

### HTTP Status Codes

- **200 OK**: Successful request
- **201 Created**: Resource created successfully
- **204 No Content**: Successful deletion
- **400 Bad Request**: Invalid request parameters
- **401 Unauthorized**: Authentication required
- **403 Forbidden**: Insufficient permissions
- **404 Not Found**: Resource not found
- **500 Internal Server Error**: Server error
- **502 Bad Gateway**: Upstream service error
- **503 Service Unavailable**: Server overloaded or maintenance

### Error Response Format

All errors return JSON:

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "details": {
    "field": "parameter_name",
    "reason": "Specific validation failure"
  }
}
```

### Common Error Scenarios

**Invalid Script**:
```json
{
  "error": "Invalid script syntax",
  "code": "INVALID_SCRIPT",
  "details": {
    "line": 5,
    "column": 10,
    "message": "unexpected symbol"
  }
}
```

**Session Not Found**:
```json
{
  "error": "Session not found",
  "code": "NOT_FOUND",
  "details": {
    "session_id": "session-123"
  }
}
```

**Provider Offline**:
```json
{
  "error": "Provider unavailable",
  "code": "PROVIDER_OFFLINE",
  "details": {
    "provider": "openai",
    "reason": "API key not configured"
  }
}
```

## See Also

- [User Guide](../user-guide/12-web-interface.md) - End-user documentation
- [Developer Guide](../developer-guide/09-web-architecture.md) - Architecture and extension guide
- [CLI Reference](../user-guide/05-cli-reference.md) - Command-line interface
