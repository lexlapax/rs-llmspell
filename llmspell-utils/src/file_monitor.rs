//! ABOUTME: File system monitoring utilities for watching file changes
//! ABOUTME: Provides functions for tracking file system events and changes

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// File system event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileEventType {
    /// File or directory was created
    Create,
    /// File or directory was modified
    Modify,
    /// File or directory was deleted
    Delete,
    /// File or directory was renamed
    Rename,
    /// Other event type
    Other,
}

/// File system event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    /// Type of event
    pub event_type: FileEventType,
    /// Path that triggered the event
    pub path: PathBuf,
    /// Optional old path (for rename events)
    pub old_path: Option<PathBuf>,
    /// Timestamp of the event
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl FileEvent {
    /// Create a new file event
    #[must_use]
    pub fn new(event_type: FileEventType, path: PathBuf) -> Self {
        Self {
            event_type,
            path,
            old_path: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new rename event
    #[must_use]
    pub fn new_rename(old_path: PathBuf, new_path: PathBuf) -> Self {
        Self {
            event_type: FileEventType::Rename,
            path: new_path,
            old_path: Some(old_path),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if event matches a glob pattern
    #[must_use]
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        match glob::Pattern::new(pattern) {
            Ok(glob_pattern) => {
                glob_pattern.matches_path(&self.path)
                    || self
                        .old_path
                        .as_ref()
                        .is_some_and(|p| glob_pattern.matches_path(p))
            }
            Err(_) => false,
        }
    }
}

/// Configuration for file watching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Paths to watch
    pub paths: Vec<PathBuf>,
    /// Recursive watching
    pub recursive: bool,
    /// Pattern filter (glob)
    pub pattern: Option<String>,
    /// Debounce duration in milliseconds
    pub debounce_ms: u64,
    /// Maximum events to buffer
    pub max_events: usize,
    /// Timeout for watching in seconds
    pub timeout_seconds: Option<u64>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            pattern: None,
            debounce_ms: 100,
            max_events: 1000,
            timeout_seconds: None,
        }
    }
}

impl WatchConfig {
    /// Create a new watch configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a path to watch
    #[must_use]
    pub fn add_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Set recursive watching
    #[must_use]
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Set pattern filter
    #[must_use]
    pub fn pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set debounce duration
    #[must_use]
    pub fn debounce(mut self, duration: Duration) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        {
            self.debounce_ms = duration.as_millis() as u64;
        }
        self
    }

    /// Set maximum events to buffer
    #[must_use]
    pub fn max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// Set timeout for watching
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout_seconds = Some(timeout.as_secs());
        self
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No paths are specified for watching
    /// - Any specified path does not exist
    /// - Max events is 0
    /// - Pattern is invalid glob syntax
    pub fn validate(&self) -> Result<()> {
        if self.paths.is_empty() {
            return Err(anyhow::anyhow!("No paths specified for watching"));
        }

        for path in &self.paths {
            if !path.exists() {
                return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
            }
        }

        if self.max_events == 0 {
            return Err(anyhow::anyhow!("Max events must be greater than 0"));
        }

        if let Some(pattern) = &self.pattern {
            glob::Pattern::new(pattern)
                .map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))?;
        }

        Ok(())
    }
}

/// Check if a path should be watched based on configuration
#[must_use]
pub fn should_watch_path(path: &Path, config: &WatchConfig) -> bool {
    if let Some(pattern) = &config.pattern {
        match glob::Pattern::new(pattern) {
            Ok(glob_pattern) => glob_pattern.matches_path(path),
            Err(_) => false,
        }
    } else {
        true
    }
}

/// Debounce events to prevent duplicates
#[must_use]
pub fn debounce_events(events: Vec<FileEvent>, debounce_ms: u64) -> Vec<FileEvent> {
    if debounce_ms == 0 {
        return events;
    }

    let mut debounced = Vec::new();
    let mut last_event_time = std::collections::HashMap::new();

    for event in events {
        let key = event.path.clone();
        let now = event.timestamp;

        if let Some(last_time) = last_event_time.get(&key) {
            let diff = now.signed_duration_since(*last_time);
            #[allow(clippy::cast_possible_wrap)]
            if diff.num_milliseconds() < (debounce_ms as i64) {
                continue;
            }
        }

        last_event_time.insert(key, now);
        debounced.push(event);
    }

    debounced
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_event_creation() {
        let event = FileEvent::new(FileEventType::Create, PathBuf::from("/test/file.txt"));
        assert_eq!(event.event_type, FileEventType::Create);
        assert_eq!(event.path, PathBuf::from("/test/file.txt"));
        assert!(event.old_path.is_none());
    }

    #[test]
    fn test_rename_event_creation() {
        let event = FileEvent::new_rename(
            PathBuf::from("/test/old.txt"),
            PathBuf::from("/test/new.txt"),
        );
        assert_eq!(event.event_type, FileEventType::Rename);
        assert_eq!(event.path, PathBuf::from("/test/new.txt"));
        assert_eq!(event.old_path, Some(PathBuf::from("/test/old.txt")));
    }

    #[test]
    fn test_pattern_matching() {
        let event = FileEvent::new(FileEventType::Create, PathBuf::from("/test/file.txt"));
        assert!(event.matches_pattern("*.txt"));
        assert!(event.matches_pattern("/test/*"));
        assert!(!event.matches_pattern("*.log"));
    }

    #[test]
    fn test_watch_config_builder() {
        let config = WatchConfig::new()
            .add_path("/test")
            .recursive(false)
            .pattern("*.txt")
            .debounce(Duration::from_millis(200))
            .max_events(500)
            .timeout(Duration::from_secs(60));

        assert_eq!(config.paths, vec![PathBuf::from("/test")]);
        assert!(!config.recursive);
        assert_eq!(config.pattern, Some("*.txt".to_string()));
        assert_eq!(config.debounce_ms, 200);
        assert_eq!(config.max_events, 500);
        assert_eq!(config.timeout_seconds, Some(60));
    }

    #[test]
    fn test_should_watch_path() {
        let config = WatchConfig::new().pattern("*.txt");
        assert!(should_watch_path(&PathBuf::from("file.txt"), &config));
        assert!(!should_watch_path(&PathBuf::from("file.log"), &config));
    }

    #[test]
    fn test_debounce_events() {
        let mut events = Vec::new();
        let base_time = chrono::Utc::now();

        // Add events with different timestamps
        let mut create_event =
            FileEvent::new(FileEventType::Create, PathBuf::from("/test/file.txt"));
        create_event.timestamp = base_time;
        events.push(create_event);

        let mut modify_event1 =
            FileEvent::new(FileEventType::Modify, PathBuf::from("/test/file.txt"));
        modify_event1.timestamp = base_time + chrono::Duration::milliseconds(50);
        events.push(modify_event1);

        let mut modify_event2 =
            FileEvent::new(FileEventType::Modify, PathBuf::from("/test/file.txt"));
        modify_event2.timestamp = base_time + chrono::Duration::milliseconds(150);
        events.push(modify_event2);

        let debounced = debounce_events(events, 100);
        assert_eq!(debounced.len(), 2); // First and third events should remain
    }
}
