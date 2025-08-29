//! ABOUTME: Embedded resources for single binary distribution
//! ABOUTME: Contains all example applications' Lua scripts and configs

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Embedded application data
#[derive(Debug, Clone)]
pub struct EmbeddedApp {
    pub name: &'static str,
    pub description: &'static str,
    pub lua_script: &'static str,
    pub config: &'static str,
    pub complexity: &'static str,
    pub agents: usize,
}

/// Registry of all embedded applications
pub static EMBEDDED_APPS: Lazy<HashMap<&'static str, EmbeddedApp>> = Lazy::new(|| {
    let mut apps = HashMap::new();

    // File Organizer - Universal Layer (3 agents)
    apps.insert(
        "file-organizer",
        EmbeddedApp {
            name: "file-organizer",
            description: "Organize messy files with AI-powered categorization",
            lua_script: include_str!("../resources/applications/file-organizer/main.lua"),
            config: include_str!("../resources/applications/file-organizer/config.toml"),
            complexity: "Universal",
            agents: 3,
        },
    );

    // Research Collector - Universal Layer (2 agents)
    apps.insert(
        "research-collector",
        EmbeddedApp {
            name: "research-collector",
            description: "Research any topic thoroughly with AI synthesis",
            lua_script: include_str!("../resources/applications/research-collector/main.lua"),
            config: include_str!("../resources/applications/research-collector/config.toml"),
            complexity: "Universal",
            agents: 2,
        },
    );

    // Content Creator - Power User Layer (4 agents)
    apps.insert(
        "content-creator",
        EmbeddedApp {
            name: "content-creator",
            description: "Create content efficiently with AI assistance",
            lua_script: include_str!("../resources/applications/content-creator/main.lua"),
            config: include_str!("../resources/applications/content-creator/config.toml"),
            complexity: "Power User",
            agents: 4,
        },
    );

    // Communication Manager - Business Layer (5 agents)
    apps.insert(
        "communication-manager",
        EmbeddedApp {
            name: "communication-manager",
            description: "Manage business communications with AI automation",
            lua_script: include_str!("../resources/applications/communication-manager/main.lua"),
            config: include_str!("../resources/applications/communication-manager/config.toml"),
            complexity: "Business",
            agents: 5,
        },
    );

    // Process Orchestrator - Professional Layer (8 agents)
    apps.insert(
        "process-orchestrator",
        EmbeddedApp {
            name: "process-orchestrator",
            description: "Orchestrate complex processes with AI coordination",
            lua_script: include_str!("../resources/applications/process-orchestrator/main.lua"),
            config: include_str!("../resources/applications/process-orchestrator/config.toml"),
            complexity: "Professional",
            agents: 8,
        },
    );

    // Code Review Assistant - Professional Layer (7 agents)
    apps.insert(
        "code-review-assistant",
        EmbeddedApp {
            name: "code-review-assistant",
            description: "Review code for quality, security, and best practices",
            lua_script: include_str!("../resources/applications/code-review-assistant/main.lua"),
            config: include_str!("../resources/applications/code-review-assistant/config.toml"),
            complexity: "Professional",
            agents: 7,
        },
    );

    // Webapp Creator - Expert Layer (20 agents)
    apps.insert(
        "webapp-creator",
        EmbeddedApp {
            name: "webapp-creator",
            description: "Create complete web applications with AI",
            lua_script: include_str!("../resources/applications/webapp-creator/main.lua"),
            config: include_str!("../resources/applications/webapp-creator/config.toml"),
            complexity: "Expert",
            agents: 20,
        },
    );

    // Knowledge Base - Universal Layer (3 agents)
    apps.insert(
        "knowledge-base",
        EmbeddedApp {
            name: "knowledge-base",
            description: "Personal knowledge management with semantic search",
            lua_script: include_str!("../resources/applications/knowledge-base/main.lua"),
            config: include_str!("../resources/applications/knowledge-base/config.toml"),
            complexity: "Universal",
            agents: 3,
        },
    );

    // Personal Assistant - Power User Layer (4 agents)
    apps.insert(
        "personal-assistant",
        EmbeddedApp {
            name: "personal-assistant",
            description: "AI-powered personal productivity assistant",
            lua_script: include_str!("../resources/applications/personal-assistant/main.lua"),
            config: include_str!("../resources/applications/personal-assistant/config.toml"),
            complexity: "Power User",
            agents: 4,
        },
    );

    apps
});

/// Get an embedded application by name
pub fn get_app(name: &str) -> Option<&'static EmbeddedApp> {
    EMBEDDED_APPS.get(name)
}

/// List all available applications
pub fn list_apps() -> Vec<&'static EmbeddedApp> {
    let mut apps: Vec<_> = EMBEDDED_APPS.values().collect();
    // Sort by complexity (Universal -> Expert)
    apps.sort_by_key(|app| match app.complexity {
        "Universal" => 1,
        "Power User" => 2,
        "Business" => 3,
        "Professional" => 4,
        "Expert" => 5,
        _ => 9,
    });
    apps
}

/// Extract an application to a temporary directory
pub fn extract_app(app_name: &str) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let app =
        get_app(app_name).ok_or_else(|| anyhow::anyhow!("Application '{}' not found", app_name))?;

    // Create temp directory for this execution
    let temp_dir =
        std::env::temp_dir().join(format!("llmspell-{}-{}", app_name, uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;

    // Write Lua script
    let lua_path = temp_dir.join("main.lua");
    std::fs::write(&lua_path, app.lua_script)?;

    // Write config
    let config_path = temp_dir.join("config.toml");
    std::fs::write(&config_path, app.config)?;

    Ok((lua_path, config_path))
}

/// Clean up extracted temporary files
pub fn cleanup_temp_dir(path: &std::path::Path) -> anyhow::Result<()> {
    if path.exists() && path.parent() == Some(std::env::temp_dir().as_path()) {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
