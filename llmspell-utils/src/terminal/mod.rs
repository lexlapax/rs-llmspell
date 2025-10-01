//! Terminal utilities for CLI output formatting
//!
//! Provides lightweight replacements for heavy CLI dependencies like tabled, colored, etc.
//! These utilities are designed to be simple, efficient, and dependency-free.

pub mod color;
/// Progress tracking and reporting utilities
pub mod progress;
pub mod prompt;
pub mod table;

// Re-export commonly used items
pub use color::{colored_text, Color, Colorize};
pub use progress::{ProgressEvent, ProgressReporter, ProgressTracker};
pub use prompt::{confirm, input, input_with_validation, select, AsyncSpinner, SimpleSpinner};
pub use table::{quick_table, SimpleTable, TableStyle};
