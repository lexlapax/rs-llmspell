# LLMSpell Web Interface Guide

Complete guide to using the LLMSpell web interface for browser-based AI workflow development.

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Dashboard](#dashboard)
4. [Script Editor](#script-editor)
5. [Sessions Management](#sessions-management)
6. [Memory & Knowledge Base](#memory--knowledge-base)
7. [Agents & Workflows](#agents--workflows)
8. [Tools & Providers](#tools--providers)
9. [Template Library](#template-library)
10. [Configuration](#configuration)
11. [WebSocket Streaming](#websocket-streaming)
12. [Troubleshooting](#troubleshooting)

## Overview

The LLMSpell web interface provides a browser-based environment for developing and executing AI workflows. It offers a visual alternative to the CLI with real-time monitoring, interactive debugging, and collaborative features.

### When to Use Web Interface vs CLI

**Use the Web Interface when:**
- Developing and testing scripts interactively
- Monitoring real-time execution and events
- Managing multiple sessions visually
- Collaborating with team members
- Exploring templates and workflows
- Debugging complex agent interactions

**Use the CLI when:**
- Running production workflows
- Automating batch processes
- Integrating with CI/CD pipelines
- Scripting and shell integration
- Headless/server environments

## Getting Started

### Starting the Web Server

Start the web server using the CLI:

```bash
# Start on default port (3000)
llmspell web start

# Start on custom port
llmspell web start --port 8080

# Start as background daemon
llmspell web start --daemon

# Start with specific profile
llmspell -p rag-prod web start
```

### Accessing the Interface

Once started, access the web interface at:
- **Default**: http://localhost:3000
- **Custom port**: http://localhost:PORT

### First-Time Setup

1. **Verify Server Status**:
   ```bash
   llmspell web status
   ```

2. **Open in Browser**:
   ```bash
   llmspell web open
   ```

3. **Configure Providers** (if needed):
   - Navigate to Configuration → Providers
   - Add API keys for LLM providers (OpenAI, Anthropic, Google, etc.)

## Dashboard

The dashboard provides an overview of system status and quick actions.

### System Status

- **Server Uptime**: How long the server has been running
- **Active Sessions**: Number of currently active sessions
- **Memory Usage**: Current memory consumption
- **Provider Status**: Health of configured LLM providers

### Quick Actions

- **New Script**: Create and execute a new script
- **New Session**: Start a new conversation session
- **Browse Templates**: Explore available workflow templates
- **View Logs**: Access real-time system logs

## Script Editor

The script editor provides an integrated development environment for Lua, JavaScript, and Python scripts.

### Creating Scripts

1. Click **"New Script"** from the dashboard or Tools page
2. Select language (Lua, JavaScript, or Python)
3. Write your script in the editor
4. Click **"Run"** to execute

### Editor Features

- **Syntax Highlighting**: Language-specific syntax coloring
- **Auto-Completion**: Intelligent code suggestions
- **Line Numbers**: Easy navigation and debugging
- **Error Highlighting**: Real-time syntax error detection

### Running Scripts

Execute scripts and view output in the integrated console:

```lua
-- Example: Simple LLM query
local response = Agent.query("Explain async programming")
print(response)
```

### Console Integration

The console displays:
- **stdout**: Standard output from your script
- **stderr**: Error messages and warnings
- **info**: System information and status messages

Output is color-coded by type for easy identification.

## Sessions Management

Sessions organize conversation history and context for AI interactions.

### Creating Sessions

1. Navigate to **Sessions** page
2. Click **"New Session"**
3. Optionally provide a name and description
4. Session is created with unique ID

### Viewing Session History

- **List View**: All sessions with metadata (created, updated, message count)
- **Detail View**: Click a session to see full conversation history
- **Filtering**: Search sessions by name, date, or content
- **Sorting**: Order by creation date, update time, or name

### Session Artifacts

Each session can store artifacts (files, outputs, results):
- View artifacts in the session detail page
- Download artifacts individually
- Delete old artifacts to save space

## Memory & Knowledge Base

Manage episodic memory and semantic knowledge for RAG workflows.

### Episodic Memory Browser

View and search conversation history:
- **Timeline View**: Chronological display of interactions
- **Search**: Full-text search across all memories
- **Filters**: Filter by date range, session, or content type

### Semantic Knowledge Graph

Visualize consolidated knowledge:
- **Graph View**: Interactive visualization of knowledge relationships
- **Node Details**: Click nodes to see full content
- **Export**: Download knowledge graph data

### RAG Document Management

Upload and manage documents for retrieval:
1. Navigate to **Knowledge Base** → **Documents**
2. Click **"Upload Document"**
3. Select files (PDF, TXT, MD, etc.)
4. Documents are automatically embedded and indexed

### Vector Search Interface

Search your knowledge base:
- **Query**: Enter natural language search query
- **Results**: Ranked by relevance with similarity scores
- **Preview**: View document snippets in results
- **Open**: Click to view full document

## Agents & Workflows

Monitor and manage AI agent instances and workflow executions.

### Viewing Active Agents

The Agents page shows:
- **Active**: Currently running agents
- **Sleeping**: Idle agents waiting for events
- **Terminated**: Completed or stopped agents

### Agent Lifecycle Management

Control agent execution:
- **Stop**: Terminate a running agent
- **Restart**: Restart a terminated agent
- **View Logs**: See agent execution logs

### Workflow Execution

Monitor workflow progress:
- **Status**: Current workflow state
- **Steps**: Completed and pending steps
- **Outputs**: Results from each step
- **Errors**: Any failures or warnings

### Agent-to-Session Linking

Agents are linked to their originating sessions:
- Click agent to see associated session
- View all agents created in a session
- Track agent lineage and relationships

## Tools & Providers

Explore available tools and manage LLM provider configurations.

### Available Tools Catalog

Browse registered tools:
- **Name**: Tool identifier
- **Description**: What the tool does
- **Parameters**: Required and optional inputs
- **Examples**: Usage examples

### Tool Execution Interface

Execute tools directly from the UI:
1. Select a tool from the catalog
2. Fill in required parameters
3. Click **"Execute"**
4. View results in the output panel

### Provider Configuration

Manage LLM provider settings:
- **Status**: Provider health (online/offline/error)
- **API Keys**: Configure authentication
- **Models**: Available models for each provider
- **Limits**: Rate limits and quotas

### API Key Management

Securely store provider API keys:
1. Navigate to **Configuration** → **Providers**
2. Click **"Add API Key"**
3. Select provider (OpenAI, Anthropic, Google, etc.)
4. Enter API key (stored encrypted)
5. Test connection

## Template Library

Browse and launch pre-built AI workflow templates.

### Browsing Templates

Templates are organized by category:
- **Research**: research-assistant, data-analysis
- **Development**: code-generator, code-review
- **Content**: content-generation, document-processor
- **Productivity**: interactive-chat, workflow-orchestrator
- **Classification**: file-classification, knowledge-management

### Template Details

Click a template to view:
- **Description**: What the template does
- **Parameters**: Required configuration
- **Example Outputs**: Sample results
- **Source Code**: View template implementation

### Launching Templates

Execute a template workflow:
1. Select template from library
2. Configure parameters in the launch modal
3. Optionally select or create a session
4. Click **"Launch"**
5. Monitor execution in Sessions or Agents page

### Template Execution Monitoring

Track template progress:
- **Status**: Running, completed, or failed
- **Steps**: Current workflow step
- **Outputs**: Intermediate and final results
- **Logs**: Execution logs and errors

## Configuration

Manage system configuration and profiles.

### Profile Management

Switch between configuration profiles:
- **Current Profile**: Active profile name
- **Available Profiles**: List of built-in and custom profiles
- **Switch**: Change active profile (requires restart)

### Static Configuration Editing

Edit configuration files directly:
1. Navigate to **Configuration** → **Static Config**
2. View current TOML configuration
3. Edit in the text editor
4. Click **"Save"**
5. Restart server to apply changes

### Runtime Configuration Updates

Modify runtime settings without restart:
- **Environment Variables**: Override config values
- **Feature Flags**: Enable/disable features
- **Logging Levels**: Adjust verbosity

### Server Restart

Apply static configuration changes:
1. Make configuration edits
2. Click **"Restart Server"**
3. Server restarts with new configuration
4. Reconnect to web interface

## WebSocket Streaming

Monitor real-time events via WebSocket connection.

### Real-Time Event Streaming

The web interface uses WebSocket for live updates:
- **Script Execution**: Real-time output streaming
- **Session Updates**: New messages and changes
- **Memory Changes**: Knowledge base updates
- **Agent Lifecycle**: Agent state transitions

### Event Filtering

Filter events by type:
- **All Events**: Show everything
- **Scripts Only**: Script execution events
- **Sessions Only**: Session-related events
- **Agents Only**: Agent lifecycle events

### Connection Management

WebSocket connection status:
- **Connected**: Green indicator, receiving events
- **Disconnected**: Red indicator, attempting reconnect
- **Reconnecting**: Yellow indicator, connection in progress

Auto-reconnect is enabled by default with exponential backoff.

## Troubleshooting

Common issues and solutions.

### Server Won't Start

**Problem**: `llmspell web start` fails

**Solutions**:
1. Check if port is already in use:
   ```bash
   lsof -i :3000
   ```
2. Try a different port:
   ```bash
   llmspell web start --port 8080
   ```
3. Check logs for errors:
   ```bash
   llmspell web start --log-level debug
   ```

### Cannot Access Web Interface

**Problem**: Browser shows "Connection refused"

**Solutions**:
1. Verify server is running:
   ```bash
   llmspell web status
   ```
2. Check firewall settings
3. Ensure correct URL (http://localhost:3000)
4. Try opening in different browser

### WebSocket Connection Fails

**Problem**: Real-time updates not working

**Solutions**:
1. Check browser console for WebSocket errors
2. Verify server is running
3. Check for proxy/firewall blocking WebSocket
4. Refresh page to reconnect

### Scripts Fail to Execute

**Problem**: Script execution returns errors

**Solutions**:
1. Check script syntax in editor
2. Verify required providers are configured
3. Check API keys are valid
4. Review error messages in console
5. Check server logs for details

### Browser Compatibility

**Supported Browsers**:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

**Known Issues**:
- Older browsers may not support WebSocket
- Some features require JavaScript enabled
- Ad blockers may interfere with WebSocket

### Network Configuration

**CORS Issues**:
If accessing from different domain, configure CORS in server settings.

**Reverse Proxy**:
When using nginx/Apache, ensure WebSocket upgrade headers are forwarded:
```nginx
proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
```

### Performance Issues

**Slow Loading**:
1. Clear browser cache
2. Check network latency
3. Reduce concurrent sessions
4. Optimize script complexity

**High Memory Usage**:
1. Close unused sessions
2. Clear old artifacts
3. Reduce vector index size
4. Restart server periodically

## See Also

- [CLI Reference](05-cli-reference.md) - Command-line interface documentation
- [Getting Started](01-getting-started.md) - Initial setup and installation
- [Configuration Guide](03-configuration.md) - Detailed configuration options
- [Developer Guide](../developer-guide/09-web-architecture.md) - Extending the web interface
- [API Reference](../technical/web-api-reference.md) - HTTP API and WebSocket protocol
