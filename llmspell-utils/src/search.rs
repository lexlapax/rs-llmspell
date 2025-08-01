// ABOUTME: File content search and pattern matching utilities
// ABOUTME: Provides regex-based search, recursive directory traversal, and context extraction

//! File and content search utilities
//!
//! This module provides utilities for searching within files and directories,
//! including pattern matching, recursive traversal, and context extraction.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_utils::search::{search_in_file, search_in_directory, SearchOptions, SearchResult};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Search for a pattern in a single file
//! let results = search_in_file(
//!     Path::new("/path/to/file.txt"),
//!     "search pattern",
//!     &SearchOptions::default()
//! )?;
//!
//! for result in results {
//!     println!("Found match at line {}: {}", result.line_number, result.line_content);
//! }
//!
//! // Search recursively in a directory
//! let results = search_in_directory(
//!     Path::new("/path/to/directory"),
//!     "pattern",
//!     &SearchOptions::default().with_recursive(true)
//! )?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Configuration options for search operations
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Whether to search recursively in subdirectories
    pub recursive: bool,
    /// Whether the search is case-sensitive
    pub case_sensitive: bool,
    /// Whether to use regex pattern matching
    pub use_regex: bool,
    /// Number of context lines to include before and after matches
    pub context_lines: usize,
    /// Maximum file size to search (in bytes)
    pub max_file_size: u64,
    /// File extensions to include (if empty, search all files)
    pub include_extensions: Vec<String>,
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
    /// Patterns to exclude directories (glob patterns)
    pub exclude_dirs: Vec<String>,
    /// Maximum number of matches per file (0 = unlimited)
    pub max_matches_per_file: usize,
    /// Maximum search depth for recursive searches (0 = unlimited)
    pub max_depth: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            case_sensitive: true,
            use_regex: false,
            context_lines: 0,
            max_file_size: 10 * 1024 * 1024, // 10MB
            include_extensions: Vec::new(),
            exclude_extensions: vec![
                "exe".to_string(),
                "bin".to_string(),
                "so".to_string(),
                "dll".to_string(),
                "dylib".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "rar".to_string(),
                "7z".to_string(),
            ],
            exclude_dirs: vec![
                ".git".to_string(),
                ".svn".to_string(),
                ".hg".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
            ],
            max_matches_per_file: 0,
            max_depth: 0,
        }
    }
}

impl SearchOptions {
    /// Create a new `SearchOptions` with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable recursive searching
    #[must_use]
    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Enable or disable case-sensitive searching
    #[must_use]
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Enable or disable regex pattern matching
    #[must_use]
    pub fn with_regex(mut self, use_regex: bool) -> Self {
        self.use_regex = use_regex;
        self
    }

    /// Set the number of context lines
    #[must_use]
    pub fn with_context_lines(mut self, context_lines: usize) -> Self {
        self.context_lines = context_lines;
        self
    }

    /// Set maximum file size to search
    #[must_use]
    pub fn with_max_file_size(mut self, max_file_size: u64) -> Self {
        self.max_file_size = max_file_size;
        self
    }

    /// Set included file extensions
    #[must_use]
    pub fn with_include_extensions(mut self, extensions: Vec<String>) -> Self {
        self.include_extensions = extensions;
        self
    }

    /// Set excluded file extensions
    #[must_use]
    pub fn with_exclude_extensions(mut self, extensions: Vec<String>) -> Self {
        self.exclude_extensions = extensions;
        self
    }

    /// Set excluded directory patterns
    #[must_use]
    pub fn with_exclude_dirs(mut self, dirs: Vec<String>) -> Self {
        self.exclude_dirs = dirs;
        self
    }

    /// Set maximum matches per file
    #[must_use]
    pub fn with_max_matches_per_file(mut self, max_matches: usize) -> Self {
        self.max_matches_per_file = max_matches;
        self
    }

    /// Set maximum search depth
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }
}

/// Represents a search match result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    /// The file path where the match was found
    pub file_path: PathBuf,
    /// Line number where the match occurred (1-based)
    pub line_number: usize,
    /// Column number where the match starts (1-based)
    pub column_number: usize,
    /// The content of the matching line
    pub line_content: String,
    /// Context lines before the match
    pub context_before: Vec<String>,
    /// Context lines after the match
    pub context_after: Vec<String>,
    /// The matched text substring
    pub matched_text: String,
}

/// Result of a search operation
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// All matches found
    pub matches: Vec<SearchMatch>,
    /// Total number of files searched
    pub files_searched: usize,
    /// Total number of matches found
    pub total_matches: usize,
    /// Files that couldn't be searched (due to errors)
    pub skipped_files: Vec<(PathBuf, String)>,
}

impl SearchResult {
    /// Create a new empty search result
    #[must_use]
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            files_searched: 0,
            total_matches: 0,
            skipped_files: Vec::new(),
        }
    }

    /// Add a match to the result
    pub fn add_match(&mut self, search_match: SearchMatch) {
        self.matches.push(search_match);
        self.total_matches += 1;
    }

    /// Add a skipped file
    pub fn add_skipped_file(&mut self, file_path: PathBuf, reason: String) {
        self.skipped_files.push((file_path, reason));
    }

    /// Increment files searched counter
    pub fn increment_files_searched(&mut self) {
        self.files_searched += 1;
    }

    /// Check if any matches were found
    #[must_use]
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    /// Get matches grouped by file
    pub fn matches_by_file(&self) -> std::collections::HashMap<PathBuf, Vec<&SearchMatch>> {
        let mut grouped = std::collections::HashMap::new();
        for search_match in &self.matches {
            grouped
                .entry(search_match.file_path.clone())
                .or_insert_with(Vec::new)
                .push(search_match);
        }
        grouped
    }
}

impl Default for SearchResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Search for a pattern in a single file
///
/// Returns all matches found in the file with optional context.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::search::{search_in_file, SearchOptions};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let matches = search_in_file(
///     Path::new("example.txt"),
///     "TODO",
///     &SearchOptions::default().with_context_lines(2)
/// )?;
///
/// for search_match in matches {
///     println!("Line {}: {}", search_match.line_number, search_match.line_content);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - File cannot be read
/// - File is too large
/// - Pattern is invalid regex (when using regex mode)
pub fn search_in_file(
    file_path: &Path,
    pattern: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchMatch>> {
    // Check file size
    let metadata = fs::metadata(file_path)
        .with_context(|| format!("Failed to read metadata for: {}", file_path.display()))?;

    if metadata.len() > options.max_file_size {
        anyhow::bail!(
            "File too large: {} bytes (max: {})",
            metadata.len(),
            options.max_file_size
        );
    }

    // Read file content
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut matches = Vec::new();

    // Prepare pattern matcher
    let pattern_matcher = if options.use_regex {
        let regex_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){pattern}")
        };

        let regex = Regex::new(&regex_pattern)
            .with_context(|| format!("Invalid regex pattern: {pattern}"))?;

        PatternMatcher::Regex(regex)
    } else {
        let search_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };
        PatternMatcher::Literal(search_pattern)
    };

    // Search through lines
    for (line_index, line) in lines.iter().enumerate() {
        let line_number = line_index + 1;

        // Check for matches in this line
        let line_matches = find_matches_in_line(line, &pattern_matcher, line_number, options);

        for mut search_match in line_matches {
            search_match.file_path = file_path.to_path_buf();

            // Add context lines if requested
            if options.context_lines > 0 {
                search_match.context_before = get_context_lines(
                    &lines,
                    line_index,
                    options.context_lines,
                    ContextDirection::Before,
                );
                search_match.context_after = get_context_lines(
                    &lines,
                    line_index,
                    options.context_lines,
                    ContextDirection::After,
                );
            }

            matches.push(search_match);

            // Check max matches limit
            if options.max_matches_per_file > 0 && matches.len() >= options.max_matches_per_file {
                break;
            }
        }

        if options.max_matches_per_file > 0 && matches.len() >= options.max_matches_per_file {
            break;
        }
    }

    Ok(matches)
}

/// Search for a pattern recursively in a directory
///
/// Returns all matches found in files within the directory tree.
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_utils::search::{search_in_directory, SearchOptions};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = search_in_directory(
///     Path::new("./src"),
///     "fn main",
///     &SearchOptions::default()
///         .with_recursive(true)
///         .with_include_extensions(vec!["rs".to_string()])
/// )?;
///
/// println!("Found {} matches in {} files", result.total_matches, result.files_searched);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Directory cannot be accessed
/// - Pattern is invalid regex (when using regex mode)
pub fn search_in_directory(
    directory: &Path,
    pattern: &str,
    options: &SearchOptions,
) -> Result<SearchResult> {
    let mut result = SearchResult::new();

    // Create directory walker
    let mut walker = WalkDir::new(directory);

    if !options.recursive {
        walker = walker.max_depth(1);
    } else if options.max_depth > 0 {
        walker = walker.max_depth(options.max_depth);
    }

    // Convert exclude patterns to lowercase for case-insensitive matching
    let exclude_dirs: HashSet<String> = options
        .exclude_dirs
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let exclude_extensions: HashSet<String> = options
        .exclude_extensions
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let include_extensions: HashSet<String> = options
        .include_extensions
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    // Walk directory tree with filtering
    for entry in walker.into_iter().filter_entry(|e| {
        // Always allow root directory
        if e.path() == directory {
            return true;
        }

        // Skip excluded directories
        if e.file_type().is_dir() {
            if let Some(dir_name) = e.path().file_name() {
                let dir_name_lower = dir_name.to_string_lossy().to_lowercase();
                if exclude_dirs.contains(&dir_name_lower) {
                    return false;
                }
            }
        }

        true
    }) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                result.add_skipped_file(directory.to_path_buf(), format!("Walk error: {e}"));
                continue;
            }
        };

        let path = entry.path();

        // Skip directories but also check for exclusion
        if entry.file_type().is_dir() {
            continue;
        }

        // Check file extension filters
        if let Some(extension) = path.extension() {
            let ext_lower = extension.to_string_lossy().to_lowercase();

            // Skip excluded extensions
            if exclude_extensions.contains(&ext_lower) {
                continue;
            }

            // If include list is specified, only include matching extensions
            if !include_extensions.is_empty() && !include_extensions.contains(&ext_lower) {
                continue;
            }
        } else if !include_extensions.is_empty() {
            // Skip files without extension if include list is specified
            continue;
        }

        // Search in this file
        match search_in_file(path, pattern, options) {
            Ok(matches) => {
                result.increment_files_searched();
                for search_match in matches {
                    result.add_match(search_match);
                }
            }
            Err(e) => {
                result.add_skipped_file(path.to_path_buf(), e.to_string());
            }
        }
    }

    Ok(result)
}

/// Internal pattern matcher enum
enum PatternMatcher {
    Literal(String),
    Regex(Regex),
}

/// Direction for context extraction
#[derive(Copy, Clone)]
enum ContextDirection {
    Before,
    After,
}

/// Find all matches of a pattern in a single line
fn find_matches_in_line(
    line: &str,
    pattern_matcher: &PatternMatcher,
    line_number: usize,
    options: &SearchOptions,
) -> Vec<SearchMatch> {
    let mut matches = Vec::new();

    match pattern_matcher {
        PatternMatcher::Literal(pattern) => {
            let search_line = if options.case_sensitive {
                line
            } else {
                &line.to_lowercase()
            };

            let mut start = 0;
            while let Some(pos) = search_line[start..].find(pattern) {
                let absolute_pos = start + pos;
                let matched_text = &line[absolute_pos..absolute_pos + pattern.len()];

                matches.push(SearchMatch {
                    file_path: PathBuf::new(), // Will be set by caller
                    line_number,
                    column_number: absolute_pos + 1, // 1-based
                    line_content: line.to_string(),
                    context_before: Vec::new(), // Will be set by caller
                    context_after: Vec::new(),  // Will be set by caller
                    matched_text: matched_text.to_string(),
                });

                start = absolute_pos + 1; // Move past this match
            }
        }
        PatternMatcher::Regex(regex) => {
            for regex_match in regex.find_iter(line) {
                matches.push(SearchMatch {
                    file_path: PathBuf::new(), // Will be set by caller
                    line_number,
                    column_number: regex_match.start() + 1, // 1-based
                    line_content: line.to_string(),
                    context_before: Vec::new(), // Will be set by caller
                    context_after: Vec::new(),  // Will be set by caller
                    matched_text: regex_match.as_str().to_string(),
                });
            }
        }
    }

    matches
}

/// Extract context lines before or after a target line
fn get_context_lines(
    lines: &[&str],
    target_index: usize,
    context_count: usize,
    direction: ContextDirection,
) -> Vec<String> {
    let mut context = Vec::new();

    match direction {
        ContextDirection::Before => {
            let start = target_index.saturating_sub(context_count);
            for line in lines.iter().take(target_index).skip(start) {
                context.push((*line).to_string());
            }
        }
        ContextDirection::After => {
            let end = std::cmp::min(target_index + 1 + context_count, lines.len());
            for line in lines.iter().take(end).skip(target_index + 1) {
                context.push((*line).to_string());
            }
        }
    }

    context
}

/// Check if a file should be searched based on its type and content
#[must_use]
pub fn should_search_file(file_path: &Path, options: &SearchOptions) -> bool {
    // Check file extension
    if let Some(extension) = file_path.extension() {
        let ext_lower = extension.to_string_lossy().to_lowercase();

        // Skip excluded extensions
        if options.exclude_extensions.contains(&ext_lower) {
            return false;
        }

        // If include list is specified, only include matching extensions
        if !options.include_extensions.is_empty()
            && !options.include_extensions.contains(&ext_lower)
        {
            return false;
        }
    } else if !options.include_extensions.is_empty() {
        // Skip files without extension if include list is specified
        return false;
    }

    // Check file size
    if let Ok(metadata) = fs::metadata(file_path) {
        if metadata.len() > options.max_file_size {
            return false;
        }
    }

    true
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions::new()
            .with_recursive(true)
            .with_case_sensitive(false)
            .with_regex(true)
            .with_context_lines(3)
            .with_max_file_size(1024)
            .with_include_extensions(vec!["txt".to_string(), "rs".to_string()])
            .with_exclude_extensions(vec!["bin".to_string()])
            .with_exclude_dirs(vec!["target".to_string()])
            .with_max_matches_per_file(10)
            .with_max_depth(5);

        assert!(options.recursive);
        assert!(!options.case_sensitive);
        assert!(options.use_regex);
        assert_eq!(options.context_lines, 3);
        assert_eq!(options.max_file_size, 1024);
        assert_eq!(options.include_extensions, vec!["txt", "rs"]);
        assert_eq!(options.exclude_extensions, vec!["bin"]);
        assert_eq!(options.exclude_dirs, vec!["target"]);
        assert_eq!(options.max_matches_per_file, 10);
        assert_eq!(options.max_depth, 5);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_file_literal() {
        let temp_dir = TempDir::new().unwrap();
        let content = "Line 1\nLine 2 with pattern\nLine 3\nAnother pattern here\nLine 5";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content);

        let options = SearchOptions::new().with_case_sensitive(true);
        let matches = search_in_file(&file_path, "pattern", &options).unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_number, 2);
        assert_eq!(matches[0].line_content, "Line 2 with pattern");
        assert_eq!(matches[0].matched_text, "pattern");
        assert_eq!(matches[0].column_number, 13);

        assert_eq!(matches[1].line_number, 4);
        assert_eq!(matches[1].line_content, "Another pattern here");
        assert_eq!(matches[1].matched_text, "pattern");
        assert_eq!(matches[1].column_number, 9);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_file_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let content = "Line 1\nLine 2 with PATTERN\nLine 3\nAnother pattern here";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content);

        let options = SearchOptions::new().with_case_sensitive(false);
        let matches = search_in_file(&file_path, "pattern", &options).unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].matched_text, "PATTERN");
        assert_eq!(matches[1].matched_text, "pattern");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_file_regex() {
        let temp_dir = TempDir::new().unwrap();
        let content = "Error: Something went wrong\nWARNING: Check this\nInfo: All good\nERROR: Another issue";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content);

        let options = SearchOptions::new()
            .with_regex(true)
            .with_case_sensitive(false);
        let matches = search_in_file(&file_path, r"(ERROR|WARNING):", &options).unwrap();

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].matched_text, "Error:");
        assert_eq!(matches[1].matched_text, "WARNING:");
        assert_eq!(matches[2].matched_text, "ERROR:");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_file_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let content = "Line 1\nLine 2\nLine 3 with pattern\nLine 4\nLine 5";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content);

        let options = SearchOptions::new().with_context_lines(2);
        let matches = search_in_file(&file_path, "pattern", &options).unwrap();

        assert_eq!(matches.len(), 1);
        let search_match = &matches[0];
        assert_eq!(search_match.context_before, vec!["Line 1", "Line 2"]);
        assert_eq!(search_match.context_after, vec!["Line 4", "Line 5"]);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_file_max_matches() {
        let temp_dir = TempDir::new().unwrap();
        let content = "pattern\npattern\npattern\npattern\npattern";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content);

        let options = SearchOptions::new().with_max_matches_per_file(3);
        let matches = search_in_file(&file_path, "pattern", &options).unwrap();

        assert_eq!(matches.len(), 3);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_directory_basic() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        create_test_file(
            temp_dir.path(),
            "file1.txt",
            "This has a pattern\nNo match here",
        );
        create_test_file(
            temp_dir.path(),
            "file2.txt",
            "Another pattern\nAnd another pattern",
        );
        create_test_file(temp_dir.path(), "file3.log", "No matches in this file");

        let options = SearchOptions::new();
        let result = search_in_directory(temp_dir.path(), "pattern", &options).unwrap();

        assert_eq!(result.files_searched, 3);
        assert_eq!(result.total_matches, 3);
        assert!(result.has_matches());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_directory_with_extension_filter() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "file1.txt", "pattern in txt");
        create_test_file(temp_dir.path(), "file2.rs", "pattern in rust");
        create_test_file(temp_dir.path(), "file3.log", "pattern in log");

        let options =
            SearchOptions::new().with_include_extensions(vec!["txt".to_string(), "rs".to_string()]);
        let result = search_in_directory(temp_dir.path(), "pattern", &options).unwrap();

        assert_eq!(result.files_searched, 2); // Only txt and rs files
        assert_eq!(result.total_matches, 2);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        create_test_file(temp_dir.path(), "root.txt", "pattern at root");
        create_test_file(&subdir, "nested.txt", "pattern in nested");

        // Non-recursive search
        let options = SearchOptions::new().with_recursive(false);
        let result = search_in_directory(temp_dir.path(), "pattern", &options).unwrap();
        assert_eq!(result.files_searched, 1); // Only root file

        // Recursive search
        let options = SearchOptions::new().with_recursive(true);
        let result = search_in_directory(temp_dir.path(), "pattern", &options).unwrap();
        assert_eq!(result.files_searched, 2); // Root and nested files
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_in_directory_exclude_dirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create directories
        let target_dir = temp_dir.path().join("target");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&target_dir).unwrap();
        fs::create_dir(&src_dir).unwrap();

        create_test_file(&target_dir, "file1.txt", "pattern in target");
        create_test_file(&src_dir, "file2.txt", "pattern in src");

        let options = SearchOptions::new()
            .with_recursive(true)
            .with_exclude_dirs(vec!["target".to_string()]);
        let result = search_in_directory(temp_dir.path(), "pattern", &options).unwrap();

        assert_eq!(result.files_searched, 1); // Only src file
        assert_eq!(result.total_matches, 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_search_result_methods() {
        let mut result = SearchResult::new();

        assert_eq!(result.files_searched, 0);
        assert_eq!(result.total_matches, 0);
        assert!(!result.has_matches());

        let search_match = SearchMatch {
            file_path: PathBuf::from("test.txt"),
            line_number: 1,
            column_number: 1,
            line_content: "test line".to_string(),
            context_before: Vec::new(),
            context_after: Vec::new(),
            matched_text: "test".to_string(),
        };

        result.add_match(search_match);
        result.increment_files_searched();

        assert_eq!(result.files_searched, 1);
        assert_eq!(result.total_matches, 1);
        assert!(result.has_matches());

        let grouped = result.matches_by_file();
        assert_eq!(grouped.len(), 1);
        assert!(grouped.contains_key(&PathBuf::from("test.txt")));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_should_search_file() {
        let temp_dir = TempDir::new().unwrap();

        // Test with default options
        let options = SearchOptions::default();

        // Should search text files
        let txt_file = temp_dir.path().join("test.txt");
        fs::write(&txt_file, "test").unwrap();
        assert!(should_search_file(&txt_file, &options));

        // Should not search binary files
        let exe_file = temp_dir.path().join("test.exe");
        fs::write(&exe_file, "test").unwrap();
        assert!(!should_search_file(&exe_file, &options));

        // Test with include extensions
        let options = SearchOptions::new().with_include_extensions(vec!["rs".to_string()]);

        let rs_file = temp_dir.path().join("test.rs");
        fs::write(&rs_file, "test").unwrap();
        assert!(should_search_file(&rs_file, &options));
        assert!(!should_search_file(&txt_file, &options)); // Not in include list
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_multiple_matches_per_line() {
        let temp_dir = TempDir::new().unwrap();
        let content = "pattern and pattern and pattern";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content);

        let options = SearchOptions::new();
        let matches = search_in_file(&file_path, "pattern", &options).unwrap();

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].column_number, 1);
        assert_eq!(matches[1].column_number, 13);
        assert_eq!(matches[2].column_number, 25);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_invalid_regex_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.txt", "test content");

        let options = SearchOptions::new().with_regex(true);
        let result = search_in_file(&file_path, "[invalid", &options);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid regex pattern"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_file_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let content = "x".repeat(2000); // 2000 bytes
        let file_path = create_test_file(temp_dir.path(), "large.txt", &content);

        let options = SearchOptions::new().with_max_file_size(1000); // 1000 bytes limit
        let result = search_in_file(&file_path, "x", &options);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File too large"));
    }
}
