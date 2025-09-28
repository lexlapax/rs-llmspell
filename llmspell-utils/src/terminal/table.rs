//! Simple table formatting for terminal output
//!
//! Provides lightweight table formatting without external dependencies.

use std::fmt::{self, Write};

/// Simple table builder for terminal output
pub struct SimpleTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
    style: TableStyle,
}

/// Table border styles
#[derive(Debug, Clone, Copy)]
pub enum TableStyle {
    /// No borders
    None,
    /// Simple ASCII borders using - and |
    Ascii,
    /// Rounded Unicode borders
    Rounded,
    /// Double-line Unicode borders
    Double,
}

impl SimpleTable {
    /// Create a new table with headers
    #[must_use]
    pub fn new(headers: Vec<String>) -> Self {
        let column_widths = headers.iter().map(String::len).collect();
        Self {
            headers,
            rows: Vec::new(),
            column_widths,
            style: TableStyle::Ascii,
        }
    }

    /// Set the table style
    #[must_use]
    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }

    /// Build the table string
    fn build_string(&self) -> String {
        let mut result = String::new();

        match self.style {
            TableStyle::None => self.format_no_borders(&mut result),
            TableStyle::Ascii => self.format_ascii_borders(&mut result),
            TableStyle::Rounded => self.format_rounded_borders(&mut result),
            TableStyle::Double => self.format_double_borders(&mut result),
        }

        result
    }

    fn format_no_borders(&self, result: &mut String) {
        // Headers
        for (i, header) in self.headers.iter().enumerate() {
            write!(result, "{:width$} ", header, width = self.column_widths[i]).unwrap();
        }
        result.push('\n');

        // Rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < self.column_widths.len() {
                    write!(result, "{:width$} ", cell, width = self.column_widths[i]).unwrap();
                }
            }
            result.push('\n');
        }
    }

    fn format_ascii_borders(&self, result: &mut String) {
        // Top border
        result.push('+');
        for width in &self.column_widths {
            result.push_str(&"-".repeat(width + 2));
            result.push('+');
        }
        result.push('\n');

        // Headers
        result.push('|');
        for (i, header) in self.headers.iter().enumerate() {
            write!(
                result,
                " {:width$} |",
                header,
                width = self.column_widths[i]
            )
            .unwrap();
        }
        result.push('\n');

        // Header separator
        result.push('+');
        for width in &self.column_widths {
            result.push_str(&"-".repeat(width + 2));
            result.push('+');
        }
        result.push('\n');

        // Rows
        for row in &self.rows {
            result.push('|');
            for (i, cell) in row.iter().enumerate() {
                if i < self.column_widths.len() {
                    write!(result, " {:width$} |", cell, width = self.column_widths[i]).unwrap();
                }
            }
            result.push('\n');
        }

        // Bottom border
        result.push('+');
        for width in &self.column_widths {
            result.push_str(&"-".repeat(width + 2));
            result.push('+');
        }
        result.push('\n');
    }

    fn format_rounded_borders(&self, result: &mut String) {
        // Top border
        result.push('╭');
        for (i, width) in self.column_widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < self.column_widths.len() - 1 {
                result.push('┬');
            }
        }
        result.push('╮');
        result.push('\n');

        // Headers
        result.push('│');
        for (i, header) in self.headers.iter().enumerate() {
            write!(
                result,
                " {:width$} │",
                header,
                width = self.column_widths[i]
            )
            .unwrap();
        }
        result.push('\n');

        // Header separator
        result.push('├');
        for (i, width) in self.column_widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < self.column_widths.len() - 1 {
                result.push('┼');
            }
        }
        result.push('┤');
        result.push('\n');

        // Rows
        for row in &self.rows {
            result.push('│');
            for (i, cell) in row.iter().enumerate() {
                if i < self.column_widths.len() {
                    write!(result, " {:width$} │", cell, width = self.column_widths[i]).unwrap();
                }
            }
            result.push('\n');
        }

        // Bottom border
        result.push('╰');
        for (i, width) in self.column_widths.iter().enumerate() {
            result.push_str(&"─".repeat(width + 2));
            if i < self.column_widths.len() - 1 {
                result.push('┴');
            }
        }
        result.push('╯');
        result.push('\n');
    }

    fn format_double_borders(&self, result: &mut String) {
        // Top border
        result.push('╔');
        for (i, width) in self.column_widths.iter().enumerate() {
            result.push_str(&"═".repeat(width + 2));
            if i < self.column_widths.len() - 1 {
                result.push('╦');
            }
        }
        result.push('╗');
        result.push('\n');

        // Headers
        result.push('║');
        for (i, header) in self.headers.iter().enumerate() {
            write!(
                result,
                " {:width$} ║",
                header,
                width = self.column_widths[i]
            )
            .unwrap();
        }
        result.push('\n');

        // Header separator
        result.push('╠');
        for (i, width) in self.column_widths.iter().enumerate() {
            result.push_str(&"═".repeat(width + 2));
            if i < self.column_widths.len() - 1 {
                result.push('╬');
            }
        }
        result.push('╣');
        result.push('\n');

        // Rows
        for row in &self.rows {
            result.push('║');
            for (i, cell) in row.iter().enumerate() {
                if i < self.column_widths.len() {
                    write!(result, " {:width$} ║", cell, width = self.column_widths[i]).unwrap();
                }
            }
            result.push('\n');
        }

        // Bottom border
        result.push('╚');
        for (i, width) in self.column_widths.iter().enumerate() {
            result.push_str(&"═".repeat(width + 2));
            if i < self.column_widths.len() - 1 {
                result.push('╩');
            }
        }
        result.push('╝');
        result.push('\n');
    }
}

impl fmt::Display for SimpleTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_string())
    }
}

/// Quick helper to create a simple table
pub fn quick_table(headers: Vec<&str>, rows: Vec<Vec<&str>>) -> String {
    let mut table = SimpleTable::new(headers.into_iter().map(String::from).collect());
    for row in rows {
        table.add_row(row.into_iter().map(String::from).collect());
    }
    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_table() {
        let mut table = SimpleTable::new(vec!["Name".into(), "Age".into()]);
        table.add_row(vec!["Alice".into(), "30".into()]);
        table.add_row(vec!["Bob".into(), "25".into()]);

        let output = table.to_string();
        assert!(output.contains("Name"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_quick_table() {
        let table = quick_table(
            vec!["ID", "Status"],
            vec![vec!["1", "OK"], vec!["2", "Error"]],
        );
        assert!(table.contains("ID"));
        assert!(table.contains("Error"));
    }
}
