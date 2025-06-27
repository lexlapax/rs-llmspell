//! ABOUTME: Info command implementation showing engine information
//! ABOUTME: Displays available engines and their capabilities

use crate::cli::{OutputFormat, ScriptEngine};
use anyhow::Result;
use serde_json::json;

/// Show engine information
pub async fn show_engine_info(
    current_engine: ScriptEngine,
    show_all: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let engines = if show_all {
        vec![
            ScriptEngine::Lua,
            ScriptEngine::Javascript,
            ScriptEngine::Python,
        ]
    } else {
        vec![ScriptEngine::Lua] // Only show available engines
    };

    match output_format {
        OutputFormat::Json => {
            let info = engines
                .iter()
                .map(|engine| {
                    json!({
                        "name": engine.as_str(),
                        "available": engine.is_available(),
                        "status": engine.availability_message(),
                        "current": *engine == current_engine,
                    })
                })
                .collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("LLMSpell Script Engines:");
            println!();
            for engine in engines {
                let marker = if engine == current_engine { "→" } else { " " };
                let status = if engine.is_available() { "✓" } else { "✗" };
                println!(
                    "{} {} {} - {}",
                    marker,
                    status,
                    engine.as_str(),
                    engine.availability_message()
                );
            }
            println!();
            println!("Current engine: {}", current_engine.as_str());
        }
    }

    Ok(())
}
