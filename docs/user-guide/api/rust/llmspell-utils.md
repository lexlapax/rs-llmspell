# llmspell-utils

## Purpose

Shared utility library providing essential building blocks for the LLMSpell framework including async operations, file handling, security utilities, rate limiting, circuit breakers, API key management, and system monitoring. This crate contains battle-tested utilities that prevent code duplication across other crates.

## Core Concepts

- **Async-First**: All IO operations are async with timeout and cancellation support
- **Security by Default**: Built-in SSRF protection, path traversal prevention, and credential sanitization  
- **Resource Protection**: Rate limiting, circuit breakers, and memory tracking
- **Cross-Platform**: File operations work consistently across Windows, macOS, and Linux
- **Performance Monitoring**: Built-in metrics and progress tracking
- **Error Recovery**: Retry logic with exponential backoff for transient failures
- **API Key Management**: Secure storage and rotation of provider API keys

## Primary Utilities

### Async Utilities

**Purpose**: Async operation helpers including timeouts, retries, concurrent execution, and cancellation.

```rust
use llmspell_utils::async_utils::{
    timeout, timeout_with_default, retry_async, 
    concurrent_map, race_to_success,
    RetryConfig, Cancellable
};
use std::time::Duration;

/// Execute with timeout
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, AsyncError>
where
    F: Future<Output = T> + Send,
    T: Send,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| AsyncError::Timeout(duration))
}

/// Execute with timeout and default value
pub async fn timeout_with_default<F, T>(
    duration: Duration,
    future: F,
    default: T,
) -> T
where
    F: Future<Output = T> + Send,
    T: Send,
{
    timeout(duration, future).await.unwrap_or(default)
}

/// Retry async operation with exponential backoff
pub async fn retry_async<F, T, E>(
    config: RetryConfig,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> BoxedResultFuture<T, E>,
    E: std::error::Error,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= config.max_attempts => return Err(e),
            Err(_) => {
                attempt += 1;
                tokio::time::sleep(delay).await;
                delay = delay.saturating_mul(config.backoff_factor);
            }
        }
    }
}

/// Execute multiple futures concurrently with limit
pub async fn concurrent_map<T, F, R>(
    items: Vec<T>,
    max_concurrent: usize,
    mapper: F,
) -> Vec<R>
where
    T: Send + 'static,
    F: Fn(T) -> BoxedResultFuture<R, AsyncError> + Clone + Send + 'static,
    R: Send + 'static,
{
    use futures::stream::{self, StreamExt};
    
    stream::iter(items)
        .map(mapper)
        .buffer_unordered(max_concurrent)
        .collect()
        .await
}

/// Race multiple operations, return first success
pub async fn race_to_success<F, T>(
    operations: Vec<F>,
) -> Result<T, AsyncError>
where
    F: Future<Output = Result<T, AsyncError>> + Send,
    T: Send,
{
    use futures::future::select_all;
    
    let (result, _, _) = select_all(operations).await;
    result
}
```

**Usage Example**:
```rust
use llmspell_utils::async_utils::{timeout, retry_async, RetryConfig};
use std::time::Duration;

// Timeout example
let result = timeout(Duration::from_secs(5), async {
    expensive_operation().await
}).await?;

// Retry example  
let config = RetryConfig {
    max_attempts: 3,
    initial_delay: Duration::from_millis(100),
    backoff_factor: 2,
    max_delay: Some(Duration::from_secs(10)),
};

let result = retry_async(config, || {
    Box::pin(async {
        unreliable_network_call().await
    })
}).await?;

// Concurrent execution
let results = concurrent_map(
    vec![1, 2, 3, 4, 5],
    2, // Max 2 concurrent
    |n| Box::pin(async move {
        process_item(n).await
    })
).await;
```

### Security Utilities

**Purpose**: Security utilities for input validation, SSRF protection, path traversal prevention, and credential protection.

```rust
use llmspell_utils::security::{
    SecurityContext, PathValidator, SSRFProtection,
    CredentialSanitizer, MemoryTracker, ExpressionValidator
};

/// Path validation to prevent traversal attacks
pub struct PathValidator {
    allowed_paths: Vec<PathBuf>,
    jail_root: Option<PathBuf>,
}

impl PathValidator {
    pub fn validate(&self, path: &Path) -> Result<PathBuf, SecurityError> {
        let canonical = path.canonicalize()
            .map_err(|_| SecurityError::InvalidPath)?;
        
        // Check jail root
        if let Some(jail) = &self.jail_root {
            if !canonical.starts_with(jail) {
                return Err(SecurityError::PathTraversal);
            }
        }
        
        // Check allowed paths
        if !self.allowed_paths.iter().any(|p| canonical.starts_with(p)) {
            return Err(SecurityError::Unauthorized);
        }
        
        Ok(canonical)
    }
}

/// SSRF protection for URLs
pub struct SSRFProtection {
    blocked_ips: HashSet<IpAddr>,
    blocked_domains: HashSet<String>,
    allowed_schemes: HashSet<String>,
}

impl SSRFProtection {
    pub async fn validate_url(&self, url: &str) -> Result<Url, SecurityError> {
        let parsed = Url::parse(url)
            .map_err(|_| SecurityError::InvalidUrl)?;
        
        // Check scheme
        if !self.allowed_schemes.contains(parsed.scheme()) {
            return Err(SecurityError::BlockedScheme);
        }
        
        // Resolve and check IP
        let host = parsed.host_str()
            .ok_or(SecurityError::InvalidUrl)?;
        
        // Check for blocked domains
        if self.blocked_domains.contains(host) {
            return Err(SecurityError::BlockedDomain);
        }
        
        // Resolve DNS and check IP
        let addr = resolve_host(host).await?;
        if self.is_private_ip(&addr) || self.blocked_ips.contains(&addr) {
            return Err(SecurityError::BlockedIP);
        }
        
        Ok(parsed)
    }
    
    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_loopback() ||
                ipv4.is_private() ||
                ipv4.is_link_local()
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() ||
                ipv6.is_unique_local()
            }
        }
    }
}

/// Credential sanitization for logs/errors
pub struct CredentialSanitizer {
    patterns: Vec<Regex>,
}

impl CredentialSanitizer {
    pub fn sanitize(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        for pattern in &self.patterns {
            result = pattern.replace_all(&result, "[REDACTED]").to_string();
        }
        
        result
    }
}
```

**Usage Example**:
```rust
use llmspell_utils::security::{PathValidator, SSRFProtection};
use std::path::PathBuf;

// Path validation
let validator = PathValidator::new()
    .with_jail_root("/app/data")
    .with_allowed_path("/app/data/uploads")
    .build();

let safe_path = validator.validate(user_provided_path)?;

// SSRF protection
let ssrf = SSRFProtection::default()
    .allow_schemes(vec!["https"])
    .block_private_ips()
    .build();

let safe_url = ssrf.validate_url(user_provided_url).await?;

// Credential sanitization
let sanitizer = CredentialSanitizer::default();
let safe_log = sanitizer.sanitize(error_message);
println!("Error: {}", safe_log); // API keys will be [REDACTED]
```

### File Utilities

**Purpose**: Cross-platform file operations with atomic writes, directory management, and metadata access.

```rust
use llmspell_utils::file_utils::{
    read_file, write_file, write_file_atomic,
    ensure_dir, list_dir, copy_file, move_file,
    file_exists, get_metadata, normalize_path,
    DirEntry, FileMetadata
};

/// Read file contents as string
pub async fn read_file(path: impl AsRef<Path>) -> Result<String, FileError> {
    tokio::fs::read_to_string(path.as_ref())
        .await
        .map_err(|e| FileError::Read {
            path: path.as_ref().to_path_buf(),
            source: e,
        })
}

/// Write file with automatic parent directory creation
pub async fn write_file(
    path: impl AsRef<Path>,
    content: impl AsRef<[u8]>,
) -> Result<(), FileError> {
    if let Some(parent) = path.as_ref().parent() {
        ensure_dir(parent).await?;
    }
    
    tokio::fs::write(path.as_ref(), content.as_ref())
        .await
        .map_err(|e| FileError::Write {
            path: path.as_ref().to_path_buf(),
            source: e,
        })
}

/// Atomic file write (write to temp, then rename)
pub async fn write_file_atomic(
    path: impl AsRef<Path>,
    content: impl AsRef<[u8]>,
) -> Result<(), FileError> {
    let temp_path = format!("{}.tmp", path.as_ref().display());
    
    // Write to temp file
    write_file(&temp_path, content).await?;
    
    // Atomic rename
    tokio::fs::rename(&temp_path, path.as_ref())
        .await
        .map_err(|e| FileError::Write {
            path: path.as_ref().to_path_buf(),
            source: e,
        })?;
    
    Ok(())
}

/// List directory contents with filtering
pub async fn list_dir(
    path: impl AsRef<Path>,
    filter: Option<FileFilter>,
) -> Result<Vec<DirEntry>, FileError> {
    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(path.as_ref()).await?;
    
    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let dir_entry = DirEntry {
            path: entry.path(),
            name: entry.file_name().to_string_lossy().to_string(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
        };
        
        if let Some(ref f) = filter {
            if !f.matches(&dir_entry) {
                continue;
            }
        }
        
        entries.push(dir_entry);
    }
    
    Ok(entries)
}
```

### Rate Limiting

**Purpose**: Provider-specific and global rate limiting with token bucket algorithm.

```rust
use llmspell_utils::rate_limiter::{RateLimiter, RateLimitConfig};
use std::time::Duration;

pub struct RateLimiter {
    capacity: usize,
    refill_rate: f64,
    tokens: Arc<Mutex<f64>>,
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            capacity: config.capacity,
            refill_rate: config.refill_per_second,
            tokens: Arc::new(Mutex::new(config.capacity as f64)),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    pub async fn acquire(&self, tokens: usize) -> Result<(), RateLimitError> {
        loop {
            self.refill().await;
            
            let mut available = self.tokens.lock().await;
            if *available >= tokens as f64 {
                *available -= tokens as f64;
                return Ok(());
            }
            
            // Calculate wait time
            let needed = tokens as f64 - *available;
            let wait_ms = (needed / self.refill_rate * 1000.0) as u64;
            
            drop(available);
            tokio::time::sleep(Duration::from_millis(wait_ms)).await;
        }
    }
    
    async fn refill(&self) {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;
        
        let elapsed = last_refill.elapsed();
        let refill_amount = elapsed.as_secs_f64() * self.refill_rate;
        
        *tokens = (*tokens + refill_amount).min(self.capacity as f64);
        *last_refill = Instant::now();
    }
}
```

### Circuit Breaker

**Purpose**: Fault tolerance pattern to prevent cascading failures.

```rust
use llmspell_utils::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, State
};

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<State>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, Box<dyn Error>>>,
    {
        // Check state
        let state = self.state.read().await;
        match *state {
            State::Open => {
                // Check if we should transition to half-open
                if self.should_attempt_reset().await {
                    drop(state);
                    *self.state.write().await = State::HalfOpen;
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
            State::HalfOpen => {
                // Allow limited requests
                if self.half_open_requests.fetch_add(1) >= self.config.half_open_max {
                    return Err(CircuitBreakerError::Open);
                }
            }
            State::Closed => {} // Normal operation
        }
        
        // Execute operation
        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }
    
    async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1) + 1;
        
        if count >= self.config.failure_threshold {
            *self.state.write().await = State::Open;
            *self.last_failure.write().await = Some(Instant::now());
        }
    }
    
    async fn record_success(&self) {
        let state = self.state.read().await;
        if matches!(*state, State::HalfOpen) {
            let count = self.success_count.fetch_add(1) + 1;
            if count >= self.config.success_threshold {
                drop(state);
                *self.state.write().await = State::Closed;
                self.failure_count.store(0, Ordering::Relaxed);
            }
        }
    }
}
```

### API Key Management

**Purpose**: Secure storage, rotation, and management of provider API keys.

```rust
use llmspell_utils::api_key_manager::{
    ApiKeyManager, ApiKey, KeyRotationPolicy
};

pub struct ApiKeyManager {
    storage: Box<dyn ApiKeyStorage>,
    encryption: Box<dyn KeyEncryption>,
    rotation_policy: KeyRotationPolicy,
}

impl ApiKeyManager {
    pub async fn store_key(
        &self,
        provider: String,
        key: String,
    ) -> Result<ApiKey, KeyError> {
        // Validate key format
        self.validate_key_format(&provider, &key)?;
        
        // Encrypt key
        let encrypted = self.encryption.encrypt(&key).await?;
        
        // Store with metadata
        let api_key = ApiKey {
            id: Uuid::new_v4(),
            provider,
            encrypted_value: encrypted,
            created_at: SystemTime::now(),
            last_used: None,
            rotation_due: self.calculate_rotation_date(),
        };
        
        self.storage.store(api_key.clone()).await?;
        Ok(api_key)
    }
    
    pub async fn get_key(&self, provider: &str) -> Result<String, KeyError> {
        let api_key = self.storage.get(provider).await?;
        
        // Check rotation
        if api_key.needs_rotation(&self.rotation_policy) {
            self.notify_rotation_needed(&api_key).await;
        }
        
        // Decrypt and return
        let decrypted = self.encryption.decrypt(&api_key.encrypted_value).await?;
        
        // Update last used
        self.storage.update_last_used(provider).await?;
        
        Ok(decrypted)
    }
    
    pub async fn rotate_key(
        &self,
        provider: &str,
        new_key: String,
    ) -> Result<(), KeyError> {
        // Store new key
        self.store_key(provider.to_string(), new_key).await?;
        
        // Archive old key
        self.storage.archive_old_key(provider).await?;
        
        Ok(())
    }
}
```

## Usage Patterns

### Retry with Circuit Breaker

**When to use**: For external service calls that may fail transiently but shouldn't overwhelm the service.

**Benefits**: Combines retry logic with circuit breaker to prevent cascading failures.

**Example**:
```rust
use llmspell_utils::{
    async_utils::{retry_async, RetryConfig},
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
};

pub struct ResilientClient {
    circuit_breaker: CircuitBreaker,
    retry_config: RetryConfig,
}

impl ResilientClient {
    pub async fn call_service<T>(&self, request: Request) -> Result<T, Error> {
        retry_async(self.retry_config.clone(), || {
            let cb = self.circuit_breaker.clone();
            let req = request.clone();
            
            Box::pin(async move {
                cb.call(async move {
                    make_http_request(req).await
                }).await
            })
        }).await
    }
}
```

### Secure File Operations

**When to use**: When handling user-provided file paths or uploads.

**Benefits**: Prevents path traversal and validates file operations.

**Example**:
```rust
use llmspell_utils::{
    security::PathValidator,
    file_utils::{read_file, write_file_atomic},
};

pub async fn process_user_file(
    user_path: &str,
    content: String,
) -> Result<(), Error> {
    // Validate path
    let validator = PathValidator::new()
        .with_jail_root("/app/user_data")
        .build();
    
    let safe_path = validator.validate(user_path)?;
    
    // Read existing content
    let existing = read_file(&safe_path).await.unwrap_or_default();
    
    // Process content
    let processed = format!("{}\n{}", existing, content);
    
    // Write atomically
    write_file_atomic(&safe_path, processed).await?;
    
    Ok(())
}
```

### Progress Tracking

**When to use**: For long-running operations that need progress reporting.

**Benefits**: Provides structured progress updates with ETA calculation.

**Example**:
```rust
use llmspell_utils::progress::{ProgressTracker, ProgressUpdate};

pub async fn process_batch(items: Vec<Item>) -> Result<Vec<Result>, Error> {
    let tracker = ProgressTracker::new(items.len());
    let results = Vec::new();
    
    for (i, item) in items.iter().enumerate() {
        // Process item
        let result = process_item(item).await?;
        results.push(result);
        
        // Update progress
        tracker.update(ProgressUpdate {
            current: i + 1,
            total: items.len(),
            message: format!("Processing item {}", item.id),
            eta: tracker.calculate_eta(),
        });
        
        // Emit progress event
        tracker.emit_progress().await;
    }
    
    tracker.complete();
    Ok(results)
}
```

## Integration Examples

### With llmspell-core Components

```rust
use llmspell_core::{BaseAgent, ExecutionContext};
use llmspell_utils::{
    async_utils::timeout,
    security::PathValidator,
    rate_limiter::RateLimiter,
};

pub struct SecureFileAgent {
    path_validator: PathValidator,
    rate_limiter: RateLimiter,
}

#[async_trait]
impl BaseAgent for SecureFileAgent {
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Rate limit
        self.rate_limiter.acquire(1).await?;
        
        // Extract and validate path
        let path = input.parameters.get("path")
            .ok_or("Missing path parameter")?;
        let safe_path = self.path_validator.validate(path)?;
        
        // Execute with timeout
        let content = timeout(Duration::from_secs(10), async {
            read_file(safe_path).await
        }).await??;
        
        Ok(AgentOutput::text(content))
    }
}
```

### With Provider API Calls

```rust
use llmspell_utils::{
    api_key_manager::ApiKeyManager,
    circuit_breaker::CircuitBreaker,
    rate_limiter::RateLimiter,
};

pub struct ProviderClient {
    key_manager: ApiKeyManager,
    circuit_breaker: CircuitBreaker,
    rate_limiter: RateLimiter,
}

impl ProviderClient {
    pub async fn call_api(&self, request: ApiRequest) -> Result<Response> {
        // Get API key
        let api_key = self.key_manager.get_key("openai").await?;
        
        // Rate limit
        self.rate_limiter.acquire(request.estimated_tokens()).await?;
        
        // Call with circuit breaker
        self.circuit_breaker.call(async {
            make_api_call(api_key, request).await
        }).await
    }
}
```

## Configuration

```toml
[utils]
# Async utilities
[utils.async]
default_timeout_ms = 30000
max_concurrent_operations = 100

# Security settings
[utils.security]
enable_path_validation = true
jail_roots = ["/app/data", "/tmp/llmspell"]
blocked_url_patterns = ["*://localhost/*", "*://127.0.0.1/*"]
credential_patterns = [
    "sk-[a-zA-Z0-9]{48}",  # OpenAI
    "claude-[a-zA-Z0-9]+",  # Anthropic
]

# Rate limiting
[utils.rate_limiting]
global_rps = 100
per_provider_limits = {
    openai = 50,
    anthropic = 30,
}

# Circuit breaker
[utils.circuit_breaker]
failure_threshold = 5
success_threshold = 2
timeout_seconds = 60
half_open_max_requests = 3

# API key management
[utils.api_keys]
storage_path = "~/.llmspell/keys"
encryption_key_env = "LLMSPELL_KEY_ENCRYPTION"
rotation_days = 90
```

## Performance Considerations

- **Async Operations**: Use `concurrent_map` for batch operations but limit concurrency to avoid resource exhaustion
- **File Operations**: Prefer `write_file_atomic` for critical data to prevent corruption
- **Rate Limiting**: Token bucket refills continuously - acquire tokens early to smooth load
- **Circuit Breaker**: Tune thresholds based on service SLAs - too sensitive causes unnecessary opens
- **Path Validation**: Cache validated paths when possible - validation has syscall overhead
- **Memory Tracking**: Enable only in development/debugging - adds overhead to allocations
- **Progress Tracking**: Batch progress updates to reduce event emission overhead

## Security Considerations

- **Path Traversal**: Always use `PathValidator` for user-provided paths
- **SSRF Protection**: Validate all URLs with `SSRFProtection` before making requests
- **Credential Leakage**: Use `CredentialSanitizer` for all logging and error messages
- **API Key Storage**: Never store API keys in plaintext - use `ApiKeyManager`
- **Resource Limits**: Enforce memory and CPU limits to prevent DoS
- **Input Validation**: Use validator utilities for all external input
- **Temporary Files**: Use secure temp directory creation with proper permissions

## Migration Guide

### From v0.5.x to v0.6.x

Breaking changes:
- Async utilities now return `AsyncError` instead of generic error
- File operations are fully async - no blocking variants
- Security utilities require explicit configuration

Migration steps:
1. Update error handling to use `AsyncError`
2. Convert synchronous file operations to async
3. Create security validators with explicit config
4. Update rate limiter to use token bucket algorithm

### From v0.6.x to v0.8.x (Phase 8)

New features:
- Enhanced security with SSRF protection
- API key rotation policies
- Memory tracking for DoS protection
- Progress tracking framework

Migration steps:
1. Add SSRF protection to URL validation
2. Implement API key rotation schedules
3. Enable memory tracking in development
4. Add progress tracking to long operations