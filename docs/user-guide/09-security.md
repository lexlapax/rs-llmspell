# Security & Permissions Guide

**Navigation**: [← User Guide](README.md) | [Configuration](configuration.md) | [Concepts](concepts.md)

---

## Overview

LLMSpell implements defense-in-depth security with a three-level security model and comprehensive sandbox system. This guide shows you how to configure permissions for network access, process execution, and file operations.

**Quick Links**:
- [Understanding Security Levels](#understanding-security-levels)
- [Configuring Permissions](#configuring-permissions)
- [Environment Variable Override](#environment-variable-override)
- [Common Scenarios](#common-scenarios)
- [Troubleshooting](#troubleshooting)

---

## Understanding Security Levels

LLMSpell uses three security levels that determine what resources tools can access:

### Safe (Pure Computation)

**What it allows**: Pure computation only - no file, network, or process access

**Use when**: Tool needs no external resources (calculations, text processing, data transformation)

**Examples**:
- `calculator` - Mathematical operations
- `hash-calculator` - Hashing and encoding
- `text-manipulator` - String processing
- `json-processor` - JSON manipulation

**Configuration**: No special configuration needed - works out of the box

---

### Restricted (Controlled Access)

**What it allows**: Explicit permissions via allowlists - requires configuration

**Use when**: Tool needs controlled access to files, network, or processes

**Examples**:
- `file-operations` - File read/write with path allowlisting
- `http-request` - Network requests to allowed domains
- `process-executor` - Execute whitelisted commands only
- `web-search` - Search with domain filtering

**Configuration**: Required via `[tools.*]` sections in config.toml (see below)

**Default behavior**: DENY unless explicitly allowed

---

### Privileged (Full System Access)

**What it allows**: Unrestricted access to all system resources

**Use when**: Trusted system administration tasks only

**⚠️ Warning**: Requires security review - should be exception, not rule

**Examples**: System monitoring, privileged administration tools

**Configuration**: Tools must explicitly declare Privileged security level

---

## Configuring Permissions

### Configuration File Structure

Add to your `config.toml`:

```toml
[tools.file_operations]
enabled = true
allowed_paths = ["/workspace", "/tmp/llmspell"]
blocked_extensions = ["exe", "dll", "so", "dylib"]
max_file_size = 50000000  # 50MB
max_depth = 10

[tools.network]
timeout_seconds = 30
max_retries = 3
verify_ssl = true

[tools.web_search]
allowed_domains = ["*"]  # Or specify domains
rate_limit_per_minute = 30

[tools.http_request]
allowed_hosts = ["*"]  # Or specify hosts
blocked_hosts = ["localhost", "127.0.0.1"]
max_request_size = 10000000  # 10MB

[tools.system]
allow_process_execution = false  # Set true to enable
allowed_commands = "echo,cat,ls,pwd"  # Comma-separated allowlist
command_timeout_seconds = 30
allowed_env_vars = "HOME,PATH"
```

---

## Environment Variable Override

For CI/CD, Docker, systemd, or quick testing, override security settings with environment variables instead of modifying `config.toml`.

### Quick Examples

**Enable file access for single script run:**
```bash
LLMSPELL_ALLOW_FILE_ACCESS=true ./target/release/llmspell run script.lua
```

**Test with relaxed permissions:**
```bash
export LLMSPELL_ALLOW_FILE_ACCESS="true"
export LLMSPELL_ALLOW_NETWORK_ACCESS="true"
export LLMSPELL_TOOLS_ALLOWED_PATHS="/tmp,/workspace"
./target/release/llmspell run test-script.lua
```

**Docker container with restricted network:**
```bash
docker run \
  -e LLMSPELL_ALLOW_FILE_ACCESS=false \
  -e LLMSPELL_ALLOW_NETWORK_ACCESS=true \
  -e LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS="api.company.com" \
  -e LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS="localhost,127.0.0.1,169.254.169.254" \
  myimage ./target/release/llmspell run script.lua
```

### Security Environment Variables Reference

All TOML security settings can be overridden via environment variables:

| TOML Config Path | Environment Variable | Example Value |
|-----------------|---------------------|---------------|
| `runtime.security.allow_file_access` | `LLMSPELL_ALLOW_FILE_ACCESS` | `true` / `false` |
| `runtime.security.allow_network_access` | `LLMSPELL_ALLOW_NETWORK_ACCESS` | `true` / `false` |
| `runtime.security.allow_process_spawn` | `LLMSPELL_ALLOW_PROCESS_SPAWN` | `true` / `false` |
| `tools.file_operations.allowed_paths` | `LLMSPELL_TOOLS_ALLOWED_PATHS` | `/tmp,/workspace,/data` |
| `tools.file_operations.max_file_size` | `LLMSPELL_TOOLS_MAX_FILE_SIZE` | `104857600` (bytes) |
| `tools.file_operations.blocked_extensions` | `LLMSPELL_TOOLS_BLOCKED_EXTENSIONS` | `exe,dll,so,dylib` |
| `tools.file_operations.max_depth` | `LLMSPELL_TOOLS_MAX_DEPTH` | `10` |
| `tools.web_search.allowed_domains` | `LLMSPELL_TOOLS_WEB_ALLOWED_DOMAINS` | `*.openai.com,github.com` |
| `tools.web_search.blocked_domains` | `LLMSPELL_TOOLS_WEB_BLOCKED_DOMAINS` | `spam.com,malware.com` |
| `tools.web_search.rate_limit_per_minute` | `LLMSPELL_TOOLS_WEB_RATE_LIMIT` | `100` |
| `tools.http_request.allowed_hosts` | `LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS` | `api.example.com,*.company.com` |
| `tools.http_request.blocked_hosts` | `LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS` | `localhost,127.0.0.1,169.254.169.254` |
| `tools.http_request.timeout_seconds` | `LLMSPELL_TOOLS_HTTP_TIMEOUT` | `60` |
| `tools.http_request.max_redirects` | `LLMSPELL_TOOLS_HTTP_MAX_REDIRECTS` | `5` |
| `tools.http_request.verify_ssl` | `LLMSPELL_TOOLS_HTTP_VERIFY_SSL` | `true` / `false` |
| `tools.system.allow_process_execution` | `LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC` | `true` / `false` |
| `tools.system.allowed_commands` | `LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS` | `echo,cat,ls,pwd,git` |
| `tools.system.command_timeout_seconds` | `LLMSPELL_TOOLS_SYSTEM_TIMEOUT` | `60` |
| `tools.system.allowed_env_vars` | `LLMSPELL_TOOLS_SYSTEM_ALLOWED_ENV` | `HOME,PATH,LANG` |
| `tools.network.timeout_seconds` | `LLMSPELL_TOOLS_NETWORK_TIMEOUT` | `30` |
| `tools.network.max_retries` | `LLMSPELL_TOOLS_NETWORK_RETRIES` | `3` |
| `tools.network.verify_ssl` | `LLMSPELL_TOOLS_NETWORK_VERIFY_SSL` | `true` / `false` |

**Complete list**: See [Configuration Guide - Environment Variables](configuration.md#security--permissions-variables) for all 50+ security environment variables.

### Precedence Order

Environment variables override TOML config:

1. **Command-line arguments** (highest priority - not yet implemented)
2. **Environment variables** ← Override TOML config
3. **Config file** (`config.toml`)
4. **Built-in defaults** (lowest priority)

**Example**: If `config.toml` sets `allow_file_access = false` but `LLMSPELL_ALLOW_FILE_ACCESS=true` is set, file access is **enabled**.

### Use Cases

#### CI/CD Integration

**GitHub Actions:**
```yaml
# .github/workflows/test.yml
env:
  LLMSPELL_ALLOW_FILE_ACCESS: "true"
  LLMSPELL_ALLOW_NETWORK_ACCESS: "true"
  LLMSPELL_TOOLS_ALLOWED_PATHS: "/github/workspace,/tmp"
  LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC: "true"
  LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS: "git,echo,cat,ls,python3"

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run LLMSpell tests
        run: ./target/release/llmspell run test-suite.lua
```

**GitLab CI:**
```yaml
# .gitlab-ci.yml
test:
  variables:
    LLMSPELL_ALLOW_FILE_ACCESS: "true"
    LLMSPELL_TOOLS_ALLOWED_PATHS: "/builds/$CI_PROJECT_PATH,/tmp"
    LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS: "git,echo,cat"
  script:
    - ./target/release/llmspell run integration-tests.lua
```

#### systemd Service

**Production service with restricted permissions:**
```ini
# /etc/systemd/system/llmspell.service
[Unit]
Description=LLMSpell Kernel Service
After=network.target

[Service]
Type=simple
User=llmspell
Group=llmspell

# Security environment variables
Environment="LLMSPELL_ALLOW_FILE_ACCESS=false"
Environment="LLMSPELL_ALLOW_NETWORK_ACCESS=true"
Environment="LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS=api.internal.company.com,*.openai.com"
Environment="LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS=localhost,127.0.0.1,169.254.169.254,10.0.0.0/8"
Environment="LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC=false"

# Runtime settings
Environment="LLMSPELL_CONFIG=/etc/llmspell/config.toml"
Environment="RUST_LOG=info"

ExecStart=/usr/local/bin/llmspell kernel start --daemon
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

#### Docker Deployment

**Dockerfile with security environment variables:**
```dockerfile
FROM rust:1.75-slim
WORKDIR /app

# Copy binary
COPY target/release/llmspell /usr/local/bin/

# Set security defaults (restrictive)
ENV LLMSPELL_ALLOW_FILE_ACCESS=false
ENV LLMSPELL_ALLOW_NETWORK_ACCESS=true
ENV LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS=api.openai.com,api.anthropic.com
ENV LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS=localhost,127.0.0.1,169.254.169.254
ENV LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC=false

CMD ["llmspell", "kernel", "start"]
```

**Docker Compose with per-environment security:**
```yaml
# docker-compose.yml
version: '3.8'

services:
  llmspell-dev:
    image: llmspell:latest
    environment:
      # Development (relaxed)
      LLMSPELL_ALLOW_FILE_ACCESS: "true"
      LLMSPELL_ALLOW_NETWORK_ACCESS: "true"
      LLMSPELL_TOOLS_ALLOWED_PATHS: "/workspace,/tmp"
      LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC: "true"
      RUST_LOG: "debug"
    volumes:
      - ./workspace:/workspace

  llmspell-prod:
    image: llmspell:latest
    environment:
      # Production (restricted)
      LLMSPELL_ALLOW_FILE_ACCESS: "false"
      LLMSPELL_ALLOW_NETWORK_ACCESS: "true"
      LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS: "api.company.com"
      LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS: "localhost,127.0.0.1,169.254.169.254,10.0.0.0/8,172.16.0.0/12"
      LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC: "false"
      RUST_LOG: "warn"
```

#### Quick Testing Patterns

**Enable file access for debugging:**
```bash
LLMSPELL_ALLOW_FILE_ACCESS=true \
LLMSPELL_TOOLS_ALLOWED_PATHS="/tmp,/workspace" \
./target/release/llmspell run debug-script.lua
```

**Test with specific domain allowlist:**
```bash
LLMSPELL_TOOLS_WEB_ALLOWED_DOMAINS="github.com,*.openai.com" \
LLMSPELL_TOOLS_WEB_RATE_LIMIT=200 \
./target/release/llmspell run web-scraper.lua
```

**Run with process execution enabled:**
```bash
LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC=true \
LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS="git,python3,echo" \
./target/release/llmspell run automation.lua
```

### Security Best Practices with Environment Variables

**1. Use restrictive defaults in production:**
```bash
# Start with deny-all, then explicitly allow
export LLMSPELL_ALLOW_FILE_ACCESS="false"
export LLMSPELL_ALLOW_NETWORK_ACCESS="true"
export LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS="api.internal.company.com"
```

**2. Always block SSRF-prone hosts:**
```bash
# Prevent Server-Side Request Forgery
export LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS="localhost,127.0.0.1,169.254.169.254,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16"
```

**3. Use environment-specific configurations:**
```bash
# development.env
LLMSPELL_ALLOW_FILE_ACCESS=true
LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC=true

# production.env
LLMSPELL_ALLOW_FILE_ACCESS=false
LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC=false
```

**4. Document security exceptions:**
```bash
# CI needs file access for test artifacts
export LLMSPELL_ALLOW_FILE_ACCESS="true"  # Required for test artifact generation
export LLMSPELL_TOOLS_ALLOWED_PATHS="/tmp/test-artifacts"
```

**5. Audit environment variables in production:**
```bash
# Check what security settings are active
env | grep LLMSPELL | sort
```

---

### Network Access Configuration

#### Allow Specific Domains

```toml
[tools.web_search]
allowed_domains = [
    "api.openai.com",
    "*.anthropic.com",  # Wildcard for subdomains
    "github.com"
]
blocked_domains = []
rate_limit_per_minute = 100
```

#### HTTP Request Permissions

```toml
[tools.http_request]
allowed_hosts = [
    "api.example.com",
    "*.company.com"
]
blocked_hosts = ["localhost", "127.0.0.1", "0.0.0.0"]  # Prevent SSRF
timeout_seconds = 30
max_redirects = 5
```

---

### Process Execution Configuration

**⚠️ Security Critical**: Process execution is disabled by default

```toml
[tools.system]
allow_process_execution = true  # Must explicitly enable
allowed_commands = "echo,cat,ls,pwd,date,whoami"  # Allowlist only these
# blocked_commands enforced by ProcessExecutorTool:
#   rm, sudo, chmod, chown, curl, wget, ssh, scp, etc.
command_timeout_seconds = 30
max_output_size = 1000000  # 1MB
allowed_env_vars = "HOME,PATH,LANG"  # Allowlist env vars
```

**Default allowed commands** (safe):
- `echo`, `cat`, `ls`, `pwd`, `date`, `whoami`, `hostname`, `env`, `printenv`, `uname`

**Default blocked commands** (dangerous):
- `rm`, `sudo`, `chmod`, `chown`, `curl`, `wget`, `ssh`, `scp`, `rsync`, `git`, `python`, `python3`, `node`, `sh`, `bash`, `zsh`

---

### File System Access Configuration

```toml
[tools.file_operations]
enabled = true
allowed_paths = [
    "/tmp",           # Safe scratch directory
    "/workspace",     # Your project directory
    "/data"           # Data directory
]
max_file_size = 50000000  # 50MB limit
atomic_writes = true
max_depth = 10  # Directory traversal depth
allowed_extensions = []  # Empty = all allowed except blocked
blocked_extensions = ["exe", "dll", "so", "dylib", "bin"]
validate_file_types = true
```

**Path Traversal Protection**: Automatically prevents `../` attacks

**Symlink Handling**: Resolves symlinks and validates against allowlist

---

## Sandbox Components

### FileSandbox

**Purpose**: Enforces file access permissions and prevents path traversal

**How it works**:
1. Validates requested path against `allowed_paths`
2. Resolves symlinks to prevent escapes
3. Checks file extension against `allowed_extensions`/`blocked_extensions`
4. Enforces `max_file_size` limit
5. Validates directory depth against `max_depth`

**SSRF Prevention**: Blocks access to system directories (`/etc`, `/usr/bin`, etc.)

---

### NetworkSandbox

**Purpose**: Controls network access with domain allowlisting and rate limiting

**How it works**:
1. Validates URL domain against `allowed_domains`
2. Blocks local addresses (localhost, 127.0.0.1, private IPs) by default
3. Enforces rate limiting (default 100 requests/minute)
4. Prevents SSRF attacks on cloud metadata endpoints

**Domain Matching**:
- Exact: `"api.example.com"` matches only that domain
- Wildcard: `"*.example.com"` matches all subdomains
- All: `"*"` allows all domains (use with caution)

---

### IntegratedSandbox

**Purpose**: Combines file, network, and resource limits for comprehensive protection

**Resource Limits** (default):
- Memory: 100MB per tool execution
- CPU Time: 5 seconds
- Open Files: 50
- Network Connections: 10

---

## Common Scenarios

### Scenario 1: Enable curl for Web Scraping

**Problem**: `Tool.execute("process_executor", {executable = "curl", ...})` fails with "Command blocked: curl"

**Solution**:

```toml
# config.toml
[tools.system]
allow_process_execution = true
allowed_commands = "echo,cat,ls,pwd,curl"  # Add curl to allowlist
command_timeout_seconds = 30
```

**Note**: `curl` is blocked by default because it enables network access from processes. Also consider using `http-request` tool instead for better security.

---

### Scenario 2: Allow API Access to Specific Domains

**Problem**: `Tool.execute("http_request", {url = "https://api.company.com/data"})` fails

**Solution**:

```toml
# config.toml
[tools.http_request]
allowed_hosts = [
    "api.company.com",
    "*.internal.company.com",  # All internal subdomains
    "api.openai.com"
]
blocked_hosts = ["localhost", "127.0.0.1"]  # Keep SSRF protection
timeout_seconds = 30
max_request_size = 10000000
```

**Wildcard Usage**:
- `"*.company.com"` matches `api.company.com`, `dev.company.com`, etc.
- `"*"` allows all domains (use with caution)

---

### Scenario 3: Enable Python Script Execution

**Problem**: Need to run Python scripts but `python3` is blocked

**Solution**:

```toml
# config.toml
[tools.system]
allow_process_execution = true
allowed_commands = "echo,cat,ls,pwd,python3"  # Add python3
command_timeout_seconds = 60  # Longer for script execution
max_output_size = 5000000  # 5MB for script output
allowed_env_vars = "HOME,PATH,PYTHONPATH,VIRTUAL_ENV"
```

**Security Consideration**: Only enable for trusted scripts. Consider using sandbox environments.

---

### Scenario 4: Extend File Access to Project Directory

**Problem**: `Tool.execute("file_operations", {operation = "read", path = "/home/user/project/README.md"})` fails with "Path not allowed"

**Solution**:

```toml
# config.toml
[tools.file_operations]
enabled = true
allowed_paths = [
    "/tmp",                    # Keep safe defaults
    "/home/user/project"       # Add your project
]
max_file_size = 50000000
blocked_extensions = ["exe", "dll", "so"]
```

**Best Practice**: Use specific paths, not `/home/user` (too broad)

---

## Troubleshooting

### "Network access denied" Error

**Error**: `Error: Host blocked: api.example.com` or `Network access not allowed`

**Diagnosis**:
1. Check if host is in `allowed_hosts` for `tools.http_request`
2. Check if not in `blocked_hosts` (localhost, 127.0.0.1 blocked by default)
3. Verify `verify_ssl = true` if using HTTPS

**Solution**:
```toml
[tools.http_request]
allowed_hosts = ["api.example.com"]  # Add your host
```

---

### "Command not allowed" / "Executable blocked" Error

**Error**: `Error: Command blocked: curl` or `Error: Executable not allowed: python3`

**Diagnosis**:
1. Check `allow_process_execution = true` in `[tools.system]`
2. Check command is in `allowed_commands` list
3. Verify command is not in default blocked list

**Solution**:
```toml
[tools.system]
allow_process_execution = true
allowed_commands = "echo,cat,ls,pwd,curl,python3"  # Add needed commands
```

**Alternative**: Use built-in tools instead:
- Instead of `curl`: Use `http-request` tool
- Instead of `python3 script.py`: Consider agent-based execution

---

### "Path not in allowlist" Error

**Error**: `Error: Path not allowed: /home/user/data` or `Access denied: /etc/passwd`

**Diagnosis**:
1. Check `allowed_paths` in `[tools.file_operations]`
2. Verify path is absolute (not relative)
3. Check for path traversal attempts (`../`)

**Solution**:
```toml
[tools.file_operations]
allowed_paths = ["/tmp", "/home/user/data"]  # Add needed paths
```

**Security Note**: Never add `/etc`, `/usr/bin`, `/var`, or other system paths to allowlist

---

### "File extension blocked" Error

**Error**: `Error: Extension not allowed: .exe` or `Blocked extension: .sh`

**Diagnosis**:
1. Check `blocked_extensions` in `[tools.file_operations]`
2. Verify `allowed_extensions` if using allowlist mode

**Solution**:
```toml
[tools.file_operations]
blocked_extensions = ["exe", "dll"]  # Remove extension if safe
# OR for allowlist mode:
allowed_extensions = ["txt", "json", "md", "sh"]  # Add needed extensions
```

**Default blocked**: `exe`, `dll`, `so`, `dylib` (platform binaries)

---

### Checking Security Violations

**View violation logs**:
```bash
# Security violations are logged by default
grep "SecurityViolation" /var/log/llmspell/security.log

# Or check with Debug global in Lua
Debug.getCapturedEntries(100)  -- See recent debug entries
```

**Audit logging** (if enabled in config):
```toml
[security.audit]
enabled = true
log_file = "/var/log/llmspell/audit.log"
log_authorization = true
```

---

## Security Best Practices

### 1. Principle of Least Privilege

**DO**: Grant minimum permissions needed
```toml
[tools.file_operations]
allowed_paths = ["/tmp/app-data"]  # Specific directory only
```

**DON'T**: Grant broad permissions
```toml
allowed_paths = ["/home/user"]  # Too broad - entire home directory
```

---

### 2. Use Allowlists, Not Denylists

**DO**: Explicitly list what's allowed
```toml
[tools.http_request]
allowed_hosts = ["api.trusted.com", "api.safe.com"]
```

**DON'T**: Try to block everything bad (incomplete)
```toml
blocked_hosts = ["malicious.com", "bad.com"]  # Can't block all threats
```

---

### 3. Enable Only Required Commands

**DO**: Minimal command set
```toml
[tools.system]
allowed_commands = "echo,date"  # Only what you need
```

**DON'T**: Enable everything
```toml
allowed_commands = "echo,ls,cat,curl,wget,python,sh,bash"  # Too permissive
```

---

### 4. Monitor and Audit

**DO**: Enable audit logging
```toml
[security.audit]
enabled = true
log_authorization = true
log_data_access = true
```

**Review logs regularly**: Check for unexpected permission requests

---

### 5. Regular Security Reviews

- Review `allowed_paths` quarterly - remove unused paths
- Audit `allowed_commands` - minimize attack surface
- Check `allowed_domains` - remove stale entries
- Update `blocked_extensions` as threats evolve

---

## Related Documentation

- **[Configuration Guide](configuration.md)** - Full configuration reference
- **[Concepts Guide](concepts.md#security-model)** - Security architecture overview
- **[Rust API: Security Integration](../developer-guide/reference/security-integration.md)** - Developer security API reference
- **[Cookbook: Security Patterns](../../examples/script-users/cookbook/security-patterns.lua)** - Input validation examples
- **[Advanced: Monitoring Security](../../examples/script-users/advanced-patterns/monitoring-security.lua)** - Security testing patterns

---

## Quick Reference

### Check Current Permissions (Lua)

```lua
-- Check if network access is allowed
if Config.isNetworkAccessAllowed() then
    -- Network operations allowed
end

-- Check if file access is allowed
if Config.isFileAccessAllowed() then
    -- File operations allowed
end

-- Get security configuration
local security = Config.getSecurity()
print(security.allow_file_access)
print(security.allow_network_access)
```

### Common Configuration Templates

**Web Scraping**:
```toml
[tools.http_request]
allowed_hosts = ["*"]
timeout_seconds = 60

[tools.web_search]
allowed_domains = ["*"]
rate_limit_per_minute = 100
```

**Data Processing**:
```toml
[tools.file_operations]
allowed_paths = ["/data", "/output"]
max_file_size = 100000000  # 100MB
```

**Git Automation**:
```toml
[tools.system]
allow_process_execution = true
allowed_commands = "git,echo,cat"
command_timeout_seconds = 60
```
