// ABOUTME: File content search tool with pattern matching and context extraction
// ABOUTME: Provides comprehensive text search within files and directories using regex or literal patterns

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::FileSandbox;
use llmspell_utils::{
    extract_optional_array, extract_optional_bool, extract_optional_string, extract_optional_u64,
    extract_parameters, extract_required_string,
    response::ResponseBuilder,
    search::{search_in_directory, search_in_file, SearchOptions, SearchResult},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument, trace, warn};

/// File search tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchConfig {
    /// Maximum number of search results to return
    pub max_results: usize,
    /// Default number of context lines to include
    pub default_context_lines: usize,
    /// Maximum file size to search (in bytes)
    pub max_file_size: u64,
    /// Maximum search depth for recursive searches
    pub max_search_depth: usize,
    /// Default file extensions to include in searches
    pub default_include_extensions: Vec<String>,
    /// Default file extensions to exclude from searches
    pub default_exclude_extensions: Vec<String>,
}

impl Default for FileSearchConfig {
    fn default() -> Self {
        Self {
            max_results: 1000,
            default_context_lines: 2,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_search_depth: 20,
            default_include_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "java".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                "h".to_string(),
                "json".to_string(),
                "xml".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "ini".to_string(),
                "cfg".to_string(),
                "conf".to_string(),
                "log".to_string(),
            ],
            default_exclude_extensions: vec![
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
                "mp3".to_string(),
                "mp4".to_string(),
                "avi".to_string(),
                "mkv".to_string(),
            ],
        }
    }
}

/// File search tool for pattern matching within files and directories
#[derive(Clone)]
pub struct FileSearchTool {
    metadata: ComponentMetadata,
    config: FileSearchConfig,
    sandbox: Arc<FileSandbox>,
}

impl FileSearchTool {
    /// Create a new file search tool
    #[must_use]
    pub fn new(config: FileSearchConfig, sandbox: Arc<FileSandbox>) -> Self {
        debug!(
            max_results = config.max_results,
            max_file_size = config.max_file_size,
            "Creating FileSearchTool"
        );
        Self {
            metadata: ComponentMetadata::new(
                "file-search".to_string(),
                "File content search with pattern matching and context extraction".to_string(),
            ),
            config,
            sandbox,
        }
    }

    /// Search within a single file
    #[allow(clippy::unused_async)]
    #[instrument(skip(self))]
    async fn search_file(
        &self,
        file_path: &Path,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<SearchResult> {
        let start = Instant::now();
        // Validate path with sandbox
        self.sandbox
            .validate_path(file_path)
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        debug!(
            "Searching in file: {} for pattern: '{}'",
            file_path.display(),
            pattern
        );

        let matches = search_in_file(file_path, pattern, options)?;
        let mut result = SearchResult::new();
        result.increment_files_searched();

        for search_match in matches {
            result.add_match(search_match);

            // Check max results limit
            if self.config.max_results > 0 && result.total_matches >= self.config.max_results {
                warn!(
                    "Reached maximum results limit ({}), stopping search",
                    self.config.max_results
                );
                break;
            }
        }

        let elapsed_ms = start.elapsed().as_millis();
        info!(
            file = ?file_path,
            pattern,
            matches = result.total_matches,
            duration_ms = elapsed_ms,
            "File search completed"
        );

        Ok(result)
    }

    /// Search within a directory
    #[allow(clippy::unused_async)]
    #[instrument(skip(self))]
    async fn search_directory(
        &self,
        directory: &Path,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<SearchResult> {
        let start = Instant::now();
        // Validate path with sandbox
        self.sandbox
            .validate_path(directory)
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        debug!(
            "Searching in directory: {} for pattern: '{}' (recursive: {})",
            directory.display(),
            pattern,
            options.recursive
        );

        let mut result = search_in_directory(directory, pattern, options)?;

        // Apply max results limit
        if self.config.max_results > 0 && result.total_matches > self.config.max_results {
            warn!(
                "Found {} matches, truncating to {} results",
                result.total_matches, self.config.max_results
            );
            result.matches.truncate(self.config.max_results);
            result.total_matches = self.config.max_results;
        }

        let elapsed_ms = start.elapsed().as_millis();
        info!(
            directory = ?directory,
            pattern,
            files_searched = result.files_searched,
            matches = result.total_matches,
            duration_ms = elapsed_ms,
            "Directory search completed"
        );

        Ok(result)
    }

    /// Build search options from parameters
    fn build_search_options(&self, params: &serde_json::Value) -> SearchOptions {
        trace!("Building search options from parameters");
        let mut options = SearchOptions::new();

        // Basic search options
        if let Some(recursive) = extract_optional_bool(params, "recursive") {
            options = options.with_recursive(recursive);
        }

        if let Some(case_sensitive) = extract_optional_bool(params, "case_sensitive") {
            options = options.with_case_sensitive(case_sensitive);
        }

        if let Some(use_regex) = extract_optional_bool(params, "use_regex") {
            options = options.with_regex(use_regex);
        }

        // Context lines
        let context_lines = usize::try_from(
            extract_optional_u64(params, "context_lines")
                .unwrap_or(self.config.default_context_lines as u64),
        )
        .unwrap_or(usize::MAX);
        options = options.with_context_lines(context_lines);

        // File size limit
        let max_file_size =
            extract_optional_u64(params, "max_file_size").unwrap_or(self.config.max_file_size);
        options = options.with_max_file_size(max_file_size);

        // File extensions
        if let Some(include_exts) = extract_optional_array(params, "include_extensions") {
            let extensions: Vec<String> = include_exts
                .iter()
                .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                .collect();
            options = options.with_include_extensions(extensions);
        } else if let Some(ext_str) = extract_optional_string(params, "include_extensions") {
            let extensions: Vec<String> =
                ext_str.split(',').map(|s| s.trim().to_string()).collect();
            options = options.with_include_extensions(extensions);
        } else {
            // Use default include extensions
            options =
                options.with_include_extensions(self.config.default_include_extensions.clone());
        }

        if let Some(exclude_exts) = extract_optional_array(params, "exclude_extensions") {
            let extensions: Vec<String> = exclude_exts
                .iter()
                .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                .collect();
            options = options.with_exclude_extensions(extensions);
        } else if let Some(ext_str) = extract_optional_string(params, "exclude_extensions") {
            let extensions: Vec<String> =
                ext_str.split(',').map(|s| s.trim().to_string()).collect();
            options = options.with_exclude_extensions(extensions);
        } else {
            // Use default exclude extensions
            options =
                options.with_exclude_extensions(self.config.default_exclude_extensions.clone());
        }

        // Directory exclusions
        if let Some(dirs_array) = extract_optional_array(params, "exclude_dirs") {
            let directories: Vec<String> = dirs_array
                .iter()
                .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                .collect();
            options = options.with_exclude_dirs(directories);
        } else if let Some(dirs_str) = extract_optional_string(params, "exclude_dirs") {
            let directories: Vec<String> =
                dirs_str.split(',').map(|s| s.trim().to_string()).collect();
            options = options.with_exclude_dirs(directories);
        }

        // Search limits
        if let Some(max_matches) = extract_optional_u64(params, "max_matches_per_file") {
            options = options
                .with_max_matches_per_file(usize::try_from(max_matches).unwrap_or(usize::MAX));
        }

        let max_depth = usize::try_from(
            extract_optional_u64(params, "max_depth")
                .unwrap_or(self.config.max_search_depth as u64),
        )
        .unwrap_or(usize::MAX);
        options = options.with_max_depth(max_depth);

        options
    }

    /// Format search results for output
    #[allow(clippy::unused_self)]
    fn format_search_results(&self, result: &SearchResult) -> serde_json::Value {
        let matches: Vec<serde_json::Value> = result
            .matches
            .iter()
            .map(|m| {
                json!({
                    "file_path": m.file_path.to_string_lossy(),
                    "line_number": m.line_number,
                    "column_number": m.column_number,
                    "line_content": m.line_content,
                    "matched_text": m.matched_text,
                    "context_before": m.context_before,
                    "context_after": m.context_after
                })
            })
            .collect();

        let skipped_files: Vec<serde_json::Value> = result
            .skipped_files
            .iter()
            .map(|(path, reason)| {
                json!({
                    "file_path": path.to_string_lossy(),
                    "reason": reason
                })
            })
            .collect();

        json!({
            "matches": matches,
            "summary": {
                "total_matches": result.total_matches,
                "files_searched": result.files_searched,
                "files_skipped": result.skipped_files.len(),
                "has_matches": result.has_matches()
            },
            "skipped_files": skipped_files
        })
    }

    /// Validate search parameters
    #[allow(clippy::unused_async)]
    #[instrument(skip(self))]
    async fn validate_search_parameters(&self, params: &serde_json::Value) -> LLMResult<()> {
        trace!("Validating search parameters");
        // Required parameters are already validated by extract_required_string
        // Just validate pattern is not empty
        if let Some(pattern) = extract_optional_string(params, "pattern") {
            if pattern.trim().is_empty() {
                return Err(LLMSpellError::Validation {
                    message: "Pattern cannot be empty".to_string(),
                    field: Some("pattern".to_string()),
                });
            }
        }

        // Validate numeric parameters
        if let Some(lines) = extract_optional_u64(params, "context_lines") {
            if lines > 50 {
                return Err(LLMSpellError::Validation {
                    message: "Context lines cannot exceed 50".to_string(),
                    field: Some("context_lines".to_string()),
                });
            }
        }

        if let Some(depth) = extract_optional_u64(params, "max_depth") {
            if depth > 100 {
                return Err(LLMSpellError::Validation {
                    message: "Max depth cannot exceed 100".to_string(),
                    field: Some("max_depth".to_string()),
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BaseAgent for FileSearchTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        let start = Instant::now();
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        self.validate_search_parameters(params).await?;

        // Extract required parameters
        let pattern = extract_required_string(params, "pattern")?;
        let path_str = extract_required_string(params, "path")?;

        info!(pattern, path = path_str, "Executing file search");

        let search_path = PathBuf::from(path_str);

        // Validate path exists
        if !search_path.exists() {
            return Err(LLMSpellError::Validation {
                message: format!("Path does not exist: {}", search_path.display()),
                field: Some("path".to_string()),
            });
        }

        // Build search options
        let options = self.build_search_options(params);

        // Perform search based on path type
        let result = if search_path.is_file() {
            debug!(
                path = ?search_path,
                "Searching in single file"
            );
            self.search_file(&search_path, pattern, &options)
                .await
                .map_err(|e| LLMSpellError::Tool {
                    message: format!("File search failed: {e}"),
                    tool_name: Some("file-search".to_string()),
                    source: None,
                })?
        } else if search_path.is_dir() {
            debug!(
                path = ?search_path,
                recursive = options.recursive,
                "Searching in directory"
            );
            self.search_directory(&search_path, pattern, &options)
                .await
                .map_err(|e| LLMSpellError::Tool {
                    message: format!("Directory search failed: {e}"),
                    tool_name: Some("file-search".to_string()),
                    source: None,
                })?
        } else {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Path is neither a file nor directory: {}",
                    search_path.display()
                ),
                field: Some("path".to_string()),
            });
        };

        // Format results
        let formatted_results = self.format_search_results(&result);

        // Create response message
        let message = if result.has_matches() {
            format!(
                "Found {} matches across {} files. Search pattern: '{}'",
                result.total_matches, result.files_searched, pattern
            )
        } else {
            format!(
                "No matches found in {} files searched. Search pattern: '{}'",
                result.files_searched, pattern
            )
        };

        let (output_text, response) = ResponseBuilder::success("search")
            .with_message(message)
            .with_result(formatted_results)
            .build_for_output();

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata
            .extra
            .insert("operation".to_string(), "search".into());
        metadata.extra.insert("response".to_string(), response);

        let elapsed_ms = start.elapsed().as_millis();
        debug!(
            pattern,
            duration_ms = elapsed_ms,
            has_matches = result.has_matches(),
            "File search operation completed"
        );

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    #[instrument(skip(self))]
    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        trace!("Validating file search input");
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        error!(
            error = %error,
            "File search error occurred"
        );
        Ok(AgentOutput::text(format!("File search error: {error}")))
    }
}

#[async_trait]
impl Tool for FileSearchTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Filesystem
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // File search requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "file-search".to_string(),
            "Search for patterns within files and directories with context extraction".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "pattern".to_string(),
            param_type: ParameterType::String,
            description: "Search pattern (literal text or regex)".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "path".to_string(),
            param_type: ParameterType::String,
            description: "File or directory path to search".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "recursive".to_string(),
            param_type: ParameterType::Boolean,
            description: "Search recursively in subdirectories".to_string(),
            required: false,
            default: Some(json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "case_sensitive".to_string(),
            param_type: ParameterType::Boolean,
            description: "Perform case-sensitive search".to_string(),
            required: false,
            default: Some(json!(true)),
        })
        .with_parameter(ParameterDef {
            name: "use_regex".to_string(),
            param_type: ParameterType::Boolean,
            description: "Treat pattern as regular expression".to_string(),
            required: false,
            default: Some(json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "context_lines".to_string(),
            param_type: ParameterType::Number,
            description: "Number of context lines before and after matches".to_string(),
            required: false,
            default: Some(json!(2)),
        })
        .with_parameter(ParameterDef {
            name: "include_extensions".to_string(),
            param_type: ParameterType::Array,
            description: "File extensions to include (empty = search all)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "exclude_extensions".to_string(),
            param_type: ParameterType::Array,
            description: "File extensions to exclude".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "exclude_dirs".to_string(),
            param_type: ParameterType::Array,
            description: "Directory names to exclude from search".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "max_file_size".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum file size to search (bytes)".to_string(),
            required: false,
            default: Some(json!(10_485_760)), // 10MB
        })
        .with_parameter(ParameterDef {
            name: "max_matches_per_file".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum matches per file (0 = unlimited)".to_string(),
            required: false,
            default: Some(json!(0)),
        })
        .with_parameter(ParameterDef {
            name: "max_depth".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum search depth for recursive searches".to_string(),
            required: false,
            default: Some(json!(20)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    use llmspell_security::sandbox::SandboxContext;
    use llmspell_testing::tool_helpers::create_test_tool_input;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs;

    fn create_test_file_search() -> (FileSearchTool, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        // Create sandbox context
        let security_requirements = SecurityRequirements {
            level: SecurityLevel::Restricted,
            file_permissions: vec!["*".to_string()],
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        };
        let resource_limits = ResourceLimits::default();
        let context = SandboxContext::new(
            "test_file_search".to_string(),
            security_requirements,
            resource_limits,
        );
        let sandbox = Arc::new(FileSandbox::new(context).unwrap());

        let config = FileSearchConfig::default();
        let tool = FileSearchTool::new(config, sandbox);

        (tool, temp_dir)
    }

    #[tokio::test]
    async fn test_search_single_file() {
        let (tool, temp_dir) = create_test_file_search();

        // Create test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(
            &test_file,
            "Line 1\nThis contains the PATTERN\nLine 3\nAnother line with pattern\nLine 5",
        )
        .await
        .unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "pattern"),
            ("path", &test_file.to_string_lossy()),
            ("case_sensitive", "false"),
            ("context_lines", "1"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Found 2 matches"));
        // Just check that the search succeeded and has some response
        assert!(!result.text.is_empty());
    }
    #[tokio::test]
    async fn test_search_directory_recursive() {
        let (tool, temp_dir) = create_test_file_search();

        // Create directory structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).await.unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");

        fs::write(&file1, "TODO: Fix this bug\nNormal line")
            .await
            .unwrap();
        fs::write(&file2, "Another TODO item\nFinal line")
            .await
            .unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "TODO"),
            ("path", &temp_dir.path().to_string_lossy()),
            ("recursive", "true"),
            ("context_lines", "0"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Found 2 matches"));
        assert!(result.text.contains("2 files"));
    }
    #[tokio::test]
    async fn test_search_with_regex() {
        let (tool, temp_dir) = create_test_file_search();

        let test_file = temp_dir.path().join("test.log");
        fs::write(
            &test_file,
            "ERROR: Something failed\nINFO: All good\nWARNING: Check this\nERROR: Another issue",
        )
        .await
        .unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "(ERROR|INFO|WARNING)"),
            ("path", &test_file.to_string_lossy()),
            ("use_regex", "true"),
            ("context_lines", "0"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Found 4 matches"));
    }
    #[tokio::test]
    async fn test_search_with_file_extension_filter() {
        let (tool, temp_dir) = create_test_file_search();

        // Create files with different extensions
        let txt_file = temp_dir.path().join("test.txt");
        let rs_file = temp_dir.path().join("test.rs");
        let bin_file = temp_dir.path().join("test.bin");

        fs::write(&txt_file, "pattern in txt").await.unwrap();
        fs::write(&rs_file, "pattern in rust").await.unwrap();
        fs::write(&bin_file, "pattern in binary").await.unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "pattern"),
            ("path", &temp_dir.path().to_string_lossy()),
            ("recursive", "false"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Found 2 matches")); // Only txt and rs files
    }
    #[tokio::test]
    async fn test_search_with_context_lines() {
        let (tool, temp_dir) = create_test_file_search();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(
            &test_file,
            "Line 1\nLine 2\nLine with MATCH\nLine 4\nLine 5",
        )
        .await
        .unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "MATCH"),
            ("path", &test_file.to_string_lossy()),
            ("context_lines", "2"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Found 1 matches"));
        // Just check that the search succeeded and has some response
        assert!(!result.text.is_empty());
    }
    #[tokio::test]
    async fn test_search_nonexistent_path() {
        let (tool, temp_dir) = create_test_file_search();

        let nonexistent = temp_dir.path().join("nonexistent.txt");

        let input = create_test_tool_input(vec![
            ("pattern", "test"),
            ("path", &nonexistent.to_string_lossy()),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
    #[tokio::test]
    async fn test_search_empty_pattern() {
        let (tool, temp_dir) = create_test_file_search();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content").await.unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", ""),
            ("path", &test_file.to_string_lossy()),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Pattern cannot be empty"));
    }
    #[tokio::test]
    async fn test_search_invalid_regex() {
        let (tool, temp_dir) = create_test_file_search();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content").await.unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "[invalid"),
            ("path", &test_file.to_string_lossy()),
            ("use_regex", "true"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_search_no_matches() {
        let (tool, temp_dir) = create_test_file_search();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "This file has no matches for the pattern")
            .await
            .unwrap();

        let input = create_test_tool_input(vec![
            ("pattern", "NONEXISTENT"),
            ("path", &test_file.to_string_lossy()),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("No matches found"));
        assert!(result.text.contains("1 files searched"));
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let (tool, _temp_dir) = create_test_file_search();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "file-search");
        assert_eq!(metadata.version, llmspell_core::Version::new(0, 1, 0));
        assert!(metadata.description.contains("File content search"));

        let schema = tool.schema();
        assert_eq!(schema.name, "file-search");
        assert_eq!(tool.category(), ToolCategory::Filesystem);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);

        // Check required parameters
        let required_params = schema.required_parameters();
        assert!(required_params.contains(&"pattern".to_string()));
        assert!(required_params.contains(&"path".to_string()));
        assert_eq!(required_params.len(), 2);
    }
    #[tokio::test]
    async fn test_parameter_validation() {
        let (tool, _temp_dir) = create_test_file_search();

        // Missing pattern
        let input1 = create_test_tool_input(vec![("path", "/tmp/test.txt")]);
        let result1 = tool.execute(input1, ExecutionContext::default()).await;
        assert!(result1.is_err());
        assert!(result1
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter"));

        // Missing path
        let input2 = create_test_tool_input(vec![("pattern", "test")]);
        let result2 = tool.execute(input2, ExecutionContext::default()).await;
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter"));

        // Invalid context_lines
        let input3 = create_test_tool_input(vec![
            ("pattern", "test"),
            ("path", "/tmp/test.txt"),
            ("context_lines", "100"),
        ]);
        let result3 = tool.execute(input3, ExecutionContext::default()).await;
        assert!(result3.is_err());
        assert!(result3
            .unwrap_err()
            .to_string()
            .contains("Context lines cannot exceed 50"));
    }
}
