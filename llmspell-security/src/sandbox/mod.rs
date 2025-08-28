//! ABOUTME: Security sandbox for safe tool execution
//! ABOUTME: Provides file system, network, and resource monitoring controls

pub mod file_sandbox;
pub mod network_sandbox;
pub mod resource_monitor;

pub use file_sandbox::FileSandbox;
pub use network_sandbox::NetworkSandbox;
pub use resource_monitor::ResourceMonitor;

use llmspell_core::{
    traits::tool::{ResourceLimits, SecurityRequirements},
    Result,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Security sandbox execution context that defines the boundaries for safe code execution.
///
/// `SandboxContext` encapsulates all the security constraints and permissions for
/// a sandboxed execution environment. It defines what file paths, network domains,
/// and environment variables a sandboxed process is allowed to access, along with
/// resource limits to prevent abuse.
///
/// # Examples
///
/// ```rust
/// use llmspell_security::SandboxContext;
/// use llmspell_core::traits::tool::{SecurityRequirements, ResourceLimits};
///
/// // Create a restricted sandbox for untrusted code
/// let security_reqs = SecurityRequirements::safe()
///     .with_file_access("/tmp")  // Only allow /tmp access
///     .with_network_access("api.example.com")  // Only allow specific API
///     .with_env_access("HOME");  // Only allow HOME environment variable
///
/// let resource_limits = ResourceLimits::strict(); // CPU/memory limits
///
/// let context = SandboxContext::new(
///     "untrusted-script-123".to_string(),
///     security_reqs,
///     resource_limits
/// );
///
/// // Check permissions before allowing operations
/// assert!(context.is_path_allowed(&std::path::Path::new("/tmp/output.txt")));
/// assert!(!context.is_path_allowed(&std::path::Path::new("/etc/passwd")));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxContext {
    /// Unique identifier for this sandbox instance.
    ///
    /// Used for logging, debugging, and tracking sandbox sessions across
    /// the system. Should be unique within the application instance.
    pub id: String,
    
    /// The security requirements that define what operations are allowed.
    ///
    /// This contains the high-level security policy that was used to create
    /// this sandbox context. Preserved for auditing and policy validation.
    pub security_requirements: SecurityRequirements,
    
    /// Resource limits to prevent resource exhaustion attacks.
    ///
    /// Defines CPU time, memory usage, disk space, and network bandwidth
    /// limits to prevent sandboxed code from consuming excessive resources.
    pub resource_limits: ResourceLimits,
    
    /// The working directory for sandboxed operations.
    ///
    /// All relative path operations will be resolved relative to this directory.
    /// Defaults to current directory or /tmp if current directory is inaccessible.
    pub working_directory: String,
    
    /// List of file paths that sandboxed code is allowed to access.
    ///
    /// Can include exact paths or path prefixes. Use "*" for unrestricted
    /// file access (only for trusted code). Paths are checked using prefix
    /// matching, so "/tmp" allows access to "/tmp/anything".
    pub allowed_paths: Vec<String>,
    
    /// List of network domains that sandboxed code can connect to.
    ///
    /// Can include exact domain names or domain suffixes (starting with ".").
    /// Use "*" for unrestricted network access (only for trusted code).
    /// Domain matching supports both exact matches and suffix matches.
    pub allowed_domains: Vec<String>,
    
    /// List of environment variables that sandboxed code can access.
    ///
    /// Only the specified environment variables will be visible to the
    /// sandboxed process. Use "*" for unrestricted environment access
    /// (only for trusted code).
    pub allowed_env_vars: Vec<String>,
}

impl SandboxContext {
    /// Create a new sandbox context
    pub fn new(
        id: String,
        security_requirements: SecurityRequirements,
        resource_limits: ResourceLimits,
    ) -> Self {
        Self {
            id,
            security_requirements: security_requirements.clone(),
            resource_limits,
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
                .to_string_lossy()
                .to_string(),
            allowed_paths: security_requirements.file_permissions,
            allowed_domains: security_requirements.network_permissions,
            allowed_env_vars: security_requirements.env_permissions,
        }
    }

    /// Check if a file path is allowed
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check wildcard permissions
        if self.allowed_paths.contains(&"*".to_string()) {
            return true;
        }

        // Check exact matches and prefix matches
        for allowed in &self.allowed_paths {
            if allowed == "*" || path_str == *allowed || path_str.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    /// Check if a domain is allowed
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        // Check wildcard permissions
        if self.allowed_domains.contains(&"*".to_string()) {
            return true;
        }

        // Check exact matches and suffix matches
        for allowed in &self.allowed_domains {
            if allowed == "*" || domain == *allowed || domain.ends_with(allowed) {
                return true;
            }
        }

        false
    }

    /// Check if an environment variable is allowed
    pub fn is_env_var_allowed(&self, var: &str) -> bool {
        // Check wildcard permissions
        if self.allowed_env_vars.contains(&"*".to_string()) {
            return true;
        }

        // Check exact matches
        self.allowed_env_vars.contains(&var.to_string())
    }
}

/// A comprehensive sandbox that integrates file, network, and resource controls.
///
/// `IntegratedSandbox` provides a unified interface for all sandbox security
/// controls, combining file system access control, network access control,
/// and resource monitoring into a single, easy-to-use interface.
///
/// # Examples
///
/// ```rust
/// use llmspell_security::{SandboxContext, IntegratedSandbox};
/// use llmspell_core::traits::tool::{SecurityRequirements, ResourceLimits};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create a sandbox context
/// let security_reqs = SecurityRequirements::safe()
///     .with_file_access("/tmp")
///     .with_network_access("api.example.com");
/// let resource_limits = ResourceLimits::strict();
/// let context = SandboxContext::new("example".to_string(), security_reqs, resource_limits);
///
/// // Create the integrated sandbox
/// let mut sandbox = IntegratedSandbox::new(context)?;
///
/// // Start monitoring for violations
/// sandbox.start_monitoring().await?;
///
/// // ... run untrusted code ...
///
/// // Check for security violations
/// if sandbox.has_violations().await {
///     let violations = sandbox.get_violations().await;
///     println!("Security violations detected: {:?}", violations);
/// }
///
/// // Stop monitoring
/// sandbox.stop_monitoring().await?;
/// # Ok(())
/// # }
/// ```
pub struct IntegratedSandbox {
    /// The sandbox context containing security policies and limits
    context: SandboxContext,
    /// File system access control component
    file_sandbox: FileSandbox,
    /// Network access control component
    network_sandbox: NetworkSandbox,
    /// Resource usage monitoring component
    resource_monitor: ResourceMonitor,
}

impl IntegratedSandbox {
    /// Create a new integrated sandbox
    pub fn new(context: SandboxContext) -> Result<Self> {
        let file_sandbox = FileSandbox::new(context.clone())?;
        let network_sandbox = NetworkSandbox::new(context.clone())?;
        let resource_monitor = ResourceMonitor::new(context.clone())?;

        Ok(Self {
            context,
            file_sandbox,
            network_sandbox,
            resource_monitor,
        })
    }

    /// Get sandbox context
    pub fn context(&self) -> &SandboxContext {
        &self.context
    }

    /// Get file sandbox
    pub fn file_sandbox(&self) -> &FileSandbox {
        &self.file_sandbox
    }

    /// Get network sandbox
    pub fn network_sandbox(&self) -> &NetworkSandbox {
        &self.network_sandbox
    }

    /// Get resource monitor
    pub fn resource_monitor(&self) -> &ResourceMonitor {
        &self.resource_monitor
    }

    /// Start monitoring resources
    pub async fn start_monitoring(&mut self) -> Result<()> {
        self.resource_monitor.start().await
    }

    /// Stop monitoring resources
    pub async fn stop_monitoring(&mut self) -> Result<()> {
        self.resource_monitor.stop().await
    }

    /// Check if the sandbox has any violations
    pub async fn has_violations(&self) -> bool {
        self.resource_monitor.has_violations().await
    }

    /// Get violation summary
    pub async fn get_violations(&self) -> Vec<String> {
        self.resource_monitor.get_violations().await
    }
}

/// Sandbox violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxViolation {
    /// File access violation
    FileAccess {
        path: String,
        operation: String,
        reason: String,
    },
    /// Network access violation
    NetworkAccess {
        domain: String,
        operation: String,
        reason: String,
    },
    /// Resource limit violation
    ResourceLimit {
        resource: String,
        limit: u64,
        actual: u64,
        reason: String,
    },
    /// Environment access violation
    EnvironmentAccess { variable: String, reason: String },
}

impl std::fmt::Display for SandboxViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxViolation::FileAccess {
                path,
                operation,
                reason,
            } => {
                write!(
                    f,
                    "File access violation: {} on '{}' - {}",
                    operation, path, reason
                )
            }
            SandboxViolation::NetworkAccess {
                domain,
                operation,
                reason,
            } => {
                write!(
                    f,
                    "Network access violation: {} to '{}' - {}",
                    operation, domain, reason
                )
            }
            SandboxViolation::ResourceLimit {
                resource,
                limit,
                actual,
                reason,
            } => {
                write!(
                    f,
                    "Resource limit violation: {} exceeded limit {} with {} - {}",
                    resource, limit, actual, reason
                )
            }
            SandboxViolation::EnvironmentAccess { variable, reason } => {
                write!(
                    f,
                    "Environment access violation: '{}' - {}",
                    variable, reason
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    #[test]
    fn test_sandbox_context_creation() {
        let security_reqs = SecurityRequirements::safe()
            .with_file_access("/tmp")
            .with_network_access("api.example.com")
            .with_env_access("HOME");

        let resource_limits = ResourceLimits::strict();

        let context =
            SandboxContext::new("test-sandbox".to_string(), security_reqs, resource_limits);

        assert_eq!(context.id, "test-sandbox");
        assert!(context.allowed_paths.contains(&"/tmp".to_string()));
        assert!(context
            .allowed_domains
            .contains(&"api.example.com".to_string()));
        assert!(context.allowed_env_vars.contains(&"HOME".to_string()));
    }
    #[test]
    fn test_path_permissions() {
        let security_reqs = SecurityRequirements::safe()
            .with_file_access("/tmp")
            .with_file_access("/var/log");

        let context =
            SandboxContext::new("test".to_string(), security_reqs, ResourceLimits::strict());

        assert!(context.is_path_allowed(Path::new("/tmp/test.txt")));
        assert!(context.is_path_allowed(Path::new("/var/log/app.log")));
        assert!(!context.is_path_allowed(Path::new("/etc/passwd")));
    }
    #[test]
    fn test_domain_permissions() {
        let security_reqs = SecurityRequirements::safe()
            .with_network_access("api.example.com")
            .with_network_access(".github.com");

        let context =
            SandboxContext::new("test".to_string(), security_reqs, ResourceLimits::strict());

        assert!(context.is_domain_allowed("api.example.com"));
        assert!(context.is_domain_allowed("api.github.com"));
        assert!(!context.is_domain_allowed("malicious.com"));
    }
    #[test]
    fn test_env_var_permissions() {
        let security_reqs = SecurityRequirements::safe()
            .with_env_access("HOME")
            .with_env_access("PATH");

        let context =
            SandboxContext::new("test".to_string(), security_reqs, ResourceLimits::strict());

        assert!(context.is_env_var_allowed("HOME"));
        assert!(context.is_env_var_allowed("PATH"));
        assert!(!context.is_env_var_allowed("SECRET_KEY"));
    }
    #[test]
    fn test_wildcard_permissions() {
        let security_reqs = SecurityRequirements::privileged();

        let context = SandboxContext::new(
            "test".to_string(),
            security_reqs,
            ResourceLimits::unlimited(),
        );

        assert!(context.is_path_allowed(Path::new("/any/path")));
        assert!(context.is_domain_allowed("any.domain.com"));
        assert!(context.is_env_var_allowed("ANY_VAR"));
    }
}
