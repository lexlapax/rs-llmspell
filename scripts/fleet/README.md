# LLMSpell Fleet Management

Fleet management system for orchestrating multiple LLMSpell kernel processes with OS-level isolation.

**✨ Now with full Docker support!** Build, deploy, and scale LLMSpell kernels using Docker containers with complete isolation, health checks, and resource management.

**📊 Comprehensive Monitoring & Observability!** Real-time dashboards, Prometheus metrics export, centralized log aggregation, and intelligent alerting for production-ready fleet management.

## Table of Contents

- [Overview](#overview)
- [Files Overview](#files-overview)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Core Scripts](#core-scripts)
  - [llmspell-fleet](#1-llmspell-fleet-shell-implementation)
  - [fleet_manager.py](#2-fleet_managerpy-python-implementation)
  - [fleet_http_service.py](#3-fleet_http_servicepy-rest-api)
- [Docker Support](#docker-support-comprehensive)
  - [Docker Architecture](#docker-architecture)
  - [Docker Files](#docker-files)
  - [Docker Quick Start](#docker-quick-start)
  - [Docker Troubleshooting](#docker-troubleshooting)
- [Monitoring & Observability](#monitoring--observability)
  - [Fleet Dashboard](#fleet-dashboard)
  - [Log Aggregator](#log-aggregator)
  - [Prometheus Metrics](#prometheus-metrics)
  - [Enhanced Metrics](#enhanced-metrics)
- [Example Scripts](#example-scripts)
- [Test Suites](#test-suites)
- [Makefile Commands](#makefile-commands)
- [Configuration](#configuration)
- [Resource Management](#resource-management)
- [Performance](#performance-characteristics)
- [Troubleshooting](#troubleshooting)
- [Production Deployment](#production-deployment)

## Overview

The fleet management system allows you to run multiple isolated LLMSpell kernels, each with its own configuration, resource limits, and client connections. This enables:

- **Multi-developer environments** - Each developer gets their own kernel
- **Collaborative sessions** - Multiple users can share a kernel for pair programming
- **Resource isolation** - OS-level process boundaries ensure true isolation
- **Service deployment** - Production-ready with systemd/Docker support

## Files Overview

```
scripts/fleet/
├── llmspell-fleet              # Shell script fleet manager
├── fleet_manager.py            # Python fleet manager with enhanced metrics
├── fleet_http_service.py       # REST API with Prometheus export
├── fleet_dashboard.py          # Terminal-based monitoring dashboard
├── log_aggregator.py           # Centralized log analysis tool
├── monitor_resources.py        # Real-time resource monitoring
├── docker-fleet.sh             # Docker management script
├── Dockerfile                  # Multi-stage Docker build
├── docker-compose.yml          # Docker orchestration config
├── Makefile                    # Automation commands
├── README.md                   # This documentation
│
├── configs/                    # Configuration files
│   ├── default.toml
│   ├── openai.toml
│   ├── anthropic.toml
│   └── local.toml
│
├── examples/                   # Example scripts
│   ├── multi_developer_setup.sh
│   ├── collaborative_session.sh
│   └── resource_management.sh
│
└── tests/                      # Test suites
    ├── test_fleet_integration.sh
    ├── test_fleet_advanced.sh
    └── test_monitoring.sh      # Monitoring feature tests
```

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

### Monitoring Tests
```bash
./test_monitoring.sh
```
- 10 monitoring-specific test cases
- Dashboard functionality
- Log aggregation validation
- Prometheus export verification
- Metrics collection testing
- Alert threshold validation

## Docker Support (Comprehensive)

### Docker Architecture

The Docker fleet provides containerized kernel orchestration with complete isolation:

```
┌────────────────────────────────────────────────────────────┐
│                    Docker Host Machine                      │
├────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────┐│
│  │ kernel-lua-openai│  │kernel-lua-anthropic│ │kernel-dev││
│  │  Container       │  │  Container         │ │Container ││
│  │  Port: 9555      │  │  Port: 9556        │ │Port: 9558││
│  │  Memory: 512MB   │  │  Memory: 512MB     │ │Mem: 2GB  ││
│  │  CPU: 0.5 core   │  │  CPU: 0.5 core     │ │CPU: 1.0  ││
│  └──────────────────┘  └──────────────────┘  └──────────┘│
│           │                      │                   │     │
│           └──────────────────────┴───────────────────┘     │
│                              │                              │
│                   ┌──────────▼──────────┐                  │
│                   │ llmspell-network     │                  │
│                   │ (Docker Bridge)      │                  │
│                   └──────────────────────┘                  │
└────────────────────────────────────────────────────────────┘
```

### Docker Files

#### 1. Dockerfile (Multi-stage Build)
Location: `scripts/fleet/Dockerfile`

**Features:**
- Multi-stage build for optimization (builder + runtime)
- Size optimization with stripped binaries
- Security hardening (non-root user: llmspell:1000)
- Minimal base image (debian:bookworm-slim)
- Health check integration
- Configurable ports (9555-9600)

**Build Process:**
```bash
# Build from fleet directory
cd scripts/fleet
docker build -f Dockerfile -t llmspell:latest ../..

# Or use Makefile
make docker-build

# Verify image
docker images | grep llmspell
```

#### 2. docker-fleet.sh Management Script
Location: `scripts/fleet/docker-fleet.sh`

**Commands:**
```bash
# Build Docker image
./docker-fleet.sh build

# Start fleet (default services)
./docker-fleet.sh up

# Start with specific profile
./docker-fleet.sh up dev         # Development profile
./docker-fleet.sh up javascript  # JavaScript kernel
./docker-fleet.sh up registry    # With registry service

# Scale services
./docker-fleet.sh scale kernel-lua-openai 3

# View logs
./docker-fleet.sh logs                    # All logs
./docker-fleet.sh logs kernel-lua-openai  # Specific service

# Health check
./docker-fleet.sh health

# Container shell access
./docker-fleet.sh shell kernel-lua-openai

# List containers
./docker-fleet.sh ps

# Stop fleet
./docker-fleet.sh down

# Clean everything (containers + images)
./docker-fleet.sh clean
```

#### 3. docker-compose.yml Configuration

**Services Defined:**
1. **kernel-lua-openai** - OpenAI provider kernel
2. **kernel-lua-anthropic** - Anthropic provider kernel
3. **kernel-javascript** - JavaScript kernel (profile: javascript)
4. **kernel-dev** - Development kernel with debug (profile: dev)
5. **fleet-registry** - Nginx registry service (profile: registry)

**Service Configuration:**
```yaml
kernel-lua-openai:
  image: llmspell:latest
  container_name: llmspell-kernel-lua-openai
  command: kernel start --daemon --port 9555
  ports:
    - "9555:9555"
  volumes:
    - ./configs/openai.toml:/etc/llmspell/config.toml:ro
    - ./connection-files:/var/lib/llmspell/connections
    - ./logs/kernel-lua-openai:/var/log/llmspell
  environment:
    LLMSPELL_CONFIG: /etc/llmspell/config.toml
    LLMSPELL_CONNECTION_FILE: /var/lib/llmspell/connections/kernel-lua-openai.json
    KERNEL_ID: kernel-lua-openai
  restart: unless-stopped
  mem_limit: 512m
  cpus: 0.5
  healthcheck:
    test: ["CMD", "nc", "-z", "localhost", "9555"]
    interval: 30s
    timeout: 3s
    retries: 3
  networks:
    - llmspell-network
```

### Docker Quick Start

```bash
# 1. Build the image
make docker-build
# or
./docker-fleet.sh build

# 2. Start the fleet
make docker-up
# or
./docker-fleet.sh up

# 3. Check status
./docker-fleet.sh ps
./docker-fleet.sh health

# 4. Connect to kernel
docker exec -it llmspell-kernel-lua-openai \
  jupyter console --existing /var/lib/llmspell/connections/kernel-lua-openai.json

# 5. View logs
./docker-fleet.sh logs kernel-lua-openai

# 6. Stop everything
make docker-down
# or
./docker-fleet.sh down
```

### Docker Resource Management

#### Memory Limits
```yaml
# Per-service in docker-compose.yml
mem_limit: 512m  # 512MB limit
mem_limit: 2g    # 2GB for dev kernel
```

#### CPU Limits
```yaml
cpus: 0.5   # 50% of one core
cpus: 1.0   # Full core for dev
```

#### Volume Mounts
- **Config files**: Read-only mount from `./configs/`
- **Connection files**: Shared volume for kernel discovery
- **Logs**: Per-kernel log directories
- **Workspace**: Development code mount (dev profile only)

### Docker Health Checks

Each container includes health checks:
```yaml
healthcheck:
  test: ["CMD", "nc", "-z", "localhost", "9555"]
  interval: 30s      # Check every 30 seconds
  timeout: 3s        # Timeout after 3 seconds
  retries: 3         # Mark unhealthy after 3 failures
  start-period: 5s   # Grace period on startup
```

Monitor health status:
```bash
# Check all container health
./docker-fleet.sh health

# Docker native health check
docker ps --format "table {{.Names}}\t{{.Status}}"
```

### Docker Networking

All containers join the `llmspell-network` bridge network:
- Internal DNS resolution by container name
- Isolated from host network
- Port mapping for external access
- Container-to-container communication enabled

### Docker Profiles

Use profiles to control which services start:

```bash
# Default (no profile) - starts basic kernels
docker-compose up

# Development profile - includes debug kernel
docker-compose --profile dev up

# JavaScript profile - includes JS kernel
docker-compose --profile javascript up

# Registry profile - includes registry service
docker-compose --profile registry up

# Multiple profiles
docker-compose --profile dev --profile registry up
```

### Docker Troubleshooting

#### Container Won't Start
```bash
# Check container logs
docker logs llmspell-kernel-lua-openai

# Inspect container
docker inspect llmspell-kernel-lua-openai

# Check build logs
docker build -f Dockerfile -t llmspell:test ../.. --progress=plain
```

#### Build Failures
```bash
# Clean build cache
docker system prune -a

# Build with no cache
docker build --no-cache -f Dockerfile -t llmspell:latest ../..

# Check disk space
docker system df
```

#### Network Issues
```bash
# List networks
docker network ls

# Inspect network
docker network inspect scripts_llmspell-network

# Test connectivity
docker exec llmspell-kernel-lua-openai ping kernel-lua-anthropic
```

#### Resource Issues
```bash
# Check resource usage
docker stats

# Limit check
docker inspect llmspell-kernel-lua-openai | grep -A 5 "HostConfig"

# Clean up unused resources
docker system prune -a --volumes
```

### Docker Production Deployment

#### Production docker-compose.yml
```yaml
services:
  kernel-production:
    image: llmspell:latest
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
    restart: always
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

#### Docker Swarm Deployment
```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml llmspell-fleet

# Scale service
docker service scale llmspell-fleet_kernel-lua-openai=5

# Update service
docker service update --limit-memory 1G llmspell-fleet_kernel-lua-openai
```

### Docker Security Best Practices

1. **Non-root user**: Containers run as `llmspell:1000`
2. **Read-only mounts**: Config files mounted read-only
3. **Network isolation**: Custom bridge network
4. **Resource limits**: Memory and CPU constraints
5. **Health checks**: Automatic unhealthy container handling
6. **Minimal base image**: Using debian:bookworm-slim
7. **No unnecessary packages**: Only runtime dependencies

## Monitoring & Observability

The fleet management system includes comprehensive monitoring and observability features for tracking kernel health, performance, and logs.

### Fleet Dashboard

**Tool**: `fleet_dashboard.py`

A terminal-based real-time monitoring dashboard for visualizing fleet status and resource usage.

**Features:**
- Real-time kernel status overview
- Resource usage visualization (memory, CPU)
- Alert thresholds for anomaly detection
- Auto-refresh with configurable intervals
- Export metrics to JSON/CSV
- Works with or without rich library

**Usage:**
```bash
# Run dashboard once
python3 fleet_dashboard.py --once

# Continuous monitoring (auto-refresh every 5 seconds)
python3 fleet_dashboard.py

# Custom refresh interval
python3 fleet_dashboard.py --refresh 10

# Export metrics
python3 fleet_dashboard.py --export metrics.json --format json
python3 fleet_dashboard.py --export metrics.csv --format csv

# Custom alert thresholds
python3 fleet_dashboard.py --threshold-memory 2000 --threshold-cpu 90
```

**Dashboard Output:**
```
================================================================================
LLMSpell Fleet Dashboard - 2025-09-27T18:29:54.084686
================================================================================

FLEET SUMMARY:
  Total Kernels: 3
  Running: 2 | Dead: 1
  Total Memory: 89.8 MB
  Total CPU: 3.4%
  Avg Memory: 44.9 MB
  Avg CPU: 1.7%

ALERTS:
  ⚠️  High memory: kernel-abc123 using 1024MB
  ⚠️  Long uptime: kernel-def456 running 26h

KERNEL DETAILS:
--------------------------------------------------------------------------------
ID           Port   Lang     Status   Memory     CPU      Uptime
--------------------------------------------------------------------------------
kernel-abc123  9555 lua      ✓ RUN      1024.0MB   45.2%     26.1h
kernel-def456  9560 python   ✓ RUN        44.9MB    1.7%      0.5h
kernel-ghi789  9565 lua      ✗ DEAD           -        -         -
--------------------------------------------------------------------------------
```

### Log Aggregator

**Tool**: `log_aggregator.py`

Centralized log collection and analysis tool for managing logs from all kernel processes.

**Features:**
- Aggregate logs from multiple kernels
- Search logs with regex patterns
- Monitor error rates with alerts
- Log rotation based on retention policy
- Real-time log tailing (like `tail -f`)
- Export logs to JSON/text formats

**Commands:**
```bash
# Tail all kernel logs
python3 log_aggregator.py tail -f
python3 log_aggregator.py tail -n 50

# Search logs with pattern
python3 log_aggregator.py search "ERROR" --context 3
python3 log_aggregator.py search "timeout" --kernel kernel-abc123

# Aggregate log summary
python3 log_aggregator.py aggregate -n 100

# Monitor error rates
python3 log_aggregator.py monitor
python3 log_aggregator.py monitor --continuous --interval 10

# Rotate old logs
python3 log_aggregator.py rotate

# Export logs
python3 log_aggregator.py export logs.json --format json
python3 log_aggregator.py export logs.txt --format text
```

**Error Monitoring:**
The log aggregator tracks various error patterns:
- ERROR/FATAL - Critical errors
- WARN/WARNING - Warning messages
- panic/crash/abort - System failures
- timeout/timed out - Timeout issues
- connection refused - Network problems
- out of memory/OOM - Memory issues
- permission denied - Access problems

Alert thresholds can be configured for automatic notifications.

### Prometheus Metrics

**Endpoint**: `/metrics/prometheus` or `/metrics?format=prometheus`

The fleet HTTP service exports metrics in Prometheus format for integration with monitoring stacks.

**Exported Metrics:**
```
# Fleet-wide metrics
llmspell_kernels_total         # Total number of kernels in registry
llmspell_kernels_active        # Number of active running kernels
llmspell_kernels_dead          # Number of dead kernels
llmspell_memory_mb_total       # Total memory usage in MB
llmspell_cpu_percent_total     # Total CPU usage percentage
llmspell_connections_total     # Total number of connections
llmspell_threads_total         # Total number of threads

# Per-kernel metrics with labels
llmspell_kernel_memory_mb{kernel_id="...", port="...", language="..."}
llmspell_kernel_cpu_percent{kernel_id="...", port="...", language="..."}
llmspell_kernel_uptime_seconds{kernel_id="...", port="...", language="..."}
llmspell_kernel_connections{kernel_id="...", port="...", language="..."}
```

**Prometheus Configuration:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'llmspell-fleet'
    static_configs:
      - targets: ['localhost:9550']
    metrics_path: '/metrics/prometheus'
    scrape_interval: 30s
```

**Grafana Dashboard Example:**
```json
{
  "dashboard": {
    "title": "LLMSpell Fleet Monitoring",
    "panels": [
      {
        "title": "Active Kernels",
        "targets": [
          {"expr": "llmspell_kernels_active"}
        ]
      },
      {
        "title": "Memory Usage by Kernel",
        "targets": [
          {"expr": "llmspell_kernel_memory_mb"}
        ]
      },
      {
        "title": "CPU Usage by Kernel",
        "targets": [
          {"expr": "llmspell_kernel_cpu_percent"}
        ]
      }
    ]
  }
}
```

### Enhanced Metrics

The `fleet_manager.py` now provides comprehensive metrics collection with aggregation:

**Metrics Structure:**
```python
{
  "timestamp": "2025-09-27T18:00:00Z",
  "total_kernels": 3,
  "kernels": [
    {
      "id": "kernel-abc123",
      "port": 9555,
      "language": "lua",
      "status": "running",
      "memory_mb": 44.5,
      "memory_percent": 0.06,
      "cpu_percent": 1.7,
      "connections": 5,
      "connection_details": [...],
      "threads": 44,
      "uptime_seconds": 1800,
      "nice": 0,
      "io": {
        "read_bytes": 1048576,
        "write_bytes": 524288
      }
    }
  ],
  "aggregated": {
    "active_kernels": 2,
    "dead_kernels": 1,
    "total_memory_mb": 89.0,
    "avg_memory_mb": 44.5,
    "total_cpu_percent": 3.4,
    "avg_cpu_percent": 1.7,
    "total_connections": 10,
    "total_threads": 88
  }
}
```

**Access Methods:**
```bash
# Via fleet_manager.py
python3 fleet_manager.py metrics

# Via HTTP API
curl http://localhost:9550/metrics

# Via Makefile
make metrics
```

### Monitoring Commands Summary

```bash
# Dashboard
python3 fleet_dashboard.py                    # Real-time dashboard
python3 fleet_dashboard.py --once            # Single snapshot
python3 fleet_dashboard.py --export data.json # Export metrics

# Logs
python3 log_aggregator.py tail -f            # Follow all logs
python3 log_aggregator.py search "ERROR"     # Search for errors
python3 log_aggregator.py monitor            # Monitor error rates

# Metrics
python3 fleet_manager.py metrics             # Get JSON metrics
curl http://localhost:9550/metrics/prometheus # Prometheus format

# Resource Monitoring
python3 monitor_resources.py                 # Real-time resources
make metrics                                  # Fleet summary
```

### Alert Configuration

Configure alert thresholds for proactive monitoring:

**Dashboard Alerts:**
```python
alert_thresholds = {
    "memory_mb": 1000,      # Alert if kernel uses > 1GB
    "cpu_percent": 80,      # Alert if CPU > 80%
    "connections": 100,     # Alert if connections > 100
    "uptime_hours": 24      # Alert if uptime > 24 hours
}
```

**Log Aggregator Alerts:**
```python
alert_thresholds = {
    'ERROR': 10,       # Alert after 10 errors
    'CRITICAL': 1,     # Alert immediately on critical
    'WARNING': 20,     # Alert after 20 warnings
    'TIMEOUT': 5       # Alert after 5 timeouts
}
```

### Performance Impact

All monitoring features are designed for minimal overhead:
- **psutil overhead**: <1% CPU, ~10MB memory
- **Metrics collection**: ~10ms per kernel
- **Log aggregation**: Streaming with minimal buffering
- **Dashboard refresh**: Configurable interval (default 5s)
- **Prometheus export**: <50ms response time

### Testing Monitoring

Run the monitoring test suite:
```bash
# Run all monitoring tests
./test_monitoring.sh

# Expected output:
=== Fleet Monitoring Test Suite ===
Testing: Enhanced metrics collection... ✓ PASS
Testing: Fleet dashboard (simple)... ✓ PASS
Testing: Dashboard export to JSON... ✓ PASS
Testing: Log aggregator help... ✓ PASS
Testing: Log aggregation... ✓ PASS
Testing: Log search functionality... ✓ PASS
Testing: Prometheus metrics endpoint... ✓ PASS
Testing: Prometheus format validation... ✓ PASS
Testing: Resource monitor script... ✓ PASS
Testing: Makefile metrics target... ✓ PASS

=== Test Summary ===
Passed: 10
Failed: 0
✓ All monitoring tests passed!
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
7. **Comprehensive Monitoring** - Real-time dashboards, metrics, logs
8. **Prometheus Integration** - Export metrics for Grafana/AlertManager
9. **Intelligent Alerting** - Configurable thresholds for anomalies
10. **Log Analysis** - Centralized aggregation with search & rotation

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
4. Monitor real-time status: `python3 fleet_dashboard.py`
5. Search logs for errors: `python3 log_aggregator.py search "ERROR"`
6. Check metrics: `python3 fleet_manager.py metrics`
7. Run monitoring tests: `./test_monitoring.sh`

## License

Part of the LLMSpell project. See main project LICENSE file.