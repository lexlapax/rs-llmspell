//! ABOUTME: Interactive setup command for first-time users
//! ABOUTME: Guides users through API key configuration and initial setup

use anyhow::Result;
use llmspell_utils::terminal::{confirm, input_with_validation, select};
use std::path::PathBuf;

/// Run interactive setup for first-time users
pub async fn run_interactive_setup(force: bool) -> Result<()> {
    println!("🎉 Welcome to LLMSpell Setup!\n");
    println!("This wizard will help you get started with LLMSpell applications.\n");

    // Check for existing config
    let config_path = dirs::home_dir()
        .map(|h| h.join(".llmspell").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("llmspell.toml"));

    if config_path.exists() && !force {
        let overwrite = confirm("Configuration already exists. Overwrite?", false)?;

        if !overwrite {
            println!("Setup cancelled. Use --force to overwrite existing configuration.");
            return Ok(());
        }
    }

    // Step 1: Choose AI provider
    println!("\n📋 Step 1: Choose your AI provider\n");

    let providers = vec!["OpenAI (Recommended)", "Anthropic", "Both", "Skip for now"];
    let provider_choice = select("Which AI provider would you like to use?", &providers)?;

    let mut config = llmspell_config::LLMSpellConfig::default();

    // Step 2: Get API keys
    match provider_choice {
        0 | 2 => {
            // OpenAI
            println!("\n🔑 Step 2: OpenAI API Key\n");
            println!("Get your API key from: https://platform.openai.com/api-keys\n");

            let api_key = input_with_validation(
                "Enter your OpenAI API key: ",
                |input: &str| {
                    if input.starts_with("sk-") && input.len() > 20 {
                        Ok(())
                    } else {
                        Err("Invalid OpenAI API key format (should start with 'sk-')")
                    }
                },
            )?;

            // Save to environment variable
            std::env::set_var("OPENAI_API_KEY", &api_key);

            // Add to config
            config.providers.providers.insert(
                "openai".to_string(),
                llmspell_config::ProviderConfig {
                    name: "openai".to_string(),
                    provider_type: "openai".to_string(),
                    enabled: true,
                    base_url: Some("https://api.openai.com/v1".to_string()),
                    api_key_env: Some("OPENAI_API_KEY".to_string()),
                    api_key: None,
                    default_model: Some("gpt-4o-mini".to_string()),
                    max_tokens: None,
                    timeout_seconds: Some(30),
                    rate_limit: None,
                    retry: None,
                    max_retries: Some(3),
                    options: std::collections::HashMap::new(),
                },
            );
        }
        _ => {}
    }

    match provider_choice {
        1 | 2 => {
            // Anthropic
            println!("\n🔑 Step 2: Anthropic API Key\n");
            println!("Get your API key from: https://console.anthropic.com/settings/keys\n");

            let api_key = input_with_validation(
                "Enter your Anthropic API key: ",
                |input: &str| {
                    if input.starts_with("sk-ant-") && input.len() > 30 {
                        Ok(())
                    } else {
                        Err("Invalid Anthropic API key format (should start with 'sk-ant-')")
                    }
                },
            )?;

            // Save to environment variable
            std::env::set_var("ANTHROPIC_API_KEY", &api_key);

            // Add to config
            config.providers.providers.insert(
                "anthropic".to_string(),
                llmspell_config::ProviderConfig {
                    name: "anthropic".to_string(),
                    provider_type: "anthropic".to_string(),
                    enabled: true,
                    base_url: Some("https://api.anthropic.com".to_string()),
                    api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                    api_key: None,
                    default_model: Some("claude-3-haiku-20240307".to_string()),
                    max_tokens: None,
                    timeout_seconds: Some(30),
                    rate_limit: None,
                    retry: None,
                    max_retries: Some(3),
                    options: std::collections::HashMap::new(),
                },
            );
        }
        _ => {}
    }

    // Step 3: Choose first application
    println!("\n🚀 Step 3: Choose your first application\n");

    let apps = vec![
        "file-organizer - Organize messy files (Simple)",
        "research-collector - Research any topic (Simple)",
        "content-creator - Create content efficiently (Intermediate)",
        "Skip - I'll explore on my own",
    ];

    let app_choice = select("Which application would you like to try first?", &apps)?;

    // Step 4: Save configuration
    println!("\n💾 Step 4: Saving configuration...\n");

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save config
    let config_content = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, config_content)?;

    println!("✅ Configuration saved to: {}", config_path.display());

    // Step 5: Show next steps
    println!("\n🎉 Setup Complete!\n");
    println!("You're ready to use LLMSpell. Here's how to get started:\n");

    match app_choice {
        0 => {
            println!("Run File Organizer:");
            println!("  llmspell apps file-organizer\n");
            println!("Or with the traditional command:");
            println!("  llmspell run examples/script-users/applications/file-organizer/main.lua");
        }
        1 => {
            println!("Run Research Collector:");
            println!("  llmspell apps research-collector\n");
            println!("Or with the traditional command:");
            println!(
                "  llmspell run examples/script-users/applications/research-collector/main.lua"
            );
        }
        2 => {
            println!("Run Content Creator:");
            println!("  llmspell apps content-creator\n");
            println!("Or with the traditional command:");
            println!("  llmspell run examples/script-users/applications/content-creator/main.lua");
        }
        _ => {
            println!("List all available applications:");
            println!("  llmspell apps list\n");
            println!("Run any application:");
            println!("  llmspell apps <app-name>");
        }
    }

    println!("\n📚 For more help: llmspell --help");

    Ok(())
}
