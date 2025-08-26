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

/// Sandbox execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxContext {
    /// Unique sandbox ID
    pub id: String,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Working directory
    pub working_directory: String,
    /// Allowed file paths
    pub allowed_paths: Vec<String>,
    /// Allowed network domains
    pub allowed_domains: Vec<String>,
    /// Environment variables allowed
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

/// Integrated sandbox that combines file, network, and resource controls
pub struct IntegratedSandbox {
    context: SandboxContext,
    file_sandbox: FileSandbox,
    network_sandbox: NetworkSandbox,
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
