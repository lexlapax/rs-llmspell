# Security Configuration Guide for LLMSpell

## Overview

This guide provides comprehensive instructions for configuring security features in LLMSpell. Proper configuration is essential for maintaining a secure deployment and protecting against various attack vectors.

## Configuration Files

### Main Configuration Structure

```toml
# config/llmspell.toml

[security]
enabled = true
mode = "enforce"  # Options: "enforce", "permissive", "disabled"
log_violations = true
audit_level = "verbose"  # Options: "minimal", "normal", "verbose"

[security.authentication]
require_api_key = true
api_key_min_length = 32
api_key_rotation_days = 90
session_timeout_minutes = 30
max_failed_attempts = 5
lockout_duration_minutes = 15

[security.authorization]
enable_rbac = true
default_role = "user"
permission_model = "least_privilege"
cache_permissions = true
cache_ttl_seconds = 300

[security.sandboxing]
enabled = true
filesystem_isolation = true
network_isolation = true
process_isolation = true
resource_limits = true

[security.rate_limiting]
enabled = true
global_rate = 1000  # requests per minute
per_user_rate = 100
per_tool_rate = 50
burst_size = 10
```

## Component-Specific Configuration

### Input Validation

```toml
[security.validation]
enabled = true
strict_mode = true
max_input_length = 10240
allowed_charsets = ["utf-8"]
reject_null_bytes = true
normalize_unicode = true

[security.validation.paths]
enabled = true
allowed_paths = [
    "/var/llmspell/data",
    "/tmp/llmspell",
    "/home/user/llmspell"
]
deny_symlinks = true
deny_hidden_files = true
max_path_depth = 10
resolve_paths = true

[security.validation.urls]
enabled = true
allowed_schemes = ["https", "http"]
allowed_domains = [
    "api.example.com",
    "data.example.com"
]
deny_local_addresses = true
deny_private_ranges = true
max_redirects = 5
```

### Sandboxing Configuration

```toml
[security.sandbox]
enabled = true
implementation = "native"  # Options: "native", "docker", "firecracker"

[security.sandbox.filesystem]
enabled = true
read_only_paths = [
    "/usr/lib",
    "/usr/share"
]
writable_paths = [
    "/tmp/llmspell",
    "/var/llmspell/scratch"
]
denied_paths = [
    "/etc",
    "/root",
    "/home"
]
max_file_size = 104857600  # 100MB
max_open_files = 100

[security.sandbox.network]
enabled = true
allow_localhost = false
allowed_ports = [80, 443]
allowed_protocols = ["tcp"]
dns_servers = ["1.1.1.1", "8.8.8.8"]
bandwidth_limit = 10485760  # 10MB/s

[security.sandbox.process]
enabled = true
max_processes = 10
max_threads = 100
cpu_quota = 0.5  # 50% of one core
memory_limit = 536870912  # 512MB
allow_fork = false
allow_exec = false
```

### Rate Limiting Configuration

```toml
[security.rate_limiting]
enabled = true
algorithm = "token_bucket"  # Options: "token_bucket", "sliding_window", "fixed_window"

[security.rate_limiting.global]
rate = 10000
burst = 100
window_seconds = 60

[security.rate_limiting.per_user]
rate = 1000
burst = 20
window_seconds = 60
track_by = "api_key"  # Options: "api_key", "ip", "session"

[security.rate_limiting.per_endpoint]
default_rate = 100
default_burst = 10

[[security.rate_limiting.endpoints]]
path = "/api/execute"
rate = 10
burst = 2

[[security.rate_limiting.endpoints]]
path = "/api/file/*"
rate = 50
burst = 5

[security.rate_limiting.penalties]
first_violation = "throttle"
repeated_violations = "temporary_ban"
ban_duration_minutes = 60
```

### Information Disclosure Prevention

```toml
[security.information_disclosure]
enabled = true
sanitize_errors = true
hide_versions = true
generic_errors = true

[security.information_disclosure.filters]
remove_stack_traces = true
remove_file_paths = true
remove_internal_ips = true
remove_usernames = true
redact_api_keys = true

[security.information_disclosure.patterns]
# Patterns to redact
api_key_pattern = "sk-[a-zA-Z0-9]{32,}"
jwt_pattern = "eyJ[a-zA-Z0-9_-]+\\.[a-zA-Z0-9_-]+\\.[a-zA-Z0-9_-]+"
email_pattern = "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"
```

## Tool-Specific Security Configuration

### File Operations Tool

```toml
[[tools.file_operations]]
name = "FileOperationsTool"

[tools.file_operations.security]
allowed_operations = ["read", "write", "create", "delete"]
allowed_paths = ["/var/llmspell/files"]
max_file_size = 10485760  # 10MB
allowed_extensions = [".txt", ".json", ".csv", ".log"]
scan_for_malware = true
check_mime_types = true
```

### API Tester Tool

```toml
[[tools.api_tester]]
name = "ApiTesterTool"

[tools.api_tester.security]
allowed_domains = [
    "api.example.com",
    "test.example.com"
]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
timeout_seconds = 30
follow_redirects = false
verify_ssl = true
max_response_size = 1048576  # 1MB
forbidden_headers = ["Authorization", "Cookie"]
```

### Calculator Tool

```toml
[[tools.calculator]]
name = "CalculatorTool"

[tools.calculator.security]
max_expression_length = 1000
max_number_size = 1e308
max_computation_time = 5
deny_imports = true
deny_exec = true
deny_file_access = true
allowed_functions = ["sin", "cos", "tan", "log", "sqrt"]
```

## Environment Variables

### Security-Related Environment Variables

```bash
# Core Security Settings
LLMSPELL_SECURITY_ENABLED=true
LLMSPELL_SECURITY_MODE=enforce
LLMSPELL_SECURITY_LOG_LEVEL=info

# Authentication
LLMSPELL_AUTH_REQUIRED=true
LLMSPELL_AUTH_API_KEY_HEADER=X-API-Key
LLMSPELL_AUTH_SESSION_SECRET=<generate-strong-secret>

# Sandboxing
LLMSPELL_SANDBOX_ENABLED=true
LLMSPELL_SANDBOX_IMPLEMENTATION=native
LLMSPELL_SANDBOX_MEMORY_LIMIT=512M

# Rate Limiting
LLMSPELL_RATELIMIT_ENABLED=true
LLMSPELL_RATELIMIT_REDIS_URL=redis://localhost:6379

# Logging
LLMSPELL_AUDIT_ENABLED=true
LLMSPELL_AUDIT_LOG_PATH=/var/log/llmspell/audit.log
LLMSPELL_SECURITY_LOG_PATH=/var/log/llmspell/security.log
```

## Deployment Configurations

### Development Environment

```toml
# config/development.toml
[security]
mode = "permissive"
log_violations = true
audit_level = "verbose"

[security.sandbox]
enabled = false  # Easier debugging

[security.rate_limiting]
enabled = false  # No limits in dev
```

### Staging Environment

```toml
# config/staging.toml
[security]
mode = "enforce"
log_violations = true
audit_level = "normal"

[security.sandbox]
enabled = true
implementation = "docker"

[security.rate_limiting]
enabled = true
global_rate = 5000
```

### Production Environment

```toml
# config/production.toml
[security]
mode = "enforce"
log_violations = true
audit_level = "normal"

[security.sandbox]
enabled = true
implementation = "firecracker"

[security.rate_limiting]
enabled = true
global_rate = 10000

[security.monitoring]
enabled = true
alert_on_violations = true
alert_email = "security@example.com"
```

## Security Headers Configuration

### HTTP Security Headers

```toml
[security.headers]
enabled = true

[security.headers.values]
"X-Content-Type-Options" = "nosniff"
"X-Frame-Options" = "DENY"
"X-XSS-Protection" = "1; mode=block"
"Strict-Transport-Security" = "max-age=31536000; includeSubDomains"
"Content-Security-Policy" = "default-src 'self'; script-src 'self'"
"Referrer-Policy" = "strict-origin-when-cross-origin"
"Permissions-Policy" = "geolocation=(), microphone=(), camera=()"
```

## Logging Configuration

### Security Audit Logging

```toml
[logging.security]
enabled = true
level = "info"
format = "json"
output = "file"
file_path = "/var/log/llmspell/security.log"
max_size_mb = 100
max_age_days = 90
compress = true

[logging.security.events]
authentication = true
authorization = true
validation_failures = true
sandbox_violations = true
rate_limit_exceeded = true
configuration_changes = true
```

### Audit Log Format

```json
{
  "timestamp": "2025-01-17T10:30:00Z",
  "event_type": "auth_failure",
  "severity": "warning",
  "user": "user123",
  "ip": "192.168.1.100",
  "tool": "FileOperationsTool",
  "action": "write",
  "result": "denied",
  "reason": "insufficient_permissions",
  "details": {
    "requested_path": "/etc/passwd",
    "allowed_paths": ["/var/llmspell/files"]
  }
}
```

## TLS/SSL Configuration

### TLS Settings

```toml
[security.tls]
enabled = true
min_version = "1.2"
preferred_version = "1.3"
cert_file = "/etc/llmspell/certs/server.crt"
key_file = "/etc/llmspell/certs/server.key"
ca_file = "/etc/llmspell/certs/ca.crt"

[security.tls.ciphers]
tls12 = [
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
]
tls13 = [
    "TLS_AES_256_GCM_SHA384",
    "TLS_AES_128_GCM_SHA256"
]

[security.tls.client]
verify_mode = "required"
check_hostname = true
```

## Database Security Configuration

### Database Encryption

```toml
[database.security]
encrypt_at_rest = true
encryption_key_path = "/etc/llmspell/keys/db.key"
encrypt_connections = true
connection_string = "postgresql://user:pass@localhost/llmspell?sslmode=require"

[database.security.access]
max_connections = 100
connection_timeout = 30
statement_timeout = 60
enable_prepared_statements = true
enable_query_logging = false  # Avoid logging sensitive data
```

## Monitoring and Alerting

### Security Monitoring

```toml
[monitoring.security]
enabled = true
metrics_port = 9090

[monitoring.security.alerts]
enabled = true
backend = "prometheus"

[[monitoring.security.alerts.rules]]
name = "high_auth_failure_rate"
condition = "rate(auth_failures[5m]) > 10"
severity = "critical"
notify = ["security@example.com", "oncall@example.com"]

[[monitoring.security.alerts.rules]]
name = "sandbox_escape_attempt"
condition = "sandbox_violations > 0"
severity = "critical"
notify = ["security@example.com"]

[[monitoring.security.alerts.rules]]
name = "unusual_data_access"
condition = "data_access_volume > 1000000"
severity = "warning"
notify = ["security@example.com"]
```

## Best Practices

### Configuration Management

1. **Version Control**: Store configs in Git (exclude secrets)
2. **Environment Separation**: Different configs per environment
3. **Secret Management**: Use vault or environment variables
4. **Validation**: Validate configs before deployment
5. **Documentation**: Document all custom settings

### Security Hardening Checklist

- [ ] Enable all security features
- [ ] Configure strict validation rules
- [ ] Set up proper sandboxing
- [ ] Enable rate limiting
- [ ] Configure audit logging
- [ ] Set up monitoring alerts
- [ ] Use strong encryption
- [ ] Implement least privilege
- [ ] Regular security reviews
- [ ] Keep configs updated

### Common Misconfigurations

1. **Too Permissive Paths**
   ```toml
   # Bad
   allowed_paths = ["/"]
   
   # Good
   allowed_paths = ["/var/llmspell/data"]
   ```

2. **Weak Rate Limits**
   ```toml
   # Bad
   global_rate = 1000000
   
   # Good
   global_rate = 10000
   ```

3. **Disabled Security Features**
   ```toml
   # Bad
   [security]
   enabled = false
   
   # Good
   [security]
   enabled = true
   mode = "enforce"
   ```

## Migration Guide

### Upgrading Security Configuration

1. **Backup Current Config**
   ```bash
   cp config/llmspell.toml config/llmspell.toml.backup
   ```

2. **Review Changes**
   ```bash
   diff config/llmspell.toml.backup config/llmspell.toml.new
   ```

3. **Test in Staging**
   ```bash
   llmspell --config config/staging.toml validate
   ```

4. **Deploy Gradually**
   - Deploy to canary
   - Monitor metrics
   - Roll out to production

## Troubleshooting

### Common Issues

1. **Permission Denied**
   - Check allowed_paths configuration
   - Verify file permissions
   - Review audit logs

2. **Rate Limit Exceeded**
   - Check rate limit settings
   - Review per-user limits
   - Consider increasing limits

3. **Sandbox Violations**
   - Review sandbox configuration
   - Check resource limits
   - Verify allowed operations

### Debug Mode

```toml
[debug]
enabled = false  # Enable only for troubleshooting
log_all_requests = true
log_all_responses = true
bypass_rate_limits = true
verbose_errors = true
```

## Compliance Configurations

### GDPR Compliance

```toml
[compliance.gdpr]
enabled = true
data_retention_days = 365
right_to_deletion = true
data_portability = true
consent_required = true
anonymize_logs = true
```

### SOC 2 Compliance

```toml
[compliance.soc2]
enabled = true
audit_logging = true
access_controls = true
encryption_required = true
change_management = true
incident_response = true
```

## Conclusion

Proper security configuration is crucial for protecting your LLMSpell deployment. Regularly review and update these settings, test changes in non-production environments, and monitor security metrics to ensure your configuration remains effective against evolving threats.