# Security Model (v0.6.0)

**Status**: Production Implementation  
**Last Updated**: August 2025  
**Validation**: Cross-referenced with llmspell-security crate and Phase 3.2 hardening

> **🛡️ Single Source of Truth**: This document consolidates all security documentation to reflect the ACTUAL security implementation in LLMSpell v0.6.0.

---

## Table of Contents

1. [Security Architecture](#security-architecture)
2. [Three-Level Security Model](#three-level-security-model)
3. [Defense Layers](#defense-layers)
4. [Threat Model](#threat-model)
5. [Implementation Details](#implementation-details)
6. [Security Guidelines](#security-guidelines)

---

## Security Architecture

### Core Security Principles

1. **Least Privilege**: Tools operate with minimal required permissions
2. **Defense in Depth**: Multiple security layers protect against threats
3. **Zero Trust**: All inputs are validated and sanitized
4. **Fail Secure**: System fails to a safe state on errors
5. **Audit Trail**: Security-relevant events are logged via hook system

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    External Network                      │
├─────────────────────────────────────────────────────────┤
│                 SECURITY BOUNDARY                        │
│                                                          │
│  Input Layer:                                           │
│  ├── Path Validation (no traversal)                     │
│  ├── Input Sanitization (encoding normalization)        │
│  └── Parameter Type Checking                            │
│                                                          │
│  Sandboxing Layer:                                      │
│  ├── Lua Stdlib Restrictions (no os.execute, io.popen)  │
│  ├── File System Sandbox (whitelist-based)              │
│  └── Network Domain Restrictions                        │
│                                                          │
│  Resource Control Layer:                                │
│  ├── Memory Limits (ResourceTracker)                    │
│  ├── CPU Time Limits                                    │
│  ├── File Size Limits                                   │
│  └── Operation Count Limits                             │
│                                                          │
│  Execution Layer:                                       │
│  ├── Bridge Security (Arc<FileSandbox> injection)       │
│  ├── Tool Security Requirements                         │
│  └── Circuit Breakers (hook system)                     │
└─────────────────────────────────────────────────────────┘
```

---

## Three-Level Security Model

**Implementation**: `llmspell-security/src/levels.rs`

```rust
pub enum SecurityLevel {
    Safe,       // No file/network access
    Restricted, // Limited, validated access  
    Privileged, // Full system access
}
```

### Level Details

#### Safe Level
- **Use Case**: Pure computation tools (calculator, uuid-generator)
- **Restrictions**: 
  - No file system access
  - No network access
  - No environment variable access
  - Memory-only operations
- **Example Tools**: calculator, hash-calculator, base64-encoder

#### Restricted Level (Default)
- **Use Case**: Most tools requiring controlled I/O
- **Restrictions**:
  - File access via validated paths only
  - Network access to whitelisted domains
  - Limited environment variable access
  - Resource limits enforced
- **Example Tools**: file-operations, web-scraper, json-processor

#### Privileged Level
- **Use Case**: System administration tools
- **Restrictions**:
  - Extended file system access
  - Broader network access
  - Process execution capability
  - Requires security review
- **Example Tools**: process-executor, system-monitor

---

## Defense Layers

### 1. Input Validation Layer

**Location**: `llmspell-utils/src/security/validation.rs`  
**Status**: ✅ Implemented

**Features**:
- Path traversal prevention (../ detection)
- Input sanitization (trim, encoding normalization)
- Parameter type validation
- Pattern-based malicious input detection

**Code Example**:
```rust
// Actual implementation pattern
let validator = PathValidator::new(allowed_paths);
let safe_path = validator.validate_path(user_input)?;
```

### 2. Sandboxing Layer

**Location**: `llmspell-security/src/sandbox.rs`  
**Status**: ✅ Implemented

**Lua Sandbox** (Phase 2):
- Removed dangerous functions: `os.execute`, `io.popen`, `loadfile`, `dofile`
- Restricted file I/O through Tool interface only
- No raw network access

**File System Sandbox** (Phase 3.2):
```rust
pub struct FileSandbox {
    allowed_paths: Vec<PathBuf>,
    max_file_size: usize,
    allow_symlinks: bool,
}
```

**Network Sandbox**:
- Domain whitelist enforcement
- No localhost/private IP access by default
- Protocol restrictions (HTTPS preferred)

### 3. Resource Control Layer

**Location**: `llmspell-utils/src/resource_tracker.rs`  
**Status**: ✅ Implemented

**ResourceTracker Features**:
```rust
pub struct ResourceLimits {
    pub memory_limit: Option<usize>,      // Bytes
    pub cpu_limit: Option<Duration>,      // Time limit
    pub file_size_limit: Option<usize>,   // Bytes
    pub operation_limit: Option<usize>,   // Count
}
```

**Enforcement**:
- Memory tracking for allocations >1MB
- CPU time limits via tokio timeouts
- File size validation before operations
- Operation counting with circuit breakers

### 4. Authentication & Authorization

**Location**: `llmspell-security/src/auth.rs`  
**Status**: 🚧 Basic Implementation

**Current Implementation**:
- Tool-level security requirements
- Bridge-enforced sandbox injection
- API key validation for providers

**Future Enhancements** (Phase 8+):
- Role-based access control
- User authentication
- Fine-grained permissions

### 5. Information Disclosure Prevention

**Location**: `llmspell-utils/src/security/information_disclosure.rs`  
**Status**: ✅ Implemented

**Features**:
- Generic error messages (no stack traces to users)
- Path obfuscation in logs
- API key redaction in state persistence
- Sensitive data patterns detection

---

## Threat Model

### STRIDE Analysis

#### Spoofing Identity
**Threats**: API key theft, session hijacking  
**Mitigations**:
- API keys stored in environment variables
- No API keys in scripts or logs
- Session isolation (Phase 6)

#### Tampering with Data
**Threats**: Path traversal, injection attacks  
**Mitigations**:
- Input validation on all parameters
- Immutable tool configurations
- State integrity checks

#### Repudiation
**Threats**: Denying malicious actions  
**Mitigations**:
- Hook-based audit logging
- Event correlation IDs
- Timestamp tracking

#### Information Disclosure
**Threats**: Error message leaks, path exposure  
**Mitigations**:
- Generic error messages
- Path sanitization
- Output filtering

#### Denial of Service
**Threats**: Resource exhaustion, infinite loops  
**Mitigations**:
- Resource limits (memory, CPU, file size)
- Operation count limits
- Circuit breakers (5 failures → open)
- Timeout enforcement

#### Elevation of Privilege
**Threats**: Sandbox escape, privilege escalation  
**Mitigations**:
- Three-level security model
- Bridge-enforced sandboxing
- No dynamic code execution

---

## Implementation Details

### Tool Security Requirements

**Every tool must declare** (Phase 3):

```rust
impl Tool for YourTool {
    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted  // Most tools use this
    }
    
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            file_access: vec!["/tmp/workspace"],
            network_domains: vec!["api.example.com"],
            env_vars: vec!["HOME"],
        }
    }
    
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits {
            memory_limit: Some(100 * 1024 * 1024), // 100MB
            cpu_limit: Some(Duration::from_secs(5)),
            file_size_limit: Some(10 * 1024 * 1024), // 10MB
            operation_limit: Some(1000),
        }
    }
}
```

### Bridge Security Enforcement

**Phase 3.2 Architecture**:

```rust
// Bridge creates and injects sandbox
let sandbox = Arc::new(FileSandbox::new(allowed_paths));
let tool = FileOperationsTool::new(config, sandbox.clone());

// Tool uses bridge-provided sandbox
impl FileOperationsTool {
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        self.sandbox.validate_path(path) // Uses injected sandbox
    }
}
```

### Security Hooks

**Phase 4 Integration**:

```rust
pub enum SecurityHooks {
    SecurityViolation,      // Triggered on violations
    ResourceLimitExceeded,  // Resource limits hit
    SandboxEscape,         // Escape attempt detected
    AuthenticationFailed,   // Auth failure
}
```

---

## Security Guidelines

### For Tool Developers

1. **Always declare security level** - Default to Restricted
2. **Use bridge-provided sandbox** - Never create your own
3. **Validate all inputs** - Use provided validators
4. **Handle errors securely** - No stack traces to users
5. **Document security requirements** - In tool documentation

### For Script Authors

1. **Never hardcode API keys** - Use environment variables
2. **Validate tool outputs** - Don't trust blindly
3. **Use appropriate security levels** - Don't request Privileged unnecessarily
4. **Handle sensitive data carefully** - Use redaction features

### Security Checklist

- [ ] Tool declares appropriate security level
- [ ] All file paths validated through sandbox
- [ ] Network access restricted to required domains
- [ ] Resource limits defined and reasonable
- [ ] Error messages don't leak information
- [ ] Sensitive data redacted in logs
- [ ] Input validation comprehensive
- [ ] No dynamic code execution
- [ ] Documentation includes security considerations

---

## Performance Impact

Security features have minimal performance impact:

| Feature | Overhead | Measurement |
|---------|----------|-------------|
| Path Validation | <1ms | Per operation |
| Input Sanitization | <0.5ms | Per input |
| Resource Tracking | <2% | Overall |
| Hook Execution | <2% | With circuit breakers |
| Sandboxing | <5ms | Initialization only |

---

## Security Evolution

### Phase 3.2 Achievements
- Three-level security model implemented
- Comprehensive input validation
- File system sandboxing
- Resource limit enforcement
- 26→37 tools secured

### Phase 4 Additions
- Hook-based security monitoring
- Circuit breakers for protection
- Event correlation for auditing

### Phase 7 Standardization
- Consistent security requirements API
- Standardized validation patterns
- Security test categories

### Future Enhancements (Phase 8+)
- GUI security controls
- Advanced authentication
- Network traffic inspection
- Security analytics dashboard

---

*This document represents the consolidated security model of LLMSpell v0.6.0, validated against implementation.*