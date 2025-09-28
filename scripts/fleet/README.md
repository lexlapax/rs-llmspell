# LLMSpell Fleet Management

Fleet management system for orchestrating multiple LLMSpell kernel processes with OS-level isolation.

## Overview

The fleet management system allows you to run multiple isolated LLMSpell kernels, each with its own configuration, resource limits, and client connections. This enables:

- **Multi-developer environments** - Each developer gets their own kernel
- **Collaborative sessions** - Multiple users can share a kernel for pair programming
- **Resource isolation** - OS-level process boundaries ensure true isolation
- **Service deployment** - Production-ready with systemd/Docker support

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Developer A   │     │   Developer B   │     │   Developer C   │
│  (OpenAI config)│     │(Anthropic config│     │ (Local config)  │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         ▼                       ▼                       ▼
    Port 9555               Port 9560               Port 9565
         │                       │                       │
┌────────┴────────┐     ┌────────┴────────┐     ┌────────┴────────┐
│  Kernel Process │     │  Kernel Process │     │  Kernel Process │
│   (PID: 12345)  │     │   (PID: 12346)  │     │   (PID: 12347)  │
│   Memory: 45MB  │     │   Memory: 45MB  │     │   Memory: 45MB  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                       │                       │
         └───────────────────────┴───────────────────────┘
                                 │
                         ┌───────▼────────┐
                         │ Fleet Registry │
                         │ (registry.json)│
                         └────────────────┘
```

## Quick Start

```bash
# 1. Make scripts executable
chmod +x llmspell-fleet

# 2. Spawn a kernel
./llmspell-fleet spawn

# 3. List running kernels
./llmspell-fleet list

# 4. Connect to a kernel (get connection info from list)
jupyter console --existing ~/.llmspell/fleet/kernel-abc123.json

# 5. Stop a kernel
./llmspell-fleet stop kernel-abc123

# 6. Stop all kernels
./llmspell-fleet stop-all
```

## Core Scripts

### 1. llmspell-fleet (Shell Implementation)

Main fleet manager script for basic operations.

**Commands:**
- `spawn [config] [language]` - Start a new kernel
- `list` - Show all running kernels
- `stop <kernel-id|port>` - Stop specific kernel
- `stop-all` - Stop all kernels
- `cleanup` - Remove dead kernel entries
- `health` - Check health of all kernels
- `logs <kernel-id|port>` - View kernel logs

**Example:**
```bash
# Spawn with default config
./llmspell-fleet spawn

# Spawn with specific config
./llmspell-fleet spawn openai.toml lua

# Stop by kernel ID
./llmspell-fleet stop kernel-abc123

# Stop by port
./llmspell-fleet stop 9555
```

### 2. fleet_manager.py (Python Implementation)

Advanced fleet management with resource monitoring.

**Features:**
- Process monitoring with psutil
- Resource limit enforcement
- Metrics collection
- Find-or-create kernel logic

**Commands:**
```bash
# Spawn with options
python3 fleet_manager.py spawn --config default.toml --language lua

# List with verbose output
python3 fleet_manager.py list --verbose

# Find or create matching kernel
python3 fleet_manager.py find --language lua --config default.toml

# Get metrics
python3 fleet_manager.py metrics

# Force stop all
python3 fleet_manager.py stop-all --force
```

### 3. fleet_http_service.py (REST API)

HTTP service for fleet management and discovery.

**Endpoints:**
- `GET /health` - Service health check
- `GET /kernels` - List all kernels
- `GET /kernels/<id>` - Get specific kernel
- `POST /kernels` - Spawn new kernel
- `DELETE /kernels/<id>` - Stop kernel
- `POST /find` - Find or create matching kernel
- `GET /metrics` - Fleet-wide metrics
- `GET /registry` - Raw registry (debug)

**Start service:**
```bash
python3 fleet_http_service.py --port 9550 --host 127.0.0.1
```

**Example API calls:**
```bash
# List kernels
curl http://localhost:9550/kernels

# Spawn kernel
curl -X POST http://localhost:9550/kernels \
  -H "Content-Type: application/json" \
  -d '{"language": "lua", "config": "default.toml"}'

# Get metrics
curl http://localhost:9550/metrics
```

## Example Scripts

### Multi-Developer Setup
```bash
./examples/multi_developer_setup.sh
```

Demonstrates:
- Multiple developers with different LLM providers
- Resource isolation between kernels
- Metrics collection across fleet
- Connection instructions for each developer

### Collaborative Session
```bash
./examples/collaborative_session.sh
```

Shows how to:
- Share a kernel between multiple users
- Implement shared data structures
- Collaborative debugging
- Real-time code review

### Resource Management
```bash
./examples/resource_management.sh
```

Covers:
- Memory limits (ulimit, cgroups)
- CPU priority (nice values)
- Process monitoring
- Resource cleanup
- Load testing

## Test Suites

### Integration Tests (Basic)
```bash
./test_fleet_integration.sh
```
- 22 test cases
- Basic functionality validation
- Quick smoke test (~30 seconds)

### Advanced Tests
```bash
./test_fleet_advanced.sh
```
- 36 comprehensive test cases
- Performance benchmarks
- Concurrent operations
- Error handling
- HTTP service validation

## Docker Support

### Docker Compose
```bash
# Start fleet with Docker
docker-compose up -d

# View specific kernel
docker-compose up kernel-lua-openai

# Stop all
docker-compose down
```

Configuration in `docker-compose.yml`:
- Per-kernel resource limits (memory, CPU)
- Health checks
- Volume management
- Network isolation

### Resource Limits
```yaml
services:
  kernel-lua-openai:
    mem_limit: 512m     # Memory limit
    cpus: 0.5          # CPU limit (50% of one core)
```

## Makefile Commands

```bash
# Initialize fleet directory
make init

# Spawn kernels
make spawn-openai
make spawn-anthropic
make spawn-local

# Management
make list
make health
make stop-all
make cleanup

# Docker
make docker-up
make docker-down
make docker-logs

# Demo & Testing
make demo
make test
make metrics
```

## Configuration

### Config Files Location
```
fleet/configs/
├── default.toml      # Default configuration
├── openai.toml       # OpenAI provider config
├── anthropic.toml    # Anthropic provider config
└── local.toml        # Local model config
```

### Registry Location
```
~/.llmspell/fleet/
├── registry.json     # Kernel registry
├── *.pid            # PID files
├── *.json           # Connection files
└── logs/
    └── *.log        # Kernel logs
```

## Resource Management

### Memory Limits
```bash
# Linux with ulimit
ulimit -m 524288 && ./llmspell-fleet spawn

# Docker
docker run --memory=512m llmspell
```

### CPU Priority
```bash
# Nice value (lower priority)
nice -n 10 ./llmspell-fleet spawn

# Docker CPU limit
docker run --cpus=0.5 llmspell
```

### Monitoring
```bash
# Real-time monitoring
python3 monitor_resources.py

# Fleet metrics
python3 fleet_manager.py metrics
```

## Performance Characteristics

- **Memory**: ~45MB per kernel (idle)
- **CPU**: 2-4% per kernel (idle)
- **Spawn Time**: <2 seconds typical, <5 seconds max
- **Port Range**: Starting from 9555
- **Concurrent Kernels**: Limited by system resources

## Troubleshooting

### Kernel Won't Start
```bash
# Check logs
cat ~/.llmspell/fleet/logs/kernel-*.log

# Clean up dead kernels
./llmspell-fleet cleanup

# Remove stale PID files
rm -f ~/.llmspell/fleet/*.pid
```

### Port Already in Use
```bash
# Find process using port
lsof -i :9555

# Kill process
kill -9 <PID>
```

### Resource Cleanup
```bash
# Complete cleanup
./cleanup_resources.sh

# Manual cleanup
pkill -f "llmspell kernel"
rm -f ~/.llmspell/fleet/*.pid
rm -f ~/.llmspell/fleet/*.json
```

## Production Deployment

### systemd Service
```ini
[Unit]
Description=LLMSpell Fleet Manager
After=network.target

[Service]
Type=simple
User=llmspell
WorkingDirectory=/opt/llmspell/fleet
ExecStart=/usr/bin/python3 /opt/llmspell/fleet/fleet_http_service.py
Restart=always

[Install]
WantedBy=multi-user.target
```

### Docker Production
```yaml
version: '3.8'
services:
  fleet-manager:
    image: llmspell:latest
    ports:
      - "9550:9550"
    volumes:
      - ./configs:/configs:ro
      - fleet-data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9550/health"]
      interval: 30s
      timeout: 3s
      retries: 3
```

## Key Features

1. **Zero Kernel Code Changes** - All orchestration is external
2. **True Process Isolation** - OS-level boundaries
3. **Simple Management** - Standard Unix tools
4. **Resource Control** - OS facilities (cgroups, ulimit, docker)
5. **Debug Isolation** - Per-process ExecutionManager
6. **Production Ready** - systemd/Docker support

## Dependencies

- **Required:**
  - Python 3.7+
  - LLMSpell kernel binary

- **Optional:**
  - psutil (Python monitoring)
  - Flask (HTTP service)
  - Docker & docker-compose
  - jq (JSON processing)

## Support

For issues or questions:
1. Check kernel logs in `~/.llmspell/fleet/logs/`
2. Run health check: `./llmspell-fleet health`
3. Review test results: `./test_fleet_advanced.sh`

## License

Part of the LLMSpell project. See main project LICENSE file.