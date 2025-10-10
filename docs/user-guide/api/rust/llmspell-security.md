# llmspell-security

**Security framework with access control and audit logging**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-security) | [Source](../../../../llmspell-security)

---

## Overview

`llmspell-security` provides comprehensive security capabilities including access control, authentication, authorization, input validation, audit logging, and threat protection. It ensures safe execution of LLMSpell scripts in multi-tenant environments.

**Key Features:**
- 🔐 Role-based access control (RBAC)
- 🛡️ Input validation and sanitization
- 📝 Comprehensive audit logging
- 🚫 Threat detection and prevention
- 🔑 API key management
- 🌐 Network policy enforcement
- 🏢 Multi-tenant isolation
- ⚡ Rate limiting and DDoS protection

## Core Components

### SecurityManager

Central security orchestration:

```rust
use async_trait::async_trait;
use std::sync::Arc;

pub struct SecurityManager {
    config: SecurityConfig,
    authenticator: Arc<dyn Authenticator>,
    authorizer: Arc<dyn Authorizer>,
    validator: Arc<dyn InputValidator>,
    auditor: Arc<dyn AuditLogger>,
    threat_detector: Arc<dyn ThreatDetector>,
    rate_limiter: Arc<dyn RateLimiter>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            authenticator: Arc::new(DefaultAuthenticator::new(&config)?),
            authorizer: Arc::new(RBACAuthorizer::new(&config)?),
            validator: Arc::new(SchemaValidator::new()),
            auditor: Arc::new(FileAuditor::new(&config.audit)?),
            threat_detector: Arc::new(HeuristicThreatDetector::new()),
            rate_limiter: Arc::new(TokenBucketRateLimiter::new(&config.rate_limiting)?),
        })
    }
    
    /// Authenticate a request
    pub async fn authenticate(&self, request: &Request) -> Result<Identity> {
        let identity = self.authenticator.authenticate(request).await?;
        
        self.auditor.log(AuditEvent::Authentication {
            identity: identity.clone(),
            success: true,
            timestamp: SystemTime::now(),
        }).await?;
        
        Ok(identity)
    }
    
    /// Authorize an action
    pub async fn authorize(&self, identity: &Identity, action: &Action) -> Result<bool> {
        let authorized = self.authorizer.authorize(identity, action).await?;
        
        self.auditor.log(AuditEvent::Authorization {
            identity: identity.clone(),
            action: action.clone(),
            granted: authorized,
            timestamp: SystemTime::now(),
        }).await?;
        
        if !authorized {
            return Err(SecurityError::Unauthorized);
        }
        
        Ok(authorized)
    }
    
    /// Validate input
    pub async fn validate_input<T: Validate>(&self, input: &T) -> Result<()> {
        // Schema validation
        self.validator.validate(input).await?;
        
        // Threat detection
        if let Some(threat) = self.threat_detector.detect(input).await? {
            self.auditor.log(AuditEvent::ThreatDetected {
                threat,
                blocked: true,
                timestamp: SystemTime::now(),
            }).await?;
            
            return Err(SecurityError::ThreatDetected);
        }
        
        Ok(())
    }
    
    /// Check rate limit
    pub async fn check_rate_limit(&self, identity: &Identity, resource: &str) -> Result<()> {
        self.rate_limiter.check(identity, resource).await
    }
}
```

### Authentication

Multi-method authentication support:

```rust
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate a request
    async fn authenticate(&self, request: &Request) -> Result<Identity>;
    
    /// Verify credentials
    async fn verify_credentials(&self, credentials: &Credentials) -> Result<Identity>;
    
    /// Refresh authentication
    async fn refresh(&self, token: &str) -> Result<Identity>;
}

#[derive(Debug, Clone)]
pub enum Credentials {
    ApiKey(String),
    Bearer(String),
    Basic { username: String, password: String },
    OAuth2(OAuth2Credentials),
    Certificate(X509Certificate),
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub principal_type: PrincipalType,
    pub attributes: HashMap<String, String>,
    pub roles: HashSet<Role>,
    pub permissions: HashSet<Permission>,
    pub session_id: Option<Uuid>,
    pub expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub enum PrincipalType {
    User,
    Service,
    Application,
    Anonymous,
}

pub struct DefaultAuthenticator {
    api_key_store: Arc<dyn ApiKeyStore>,
    token_validator: Arc<dyn TokenValidator>,
}

#[async_trait]
impl Authenticator for DefaultAuthenticator {
    async fn authenticate(&self, request: &Request) -> Result<Identity> {
        // Check for API key
        if let Some(api_key) = request.header("X-API-Key") {
            return self.api_key_store.validate_key(api_key).await;
        }
        
        // Check for Bearer token
        if let Some(auth_header) = request.header("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                return self.token_validator.validate_token(token).await;
            }
        }
        
        // Check for Basic auth
        if let Some(auth_header) = request.header("Authorization") {
            if auth_header.starts_with("Basic ") {
                let encoded = &auth_header[6..];
                let decoded = base64::decode(encoded)?;
                let credentials = String::from_utf8(decoded)?;
                let parts: Vec<&str> = credentials.splitn(2, ':').collect();
                
                if parts.len() == 2 {
                    return self.verify_credentials(&Credentials::Basic {
                        username: parts[0].to_string(),
                        password: parts[1].to_string(),
                    }).await;
                }
            }
        }
        
        // Anonymous access
        Ok(Identity {
            id: "anonymous".to_string(),
            principal_type: PrincipalType::Anonymous,
            attributes: HashMap::new(),
            roles: HashSet::new(),
            permissions: HashSet::new(),
            session_id: None,
            expires_at: None,
        })
    }
}
```

### Authorization

Role-based access control:

```rust
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if identity is authorized for action
    async fn authorize(&self, identity: &Identity, action: &Action) -> Result<bool>;
    
    /// Get effective permissions for identity
    async fn get_permissions(&self, identity: &Identity) -> Result<HashSet<Permission>>;
    
    /// Check resource access
    async fn check_resource_access(&self, identity: &Identity, resource: &Resource) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct Action {
    pub operation: Operation,
    pub resource: Resource,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Read,
    Write,
    Execute,
    Delete,
    Admin,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub id: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    Agent,
    Tool,
    Workflow,
    State,
    Session,
    Artifact,
    Custom(String),
}

pub struct RBACAuthorizer {
    role_store: Arc<dyn RoleStore>,
    policy_engine: Arc<PolicyEngine>,
}

#[async_trait]
impl Authorizer for RBACAuthorizer {
    async fn authorize(&self, identity: &Identity, action: &Action) -> Result<bool> {
        // Check direct permissions
        if identity.permissions.contains(&Permission::from_action(action)) {
            return Ok(true);
        }
        
        // Check role-based permissions
        for role in &identity.roles {
            let role_permissions = self.role_store.get_permissions(role).await?;
            if role_permissions.contains(&Permission::from_action(action)) {
                return Ok(true);
            }
        }
        
        // Check policies
        self.policy_engine.evaluate(identity, action).await
    }
}
```

### Input Validation

Comprehensive input sanitization:

```rust
#[async_trait]
pub trait InputValidator: Send + Sync {
    /// Validate input against schema
    async fn validate<T: Validate>(&self, input: &T) -> Result<()>;
    
    /// Sanitize input
    async fn sanitize<T: Sanitize>(&self, input: &mut T) -> Result<()>;
}

pub trait Validate {
    fn validation_schema(&self) -> Schema;
    fn validate(&self) -> Result<()>;
}

pub trait Sanitize {
    fn sanitize(&mut self) -> Result<()>;
}

pub struct SchemaValidator {
    schemas: HashMap<String, Schema>,
}

impl SchemaValidator {
    /// Validate JSON against schema
    pub fn validate_json(&self, data: &Value, schema: &Schema) -> Result<()> {
        let compiled = JSONSchema::compile(schema)?;
        
        if let Err(errors) = compiled.validate(data) {
            let error_messages: Vec<String> = errors
                .map(|e| format!("{}: {}", e.instance_path, e.to_string()))
                .collect();
            
            return Err(ValidationError::SchemaViolation(error_messages));
        }
        
        Ok(())
    }
    
    /// Validate string input
    pub fn validate_string(&self, input: &str, rules: &StringRules) -> Result<()> {
        // Length check
        if let Some(min) = rules.min_length {
            if input.len() < min {
                return Err(ValidationError::TooShort(min));
            }
        }
        
        if let Some(max) = rules.max_length {
            if input.len() > max {
                return Err(ValidationError::TooLong(max));
            }
        }
        
        // Pattern check
        if let Some(ref pattern) = rules.pattern {
            let re = Regex::new(pattern)?;
            if !re.is_match(input) {
                return Err(ValidationError::PatternMismatch);
            }
        }
        
        // Blacklist check
        for blacklisted in &rules.blacklist {
            if input.contains(blacklisted) {
                return Err(ValidationError::BlacklistedContent);
            }
        }
        
        Ok(())
    }
}
```

### Threat Detection

Identify and prevent security threats:

```rust
#[async_trait]
pub trait ThreatDetector: Send + Sync {
    /// Detect threats in input
    async fn detect<T>(&self, input: &T) -> Result<Option<Threat>>;
    
    /// Analyze behavior for threats
    async fn analyze_behavior(&self, behavior: &BehaviorProfile) -> Result<ThreatLevel>;
}

#[derive(Debug, Clone)]
pub struct Threat {
    pub threat_type: ThreatType,
    pub severity: ThreatLevel,
    pub description: String,
    pub indicators: Vec<String>,
    pub recommended_action: Action,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    SqlInjection,
    CommandInjection,
    PathTraversal,
    XXE,
    SSRF,
    DoS,
    DataExfiltration,
    PrivilegeEscalation,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct HeuristicThreatDetector {
    patterns: Vec<ThreatPattern>,
    ml_model: Option<Arc<dyn MLModel>>,
}

impl HeuristicThreatDetector {
    /// Check for SQL injection
    fn check_sql_injection(&self, input: &str) -> Option<Threat> {
        let sql_patterns = [
            r"(\b(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)\b)",
            r"(--|#|/\*|\*/)",
            r"(\bOR\b.*=.*)",
            r"(\bUNION\b.*\bSELECT\b)",
        ];
        
        for pattern in &sql_patterns {
            if Regex::new(pattern).unwrap().is_match(input) {
                return Some(Threat {
                    threat_type: ThreatType::SqlInjection,
                    severity: ThreatLevel::High,
                    description: "Potential SQL injection detected".to_string(),
                    indicators: vec![pattern.to_string()],
                    recommended_action: Action::Block,
                });
            }
        }
        
        None
    }
    
    /// Check for command injection
    fn check_command_injection(&self, input: &str) -> Option<Threat> {
        let cmd_patterns = [
            r"(;|\||&&|\$\(|\`)",
            r"(rm\s+-rf|dd\s+if=|mkfs|format)",
            r"(/etc/passwd|/etc/shadow)",
        ];
        
        for pattern in &cmd_patterns {
            if Regex::new(pattern).unwrap().is_match(input) {
                return Some(Threat {
                    threat_type: ThreatType::CommandInjection,
                    severity: ThreatLevel::Critical,
                    description: "Potential command injection detected".to_string(),
                    indicators: vec![pattern.to_string()],
                    recommended_action: Action::Block,
                });
            }
        }
        
        None
    }
}
```

### Audit Logging

Comprehensive security audit trail:

```rust
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    async fn log(&self, event: AuditEvent) -> Result<()>;
    
    /// Query audit logs
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>>;
    
    /// Export audit logs
    async fn export(&self, format: ExportFormat, filter: Option<AuditFilter>) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    Authentication {
        identity: Identity,
        success: bool,
        timestamp: SystemTime,
    },
    Authorization {
        identity: Identity,
        action: Action,
        granted: bool,
        timestamp: SystemTime,
    },
    DataAccess {
        identity: Identity,
        resource: Resource,
        operation: Operation,
        timestamp: SystemTime,
    },
    ConfigChange {
        identity: Identity,
        setting: String,
        old_value: Option<Value>,
        new_value: Value,
        timestamp: SystemTime,
    },
    ThreatDetected {
        threat: Threat,
        blocked: bool,
        timestamp: SystemTime,
    },
    SecurityViolation {
        identity: Identity,
        violation_type: String,
        details: String,
        timestamp: SystemTime,
    },
}

pub struct FileAuditor {
    file_path: PathBuf,
    rotation_policy: RotationPolicy,
    encryption: Option<Box<dyn Encryptor>>,
}

#[async_trait]
impl AuditLogger for FileAuditor {
    async fn log(&self, event: AuditEvent) -> Result<()> {
        let mut line = serde_json::to_string(&event)?;
        
        // Encrypt if configured
        if let Some(ref encryptor) = self.encryption {
            line = encryptor.encrypt(&line)?;
        }
        
        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await?;
        
        file.write_all(format!("{}\n", line).as_bytes()).await?;
        file.sync_all().await?;
        
        // Check rotation
        self.check_rotation().await?;
        
        Ok(())
    }
}
```

### Rate Limiting

Prevent abuse and DoS attacks:

```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is within rate limit
    async fn check(&self, identity: &Identity, resource: &str) -> Result<()>;
    
    /// Get current usage
    async fn get_usage(&self, identity: &Identity) -> Result<Usage>;
    
    /// Reset limits for identity
    async fn reset(&self, identity: &Identity) -> Result<()>;
}

pub struct TokenBucketRateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: usize,
    tokens: AtomicUsize,
    refill_rate: usize,
    last_refill: AtomicU64,
}

impl TokenBucket {
    /// Try to consume tokens
    pub fn try_consume(&self, tokens: usize) -> bool {
        // Refill bucket
        self.refill();
        
        // Try to consume
        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < tokens {
                return false;
            }
            
            match self.tokens.compare_exchange(
                current,
                current - tokens,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }
    
    /// Refill tokens based on elapsed time
    fn refill(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last = self.last_refill.load(Ordering::Relaxed);
        let elapsed = now - last;
        
        if elapsed > 0 {
            let tokens_to_add = (elapsed as usize * self.refill_rate).min(self.capacity);
            let current = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current + tokens_to_add).min(self.capacity);
            
            self.tokens.store(new_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
        }
    }
}
```

### Network Security

Control network access:

```rust
pub struct NetworkSecurityManager {
    firewall: Arc<dyn Firewall>,
    dns_filter: Arc<dyn DnsFilter>,
    proxy: Option<Arc<dyn Proxy>>,
}

impl NetworkSecurityManager {
    /// Check if URL is allowed
    pub async fn check_url(&self, url: &Url) -> Result<bool> {
        // Check firewall rules
        if !self.firewall.is_allowed(url).await? {
            return Ok(false);
        }
        
        // Check DNS filtering
        if !self.dns_filter.is_safe(url.host_str().unwrap_or("")).await? {
            return Ok(false);
        }
        
        // Check for SSRF
        if self.is_internal_address(url)? {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Check for SSRF attempts
    fn is_internal_address(&self, url: &Url) -> Result<bool> {
        let host = url.host_str().ok_or(NetworkError::InvalidHost)?;
        
        // Check for localhost
        if host == "localhost" || host == "127.0.0.1" || host == "::1" {
            return Ok(true);
        }
        
        // Check for private networks
        if let Ok(ip) = host.parse::<IpAddr>() {
            match ip {
                IpAddr::V4(ipv4) => {
                    if ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local() {
                        return Ok(true);
                    }
                }
                IpAddr::V6(ipv6) => {
                    if ipv6.is_loopback() || ipv6.is_unique_local() {
                        return Ok(true);
                    }
                }
            }
        }
        
        // Check for metadata endpoints
        let metadata_endpoints = [
            "169.254.169.254",  // AWS
            "metadata.google.internal",  // GCP
            "169.254.169.254",  // Azure
        ];
        
        if metadata_endpoints.contains(&host) {
            return Ok(true);
        }
        
        Ok(false)
    }
}
```

---

## Sandbox System

The sandbox system provides defense-in-depth security through file, network, and resource isolation. It enforces the three-level security model (Safe, Restricted, Privileged) at runtime.

> **📚 User Guide**: See [Security & Permissions Guide](../../security-and-permissions.md) for complete configuration details and troubleshooting.

### SecurityRequirements

Define permissions for tools and agents using a fluent API:

```rust
use llmspell_security::SecurityRequirements;

// Safe - no external access (pure computation)
let safe_reqs = SecurityRequirements::safe();

// Restricted - explicit permissions required
let restricted_reqs = SecurityRequirements::restricted()
    .with_file_access("/workspace")
    .with_file_access("/tmp")
    .with_network_access("api.openai.com")
    .with_network_access("*.github.com")  // Wildcard support
    .with_env_access("HOME")
    .with_env_access("PATH");

// Privileged - full access (use sparingly, requires review)
let privileged_reqs = SecurityRequirements::privileged();
```

**Security Levels**:
- **Safe**: No file/network/process access - pure computation only
- **Restricted**: Requires explicit allowlists via `with_*` methods (DEFAULT)
- **Privileged**: Full system access - should be exception, not rule

---

### FileSandbox

Path-based file system isolation with traversal protection:

```rust
use llmspell_security::sandbox::{FileSandbox, SandboxContext};
use llmspell_security::{SecurityRequirements, ResourceLimits};
use std::path::Path;

// Create sandbox context
let context = SandboxContext::new(
    "my-sandbox",
    SecurityRequirements::restricted()
        .with_file_access("/workspace")
        .with_file_access("/tmp"),
    ResourceLimits::default(),
);

let sandbox = FileSandbox::new(context)?;

// Validate paths before use
sandbox.validate_path(Path::new("/workspace/data.txt"))?;  // ✅ OK
sandbox.validate_path(Path::new("/tmp/output.json"))?;     // ✅ OK
sandbox.validate_path(Path::new("/etc/passwd"))?;          // ❌ ERROR: not in allowlist
sandbox.validate_path(Path::new("/workspace/../etc/passwd"))?;  // ❌ ERROR: path traversal blocked
```

**Features**:
- Path allowlisting enforcement
- Path traversal protection (`../` attacks)
- Symlink resolution with validation
- File extension filtering
- Size limits

---

### NetworkSandbox

Domain-based network isolation with rate limiting and SSRF prevention:

```rust
use llmspell_security::sandbox::{NetworkSandbox, RateLimitConfig};

// Create network sandbox
let context = SandboxContext::new(
    "network-sandbox",
    SecurityRequirements::restricted()
        .with_network_access("api.example.com")
        .with_network_access("*.github.com"),
    ResourceLimits::default(),
);

let mut sandbox = NetworkSandbox::new(
    context,
    RateLimitConfig {
        max_requests: 100,
        window_seconds: 60,
    }
)?;

// Validate requests before execution
sandbox.validate_request("https://api.example.com/data", "GET").await?;  // ✅ OK
sandbox.validate_request("https://api.github.com/repos", "GET").await?;  // ✅ OK (wildcard)
sandbox.validate_request("https://evil.com/data", "GET").await?;         // ❌ ERROR: domain blocked
sandbox.validate_request("http://localhost/admin", "GET").await?;        // ❌ ERROR: SSRF blocked

// Make safe HTTP requests
let response = sandbox.get("https://api.example.com/data").await?;
```

**Features**:
- Domain allowlisting with wildcard support
- Rate limiting (default: 100 requests/minute)
- SSRF prevention (blocks localhost, private IPs, cloud metadata)
- Request/response size limits

**SSRF Protection**: Automatically blocks:
- `localhost`, `127.0.0.1`, `0.0.0.0`
- Private IP ranges (`10.0.0.0/8`, `192.168.0.0/16`, `172.16.0.0/12`)
- Cloud metadata endpoints (`169.254.169.254`)

---

### IntegratedSandbox

Combined file, network, and resource isolation for comprehensive protection:

```rust
use llmspell_security::sandbox::IntegratedSandbox;
use llmspell_security::ResourceLimits;
use std::time::Duration;

// Create integrated sandbox with all restrictions
let sandbox = IntegratedSandbox::builder()
    .with_file_permissions(vec!["/workspace", "/tmp"])
    .with_network_policy(vec!["api.openai.com", "*.anthropic.com"])
    .with_resource_limits(ResourceLimits {
        max_memory: 512 * 1024 * 1024,      // 512MB
        max_cpu_time: Duration::from_secs(300),  // 5 minutes
        max_file_size: 100 * 1024 * 1024,   // 100MB
        max_open_files: 50,
        max_network_connections: 10,
    })
    .build()?;

// Execute with comprehensive monitoring
sandbox.execute_with_monitoring(|| async {
    // Your code here - runs with all restrictions enforced
    // File access: only /workspace and /tmp
    // Network: only allowed domains
    // Resources: memory, CPU, and connection limits

    Ok(())
}).await?;

// Check for violations
if sandbox.has_violations().await {
    for violation in sandbox.get_violations().await {
        eprintln!("Security violation: {:?}", violation);
        // Log to audit system, alert, etc.
    }
}
```

**Resource Limits**:
- **Memory**: Per-execution memory cap (default 100MB)
- **CPU Time**: Maximum execution duration (default 5 seconds)
- **File Size**: Maximum file read/write size (default 50MB)
- **Open Files**: Maximum concurrent file handles (default 50)
- **Network Connections**: Maximum concurrent connections (default 10)

---

### SandboxContext

The `SandboxContext` ties together security requirements and resource limits:

```rust
use llmspell_security::sandbox::SandboxContext;
use llmspell_security::{SecurityRequirements, ResourceLimits};

let context = SandboxContext::new(
    "my-context",
    SecurityRequirements::restricted()
        .with_file_access("/workspace")
        .with_network_access("*.api.com"),
    ResourceLimits {
        max_memory: 256 * 1024 * 1024,  // 256MB
        max_cpu_time: Duration::from_secs(10),
        ..Default::default()
    },
);

// Check domain allowance
if context.is_domain_allowed("api.api.com") {  // ✅ Matches wildcard
    println!("Domain allowed");
}

// Check path allowance
if context.is_path_allowed(Path::new("/workspace/data.txt")) {  // ✅ OK
    println!("Path allowed");
}
```

---

### Configuration Integration

Sandbox permissions are typically configured via TOML rather than hardcoded:

```toml
# config.toml

[tools.file_operations]
allowed_paths = ["/workspace", "/tmp", "/data"]
max_file_size = 50000000
blocked_extensions = ["exe", "dll", "so"]

[tools.web_search]
allowed_domains = ["api.openai.com", "*.github.com"]
rate_limit_per_minute = 100

[tools.http_request]
allowed_hosts = ["*.example.com"]
blocked_hosts = ["localhost", "127.0.0.1"]

[tools.system]
allow_process_execution = false
allowed_commands = "echo,cat,ls,pwd"
```

These configurations are automatically wired to sandbox instances via `llmspell-bridge`.

**For complete configuration guide**, see [Security & Permissions Guide](../../security-and-permissions.md).

---

## Usage Examples

### Basic Security Setup

```rust
use llmspell_security::{SecurityManager, SecurityConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure security
    let config = SecurityConfig {
        authentication: AuthConfig {
            methods: vec![AuthMethod::ApiKey, AuthMethod::Bearer],
            session_timeout: Duration::from_hours(24),
        },
        authorization: AuthzConfig {
            mode: AuthzMode::RBAC,
            default_policy: PolicyEffect::Deny,
        },
        audit: AuditConfig {
            enabled: true,
            log_path: "./audit.log".into(),
            rotation: RotationPolicy::Daily,
            encryption: true,
        },
        rate_limiting: RateLimitConfig {
            enabled: true,
            default_limit: 1000,
            window: Duration::from_mins(1),
        },
        threat_detection: ThreatConfig {
            enabled: true,
            ml_model: Some("./threat_model.bin".into()),
        },
    };
    
    let security = SecurityManager::new(config)?;
    
    // Use security manager
    let request = create_request();
    let identity = security.authenticate(&request).await?;
    
    let action = Action {
        operation: Operation::Execute,
        resource: Resource {
            resource_type: ResourceType::Agent,
            id: "agent123".to_string(),
            attributes: HashMap::new(),
        },
        context: HashMap::new(),
    };
    
    security.authorize(&identity, &action).await?;
    
    Ok(())
}
```

### API Key Management

```rust
use llmspell_security::ApiKeyManager;

async fn manage_api_keys(manager: &ApiKeyManager) -> Result<()> {
    // Generate new API key
    let key = manager.generate_key(ApiKeyOptions {
        name: "Production API".to_string(),
        expires_at: Some(SystemTime::now() + Duration::from_days(365)),
        permissions: vec![
            Permission::ReadState,
            Permission::ExecuteAgent,
        ],
        rate_limit: Some(10000),
    }).await?;
    
    println!("Generated API key: {}", key.key);
    
    // List API keys
    let keys = manager.list_keys().await?;
    for key in keys {
        println!("Key: {} (expires: {:?})", key.name, key.expires_at);
    }
    
    // Revoke API key
    manager.revoke_key(&key.id).await?;
    
    Ok(())
}
```

### Input Validation

```rust
use llmspell_security::{SchemaValidator, StringRules};

async fn validate_user_input(validator: &SchemaValidator) -> Result<()> {
    // Define validation schema
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 100,
                "pattern": "^[a-zA-Z0-9_-]+$"
            },
            "email": {
                "type": "string",
                "format": "email"
            },
            "age": {
                "type": "integer",
                "minimum": 0,
                "maximum": 150
            }
        },
        "required": ["name", "email"]
    });
    
    // Validate input
    let input = json!({
        "name": "john_doe",
        "email": "john@example.com",
        "age": 30
    });
    
    validator.validate_json(&input, &schema)?;
    
    // String validation
    let username = "john_doe_123";
    validator.validate_string(username, &StringRules {
        min_length: Some(3),
        max_length: Some(20),
        pattern: Some(r"^[a-zA-Z0-9_]+$".to_string()),
        blacklist: vec!["admin".to_string(), "root".to_string()],
    })?;
    
    Ok(())
}
```

### Threat Detection

```rust
async fn detect_threats(detector: &HeuristicThreatDetector) -> Result<()> {
    let inputs = vec![
        "SELECT * FROM users WHERE id = 1 OR 1=1",
        "../../etc/passwd",
        "normal input text",
        "; rm -rf /",
    ];
    
    for input in inputs {
        if let Some(threat) = detector.detect(&input).await? {
            println!("THREAT DETECTED in '{}': {:?} ({})", 
                input, threat.threat_type, threat.severity);
            
            // Take action based on severity
            match threat.severity {
                ThreatLevel::Critical => {
                    // Block immediately and alert
                    block_request();
                    send_security_alert(&threat);
                }
                ThreatLevel::High => {
                    // Block and log
                    block_request();
                    log_threat(&threat);
                }
                ThreatLevel::Medium => {
                    // Log and monitor
                    log_threat(&threat);
                }
                ThreatLevel::Low => {
                    // Just log
                    debug!("Low threat: {:?}", threat);
                }
            }
        } else {
            println!("Input '{}' is safe", input);
        }
    }
    
    Ok(())
}
```

### Audit Log Analysis

```rust
async fn analyze_audit_logs(auditor: &FileAuditor) -> Result<()> {
    // Query recent authentication failures
    let failed_auths = auditor.query(AuditFilter {
        event_types: Some(vec![AuditEventType::Authentication]),
        time_range: Some((SystemTime::now() - Duration::from_hours(24), SystemTime::now())),
        custom_filter: Some(Box::new(|event| {
            match event {
                AuditEvent::Authentication { success, .. } => !success,
                _ => false,
            }
        })),
    }).await?;
    
    println!("Failed authentications in last 24h: {}", failed_auths.len());
    
    // Find suspicious patterns
    let mut ip_failure_count: HashMap<String, usize> = HashMap::new();
    for event in failed_auths {
        if let AuditEvent::Authentication { identity, .. } = event {
            *ip_failure_count.entry(identity.attributes.get("ip").cloned().unwrap_or_default())
                .or_insert(0) += 1;
        }
    }
    
    // Alert on IPs with many failures
    for (ip, count) in ip_failure_count {
        if count > 10 {
            println!("ALERT: IP {} has {} failed auth attempts", ip, count);
        }
    }
    
    // Export logs for compliance
    let export_data = auditor.export(ExportFormat::Json, Some(AuditFilter {
        time_range: Some((SystemTime::now() - Duration::from_days(30), SystemTime::now())),
        ..Default::default()
    })).await?;
    
    std::fs::write("audit_export.json", export_data)?;
    
    Ok(())
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sql_injection_detection() {
        let detector = HeuristicThreatDetector::new();
        
        let malicious_inputs = vec![
            "' OR '1'='1",
            "1; DROP TABLE users",
            "admin'--",
        ];
        
        for input in malicious_inputs {
            let threat = detector.detect(&input).await.unwrap();
            assert!(threat.is_some());
            assert_eq!(threat.unwrap().threat_type, ThreatType::SqlInjection);
        }
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = TokenBucketRateLimiter::new(RateLimitConfig {
            enabled: true,
            default_limit: 10,
            window: Duration::from_secs(1),
        });
        
        let identity = Identity {
            id: "test_user".to_string(),
            ..Default::default()
        };
        
        // Should allow first 10 requests
        for _ in 0..10 {
            assert!(limiter.check(&identity, "api").await.is_ok());
        }
        
        // Should block 11th request
        assert!(limiter.check(&identity, "api").await.is_err());
        
        // Wait for refill
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should allow again
        assert!(limiter.check(&identity, "api").await.is_ok());
    }
}
```

## Security Best Practices

1. **Defense in Depth**: Layer multiple security controls
2. **Least Privilege**: Grant minimum necessary permissions
3. **Input Validation**: Never trust user input
4. **Audit Everything**: Log all security-relevant events
5. **Fail Secure**: Default to denying access
6. **Regular Updates**: Keep security patterns current
7. **Threat Modeling**: Anticipate attack vectors

## Related Documentation

- [llmspell-tenancy](llmspell-tenancy.md) - Multi-tenant isolation
- [llmspell-sessions](llmspell-sessions.md) - Session security contexts
- [llmspell-utils](llmspell-utils.md) - Security utilities