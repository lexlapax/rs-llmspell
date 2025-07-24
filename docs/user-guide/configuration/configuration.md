# ABOUTME: Comprehensive configuration guide for rs-llmspell operators
# ABOUTME: Security settings, resource limits, deployment guidance, and operations

# Configuration Guide

⚠️ **EVOLVING CONFIGURATION**: This guide includes both current Phase 3.3 configuration options and planned features for production deployment. Some advanced configuration options may require future implementation.

This comprehensive guide covers all configuration aspects of rs-llmspell including security settings, resource limits, deployment options, and operational procedures.

## Table of Contents

1. [Configuration Overview](#configuration-overview)
2. [Security Configuration](#security-configuration)
3. [Resource Limits](#resource-limits)
4. [Tool-Specific Settings](#tool-specific-settings)
5. [Deployment Configurations](#deployment-configurations)
6. [Monitoring and Logging](#monitoring-and-logging)
7. [Incident Response](#incident-response)
8. [Best Practices](#best-practices)

## Configuration Overview

### Configuration Files

```toml
# config/llmspell.toml - Main configuration file

[security]
enabled = true
mode = "enforce"  # Options: "enforce", "permissive", "disabled"
log_violations = true
audit_level = "verbose"  # Options: "minimal", "normal", "verbose"

[resources]
default_limits = "default"  # Options: "strict", "default", "relaxed", "unlimited"
enforce_limits = true
track_usage = true
```

### Environment Variables

```bash
# Core Security Settings
LLMSPELL_SECURITY_ENABLED=true
LLMSPELL_SECURITY_MODE=enforce
LLMSPELL_SECURITY_LOG_LEVEL=info

# Authentication
LLMSPELL_AUTH_REQUIRED=true
LLMSPELL_AUTH_API_KEY_HEADER=X-API-Key
LLMSPELL_AUTH_SESSION_SECRET=<generate-strong-secret>

# Resource Limits
LLMSPELL_RESOURCE_LIMITS_ENABLED=true
LLMSPELL_RESOURCE_LIMITS_DEFAULT=default
LLMSPELL_RESOURCE_TRACKING=true
```

## Security Configuration

### Authentication Settings

```toml
[security.authentication]
require_api_key = true
api_key_min_length = 32
api_key_rotation_days = 90
session_timeout_minutes = 30
max_failed_attempts = 5
lockout_duration_minutes = 15

# API Key validation
[security.authentication.api_keys]
hash_algorithm = "argon2"
rate_limit_per_key = 1000  # requests per hour
allow_multiple_keys_per_user = false
```

### Authorization Settings

```toml
[security.authorization]
enable_rbac = true
default_role = "user"
permission_model = "least_privilege"
cache_permissions = true
cache_ttl_seconds = 300

# Role definitions
[[security.authorization.roles]]
name = "admin"
permissions = ["*"]

[[security.authorization.roles]]
name = "user"
permissions = ["tools:execute", "tools:list", "files:read"]

[[security.authorization.roles]]
name = "guest"
permissions = ["tools:list"]
```

### Sandboxing Configuration

```toml
[security.sandboxing]
enabled = true
implementation = "native"  # Options: "native", "docker", "firecracker"

[security.sandboxing.filesystem]
enabled = true
allowed_paths = [
    "/var/llmspell/data",
    "/tmp/llmspell",
    "/workspace"
]
denied_patterns = ["*.exe", "*.sh", "*.bat"]
max_file_size = "10MB"
max_open_files = 100

[security.sandboxing.network]
enabled = true
allowed_domains = [
    "api.example.com",
    "*.trusted-domain.com"
]
allowed_ports = [80, 443]
deny_local_addresses = true
bandwidth_limit = "10MB/s"

[security.sandboxing.process]
enabled = true
max_processes = 10
max_threads = 100
cpu_quota = 0.5  # 50% of one core
memory_limit = "512MB"
allow_fork = false
```

### Rate Limiting

```toml
[security.rate_limiting]
enabled = true
algorithm = "token_bucket"

[security.rate_limiting.global]
rate = 10000  # per minute
burst = 100
window_seconds = 60

[security.rate_limiting.per_user]
rate = 1000
burst = 20
track_by = "api_key"

[[security.rate_limiting.endpoints]]
path = "/api/execute"
rate = 10
burst = 2

[[security.rate_limiting.endpoints]]
path = "/api/tools/*"
rate = 100
burst = 10
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
redact_api_keys = true
```

## Resource Limits

### Overview

The Resource Limit Framework provides comprehensive resource management for all tools:
- **Memory Limits**: Track and limit memory allocation
- **CPU Time Limits**: Monitor and restrict CPU usage
- **File Size Limits**: Control maximum file sizes for I/O operations
- **Operation Count Limits**: Limit the number of operations performed
- **Concurrent Operation Limits**: Control parallel execution
- **Operation Timeouts**: Prevent long-running operations

### Predefined Configurations

#### Default Limits
```toml
[resources.profiles.default]
max_memory_bytes = 104857600  # 100MB
max_cpu_time_ms = 30000       # 30 seconds
max_file_size = 52428800      # 50MB
max_operations = 1000000      # 1M operations
max_concurrent_operations = 100
operation_timeout_seconds = 60
```

#### Strict Limits (Untrusted Operations)
```toml
[resources.profiles.strict]
max_memory_bytes = 10485760   # 10MB
max_cpu_time_ms = 5000        # 5 seconds
max_file_size = 5242880       # 5MB
max_operations = 10000        # 10K operations
max_concurrent_operations = 10
operation_timeout_seconds = 10
```

#### Relaxed Limits (Trusted Operations)
```toml
[resources.profiles.relaxed]
max_memory_bytes = 1073741824  # 1GB
max_cpu_time_ms = 300000       # 5 minutes
max_file_size = 524288000      # 500MB
max_operations = 100000000     # 100M operations
max_concurrent_operations = 1000
operation_timeout_seconds = 300
```

### Per-Tool Resource Configuration

```toml
[[resources.tool_limits]]
tool_name = "FileOperationsTool"
profile = "default"
# Override specific limits
max_file_size = 104857600  # 100MB for file operations

[[resources.tool_limits]]
tool_name = "CalculatorTool"
profile = "strict"
max_cpu_time_ms = 1000  # 1 second max for calculations

[[resources.tool_limits]]
tool_name = "DatabaseConnectorTool"
profile = "relaxed"
max_memory_bytes = 536870912  # 512MB for database operations
```

### Resource Monitoring

```toml
[resources.monitoring]
enabled = true
warning_threshold = 0.8  # Warn at 80% usage
check_interval_seconds = 5
alert_on_limit_exceeded = true
collect_statistics = true
```

## Tool-Specific Settings

### File Operations Tool

```toml
[[tools.file_operations]]
name = "FileOperationsTool"

[tools.file_operations.security]
allowed_operations = ["read", "write", "create", "delete"]
allowed_paths = ["/var/llmspell/files"]
max_file_size = "10MB"
allowed_extensions = [".txt", ".json", ".csv", ".log"]
scan_for_malware = true
check_mime_types = true
```

### API Tester Tool

```toml
[[tools.api_tester]]
name = "ApiTesterTool"

[tools.api_tester.security]
allowed_domains = ["api.example.com", "test.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
timeout_seconds = 30
follow_redirects = false
verify_ssl = true
max_response_size = "1MB"
forbidden_headers = ["Authorization", "Cookie"]
```

### Process Executor Tool

```toml
[[tools.process_executor]]
name = "ProcessExecutorTool"

[tools.process_executor.security]
allowed_commands = ["ls", "echo", "date"]
allowed_paths = ["/usr/bin", "/bin"]
max_execution_time = 30
max_output_size = "1MB"
environment_whitelist = ["PATH", "HOME"]
deny_shell_operators = true
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

[resources]
default_limits = "relaxed"
enforce_limits = false  # Just track, don't enforce
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

[resources]
default_limits = "default"
enforce_limits = true
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

[resources]
default_limits = "default"
enforce_limits = true
track_usage = true
alert_on_limit_exceeded = true
```

### Container Deployment

```bash
# Secure Docker deployment
docker run --security-opt=no-new-privileges \
           --cap-drop=ALL \
           --cap-add=NET_BIND_SERVICE \
           --read-only \
           --tmpfs /tmp:noexec,nosuid,size=100M \
           --user 1000:1000 \
           -v /path/to/config:/config:ro \
           -v /path/to/data:/data:rw \
           rs-llmspell

# Kubernetes SecurityContext
apiVersion: v1
kind: Pod
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 1000
  containers:
  - name: llmspell
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
    resources:
      limits:
        memory: "512Mi"
        cpu: "1000m"
      requests:
        memory: "256Mi"
        cpu: "500m"
```

## Monitoring and Logging

### Security Event Logging

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
resource_limit_exceeded = true
```

### Audit Log Format

```json
{
  "timestamp": "2025-01-17T10:30:00Z",
  "event_type": "resource_limit_exceeded",
  "severity": "warning",
  "user": "user123",
  "ip": "192.168.1.100",
  "tool": "FileOperationsTool",
  "resource": "memory",
  "limit": 104857600,
  "requested": 209715200,
  "action": "denied"
}
```

### Monitoring Alerts

```toml
[monitoring.security.alerts]
enabled = true
backend = "prometheus"

[[monitoring.security.alerts.rules]]
name = "high_auth_failure_rate"
condition = "rate(auth_failures[5m]) > 10"
severity = "critical"
notify = ["security@example.com", "oncall@example.com"]

[[monitoring.security.alerts.rules]]
name = "resource_exhaustion"
condition = "resource_usage_percent > 90"
severity = "warning"
notify = ["ops@example.com"]

[[monitoring.security.alerts.rules]]
name = "sandbox_escape_attempt"
condition = "sandbox_violations > 0"
severity = "critical"
notify = ["security@example.com"]
```

## Incident Response

### Quick Response Guide

#### 1. Detection Phase (0-15 minutes)
```bash
# Check for active threats
tail -f /var/log/llmspell/security.log | grep -E "(CRITICAL|violation)"

# Review resource exhaustion
grep "resource_limit_exceeded" /var/log/llmspell/security.log | tail -20

# Check system resources
top -b -n 1 | head -20
netstat -tulpn | grep llmspell
```

#### 2. Containment Phase (15-60 minutes)
```bash
# Block suspicious IP
iptables -A INPUT -s <SUSPICIOUS_IP> -j DROP

# Disable compromised API key
echo "<API_KEY>" >> /etc/llmspell/blocked_keys.txt
systemctl reload llmspell

# Enable emergency mode
sed -i 's/mode = "enforce"/mode = "lockdown"/' /etc/llmspell/config.toml

# Reduce resource limits temporarily
sed -i 's/default_limits = "default"/default_limits = "strict"/' /etc/llmspell/config.toml
```

#### 3. Investigation Phase
```bash
# Collect evidence
mkdir /tmp/incident-$(date +%Y%m%d)
cp -r /var/log/llmspell/* /tmp/incident-*/
tar -czf incident-logs.tar.gz /tmp/incident-*

# Analyze patterns
grep -E "(failed|denied|violation|exceeded)" /var/log/llmspell/security.log | \
  awk '{print $5}' | sort | uniq -c | sort -nr
```

### Incident Types and Responses

#### Resource Exhaustion Attack
```yaml
Detection: Multiple resource limit violations
Response:
  - Enable strict resource limits
  - Identify attack source
  - Scale resources if legitimate
  - Block malicious actors
```

#### Unauthorized Access Attempt
```yaml
Detection: Multiple auth failures from same IP
Response:
  - Block IP address
  - Review API key usage
  - Check for data access
  - Rotate affected keys
```

## Best Practices

### Configuration Management

1. **Version Control**
   ```bash
   # Store configs in Git (exclude secrets)
   git add config/*.toml
   git commit -m "Update security configuration"
   ```

2. **Secret Management**
   ```bash
   # Use environment variables for secrets
   export LLMSPELL_DB_PASSWORD=$(vault read -field=password secret/llmspell/db)
   export LLMSPELL_API_KEY=$(vault read -field=key secret/llmspell/api)
   ```

3. **Configuration Validation**
   ```bash
   # Validate before deployment
   llmspell config validate --file config/production.toml
   llmspell config diff --from config/current.toml --to config/new.toml
   ```

### Security Hardening Checklist

- [ ] Enable all security features
- [ ] Configure authentication requirements
- [ ] Set up authorization roles
- [ ] Enable sandboxing for all tools
- [ ] Configure rate limiting
- [ ] Set appropriate resource limits
- [ ] Enable audit logging
- [ ] Set up monitoring alerts
- [ ] Configure TLS/SSL
- [ ] Review firewall rules
- [ ] Schedule security audits

### Common Misconfigurations to Avoid

1. **Too Permissive Paths**
   ```toml
   # ❌ Bad
   allowed_paths = ["/"]
   
   # ✅ Good
   allowed_paths = ["/var/llmspell/data"]
   ```

2. **Weak Rate Limits**
   ```toml
   # ❌ Bad
   global_rate = 1000000
   
   # ✅ Good
   global_rate = 10000
   ```

3. **Disabled Security Features**
   ```toml
   # ❌ Bad
   [security]
   enabled = false
   
   # ✅ Good
   [security]
   enabled = true
   mode = "enforce"
   ```

4. **Excessive Resource Limits**
   ```toml
   # ❌ Bad
   default_limits = "unlimited"
   
   # ✅ Good
   default_limits = "default"
   ```

### Troubleshooting

#### Permission Denied Errors
```bash
# Check allowed paths
grep "allowed_paths" /etc/llmspell/config.toml

# Review recent denials
grep "permission_denied" /var/log/llmspell/security.log | tail -10

# Test with permissive mode (temporary)
llmspell --security-mode=permissive test-command
```

#### Resource Limit Issues
```bash
# Check current limits
llmspell config show resources

# Monitor resource usage
watch -n 1 'grep "resource_limit_exceeded" /var/log/llmspell/security.log | wc -l'

# Temporarily increase limits (careful!)
llmspell config set resources.profiles.default.max_memory_bytes 209715200
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

## Regular Maintenance

### Daily Tasks
- Review security alerts
- Check authentication failures
- Monitor resource usage
- Verify backup completion

### Weekly Tasks
- Analyze security trends
- Review rate limit effectiveness
- Update threat intelligence
- Test incident procedures

### Monthly Tasks
- Rotate API keys
- Security configuration review
- Vulnerability scanning
- Update security patches

## Conclusion

Proper configuration is crucial for securing and optimizing your rs-llmspell deployment. This guide combines security settings and resource management into a unified configuration approach. Regular monitoring, timely updates, and adherence to these guidelines will help maintain a strong security posture while ensuring optimal performance.

Remember to test all changes in non-production environments first and maintain comprehensive audit logs for compliance and forensics.