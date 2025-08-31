//! ABOUTME: Command-line interface library for rs-llmspell
//! ABOUTME: Provides CLI argument parsing and command handling functionality

pub mod cli;
pub mod commands;
pub mod config;
pub mod embedded_resources;
pub mod kernel;
pub mod output;
pub mod repl_interface;

#[cfg(test)]
pub mod test_helpers;

// Re-export commonly used types for testing
pub use cli::OutputFormat;
