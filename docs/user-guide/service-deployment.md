# Service Deployment Guide

**Version**: 0.9.0
**Last Updated**: December 2024

> **🚀 Production Deployment**: Deploy LLMSpell kernel as a system service with systemd (Linux) or launchd (macOS).

**🔗 Navigation**: [← User Guide](README.md) | [Configuration →](configuration.md) | [Troubleshooting →](troubleshooting.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Deployment Modes](#deployment-modes)
3. [systemd Deployment (Linux)](#systemd-deployment-linux)
4. [launchd Deployment (macOS)](#launchd-deployment-macos)
5. [Configuration](#configuration)
6. [Managing Services](#managing-services)
7. [Monitoring & Logging](#monitoring--logging)
8. [Security Best Practices](#security-best-practices)
9. [Troubleshooting](#troubleshooting)

---

## Quick Start

Install LLMSpell kernel as a service with auto-detection:

```bash
# Auto-detect platform and install as user service
./target/release/llmspell kernel install-service

# Install as system service with custom port
./target/release/llmspell kernel install-service --system --port 9600

# Enable and start immediately
./target/release/llmspell kernel install-service --enable --start
```

## Deployment Modes

### User Service
- Runs under your user account
- Starts when you log in
- Access to your home directory and environment
- Recommended for development and single-user systems

### System Service
- Runs as dedicated service user
- Starts at boot
- Isolated from user sessions
- Recommended for production servers

### Daemon Mode
- Background process with detached TTY
- PID file management
- Signal handling for graceful shutdown
- Automatic log rotation support

## systemd Deployment (Linux)

### Installation

```bash
# Install as user service
./target/release/llmspell kernel install-service --service-type systemd

# Install as system service
sudo ./target/release/llmspell kernel install-service --system --service-type systemd

# Custom configuration
./target/release/llmspell kernel install-service \
  --name my-llmspell \
  --port 9600 \
  --log-file /var/log/llmspell/kernel.log \
  --pid-file /var/run/llmspell/kernel.pid
```

### Generated Service File

The installation creates a service file at:
- User service: `~/.config/systemd/user/llmspell-kernel.service`
- System service: `/etc/systemd/system/llmspell-kernel.service`

Example service file:

```ini
[Unit]
Description=LLMSpell Kernel Service
After=network.target

[Service]
Type=forking
ExecStart=/usr/local/bin/llmspell kernel start --daemon --port 9555 --log-file /var/log/llmspell/kernel.log --pid-file /var/run/llmspell/kernel.pid
ExecStop=/usr/local/bin/llmspell kernel stop --pid-file /var/run/llmspell/kernel.pid
PIDFile=/var/run/llmspell/kernel.pid
Restart=on-failure
RestartSec=10

# Security hardening
PrivateTmp=yes
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/log/llmspell /var/run/llmspell

[Install]
WantedBy=multi-user.target
```

### Management Commands

```bash
# Enable service to start at boot
systemctl --user enable llmspell-kernel  # User service
sudo systemctl enable llmspell-kernel    # System service

# Start/stop/restart
systemctl --user start llmspell-kernel
systemctl --user stop llmspell-kernel
systemctl --user restart llmspell-kernel

# Check status
systemctl --user status llmspell-kernel

# View logs
journalctl --user -u llmspell-kernel -f
```

## launchd Deployment (macOS)

### Installation

```bash
# Install as user agent
./target/release/llmspell kernel install-service --service-type launchd

# Install as system daemon
sudo ./target/release/llmspell kernel install-service --system --service-type launchd

# With custom settings
./target/release/llmspell kernel install-service \
  --name com.example.llmspell \
  --port 9600 \
  --enable \
  --start
```

### Generated Plist File

The installation creates a plist file at:
- User agent: `~/Library/LaunchAgents/com.llmspell.kernel.plist`
- System daemon: `/Library/LaunchDaemons/com.llmspell.kernel.plist`

Example plist:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.llmspell.kernel</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/llmspell</string>
        <string>kernel</string>
        <string>start</string>
        <string>--daemon</string>
        <string>--port</string>
        <string>9555</string>
        <string>--log-file</string>
        <string>/usr/local/var/log/llmspell/kernel.log</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/llmspell/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/llmspell/stderr.log</string>
</dict>
</plist>
```

### Management Commands

```bash
# Load/unload service
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist
launchctl unload ~/Library/LaunchAgents/com.llmspell.kernel.plist

# Start/stop
launchctl start com.llmspell.kernel
launchctl stop com.llmspell.kernel

# Check status
launchctl list | grep llmspell

# View logs
tail -f /usr/local/var/log/llmspell/kernel.log
```

## Configuration

### Service Configuration File

Create a dedicated configuration file for the service:

```toml
# /etc/llmspell/kernel.toml (Linux)
# /usr/local/etc/llmspell/kernel.toml (macOS)

[kernel]
port = 9555
connection_file = "/var/lib/llmspell/kernel.json"
idle_timeout = 0  # Never timeout for service

[daemon]
daemonize = true
pid_file = "/var/run/llmspell/kernel.pid"
working_dir = "/var/lib/llmspell"
umask = 0o077  # Restrictive permissions

[logging]
log_file = "/var/log/llmspell/kernel.log"
log_level = "info"
max_size_mb = 100
max_backups = 5
compress = true

[security]
hmac_key = "your-secret-key-here"
allowed_origins = ["localhost", "127.0.0.1"]
max_message_size_mb = 10
rate_limit_per_minute = 100
```

### Environment Variables

Set environment variables in the service file:

```bash
# systemd environment file: /etc/default/llmspell-kernel
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
RUST_LOG=info
LLMSPELL_CONFIG=/etc/llmspell/kernel.toml
```

## Managing Services

### Start/Stop Operations

```bash
# Start kernel daemon manually
./target/release/llmspell kernel start --daemon --port 9555

# Check status
./target/release/llmspell kernel status

# Stop by PID file
./target/release/llmspell kernel stop --pid-file /var/run/llmspell/kernel.pid

# Stop by kernel ID
./target/release/llmspell kernel stop abc123
```

### Signal Handling

The daemon responds to standard Unix signals:

- `SIGTERM`: Graceful shutdown
- `SIGINT`: Graceful shutdown (Ctrl+C)
- `SIGHUP`: Reload configuration
- `SIGUSR1`: Dump statistics
- `SIGUSR2`: Toggle debug logging

```bash
# Graceful reload
kill -HUP $(cat /var/run/llmspell/kernel.pid)

# Get statistics
kill -USR1 $(cat /var/run/llmspell/kernel.pid)
```

## Monitoring & Logging

### Health Checks

```bash
# Built-in health endpoint
curl http://localhost:9555/health

# Detailed metrics
curl http://localhost:9555/metrics
```

### Log Management

```bash
# View real-time logs
tail -f /var/log/llmspell/kernel.log

# Rotate logs manually
logrotate -f /etc/logrotate.d/llmspell-kernel
```

Example logrotate configuration:

```
/var/log/llmspell/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 llmspell llmspell
    postrotate
        kill -USR1 $(cat /var/run/llmspell/kernel.pid) 2>/dev/null || true
    endscript
}
```

## Security Best Practices

### 1. Run as Dedicated User

```bash
# Create service user
sudo useradd -r -s /bin/false -d /var/lib/llmspell llmspell

# Set ownership
sudo chown -R llmspell:llmspell /var/lib/llmspell
sudo chown -R llmspell:llmspell /var/log/llmspell
```

### 2. File Permissions

```bash
# Restrictive permissions
chmod 600 /etc/llmspell/kernel.toml      # Config file
chmod 644 /var/run/llmspell/kernel.pid   # PID file
chmod 640 /var/log/llmspell/kernel.log   # Log file
```

### 3. Network Security

```toml
[security]
# Enable HMAC authentication
hmac_key = "$(openssl rand -base64 32)"

# Restrict origins
allowed_origins = ["localhost"]

# Enable TLS (requires certificates)
tls_enabled = true
tls_cert = "/etc/llmspell/cert.pem"
tls_key = "/etc/llmspell/key.pem"
```

### 4. Resource Limits

```ini
# systemd resource limits
[Service]
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
CPUQuota=200%
```

## Troubleshooting

### Service Won't Start

1. Check permissions:
```bash
ls -la /var/run/llmspell/
ls -la /var/log/llmspell/
```

2. Check port availability:
```bash
lsof -i :9555
netstat -an | grep 9555
```

3. Verify configuration:
```bash
./target/release/llmspell kernel start --dry-run
```

### Connection Issues

1. Check if service is running:
```bash
systemctl --user status llmspell-kernel
ps aux | grep llmspell
```

2. Test connectivity:
```bash
nc -zv localhost 9555
curl http://localhost:9555/health
```

3. Check firewall:
```bash
sudo iptables -L -n | grep 9555
sudo ufw status | grep 9555
```

### Performance Issues

1. Monitor resource usage:
```bash
systemctl --user status llmspell-kernel
top -p $(cat /var/run/llmspell/kernel.pid)
```

2. Check logs for warnings:
```bash
grep WARN /var/log/llmspell/kernel.log
grep ERROR /var/log/llmspell/kernel.log
```

3. Enable debug logging:
```bash
kill -USR2 $(cat /var/run/llmspell/kernel.pid)
```

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Address already in use" | Port conflict | Change port or stop conflicting service |
| "Permission denied" | Insufficient privileges | Check file permissions and user |
| "Connection refused" | Service not running | Start service and check logs |
| "Too many open files" | ulimit too low | Increase file descriptor limit |
| "Cannot create PID file" | Directory doesn't exist | Create directory with proper permissions |

---

**🔗 Next Steps**: [IDE Integration →](ide-integration.md) | [API Reference →](api/README.md)