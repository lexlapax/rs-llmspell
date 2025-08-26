//! Debug output handlers for various destinations
//!
//! Provides trait and implementations for routing debug output
//! to stdout, files, buffers, and other destinations.

use crate::debug::entry::DebugEntry;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

/// Trait for debug output handlers
pub trait DebugOutput: Send + Sync {
    /// Write a debug entry to this output
    fn write(&self, entry: &DebugEntry);

    /// Flush any buffered output
    fn flush(&self);

    /// Clear the output (if applicable)
    fn clear(&self) {}

    /// Get a description of this output handler
    fn description(&self) -> &'static str;
}

/// Output handler that writes to stdout
pub struct StdoutOutput {
    colored: bool,
}

impl StdoutOutput {
    /// Create a new stdout output handler
    #[must_use]
    pub fn new(colored: bool) -> Self {
        Self { colored }
    }
}

impl DebugOutput for StdoutOutput {
    fn write(&self, entry: &DebugEntry) {
        println!("{}", entry.format_text(self.colored));
    }

    fn flush(&self) {
        let _ = io::stdout().flush();
    }

    fn description(&self) -> &'static str {
        "stdout"
    }
}

/// Output handler that writes to a file
pub struct FileOutput {
    file: Arc<RwLock<File>>,
    #[allow(dead_code)]
    path: PathBuf,
    format: FileFormat,
}

/// Format for file output
#[derive(Debug, Clone, Copy)]
pub enum FileFormat {
    /// Plain text format
    Text,
    /// JSON format
    Json,
    /// Pretty-printed JSON format
    JsonPretty,
}

impl FileOutput {
    /// Create a new file output handler
    ///
    /// # Errors
    /// Returns error if file cannot be created or opened
    pub fn new(path: PathBuf, format: FileFormat, append: bool) -> io::Result<Self> {
        let file = if append {
            OpenOptions::new().create(true).append(true).open(&path)?
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)?
        };

        Ok(Self {
            file: Arc::new(RwLock::new(file)),
            path,
            format,
        })
    }
}

impl DebugOutput for FileOutput {
    fn write(&self, entry: &DebugEntry) {
        let formatted = match self.format {
            FileFormat::Text => entry.format_text(false),
            FileFormat::Json => entry.format_json(),
            FileFormat::JsonPretty => entry.format_json_pretty(),
        };

        let mut file = self.file.write();
        let _ = writeln!(file, "{formatted}");
    }

    fn flush(&self) {
        let mut file = self.file.write();
        let _ = file.flush();
    }

    fn description(&self) -> &'static str {
        "file"
    }
}

/// Output handler that buffers entries in memory
pub struct BufferOutput {
    buffer: Arc<RwLock<VecDeque<DebugEntry>>>,
    max_size: usize,
}

impl BufferOutput {
    /// Create a new buffer output handler
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    /// Get all buffered entries
    #[must_use]
    pub fn get_entries(&self) -> Vec<DebugEntry> {
        self.buffer.read().iter().cloned().collect()
    }

    /// Get the last N entries
    #[must_use]
    pub fn get_last_entries(&self, n: usize) -> Vec<DebugEntry> {
        let buffer = self.buffer.read();
        buffer.iter().rev().take(n).rev().cloned().collect()
    }
}

impl DebugOutput for BufferOutput {
    fn write(&self, entry: &DebugEntry) {
        let mut buffer = self.buffer.write();

        // If at capacity, remove oldest entry
        if buffer.len() >= self.max_size {
            buffer.pop_front();
        }

        buffer.push_back(entry.clone());
    }

    fn flush(&self) {
        // No-op for buffer
    }

    fn clear(&self) {
        self.buffer.write().clear();
    }

    fn description(&self) -> &'static str {
        "buffer"
    }
}

/// Multi-output handler that writes to multiple destinations
pub struct MultiOutput {
    outputs: Vec<Box<dyn DebugOutput>>,
}

impl MultiOutput {
    /// Create a new multi-output handler
    #[must_use]
    pub fn new(outputs: Vec<Box<dyn DebugOutput>>) -> Self {
        Self { outputs }
    }

    /// Add an output handler
    pub fn add_output(&mut self, output: Box<dyn DebugOutput>) {
        self.outputs.push(output);
    }
}

impl DebugOutput for MultiOutput {
    fn write(&self, entry: &DebugEntry) {
        for output in &self.outputs {
            output.write(entry);
        }
    }

    fn flush(&self) {
        for output in &self.outputs {
            output.flush();
        }
    }

    fn clear(&self) {
        for output in &self.outputs {
            output.clear();
        }
    }

    fn description(&self) -> &'static str {
        "multi"
    }
}

/// Null output handler that discards all entries
pub struct NullOutput;

impl DebugOutput for NullOutput {
    fn write(&self, _entry: &DebugEntry) {
        // Intentionally do nothing
    }

    fn flush(&self) {
        // No-op
    }

    fn description(&self) -> &'static str {
        "null"
    }
}

impl fmt::Debug for dyn DebugOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugOutput({})", self.description())
    }
}

/// Implementation of `DebugOutput` for `Arc<T>` to allow shared ownership
impl<T: DebugOutput> DebugOutput for Arc<T> {
    fn write(&self, entry: &DebugEntry) {
        (**self).write(entry);
    }

    fn flush(&self) {
        (**self).flush();
    }

    fn clear(&self) {
        (**self).clear();
    }

    fn description(&self) -> &'static str {
        (**self).description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::levels::DebugLevel;

    #[test]
    fn test_buffer_output() {
        let buffer = BufferOutput::new(3);

        for i in 0..5 {
            let entry = DebugEntry::new(DebugLevel::Info, format!("Message {i}"));
            buffer.write(&entry);
        }

        let entries = buffer.get_entries();
        assert_eq!(entries.len(), 3); // Max size is 3
        assert_eq!(entries[0].message, "Message 2"); // Oldest kept entry
        assert_eq!(entries[2].message, "Message 4"); // Newest entry
    }

    #[test]
    fn test_multi_output() {
        let buffer1 = Arc::new(BufferOutput::new(10));
        let buffer2 = Arc::new(BufferOutput::new(10));

        let multi = MultiOutput::new(vec![
            Box::new(buffer1.clone()) as Box<dyn DebugOutput>,
            Box::new(buffer2.clone()) as Box<dyn DebugOutput>,
        ]);

        let entry = DebugEntry::new(DebugLevel::Debug, "Test");
        multi.write(&entry);

        assert_eq!(buffer1.get_entries().len(), 1);
        assert_eq!(buffer2.get_entries().len(), 1);
    }
}
