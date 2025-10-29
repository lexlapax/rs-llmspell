//! File Classification Template - Scan-Classify-Act Pattern
//!
//! Provides automated file organization with multiple classification strategies.
//! Supports extension-based, content-based, and AI-based classification with
//! dry-run mode for safe preview before execution.

use crate::context::ExecutionContext;
use crate::core::{
    memory_parameters, provider_parameters, CostEstimate, TemplateCategory, TemplateMetadata,
    TemplateOutput, TemplateParams, TemplateResult,
};
use crate::error::{TemplateError, ValidationError};
use crate::validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Category definition for file classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub extensions: Vec<String>,
    pub keywords: Vec<String>,
    pub destination: Option<String>,
}

/// Classification result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClassificationResult {
    file_path: String,
    category: String,
    confidence: f32,
    destination: String,
    action: String,
}

/// File Classification Template
#[derive(Debug)]
pub struct FileClassificationTemplate {
    metadata: TemplateMetadata,
}

impl Default for FileClassificationTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl FileClassificationTemplate {
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "file-classification".to_string(),
                name: "File Classification".to_string(),
                description: "Automated file organization with scan-classify-act pattern. Supports multiple classification strategies (extension, content, AI-based) with dry-run mode for safe preview.".to_string(),
                category: TemplateCategory::Custom("Productivity".to_string()),
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec![],
                tags: vec![
                    "files".to_string(),
                    "organization".to_string(),
                    "classification".to_string(),
                    "automation".to_string(),
                    "productivity".to_string(),
                ],
            },
        }
    }

    /// Get predefined category presets
    fn get_category_preset(&self, preset: &str) -> Vec<Category> {
        match preset {
            "documents" => vec![
                Category {
                    name: "PDFs".to_string(),
                    extensions: vec![".pdf".to_string()],
                    keywords: vec![],
                    destination: Some("Documents/PDFs".to_string()),
                },
                Category {
                    name: "Word Documents".to_string(),
                    extensions: vec![".doc".to_string(), ".docx".to_string()],
                    keywords: vec![],
                    destination: Some("Documents/Word".to_string()),
                },
                Category {
                    name: "Spreadsheets".to_string(),
                    extensions: vec![".xls".to_string(), ".xlsx".to_string(), ".csv".to_string()],
                    keywords: vec![],
                    destination: Some("Documents/Spreadsheets".to_string()),
                },
                Category {
                    name: "Text Files".to_string(),
                    extensions: vec![".txt".to_string(), ".md".to_string(), ".rtf".to_string()],
                    keywords: vec![],
                    destination: Some("Documents/Text".to_string()),
                },
                Category {
                    name: "Presentations".to_string(),
                    extensions: vec![".ppt".to_string(), ".pptx".to_string()],
                    keywords: vec![],
                    destination: Some("Documents/Presentations".to_string()),
                },
            ],
            "media" => vec![
                Category {
                    name: "Photos".to_string(),
                    extensions: vec![
                        ".jpg".to_string(),
                        ".jpeg".to_string(),
                        ".png".to_string(),
                        ".gif".to_string(),
                        ".bmp".to_string(),
                        ".webp".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Media/Photos".to_string()),
                },
                Category {
                    name: "Videos".to_string(),
                    extensions: vec![
                        ".mp4".to_string(),
                        ".avi".to_string(),
                        ".mov".to_string(),
                        ".mkv".to_string(),
                        ".wmv".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Media/Videos".to_string()),
                },
                Category {
                    name: "Audio".to_string(),
                    extensions: vec![
                        ".mp3".to_string(),
                        ".wav".to_string(),
                        ".flac".to_string(),
                        ".aac".to_string(),
                        ".m4a".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Media/Audio".to_string()),
                },
            ],
            "code" => vec![
                Category {
                    name: "Rust".to_string(),
                    extensions: vec![".rs".to_string(), ".toml".to_string()],
                    keywords: vec![],
                    destination: Some("Code/Rust".to_string()),
                },
                Category {
                    name: "Python".to_string(),
                    extensions: vec![".py".to_string(), ".pyw".to_string()],
                    keywords: vec![],
                    destination: Some("Code/Python".to_string()),
                },
                Category {
                    name: "JavaScript".to_string(),
                    extensions: vec![
                        ".js".to_string(),
                        ".jsx".to_string(),
                        ".ts".to_string(),
                        ".tsx".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Code/JavaScript".to_string()),
                },
                Category {
                    name: "Go".to_string(),
                    extensions: vec![".go".to_string()],
                    keywords: vec![],
                    destination: Some("Code/Go".to_string()),
                },
                Category {
                    name: "Other Code".to_string(),
                    extensions: vec![
                        ".java".to_string(),
                        ".c".to_string(),
                        ".cpp".to_string(),
                        ".h".to_string(),
                        ".hpp".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Code/Other".to_string()),
                },
            ],
            "downloads" => vec![
                Category {
                    name: "Archives".to_string(),
                    extensions: vec![
                        ".zip".to_string(),
                        ".tar".to_string(),
                        ".gz".to_string(),
                        ".rar".to_string(),
                        ".7z".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Downloads/Archives".to_string()),
                },
                Category {
                    name: "Installers".to_string(),
                    extensions: vec![
                        ".exe".to_string(),
                        ".dmg".to_string(),
                        ".pkg".to_string(),
                        ".deb".to_string(),
                        ".rpm".to_string(),
                    ],
                    keywords: vec![],
                    destination: Some("Downloads/Installers".to_string()),
                },
                Category {
                    name: "Documents".to_string(),
                    extensions: vec![".pdf".to_string(), ".doc".to_string(), ".docx".to_string()],
                    keywords: vec![],
                    destination: Some("Downloads/Documents".to_string()),
                },
            ],
            _ => vec![],
        }
    }

    /// Scan directory for files
    fn scan_files(
        &self,
        source_path: &str,
        recursive: bool,
    ) -> Result<Vec<PathBuf>, TemplateError> {
        let path = Path::new(source_path);

        if !path.exists() {
            return Err(ValidationError::invalid_value(
                "source_path",
                format!("Source path does not exist: {}", source_path),
            )
            .into());
        }

        let mut files = Vec::new();

        if path.is_file() {
            // Single file
            files.push(path.to_path_buf());
        } else if path.is_dir() {
            // Directory - scan for files
            Self::scan_directory(path, recursive, &mut files)?;
        } else {
            return Err(ValidationError::invalid_value(
                "source_path",
                format!("Invalid path (not file or directory): {}", source_path),
            )
            .into());
        }

        Ok(files)
    }

    /// Recursively scan directory
    fn scan_directory(
        dir: &Path,
        recursive: bool,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        use std::fs;

        let entries = fs::read_dir(dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to read directory {:?}: {}", dir, e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                TemplateError::ExecutionFailed(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() && recursive {
                Self::scan_directory(&path, recursive, files)?;
            }
        }

        Ok(())
    }

    /// Classify file using extension-based strategy
    fn classify_extension_based(
        &self,
        file_path: &Path,
        categories: &[Category],
    ) -> Option<(String, f32)> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext.to_lowercase()));

        if let Some(ext) = extension {
            for category in categories {
                if category.extensions.contains(&ext) {
                    return Some((category.name.clone(), 1.0));
                }
            }
        }

        None
    }

    /// Classify file using content-based strategy
    fn classify_content_based(
        &self,
        file_path: &Path,
        categories: &[Category],
    ) -> Option<(String, f32)> {
        use std::fs;

        // Try to read first 1KB of file
        let content = fs::read_to_string(file_path).ok()?;
        let content_lower = content.to_lowercase();

        for category in categories {
            if category.keywords.is_empty() {
                continue;
            }

            let mut matches = 0;
            for keyword in &category.keywords {
                if content_lower.contains(&keyword.to_lowercase()) {
                    matches += 1;
                }
            }

            if matches > 0 {
                let confidence = matches as f32 / category.keywords.len() as f32;
                return Some((category.name.clone(), confidence));
            }
        }

        None
    }

    /// Classify file using the specified strategy
    fn classify_file(
        &self,
        file_path: &Path,
        strategy: &str,
        categories: &[Category],
    ) -> (String, f32) {
        match strategy {
            "extension" => {
                if let Some((category, confidence)) =
                    self.classify_extension_based(file_path, categories)
                {
                    (category, confidence)
                } else {
                    ("Uncategorized".to_string(), 0.0)
                }
            }
            "content" => {
                // Try content-based first
                if let Some((category, confidence)) =
                    self.classify_content_based(file_path, categories)
                {
                    (category, confidence)
                } else if let Some((category, confidence)) =
                    self.classify_extension_based(file_path, categories)
                {
                    // Fallback to extension
                    (category, confidence * 0.8)
                } else {
                    ("Uncategorized".to_string(), 0.0)
                }
            }
            "hybrid" => {
                // Try extension first (fast)
                if let Some((category, confidence)) =
                    self.classify_extension_based(file_path, categories)
                {
                    (category, confidence)
                } else if let Some((category, confidence)) =
                    self.classify_content_based(file_path, categories)
                {
                    // Fallback to content
                    (category, confidence)
                } else {
                    ("Uncategorized".to_string(), 0.0)
                }
            }
            _ => {
                // Default to extension-based
                self.classify_extension_based(file_path, categories)
                    .unwrap_or_else(|| ("Uncategorized".to_string(), 0.0))
            }
        }
    }

    /// Get destination path for a category
    fn get_destination_path(
        &self,
        category: &str,
        categories: &[Category],
        destination_base: Option<&str>,
    ) -> String {
        // Find category definition
        for cat in categories {
            if cat.name == category {
                if let Some(dest) = &cat.destination {
                    return if let Some(base) = destination_base {
                        format!("{}/{}", base, dest)
                    } else {
                        dest.clone()
                    };
                }
            }
        }

        // Fallback: use category name
        if let Some(base) = destination_base {
            format!("{}/{}", base, category)
        } else {
            category.to_string()
        }
    }

    /// Execute action on classified file
    fn execute_action(
        &self,
        file_path: &Path,
        destination: &str,
        action: &str,
        dry_run: bool,
    ) -> Result<String, TemplateError> {
        use std::fs;

        if dry_run {
            // Dry-run mode: just return what would happen
            return Ok(format!(
                "[DRY-RUN] Would {} {} to {}",
                action,
                file_path.display(),
                destination
            ));
        }

        match action {
            "move" => {
                // Create destination directory
                let dest_path = Path::new(destination);
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        TemplateError::ExecutionFailed(format!(
                            "Failed to create destination directory: {}",
                            e
                        ))
                    })?;
                }

                // Move file
                let final_dest = dest_path
                    .join(file_path.file_name().ok_or_else(|| {
                        TemplateError::ExecutionFailed("Invalid file name".to_string())
                    })?)
                    .to_string_lossy()
                    .to_string();

                fs::rename(file_path, &final_dest).map_err(|e| {
                    TemplateError::ExecutionFailed(format!("Failed to move file: {}", e))
                })?;

                Ok(format!("Moved {} to {}", file_path.display(), final_dest))
            }
            "copy" => {
                // Create destination directory
                let dest_path = Path::new(destination);
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        TemplateError::ExecutionFailed(format!(
                            "Failed to create destination directory: {}",
                            e
                        ))
                    })?;
                }

                // Copy file
                let final_dest = dest_path
                    .join(file_path.file_name().ok_or_else(|| {
                        TemplateError::ExecutionFailed("Invalid file name".to_string())
                    })?)
                    .to_string_lossy()
                    .to_string();

                fs::copy(file_path, &final_dest).map_err(|e| {
                    TemplateError::ExecutionFailed(format!("Failed to copy file: {}", e))
                })?;

                Ok(format!("Copied {} to {}", file_path.display(), final_dest))
            }
            "report" => {
                // Report-only mode: just log classification
                Ok(format!(
                    "Classified {} → {}",
                    file_path.display(),
                    destination
                ))
            }
            _ => Err(ValidationError::invalid_value(
                "action",
                format!("Unknown action: {}", action),
            )
            .into()),
        }
    }

    /// Generate classification report
    fn generate_report(
        &self,
        results: &[ClassificationResult],
        output_format: &str,
    ) -> Result<String, TemplateError> {
        match output_format {
            "json" => self.format_json_report(results),
            "markdown" => Ok(self.format_markdown_report(results)),
            _ => Ok(self.format_text_report(results)),
        }
    }

    /// Format report as JSON
    fn format_json_report(
        &self,
        results: &[ClassificationResult],
    ) -> Result<String, TemplateError> {
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            *category_counts.entry(result.category.clone()).or_insert(0) += 1;
        }

        let output = json!({
            "total_files": results.len(),
            "categories": category_counts,
            "classifications": results
        });

        serde_json::to_string_pretty(&output).map_err(|e| {
            TemplateError::ExecutionFailed(format!("JSON serialization failed: {}", e))
        })
    }

    /// Format report as markdown
    fn format_markdown_report(&self, results: &[ClassificationResult]) -> String {
        let mut output = String::from("# File Classification Report\n\n");

        // Summary
        output.push_str(&format!("**Total Files**: {}\n\n", results.len()));

        // Category breakdown
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            *category_counts.entry(result.category.clone()).or_insert(0) += 1;
        }

        output.push_str("## Category Breakdown\n\n");
        for (category, count) in &category_counts {
            output.push_str(&format!("- **{}**: {} files\n", category, count));
        }

        // Detailed results
        output.push_str("\n## Detailed Classifications\n\n");
        for result in results {
            output.push_str(&format!(
                "- `{}` → **{}** (confidence: {:.2}) - {}\n",
                result.file_path, result.category, result.confidence, result.destination
            ));
        }

        output
    }

    /// Format report as plain text
    fn format_text_report(&self, results: &[ClassificationResult]) -> String {
        let mut output = String::from("=== FILE CLASSIFICATION REPORT ===\n\n");

        output.push_str(&format!("Total Files: {}\n\n", results.len()));

        // Category breakdown
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            *category_counts.entry(result.category.clone()).or_insert(0) += 1;
        }

        output.push_str("CATEGORY BREAKDOWN:\n");
        for (category, count) in &category_counts {
            output.push_str(&format!("  {} → {} files\n", category, count));
        }

        output.push_str("\nDETAILED CLASSIFICATIONS:\n");
        for result in results {
            output.push_str(&format!(
                "  {} → {} (confidence: {:.2})\n    Destination: {}\n",
                result.file_path, result.category, result.confidence, result.destination
            ));
        }

        output
    }
}

#[async_trait]
impl crate::core::Template for FileClassificationTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        let mut params = vec![
            // source_path (required)
            ParameterSchema::required(
                "source_path",
                "Directory or file path to classify",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(1),
                ..Default::default()
            }),
            // classification_strategy (optional enum with default)
            ParameterSchema::optional(
                "classification_strategy",
                "Classification method",
                ParameterType::String,
                json!("extension"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![json!("extension"), json!("content"), json!("hybrid")]),
                ..Default::default()
            }),
            // category_preset (optional enum)
            ParameterSchema::optional(
                "category_preset",
                "Predefined category set",
                ParameterType::String,
                json!(null),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("documents"),
                    json!("media"),
                    json!("code"),
                    json!("downloads"),
                ]),
                ..Default::default()
            }),
            // action (optional enum with default)
            ParameterSchema::optional(
                "action",
                "Action to perform on files",
                ParameterType::String,
                json!("report"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![json!("move"), json!("copy"), json!("report")]),
                ..Default::default()
            }),
            // destination_base (optional)
            ParameterSchema::optional(
                "destination_base",
                "Base path for move/copy actions",
                ParameterType::String,
                json!(null),
            ),
            // dry_run (optional bool with default)
            ParameterSchema::optional(
                "dry_run",
                "Preview without executing actions",
                ParameterType::Boolean,
                json!(true),
            ),
            // recursive (optional bool with default)
            ParameterSchema::optional(
                "recursive",
                "Scan subdirectories",
                ParameterType::Boolean,
                json!(false),
            ),
            // output_format (optional enum with default)
            ParameterSchema::optional(
                "output_format",
                "Report format",
                ParameterType::String,
                json!("text"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![json!("text"), json!("markdown"), json!("json")]),
                ..Default::default()
            }),
        ];

        // Add memory parameters (Task 13.11.1)
        params.extend(memory_parameters());

        // Add provider parameters (Task 13.5.7d)
        params.extend(provider_parameters());

        tracing::debug!(
            "FileClassification: Generated config schema with {} parameters",
            params.len()
        );
        ConfigSchema::new(params)
    }

    async fn execute(
        &self,
        params: TemplateParams,
        _context: ExecutionContext,
    ) -> Result<TemplateOutput, TemplateError> {
        let start_time = std::time::Instant::now();
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params.clone(),
        );

        // Extract parameters
        let source_path: String = params.get("source_path")?;
        let classification_strategy: String =
            params.get_or("classification_strategy", "extension".to_string());
        let category_preset: Option<String> = params.get_optional("category_preset");
        let action: String = params.get_or("action", "report".to_string());
        let destination_base: Option<String> = params.get_optional("destination_base");
        let dry_run: bool = params.get_or("dry_run", true);
        let recursive: bool = params.get_or("recursive", false);
        let output_format: String = params.get_or("output_format", "text".to_string());

        info!(
            "Starting file classification: source={}, strategy={}, action={}, dry_run={}",
            source_path, classification_strategy, action, dry_run
        );

        // Get categories
        let categories = if let Some(preset) = &category_preset {
            self.get_category_preset(preset)
        } else {
            // Default to documents preset
            self.get_category_preset("documents")
        };

        if categories.is_empty() {
            return Err(ValidationError::invalid_value(
                "category_preset",
                "No categories defined (invalid preset or no custom categories)",
            )
            .into());
        }

        info!(
            "Using {} categories: {:?}",
            categories.len(),
            categories.iter().map(|c| &c.name).collect::<Vec<_>>()
        );

        // Phase 1: Scan files
        info!("Phase 1: Scanning files from {}", source_path);
        let files = self.scan_files(&source_path, recursive)?;
        info!("Found {} files to classify", files.len());

        // Phase 2: Classify files
        info!(
            "Phase 2: Classifying files using {} strategy",
            classification_strategy
        );
        let mut results = Vec::new();

        for (index, file_path) in files.iter().enumerate() {
            if index > 0 && index % 10 == 0 {
                info!("Progress: {}/{} files classified", index, files.len());
            }

            let (category, confidence) =
                self.classify_file(file_path, &classification_strategy, &categories);

            let destination =
                self.get_destination_path(&category, &categories, destination_base.as_deref());

            results.push(ClassificationResult {
                file_path: file_path.to_string_lossy().to_string(),
                category: category.clone(),
                confidence,
                destination: destination.clone(),
                action: action.clone(),
            });
        }

        info!(
            "Classification complete: {} files classified",
            results.len()
        );

        // Phase 3: Execute actions (if not report-only)
        if action != "report" {
            info!(
                "Phase 3: Executing {} actions (dry_run={})",
                action, dry_run
            );

            for result in &results {
                let file_path = Path::new(&result.file_path);
                match self.execute_action(file_path, &result.destination, &action, dry_run) {
                    Ok(message) => {
                        info!("{}", message);
                    }
                    Err(e) => {
                        warn!("Action failed for {}: {}", result.file_path, e);
                    }
                }
            }
        }

        // Generate report
        let report = self.generate_report(&results, &output_format)?;

        // Set output
        output.result = TemplateResult::text(report);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("files_scanned", json!(files.len()));
        output.add_metric("files_classified", json!(results.len()));
        output.add_metric("dry_run", json!(dry_run));
        output.add_metric("classification_strategy", json!(classification_strategy));

        // Category breakdown
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for result in &results {
            *category_counts.entry(result.category.clone()).or_insert(0) += 1;
        }
        output.add_metric("category_breakdown", json!(category_counts));

        info!(
            "File classification complete (duration: {}ms, files: {})",
            output.metrics.duration_ms,
            results.len()
        );

        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        let recursive: bool = params.get_or("recursive", false);

        // Cost estimation for file classification
        // This is mainly I/O bound, minimal token usage (no LLM calls in current implementation)
        let base_tokens = 100; // Minimal overhead

        // Estimate based on strategy
        let strategy: String = params.get_or("classification_strategy", "extension".to_string());
        let estimated_tokens = match strategy.as_str() {
            "extension" => base_tokens,     // Fast, no content reading
            "content" => base_tokens + 500, // Content reading overhead
            "hybrid" => base_tokens + 250,  // Mix of both
            _ => base_tokens,
        };

        // Duration estimates (in milliseconds)
        let base_duration = if recursive { 1000 } else { 500 };
        let estimated_duration = base_duration
            + match strategy.as_str() {
                "extension" => 100,
                "content" => 500,
                "hybrid" => 300,
                _ => 100,
            };

        CostEstimate::new(estimated_tokens, 0.0, estimated_duration, 0.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create test directory structure
    fn create_test_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test files with different extensions
        fs::write(base_path.join("document.pdf"), b"PDF content").unwrap();
        fs::write(base_path.join("report.docx"), b"Word document").unwrap();
        fs::write(base_path.join("data.xlsx"), b"Excel spreadsheet").unwrap();
        fs::write(base_path.join("notes.txt"), b"Plain text file").unwrap();
        fs::write(base_path.join("photo.jpg"), b"JPEG image").unwrap();
        fs::write(base_path.join("video.mp4"), b"MP4 video").unwrap();
        fs::write(base_path.join("song.mp3"), b"MP3 audio").unwrap();
        fs::write(base_path.join("script.rs"), b"fn main() {}").unwrap();
        fs::write(base_path.join("config.toml"), b"[package]").unwrap();
        fs::write(base_path.join("app.py"), b"print('hello')").unwrap();
        fs::write(base_path.join("unknown.xyz"), b"Unknown file").unwrap();

        // Create subdirectory for recursive tests
        let sub_dir = base_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("nested.pdf"), b"Nested PDF").unwrap();
        fs::write(sub_dir.join("deep.txt"), b"Deep text").unwrap();

        temp_dir
    }

    #[test]
    fn test_template_metadata() {
        let template = FileClassificationTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "file-classification");
        assert_eq!(metadata.name, "File Classification");
        assert!(metadata.description.contains("scan-classify-act"));
        assert_eq!(metadata.version, "0.1.0");
        assert!(metadata.tags.contains(&"files".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = FileClassificationTemplate::new();
        let schema = template.config_schema();

        assert!(!schema.parameters.is_empty());

        // Verify required parameters
        let source_path = schema.parameters.iter().find(|p| p.name == "source_path");
        assert!(source_path.is_some());
        assert!(source_path.unwrap().required);

        // Verify optional parameters with defaults
        let strategy = schema
            .parameters
            .iter()
            .find(|p| p.name == "classification_strategy");
        assert!(strategy.is_some());
        assert!(!strategy.unwrap().required);

        let dry_run = schema.parameters.iter().find(|p| p.name == "dry_run");
        assert!(dry_run.is_some());
        assert!(!dry_run.unwrap().required);
    }

    #[test]
    fn test_category_preset_documents() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("documents");

        assert!(!categories.is_empty());
        assert!(categories.len() >= 4); // PDFs, Word, Spreadsheets, Text, Presentations

        // Verify PDFs category
        let pdf_cat = categories.iter().find(|c| c.name == "PDFs");
        assert!(pdf_cat.is_some());
        let pdf_cat = pdf_cat.unwrap();
        assert!(pdf_cat.extensions.contains(&".pdf".to_string()));
        assert!(pdf_cat.destination.is_some());
    }

    #[test]
    fn test_category_preset_media() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("media");

        assert!(!categories.is_empty());

        // Verify Photos category
        let photos = categories.iter().find(|c| c.name == "Photos");
        assert!(photos.is_some());
        assert!(photos.unwrap().extensions.contains(&".jpg".to_string()));
    }

    #[test]
    fn test_category_preset_code() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("code");

        assert!(!categories.is_empty());

        // Verify Rust category
        let rust_cat = categories.iter().find(|c| c.name == "Rust");
        assert!(rust_cat.is_some());
        assert!(rust_cat.unwrap().extensions.contains(&".rs".to_string()));
    }

    #[test]
    fn test_category_preset_downloads() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("downloads");

        assert!(!categories.is_empty());

        // Verify Archives category
        let archives = categories.iter().find(|c| c.name == "Archives");
        assert!(archives.is_some());
        assert!(archives.unwrap().extensions.contains(&".zip".to_string()));
    }

    #[test]
    fn test_classify_extension_based() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("documents");

        let temp_dir = TempDir::new().unwrap();
        let pdf_file = temp_dir.path().join("test.pdf");
        fs::write(&pdf_file, b"PDF content").unwrap();

        let (category, confidence) = template
            .classify_extension_based(&pdf_file, &categories)
            .unwrap();
        assert_eq!(category, "PDFs");
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_classify_extension_based_uncategorized() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("documents");

        let temp_dir = TempDir::new().unwrap();
        let unknown_file = temp_dir.path().join("test.xyz");
        fs::write(&unknown_file, b"Unknown content").unwrap();

        let result = template.classify_extension_based(&unknown_file, &categories);
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_content_based() {
        let template = FileClassificationTemplate::new();

        // Create category with keywords
        let categories = vec![Category {
            name: "Invoices".to_string(),
            extensions: vec![],
            keywords: vec!["invoice".to_string(), "payment".to_string()],
            destination: Some("Documents/Invoices".to_string()),
        }];

        let temp_dir = TempDir::new().unwrap();
        let invoice_file = temp_dir.path().join("document.txt");
        fs::write(&invoice_file, b"This is an invoice for payment").unwrap();

        let (category, confidence) = template
            .classify_content_based(&invoice_file, &categories)
            .unwrap();
        assert_eq!(category, "Invoices");
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_classify_file_extension_strategy() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("documents");

        let temp_dir = TempDir::new().unwrap();
        let pdf_file = temp_dir.path().join("test.pdf");
        fs::write(&pdf_file, b"PDF content").unwrap();

        let (category, confidence) = template.classify_file(&pdf_file, "extension", &categories);
        assert_eq!(category, "PDFs");
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_classify_file_hybrid_strategy() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("documents");

        let temp_dir = TempDir::new().unwrap();
        let pdf_file = temp_dir.path().join("test.pdf");
        fs::write(&pdf_file, b"PDF content").unwrap();

        let (category, confidence) = template.classify_file(&pdf_file, "hybrid", &categories);
        assert_eq!(category, "PDFs");
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_scan_files_single_file() {
        let template = FileClassificationTemplate::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"content").unwrap();

        let files = template
            .scan_files(&test_file.to_string_lossy(), false)
            .unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[test]
    fn test_scan_files_directory_non_recursive() {
        let temp_dir = create_test_directory();
        let template = FileClassificationTemplate::new();

        let files = template
            .scan_files(&temp_dir.path().to_string_lossy(), false)
            .unwrap();

        // Should find 11 files in root (not including subdirectory files)
        assert!(files.len() >= 10);
        assert!(files
            .iter()
            .any(|f| f.file_name().unwrap() == "document.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "script.rs"));

        // Should NOT include nested files
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "nested.pdf"));
    }

    #[test]
    fn test_scan_files_directory_recursive() {
        let temp_dir = create_test_directory();
        let template = FileClassificationTemplate::new();

        let files = template
            .scan_files(&temp_dir.path().to_string_lossy(), true)
            .unwrap();

        // Should find all files including subdirectory
        assert!(files.len() >= 12); // 11 root + 2 nested

        // Should include nested files
        assert!(files.iter().any(|f| f.file_name().unwrap() == "nested.pdf"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "deep.txt"));
    }

    #[test]
    fn test_scan_files_nonexistent_path() {
        let template = FileClassificationTemplate::new();
        let result = template.scan_files("/nonexistent/path/to/nowhere", false);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TemplateError::ValidationFailed(_)
        ));
    }

    #[test]
    fn test_get_destination_path() {
        let template = FileClassificationTemplate::new();
        let categories = template.get_category_preset("documents");

        // With destination_base
        let dest = template.get_destination_path("PDFs", &categories, Some("/home/user"));
        assert_eq!(dest, "/home/user/Documents/PDFs");

        // Without destination_base
        let dest = template.get_destination_path("PDFs", &categories, None);
        assert_eq!(dest, "Documents/PDFs");

        // Unknown category with base
        let dest = template.get_destination_path("Unknown", &categories, Some("/home/user"));
        assert_eq!(dest, "/home/user/Unknown");

        // Unknown category without base
        let dest = template.get_destination_path("Unknown", &categories, None);
        assert_eq!(dest, "Unknown");
    }

    #[test]
    fn test_execute_action_dry_run() {
        let template = FileClassificationTemplate::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.pdf");
        fs::write(&test_file, b"PDF content").unwrap();

        let result = template.execute_action(&test_file, "Documents/PDFs", "move", true);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.contains("[DRY-RUN]"));
        assert!(message.contains("move"));

        // File should still exist (dry-run)
        assert!(test_file.exists());
    }

    #[test]
    fn test_execute_action_report() {
        let template = FileClassificationTemplate::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.pdf");
        fs::write(&test_file, b"PDF content").unwrap();

        let result = template.execute_action(&test_file, "Documents/PDFs", "report", false);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.contains("Classified"));
    }

    #[test]
    fn test_execute_action_unknown_action() {
        let template = FileClassificationTemplate::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.pdf");
        fs::write(&test_file, b"PDF content").unwrap();

        let result = template.execute_action(&test_file, "Documents/PDFs", "invalid_action", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_text_report() {
        let template = FileClassificationTemplate::new();
        let results = vec![
            ClassificationResult {
                file_path: "/tmp/file1.pdf".to_string(),
                category: "PDFs".to_string(),
                confidence: 1.0,
                destination: "Documents/PDFs".to_string(),
                action: "move".to_string(),
            },
            ClassificationResult {
                file_path: "/tmp/file2.txt".to_string(),
                category: "Text Files".to_string(),
                confidence: 1.0,
                destination: "Documents/Text".to_string(),
                action: "move".to_string(),
            },
        ];

        let report = template.format_text_report(&results);

        assert!(report.contains("FILE CLASSIFICATION REPORT"));
        assert!(report.contains("Total Files: 2"));
        assert!(report.contains("PDFs"));
        assert!(report.contains("Text Files"));
        assert!(report.contains("/tmp/file1.pdf"));
    }

    #[test]
    fn test_format_markdown_report() {
        let template = FileClassificationTemplate::new();
        let results = vec![ClassificationResult {
            file_path: "/tmp/file1.pdf".to_string(),
            category: "PDFs".to_string(),
            confidence: 1.0,
            destination: "Documents/PDFs".to_string(),
            action: "move".to_string(),
        }];

        let report = template.format_markdown_report(&results);

        assert!(report.contains("# File Classification Report"));
        assert!(report.contains("**Total Files**: 1"));
        assert!(report.contains("## Category Breakdown"));
        assert!(report.contains("## Detailed Classifications"));
        assert!(report.contains("`/tmp/file1.pdf`"));
    }

    #[test]
    fn test_format_json_report() {
        let template = FileClassificationTemplate::new();
        let results = vec![ClassificationResult {
            file_path: "/tmp/file1.pdf".to_string(),
            category: "PDFs".to_string(),
            confidence: 1.0,
            destination: "Documents/PDFs".to_string(),
            action: "move".to_string(),
        }];

        let report = template.format_json_report(&results).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();

        assert_eq!(parsed["total_files"], 1);
        assert!(parsed["categories"].is_object());
        assert!(parsed["classifications"].is_array());
    }

    #[tokio::test]
    async fn test_estimate_cost_extension() {
        let template = FileClassificationTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("classification_strategy", json!("extension"));

        let estimate = template.estimate_cost(&params).await;

        assert!(estimate.estimated_tokens.unwrap_or(0) > 0);
        assert!(estimate.estimated_duration_ms.unwrap_or(0) > 0);
        assert!(estimate.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_estimate_cost_content() {
        let template = FileClassificationTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("classification_strategy", json!("content"));

        let estimate = template.estimate_cost(&params).await;

        // Content-based should have higher token estimate
        assert!(estimate.estimated_tokens.unwrap_or(0) > 100);
    }

    #[tokio::test]
    async fn test_estimate_cost_recursive() {
        let template = FileClassificationTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("recursive", json!(true));

        let estimate = template.estimate_cost(&params).await;

        // Recursive should have higher duration estimate
        assert!(estimate.estimated_duration_ms.unwrap_or(0) >= 1000);
    }

    #[test]
    fn test_default_impl() {
        let template1 = FileClassificationTemplate::new();
        let template2 = FileClassificationTemplate::default();

        assert_eq!(template1.metadata().id, template2.metadata().id);
    }
}
