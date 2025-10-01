//! Simple ANSI color support for terminal output
//!
//! Provides lightweight colored text without external dependencies.

/// ANSI color codes
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Red color
    Red,
    /// Green color
    Green,
    /// Yellow color
    Yellow,
    /// Blue color
    Blue,
    /// Cyan color
    Cyan,
    /// Magenta color
    Magenta,
    /// White color
    White,
    /// Black color
    Black,
}

impl Color {
    /// Get ANSI escape code for this color
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::Black => "\x1b[30m",
        }
    }
}

/// ANSI formatting codes
pub const RESET: &str = "\x1b[0m";
/// Bold text formatting
pub const BOLD: &str = "\x1b[1m";
/// Dim text formatting
pub const DIM: &str = "\x1b[2m";
/// Italic text formatting
pub const ITALIC: &str = "\x1b[3m";
/// Underline text formatting
pub const UNDERLINE: &str = "\x1b[4m";

/// Simple function to color text
#[must_use]
pub fn colored_text(text: &str, color: Color) -> String {
    format!("{}{text}{RESET}", color.code())
}

/// Extension trait for &str to add color methods
pub trait Colorize {
    /// Apply red color
    fn red(&self) -> String;
    /// Apply green color
    fn green(&self) -> String;
    /// Apply yellow color
    fn yellow(&self) -> String;
    /// Apply blue color
    fn blue(&self) -> String;
    /// Apply cyan color
    fn cyan(&self) -> String;
    /// Apply magenta color
    fn magenta(&self) -> String;
    /// Apply white color
    fn white(&self) -> String;
    /// Apply black color
    fn black(&self) -> String;
    /// Apply bold formatting
    fn bold(&self) -> String;
    /// Apply dim formatting
    fn dim(&self) -> String;
    /// Apply italic formatting
    fn italic(&self) -> String;
    /// Apply underline formatting
    fn underline(&self) -> String;
}

impl Colorize for &str {
    fn red(&self) -> String {
        colored_text(self, Color::Red)
    }

    fn green(&self) -> String {
        colored_text(self, Color::Green)
    }

    fn yellow(&self) -> String {
        colored_text(self, Color::Yellow)
    }

    fn blue(&self) -> String {
        colored_text(self, Color::Blue)
    }

    fn cyan(&self) -> String {
        colored_text(self, Color::Cyan)
    }

    fn magenta(&self) -> String {
        colored_text(self, Color::Magenta)
    }

    fn white(&self) -> String {
        colored_text(self, Color::White)
    }

    fn black(&self) -> String {
        colored_text(self, Color::Black)
    }

    fn bold(&self) -> String {
        format!("{BOLD}{self}{RESET}")
    }

    fn dim(&self) -> String {
        format!("{DIM}{self}{RESET}")
    }

    fn italic(&self) -> String {
        format!("{ITALIC}{self}{RESET}")
    }

    fn underline(&self) -> String {
        format!("{UNDERLINE}{self}{RESET}")
    }
}

impl Colorize for String {
    fn red(&self) -> String {
        self.as_str().red()
    }

    fn green(&self) -> String {
        self.as_str().green()
    }

    fn yellow(&self) -> String {
        self.as_str().yellow()
    }

    fn blue(&self) -> String {
        self.as_str().blue()
    }

    fn cyan(&self) -> String {
        self.as_str().cyan()
    }

    fn magenta(&self) -> String {
        self.as_str().magenta()
    }

    fn white(&self) -> String {
        self.as_str().white()
    }

    fn black(&self) -> String {
        self.as_str().black()
    }

    fn bold(&self) -> String {
        self.as_str().bold()
    }

    fn dim(&self) -> String {
        self.as_str().dim()
    }

    fn italic(&self) -> String {
        self.as_str().italic()
    }

    fn underline(&self) -> String {
        self.as_str().underline()
    }
}

/// Check if terminal supports colors (basic check)
#[must_use]
pub fn supports_color() -> bool {
    // Check NO_COLOR env var (standard)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check TERM env var
    if let Ok(term) = std::env::var("TERM") {
        return term != "dumb";
    }

    // Default to true on most systems
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_text() {
        let red_text = colored_text("Error", Color::Red);
        assert_eq!(red_text, "\x1b[31mError\x1b[0m");
    }

    #[test]
    fn test_colorize_trait() {
        let text = "Hello";
        assert_eq!(text.red(), "\x1b[31mHello\x1b[0m");
        assert_eq!(text.bold(), "\x1b[1mHello\x1b[0m");
    }
}
