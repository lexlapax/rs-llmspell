//! Version command implementation
//!
//! Provides detailed version and build information as a subcommand
//! This is the single source of truth for all version information.

use anyhow::Result;
use clap::Parser;
use crate::cli::OutputFormat;
use std::fmt;

/// Show version information
#[derive(Debug, Parser)]
pub struct VersionCommand {
    /// Show verbose version information
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Show version of specific component
    #[arg(long, value_enum)]
    pub component: Option<Component>,

    /// Show short commit hash only
    #[arg(long)]
    pub short: bool,

    /// Show client version only (useful for scripts)
    #[arg(long)]
    pub client: bool,
}

/// Components that can have version information
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Component {
    /// CLI version
    Cli,
    /// Kernel version
    Kernel,
    /// Bridge/Language runtime versions
    Bridge,
    /// All components
    All,
}

/// Version information structure containing all build metadata
struct VersionInfo {
    version: String,
    git_commit: Option<String>,
    git_commit_short: Option<String>,
    git_branch: Option<String>,
    git_commit_date: Option<String>,
    git_dirty: bool,
    build_timestamp: String,
    build_profile: String,
    target_triple: String,
    host_triple: String,
    rustc_version: String,
    features: String,
}

impl VersionInfo {
    /// Create version info from compile-time environment variables
    fn from_env() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("LLMSPELL_GIT_HASH").map(String::from),
            git_commit_short: option_env!("LLMSPELL_GIT_SHORT_HASH").map(String::from),
            git_branch: option_env!("LLMSPELL_GIT_BRANCH").map(String::from),
            git_commit_date: option_env!("LLMSPELL_GIT_COMMIT_DATE").map(String::from),
            git_dirty: option_env!("LLMSPELL_GIT_DIRTY")
                .map(|s| s == "true")
                .unwrap_or(false),
            build_timestamp: option_env!("LLMSPELL_BUILD_TIMESTAMP")
                .unwrap_or("unknown")
                .to_string(),
            build_profile: option_env!("LLMSPELL_BUILD_PROFILE")
                .unwrap_or("unknown")
                .to_string(),
            target_triple: option_env!("LLMSPELL_TARGET")
                .unwrap_or("unknown")
                .to_string(),
            host_triple: option_env!("LLMSPELL_HOST")
                .unwrap_or("unknown")
                .to_string(),
            rustc_version: option_env!("LLMSPELL_RUSTC_VERSION")
                .unwrap_or("unknown")
                .to_string(),
            features: option_env!("LLMSPELL_FEATURES")
                .unwrap_or("default")
                .to_string(),
        }
    }

    /// Get short version string (for -V flag)
    fn short(&self) -> String {
        if let Some(ref commit_short) = self.git_commit_short {
            let dirty_suffix = if self.git_dirty { "-modified" } else { "" };
            format!(
                "llmspell {} ({}{} {})",
                self.version,
                commit_short,
                dirty_suffix,
                self.git_commit_date.as_ref().unwrap_or(&"unknown".to_string())
            )
        } else {
            format!("llmspell {}", self.version)
        }
    }

    /// Get verbose version string
    fn verbose(&self) -> String {
        let mut output = String::new();

        // First line matches short format
        output.push_str(&self.short());
        output.push('\n');

        // Binary name
        output.push_str("binary: llmspell\n");

        // Git information
        if let Some(ref commit) = self.git_commit {
            output.push_str(&format!("commit-hash: {}\n", commit));
        }

        if let Some(ref date) = self.git_commit_date {
            output.push_str(&format!("commit-date: {}\n", date));
        }

        if let Some(ref branch) = self.git_branch {
            output.push_str(&format!("branch: {}\n", branch));
        }

        if self.git_dirty {
            output.push_str("working-tree: modified\n");
        }

        // Build information
        output.push_str(&format!("build-timestamp: {}\n", self.build_timestamp));
        output.push_str(&format!("build-profile: {}\n", self.build_profile));

        // Platform information
        output.push_str(&format!("host: {}\n", self.host_triple));
        output.push_str(&format!("target: {}\n", self.target_triple));

        // Rust compiler information
        output.push_str(&format!("rustc: {}\n", self.rustc_version));

        // Feature flags
        output.push_str(&format!("features: {}\n", self.features));

        output
    }

    /// Get JSON representation of version info
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "git": {
                "commit": self.git_commit,
                "commit_short": self.git_commit_short,
                "branch": self.git_branch,
                "commit_date": self.git_commit_date,
                "dirty": self.git_dirty,
            },
            "build": {
                "timestamp": self.build_timestamp,
                "profile": self.build_profile,
                "host": self.host_triple,
                "target": self.target_triple,
                "rustc": self.rustc_version,
                "features": self.features.split(',').collect::<Vec<_>>(),
            }
        })
    }
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short())
    }
}

/// Execute version command
pub async fn execute(cmd: VersionCommand, format: OutputFormat) -> Result<()> {
    let info = VersionInfo::from_env();

    match format {
        OutputFormat::Json => {
            let json_output = if cmd.client {
                serde_json::json!({
                    "clientVersion": {
                        "version": info.version,
                        "gitCommit": info.git_commit_short,
                        "gitTreeState": if info.git_dirty { "dirty" } else { "clean" },
                        "buildDate": info.build_timestamp,
                        "platform": format!("{}/{}",
                            info.target_triple.split('-').next().unwrap_or("unknown"),
                            info.target_triple.split('-').nth(2).unwrap_or("unknown")
                        ),
                    }
                })
            } else {
                info.to_json()
            };
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            if cmd.short {
                // Just the version number
                println!("{}", info.version);
            } else if cmd.client {
                // Client version only (like kubectl)
                println!("Client Version: {}", info.short());
            } else if cmd.verbose {
                // Full verbose output
                print!("{}", info.verbose());

                // If requested, show component versions
                if let Some(Component::All) = cmd.component {
                    println!("\nComponents:");
                    print_component_versions().await;
                }
            } else {
                // Standard version output
                println!("{}", info.short());
            }
        }
    }

    Ok(())
}

/// Print versions of all components
async fn print_component_versions() {
    // CLI version (already shown above)

    // Kernel version - would need to query if connected
    println!("  kernel: {} (embedded)", env!("CARGO_PKG_VERSION"));

    // Bridge/runtime versions
    #[cfg(feature = "lua")]
    println!("  lua: enabled");

    #[cfg(feature = "javascript")]
    println!("  javascript: enabled");

    #[cfg(feature = "python")]
    println!("  python: enabled");

    // Feature flags
    let features = option_env!("LLMSPELL_FEATURES").unwrap_or("default");
    if features != "default" {
        println!("  features: {}", features);
    }
}

/// Simple version display for -V flag (called from main.rs)
pub fn show_version_simple() {
    let info = VersionInfo::from_env();
    println!("{}", info.short());
}