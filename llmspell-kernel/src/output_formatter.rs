//! Output formatting for CLI clients
//!
//! This module provides a clean abstraction for handling output in CLI contexts,
//! replacing direct println!/print! calls with a structured approach that can
//! be easily tested and redirected as needed.

use llmspell_core::io::{IOContext, IOStream};
use std::io::{self, Write};
use std::sync::Arc;

/// Stream type for output routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputChannel {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}

/// Output formatter for CLI applications
///
/// Provides structured output handling that can be redirected to IOContext
/// or standard streams depending on the execution context.
pub struct OutputFormatter {
    io_context: Option<Arc<IOContext>>,
    use_color: bool,
}

impl OutputFormatter {
    /// Create a new formatter with direct stdout/stderr output
    #[must_use]
    pub fn new() -> Self {
        Self {
            io_context: None,
            use_color: Self::should_use_color(),
        }
    }

    /// Check if we should use color output
    fn should_use_color() -> bool {
        // Simple check - use color if stdout is a terminal
        // In the future, could use a proper crate like atty or supports-color
        std::env::var("NO_COLOR").is_err() && !cfg!(windows)
    }

    /// Create a new formatter that routes through IOContext
    #[must_use]
    pub fn with_io_context(io_context: Arc<IOContext>) -> Self {
        Self {
            io_context: Some(io_context),
            use_color: false, // Disable color when routing through IOContext
        }
    }

    /// Write output to the specified channel
    pub fn write(&self, channel: OutputChannel, text: &str) -> io::Result<()> {
        if let Some(ref io_context) = self.io_context {
            // Route through IOContext
            match channel {
                OutputChannel::Stdout => io_context
                    .stdout
                    .write(text)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
                OutputChannel::Stderr => io_context
                    .stderr
                    .write(text)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
            }
        } else {
            // Direct output to standard streams
            match channel {
                OutputChannel::Stdout => {
                    print!("{}", text);
                    io::stdout().flush()
                }
                OutputChannel::Stderr => {
                    eprint!("{}", text);
                    io::stderr().flush()
                }
            }
        }
    }

    /// Write a line to the specified channel
    pub fn write_line(&self, channel: OutputChannel, line: &str) -> io::Result<()> {
        if let Some(ref io_context) = self.io_context {
            // Route through IOContext
            match channel {
                OutputChannel::Stdout => io_context
                    .stdout
                    .write_line(line)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
                OutputChannel::Stderr => io_context
                    .stderr
                    .write_line(line)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
            }
        } else {
            // Direct output to standard streams
            match channel {
                OutputChannel::Stdout => writeln!(io::stdout(), "{}", line),
                OutputChannel::Stderr => writeln!(io::stderr(), "{}", line),
            }
        }
    }

    /// Write an error message
    pub fn error(&self, message: &str) -> io::Result<()> {
        if self.use_color {
            self.write_line(
                OutputChannel::Stderr,
                &format!("\x1b[31m{}\x1b[0m", message),
            )
        } else {
            self.write_line(OutputChannel::Stderr, message)
        }
    }

    /// Write a warning message
    pub fn warning(&self, message: &str) -> io::Result<()> {
        if self.use_color {
            self.write_line(
                OutputChannel::Stderr,
                &format!("\x1b[33m{}\x1b[0m", message),
            )
        } else {
            self.write_line(OutputChannel::Stderr, message)
        }
    }

    /// Write an info message
    pub fn info(&self, message: &str) -> io::Result<()> {
        self.write_line(OutputChannel::Stdout, message)
    }

    /// Write a success message
    pub fn success(&self, message: &str) -> io::Result<()> {
        if self.use_color {
            self.write_line(
                OutputChannel::Stdout,
                &format!("\x1b[32m{}\x1b[0m", message),
            )
        } else {
            self.write_line(OutputChannel::Stdout, message)
        }
    }

    /// Flush all output streams
    pub fn flush(&self) -> io::Result<()> {
        if let Some(ref io_context) = self.io_context {
            io_context
                .stdout
                .flush()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            io_context
                .stderr
                .flush()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        } else {
            io::stdout().flush()?;
            io::stderr().flush()?;
        }
        Ok(())
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard output implementation for direct CLI usage
pub struct StdoutIOStream;

impl IOStream for StdoutIOStream {
    fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        print!("{}", data);
        io::stdout()
            .flush()
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "stdout write".to_string(),
                source: e,
            })
    }

    fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        println!("{}", line);
        Ok(())
    }

    fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
        io::stdout()
            .flush()
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "stdout flush".to_string(),
                source: e,
            })
    }
}

/// Standard error implementation for direct CLI usage
pub struct StderrIOStream;

impl IOStream for StderrIOStream {
    fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        eprint!("{}", data);
        io::stderr()
            .flush()
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "stderr write".to_string(),
                source: e,
            })
    }

    fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        eprintln!("{}", line);
        Ok(())
    }

    fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
        io::stderr()
            .flush()
            .map_err(|e| llmspell_core::error::LLMSpellError::Io {
                operation: "stderr flush".to_string(),
                source: e,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_formatter_creation() {
        let formatter = OutputFormatter::new();
        assert!(formatter.io_context.is_none());

        let formatter_default = OutputFormatter::default();
        assert!(formatter_default.io_context.is_none());
    }

    #[test]
    fn test_channel_equality() {
        assert_eq!(OutputChannel::Stdout, OutputChannel::Stdout);
        assert_ne!(OutputChannel::Stdout, OutputChannel::Stderr);
    }
}
