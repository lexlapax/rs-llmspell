# ABOUTME: Comprehensive security architecture for rs-llmspell
# ABOUTME: Covers architecture layers, threat model, and security controls

# rs-llmspell Security Architecture

**Version**: Phase 3.2 Security Hardening  
**Status**: ✅ **CURRENT** - Comprehensive defense-in-depth implementation  
**Last Updated**: July 2025

> **🛡️ SECURITY MODEL**: This document outlines the complete security architecture including STRIDE threat analysis, defense layers, and Phase 3.2 security hardening results.

## Overview

rs-llmspell implements a comprehensive defense-in-depth security architecture to protect against various threats when executing LLM-driven tools and scripts. This document outlines the security layers, threat model, and mechanisms that ensure safe operation.

## Core Security Principles

1. **Least Privilege**: Tools operate with minimal required permissions
2. **Defense in Depth**: Multiple security layers protect against threats
3. **Zero Trust**: All inputs are validated and sanitized
4. **Fail Secure**: System fails to a safe state on errors
5. **Audit Trail**: All security-relevant events are logged

## Security Architecture Layers

### 1. Input Validation Layer

**Location**: `llmspell-utils/src/security/validation.rs`

- **Purpose**: First line of defense against malicious inputs
- **Components**:
  - Path traversal prevention
  - Input sanitization
  - Parameter validation
  - Type checking
- **Key Features**:
  - Configurable validation rules
  - Pattern-based detection
  - Encoding normalization

### 2. Authentication & Authorization Layer

**Location**: `llmspell-utils/src/security/auth.rs`

- **Purpose**: Control access to tools and resources
- **Components**:
  - Permission checking
  - Role-based access control
  - API key validation
  - Session management
- **Key Features**:
  - Fine-grained permissions
  - Dynamic authorization
  - Audit logging

### 3. Sandboxing Layer

**Location**: `llmspell-utils/src/security/sandbox.rs`

- **Purpose**: Isolate tool execution from system resources
- **Components**:
  - Filesystem sandboxing
  - Network restrictions
  - Process isolation
  - Resource limits
- **Key Features**:
  - Configurable sandbox policies
  - Resource quotas
  - Capability-based security

### 4. Rate Limiting & DoS Protection

**Location**: `llmspell-utils/src/security/rate_limiting.rs`

- **Purpose**: Prevent resource exhaustion attacks
- **Components**:
  - Request rate limiting
  - Resource usage monitoring
  - Circuit breakers
  - Backpressure handling
- **Key Features**:
  - Adaptive rate limiting
  - Per-user/per-tool limits
  - Graceful degradation

### 5. Information Disclosure Prevention

**Location**: `llmspell-utils/src/security/information_disclosure.rs`

- **Purpose**: Prevent sensitive data leakage
- **Components**:
  - Output filtering
  - Error message sanitization
  - Path obfuscation
  - Secret redaction
- **Key Features**:
  - Pattern-based filtering
  - Context-aware redaction
  - Configurable sensitivity levels

## Threat Model (STRIDE Analysis)

### System Components and Trust Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    External Network                      │
├─────────────────────────────────────────────────────────┤
│                    System Boundary                       │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │     CLI     │  │ Script Engine │  │  Tool Library │ │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘ │
│         │                 │                   │         │
│  ┌──────▼─────────────────▼──────────────────▼───────┐ │
│  │              Bridge Layer (Trust Boundary)         │ │
│  └──────┬─────────────────┬──────────────────┬───────┘ │
│         │                 │                   │         │
│  ┌──────▼──────┐  ┌──────▼───────┐  ┌───────▼───────┐ │
│  │   Storage   │  │   Providers   │  │    Agents     │ │
│  └─────────────┘  └──────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### STRIDE Threats by Category

#### 1. Spoofing Identity
**Threat**: Impersonation of legitimate users or services
- **Attack Vectors**: Stolen API keys, session hijacking, man-in-the-middle
- **Risk Level**: HIGH
- **Mitigations**: Strong authentication, API key rotation, TLS everywhere

#### 2. Tampering with Data
**Threat**: Unauthorized modification of data or code
- **Attack Vectors**: SQL injection, file manipulation, memory corruption
- **Risk Level**: HIGH
- **Mitigations**: Input validation, integrity checks, immutable storage

#### 3. Repudiation
**Threat**: Users denying actions they performed
- **Risk Level**: MEDIUM
- **Mitigations**: Comprehensive audit logging, tamper-proof logs, cryptographic signatures

#### 4. Information Disclosure
**Threat**: Sensitive information exposure
- **Attack Vectors**: Error message leaks, path traversal, memory dumps, side-channel attacks
- **Risk Level**: HIGH
- **Mitigations**: Generic error messages, path sanitization, output filtering

#### 5. Denial of Service
**Threat**: System resource exhaustion
- **Attack Vectors**: Infinite loops, memory bombs, fork bombs, network flooding
- **Risk Level**: HIGH
- **Mitigations**: Execution timeouts, memory limits, rate limiting, resource quotas

#### 6. Elevation of Privilege
**Threat**: Gaining unauthorized permissions
- **Attack Vectors**: Sandbox escape, permission bypass, role manipulation
- **Risk Level**: CRITICAL
- **Mitigations**: Strict sandboxing, capability-based security, least privilege

### LLM-Specific Threats

#### 1. Prompt Injection
**Threat**: Malicious prompts cause unintended tool execution
- **Risk Level**: CRITICAL
- **Mitigations**: Prompt validation, execution context isolation, tool whitelisting

#### 2. Model Manipulation
**Threat**: Adversarial inputs cause model misbehavior
- **Risk Level**: MEDIUM
- **Mitigations**: Input preprocessing, anomaly detection, rate limiting

#### 3. Context Confusion
**Threat**: LLM confuses contexts leading to data leakage
- **Risk Level**: HIGH
- **Mitigations**: Context isolation, memory clearing, session management

## Tool Security Model

### Tool Categories by Risk Level

1. **Low Risk Tools**
   - Read-only operations
   - No external network access
   - Examples: JsonProcessorTool, DateTimeHandler

2. **Medium Risk Tools**
   - Limited file system access
   - Controlled network access
   - Examples: FileOperationsTool, WebSearchTool

3. **High Risk Tools**
   - Execute external commands
   - Modify system state
   - Examples: ProcessExecutorTool, ApiTesterTool

### Security Controls per Tool

Each tool implements:

```rust
pub struct ToolSecurityConfig {
    pub requires_auth: bool,
    pub allowed_paths: Vec<PathBuf>,
    pub network_access: NetworkPolicy,
    pub max_execution_time: Duration,
    pub max_memory: usize,
    pub allowed_operations: Vec<Operation>,
}
```

## Phase 3.2 Security Enhancements

During Phase 3.2, comprehensive security hardening was implemented:

### Critical Vulnerabilities Fixed
1. **Calculator DoS**: Fixed regex timeout vulnerability (Phase 3.2)
2. **Path Traversal** (multiple tools): Standardized path validation
3. **Information Disclosure**: Sanitized error messages across all tools
4. **Resource Exhaustion**: Implemented timeouts and limits

### Security Infrastructure Added
- Centralized validation utilities in `llmspell-utils`
- Comprehensive security test framework (200+ tests)
- Automated vulnerability scanning in CI/CD
- Security configuration system

## Security Configuration

### Global Security Settings

```toml
[security]
enable_sandboxing = true
enable_rate_limiting = true
enable_audit_logging = true
max_execution_time = 30s
max_memory_per_tool = 100MB
allowed_network_domains = ["api.example.com"]
```

### Per-Tool Security Settings

```toml
[[tools.security]]
name = "FileOperationsTool"
allowed_paths = ["/tmp/llmspell", "/var/llmspell/data"]
max_file_size = 10MB
allowed_operations = ["read", "write", "create"]
```

## Security Testing

### Test Categories

1. **Unit Tests**: Test individual security components
2. **Integration Tests**: Test security across components
3. **Security Tests**: Dedicated security test suite
   - Path traversal tests (50+ cases)
   - Injection tests (50+ cases)
   - DoS tests (30+ cases)
   - Information disclosure tests (30+ cases)

### Security Test Framework

Location: `llmspell-tools/tests/security/`

- `test_framework.rs`: Core security testing utilities
- `path_security_tests.rs`: Path traversal prevention
- `input_validation_tests.rs`: Injection prevention
- `auth_tests.rs`: Authentication/authorization
- `rate_limit_tests.rs`: DoS protection
- `data_exposure_tests.rs`: Information disclosure

## Monitoring and Auditing

### Security Events

All security-relevant events are logged:
- Authentication attempts
- Authorization failures
- Rate limit violations
- Sandbox escapes
- Validation failures

### Audit Log Format

```json
{
  "timestamp": "2025-01-17T10:30:00Z",
  "event_type": "auth_failure",
  "tool": "ApiTesterTool",
  "user": "user123",
  "details": {
    "reason": "invalid_api_key",
    "ip": "192.168.1.100"
  }
}
```

## Tool-Specific Threat Models

### File System Tools
**Threats**: Path traversal, symlink attacks, file disclosure
**Mitigations**: 
- Canonical path resolution
- Symlink detection
- Sandbox restrictions
- Size limits

### Network Tools
**Threats**: SSRF, data exfiltration, DNS rebinding
**Mitigations**:
- URL validation
- Domain whitelisting
- Request timeouts
- Response filtering

### Code Execution Tools
**Threats**: Command injection, sandbox escape, resource exhaustion
**Mitigations**:
- Argument sanitization
- Process isolation
- Resource limits
- Execution timeouts

### Data Processing Tools
**Threats**: Injection attacks, parser DoS, memory exhaustion
**Mitigations**:
- Input validation
- Size limits
- Timeout controls
- Safe parsing libraries

## Risk Assessment Matrix

```
         Impact →
    │ Low │ Med │ High │ Crit │
────┼─────┼─────┼──────┼──────┤
L   │     │     │      │      │
o L │  1  │  2  │  3   │  4   │
w o ├─────┼─────┼──────┼──────┤
↓ w │  2  │  3  │  4   │  5   │
    ├─────┼─────┼──────┼──────┤
L M │  3  │  4  │  5   │  6   │
i e ├─────┼─────┼──────┼──────┤
k d │  4  │  5  │  6   │  7   │
e   ├─────┼─────┼──────┼──────┤
l H │  5  │  6  │  7   │  8   │
i i ├─────┼─────┼──────┼──────┤
h g │  6  │  7  │  8   │  9   │
o h └─────┴─────┴──────┴──────┘
o
d
```

### Risk Ratings
- **Critical (7-9)**: Immediate action required
- **High (5-6)**: Address within sprint
- **Medium (3-4)**: Address within quarter
- **Low (1-2)**: Monitor and address as able

## Compliance Considerations

### Standards Alignment
- OWASP Top 10
- CWE/SANS Top 25
- NIST Cybersecurity Framework
- ISO 27001/27002

### Regulatory Requirements
- GDPR (data protection)
- CCPA (privacy)
- SOC 2 (security controls)
- HIPAA (if healthcare data)

## Future Enhancements

1. **Hardware Security Module (HSM)** integration
2. **Advanced threat detection** using ML
3. **Zero-knowledge proof** for sensitive operations
4. **Blockchain-based** audit trail
5. **Quantum-resistant** cryptography

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CWE/SANS Top 25](https://cwe.mitre.org/top25/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)