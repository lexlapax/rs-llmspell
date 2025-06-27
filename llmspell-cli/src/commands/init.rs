//! ABOUTME: Init command implementation for creating configuration files
//! ABOUTME: Generates default configuration with helpful comments

use crate::config;
use anyhow::Result;
use std::path::PathBuf;

/// Initialize configuration file
pub async fn init_config(output: PathBuf, force: bool) -> Result<()> {
    // Check if file already exists
    if output.exists() && !force {
        anyhow::bail!(
            "Configuration file already exists: {}. Use --force to overwrite.",
            output.display()
        );
    }
    
    // Create the configuration file
    config::create_default_config(&output).await?;
    
    println!("✓ Created configuration file: {}", output.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to configure your settings", output.display());
    println!("  2. Set API keys:");
    println!("     - OPENAI_API_KEY for OpenAI provider");
    println!("     - ANTHROPIC_API_KEY for Anthropic provider");
    println!("     - COHERE_API_KEY for Cohere provider");
    println!("  3. Run 'llmspell validate' to check your configuration");
    println!("  4. Run 'llmspell run <script>' to execute scripts");
    
    Ok(())
}