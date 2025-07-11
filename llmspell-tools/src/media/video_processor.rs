// ABOUTME: Video processing tool for format detection, metadata extraction, and thumbnail generation
// ABOUTME: Provides basic video file operations without external dependencies

// TODO: Phase 3+ - Add advanced video processing functionality:
// - Integrate ffmpeg or gstreamer for full codec support
// - Implement video transcoding with configurable settings
// - Add frame extraction at specific timestamps
// - Support video trimming and concatenation
// - Extract full metadata (codec, bitrate, framerate, audio tracks)
// - Add subtitle extraction and manipulation
// - Support streaming processing for large files

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::SandboxContext;
use llmspell_utils::{
    extract_optional_f64, extract_optional_string, extract_parameters, extract_required_string,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Video format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    Mp4,
    Avi,
    Mkv,
    Mov,
    Webm,
    Flv,
    Wmv,
    Unknown,
}

impl VideoFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("mp4") => Self::Mp4,
            Some("avi") => Self::Avi,
            Some("mkv") => Self::Mkv,
            Some("mov") => Self::Mov,
            Some("webm") => Self::Webm,
            Some("flv") => Self::Flv,
            Some("wmv") => Self::Wmv,
            _ => Self::Unknown,
        }
    }
}

/// Video resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResolution {
    pub width: u32,
    pub height: u32,
}

impl VideoResolution {
    /// Get resolution name (e.g., "1080p", "4K")
    pub fn name(&self) -> &'static str {
        match (self.width, self.height) {
            (w, h) if w >= 7680 && h >= 4320 => "8K",
            (w, h) if w >= 3840 && h >= 2160 => "4K",
            (w, h) if w >= 2560 && h >= 1440 => "1440p",
            (w, h) if w >= 1920 && h >= 1080 => "1080p",
            (w, h) if w >= 1280 && h >= 720 => "720p",
            (w, h) if w >= 854 && h >= 480 => "480p",
            (w, h) if w >= 640 && h >= 360 => "360p",
            _ => "SD",
        }
    }

    /// Get aspect ratio as a string
    pub fn aspect_ratio(&self) -> String {
        let gcd = self.gcd(self.width, self.height);
        let w = self.width / gcd;
        let h = self.height / gcd;
        format!("{}:{}", w, h)
    }

    fn gcd(&self, _a: u32, b: u32) -> u32 {
        let mut a = _a;
        let mut b = b;
        while b != 0 {
            let tmp = b;
            b = a % b;
            a = tmp;
        }
        a
    }
}

/// Video metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// Video format
    pub format: VideoFormat,
    /// Duration in seconds
    pub duration_seconds: Option<f64>,
    /// Video resolution
    pub resolution: Option<VideoResolution>,
    /// Frames per second
    pub fps: Option<f64>,
    /// Video bitrate in bits per second
    pub video_bitrate: Option<u32>,
    /// Audio bitrate in bits per second
    pub audio_bitrate: Option<u32>,
    /// Number of audio channels
    pub audio_channels: Option<u8>,
    /// File size in bytes
    pub file_size: u64,
    /// Video codec
    pub video_codec: Option<String>,
    /// Audio codec
    pub audio_codec: Option<String>,
}

/// Video processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoProcessorConfig {
    /// Maximum file size to process (in bytes)
    pub max_file_size: u64,
    /// Whether to extract detailed metadata
    pub extract_metadata: bool,
    /// Maximum thumbnail width
    pub max_thumbnail_width: u32,
    /// Maximum thumbnail height
    pub max_thumbnail_height: u32,
    /// Supported formats for detection
    pub supported_formats: Vec<VideoFormat>,
}

impl Default for VideoProcessorConfig {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024 * 1024, // 1GB
            extract_metadata: true,
            max_thumbnail_width: 1920,
            max_thumbnail_height: 1080,
            supported_formats: vec![
                VideoFormat::Mp4,
                VideoFormat::Avi,
                VideoFormat::Mkv,
                VideoFormat::Mov,
                VideoFormat::Webm,
            ],
        }
    }
}

/// Video processor tool for format detection and metadata extraction
#[derive(Clone)]
pub struct VideoProcessorTool {
    metadata: ComponentMetadata,
    config: VideoProcessorConfig,
    #[allow(dead_code)] // Reserved for future sandbox integration
    sandbox_context: Option<Arc<SandboxContext>>,
}

impl VideoProcessorTool {
    /// Create a new video processor tool
    pub fn new(config: VideoProcessorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "video_processor".to_string(),
                "Video file processing for format detection, metadata extraction, and thumbnails"
                    .to_string(),
            ),
            config,
            sandbox_context: None,
        }
    }

    /// Create a new video processor tool with sandbox context
    pub fn with_sandbox(
        config: VideoProcessorConfig,
        sandbox_context: Arc<SandboxContext>,
    ) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "video_processor".to_string(),
                "Video file processing for format detection, metadata extraction, and thumbnails"
                    .to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
        }
    }

    /// Detect video format from file
    async fn detect_format(&self, file_path: &Path) -> LLMResult<VideoFormat> {
        // First try extension-based detection
        let format = VideoFormat::from_extension(file_path);

        if format != VideoFormat::Unknown {
            debug!("Detected video format from extension: {:?}", format);
            return Ok(format);
        }

        // TODO: Phase 3+ - Implement container format detection by reading file headers
        // For now, we'll return Unknown for unsupported formats
        debug!("Could not detect video format for: {:?}", file_path);
        Ok(VideoFormat::Unknown)
    }

    /// Extract metadata from video file
    async fn extract_metadata(&self, file_path: &Path) -> LLMResult<VideoMetadata> {
        // Get file size
        let file_metadata = std::fs::metadata(file_path).map_err(|e| LLMSpellError::Tool {
            message: format!("Failed to read file metadata: {}", e),
            tool_name: Some("video_processor".to_string()),
            source: None,
        })?;

        let file_size = file_metadata.len();

        // Check file size limit
        if file_size > self.config.max_file_size {
            return Err(LLMSpellError::Tool {
                message: format!(
                    "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                    file_size, self.config.max_file_size
                ),
                tool_name: Some("video_processor".to_string()),
                source: None,
            });
        }

        // Detect format
        let format = self.detect_format(file_path).await?;

        // Create basic metadata
        let metadata = VideoMetadata {
            format: format.clone(),
            duration_seconds: None,
            resolution: None,
            fps: None,
            video_bitrate: None,
            audio_bitrate: None,
            audio_channels: None,
            file_size,
            video_codec: None,
            audio_codec: None,
        };

        // TODO: Phase 3+ - Extract actual metadata using video processing library
        // For now, we provide basic metadata based on format detection

        // For MP4 files, we could parse the basic atoms/boxes structure
        // but that would require significant implementation for Phase 2

        Ok(metadata)
    }

    /// Generate thumbnail from video
    async fn generate_thumbnail(
        &self,
        _video_path: &Path,
        _output_path: &Path,
        _timestamp_seconds: Option<f64>,
    ) -> LLMResult<()> {
        // TODO: Phase 3+ - Implement actual thumbnail generation
        // This would require video decoding capabilities

        // For now, we'll create a placeholder response
        Err(LLMSpellError::Tool {
            message: "Thumbnail generation is not implemented in this basic version. Video processing capabilities will be added in Phase 3+".to_string(),
            tool_name: Some("video_processor".to_string()),
            source: None,
        })
    }

    /// Extract frame at specific timestamp
    async fn extract_frame(
        &self,
        _video_path: &Path,
        _output_path: &Path,
        timestamp_seconds: f64,
    ) -> LLMResult<()> {
        // TODO: Phase 3+ - Implement actual frame extraction
        // This would require video decoding capabilities

        // For now, we'll create a placeholder response
        Err(LLMSpellError::Tool {
            message: format!(
                "Frame extraction at {}s is not implemented in this basic version. Video processing capabilities will be added in Phase 3+",
                timestamp_seconds
            ),
            tool_name: Some("video_processor".to_string()),
            source: None,
        })
    }

    /// Validate processing parameters
    async fn validate_parameters(&self, params: &serde_json::Value) -> LLMResult<()> {
        // Validate operation
        if let Some(operation) = extract_optional_string(params, "operation") {
            match operation {
                "detect" | "metadata" | "thumbnail" | "extract_frame" => {}
                _ => {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "Invalid operation: {}. Supported operations: detect, metadata, thumbnail, extract_frame",
                            operation
                        ),
                        field: Some("operation".to_string()),
                    });
                }
            }
        }

        // Validate file_path
        if let Some(file_path) = extract_optional_string(params, "file_path") {
            if file_path.is_empty() {
                return Err(LLMSpellError::Validation {
                    message: "File path cannot be empty".to_string(),
                    field: Some("file_path".to_string()),
                });
            }
        }

        // Validate thumbnail/frame extraction parameters
        if matches!(
            extract_optional_string(params, "operation"),
            Some("thumbnail") | Some("extract_frame")
        ) && params.get("output_path").is_none()
        {
            return Err(LLMSpellError::Validation {
                message: "output_path is required for thumbnail/extract_frame operations"
                    .to_string(),
                field: Some("output_path".to_string()),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl BaseAgent for VideoProcessorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        self.validate_parameters(params).await?;

        let operation = extract_optional_string(params, "operation").unwrap_or("metadata");

        match operation {
            "detect" => {
                let file_path = extract_required_string(params, "file_path")?;

                let path = Path::new(file_path);
                let format = self.detect_format(path).await?;

                let response = json!({
                    "operation": "detect",
                    "file_path": file_path,
                    "format": format,
                    "supported": format != VideoFormat::Unknown && self.config.supported_formats.contains(&format)
                });

                Ok(
                    AgentOutput::text(format!("Detected video format: {:?}", format))
                        .with_metadata(serde_json::from_value(response).unwrap_or_default()),
                )
            }

            "metadata" => {
                let file_path = extract_required_string(params, "file_path")?;

                let path = Path::new(file_path);
                let metadata = self.extract_metadata(path).await?;

                let response = json!({
                    "operation": "metadata",
                    "file_path": file_path,
                    "metadata": metadata
                });

                let mut message = format!("Video file: {:?} format", metadata.format);

                if let Some(resolution) = &metadata.resolution {
                    message.push_str(&format!(
                        ", Resolution: {}x{} ({})",
                        resolution.width,
                        resolution.height,
                        resolution.name()
                    ));
                }

                if let Some(duration) = metadata.duration_seconds {
                    message.push_str(&format!(", Duration: {:.1}s", duration));
                }

                message.push_str(&format!(", Size: {} bytes", metadata.file_size));

                Ok(AgentOutput::text(message)
                    .with_metadata(serde_json::from_value(response).unwrap_or_default()))
            }

            "thumbnail" => {
                let video_path = extract_required_string(params, "file_path")?;
                let output_path = extract_required_string(params, "output_path")?;
                let timestamp = extract_optional_f64(params, "timestamp_seconds");

                let video = Path::new(video_path);
                let output = Path::new(output_path);

                self.generate_thumbnail(video, output, timestamp).await?;

                let response = json!({
                    "operation": "thumbnail",
                    "video_path": video_path,
                    "output_path": output_path,
                    "timestamp_seconds": timestamp,
                    "success": true
                });

                Ok(AgentOutput::text(format!(
                    "Generated thumbnail from {} to {}",
                    video_path, output_path
                ))
                .with_metadata(serde_json::from_value(response).unwrap_or_default()))
            }

            "extract_frame" => {
                let video_path = extract_required_string(params, "file_path")?;
                let output_path = extract_required_string(params, "output_path")?;
                let timestamp = extract_optional_f64(params, "timestamp_seconds").unwrap_or(0.0);

                let video = Path::new(video_path);
                let output = Path::new(output_path);

                self.extract_frame(video, output, timestamp).await?;

                let response = json!({
                    "operation": "extract_frame",
                    "video_path": video_path,
                    "output_path": output_path,
                    "timestamp_seconds": timestamp,
                    "success": true
                });

                Ok(AgentOutput::text(format!(
                    "Extracted frame at {}s from {} to {}",
                    timestamp, video_path, output_path
                ))
                .with_metadata(serde_json::from_value(response).unwrap_or_default()))
            }

            _ => unreachable!(), // Already validated
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Video processor error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for VideoProcessorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Media
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "video_processor".to_string(),
            "Process video files for format detection, metadata extraction, and thumbnails"
                .to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: detect, metadata, thumbnail, extract_frame"
                .to_string(),
            required: false,
            default: Some(json!("metadata")),
        })
        .with_parameter(ParameterDef {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the video file".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "output_path".to_string(),
            param_type: ParameterType::String,
            description: "Output path for thumbnail or frame extraction".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "timestamp_seconds".to_string(),
            param_type: ParameterType::Number,
            description: "Timestamp in seconds for thumbnail/frame extraction".to_string(),
            required: false,
            default: Some(json!(0.0)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tool() -> VideoProcessorTool {
        let config = VideoProcessorConfig::default();
        VideoProcessorTool::new(config)
    }

    fn create_test_input(text: &str, params: serde_json::Value) -> AgentInput {
        AgentInput {
            text: text.to_string(),
            media: vec![],
            context: None,
            parameters: {
                let mut map = HashMap::new();
                map.insert("parameters".to_string(), params);
                map
            },
            output_modalities: vec![],
        }
    }

    #[tokio::test]
    async fn test_format_detection_by_extension() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();

        // Test various extensions
        let test_cases = vec![
            ("test.mp4", VideoFormat::Mp4),
            ("test.avi", VideoFormat::Avi),
            ("test.mkv", VideoFormat::Mkv),
            ("test.mov", VideoFormat::Mov),
            ("test.webm", VideoFormat::Webm),
            ("test.flv", VideoFormat::Flv),
            ("test.wmv", VideoFormat::Wmv),
            ("test.unknown", VideoFormat::Unknown),
        ];

        for (filename, expected_format) in test_cases {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, b"dummy").unwrap();

            let format = tool.detect_format(&file_path).await.unwrap();
            assert_eq!(format, expected_format);
        }
    }

    #[tokio::test]
    async fn test_resolution_naming() {
        let res_4k = VideoResolution {
            width: 3840,
            height: 2160,
        };
        assert_eq!(res_4k.name(), "4K");
        assert_eq!(res_4k.aspect_ratio(), "16:9");

        let res_1080p = VideoResolution {
            width: 1920,
            height: 1080,
        };
        assert_eq!(res_1080p.name(), "1080p");
        assert_eq!(res_1080p.aspect_ratio(), "16:9");

        let res_720p = VideoResolution {
            width: 1280,
            height: 720,
        };
        assert_eq!(res_720p.name(), "720p");
        assert_eq!(res_720p.aspect_ratio(), "16:9");

        let res_square = VideoResolution {
            width: 1000,
            height: 1000,
        };
        assert_eq!(res_square.aspect_ratio(), "1:1");
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("video.mp4");

        fs::write(&file_path, b"dummy mp4 content").unwrap();

        let input = create_test_input(
            "Extract metadata",
            json!({
                "operation": "metadata",
                "file_path": file_path.to_str().unwrap()
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Video file"));
        assert!(result.text.contains("Mp4"));
        assert!(result.text.contains("Size: 17 bytes"));
    }

    #[tokio::test]
    async fn test_format_detection_operation() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mkv");

        fs::write(&file_path, b"dummy").unwrap();

        let input = create_test_input(
            "Detect format",
            json!({
                "operation": "detect",
                "file_path": file_path.to_str().unwrap()
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Detected video format: Mkv"));
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let mut config = VideoProcessorConfig::default();
        config.max_file_size = 10; // Very small limit
        let tool = VideoProcessorTool::new(config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.mp4");

        // Create a file larger than the limit
        fs::write(&file_path, vec![0u8; 100]).unwrap();

        let input = create_test_input(
            "Extract metadata",
            json!({
                "operation": "metadata",
                "file_path": file_path.to_str().unwrap()
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[tokio::test]
    async fn test_thumbnail_not_implemented() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let video_path = temp_dir.path().join("video.mp4");
        let output_path = temp_dir.path().join("thumb.jpg");

        fs::write(&video_path, b"dummy").unwrap();

        let input = create_test_input(
            "Generate thumbnail",
            json!({
                "operation": "thumbnail",
                "file_path": video_path.to_str().unwrap(),
                "output_path": output_path.to_str().unwrap()
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not implemented"));
    }

    #[tokio::test]
    async fn test_extract_frame_not_implemented() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let video_path = temp_dir.path().join("video.mp4");
        let output_path = temp_dir.path().join("frame.jpg");

        fs::write(&video_path, b"dummy").unwrap();

        let input = create_test_input(
            "Extract frame",
            json!({
                "operation": "extract_frame",
                "file_path": video_path.to_str().unwrap(),
                "output_path": output_path.to_str().unwrap(),
                "timestamp_seconds": 5.0
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not implemented"));
        assert!(error_msg.contains("5s"));
    }

    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = create_test_tool();

        let input = create_test_input(
            "Invalid operation",
            json!({
                "operation": "invalid"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid operation"));
    }

    #[tokio::test]
    async fn test_missing_required_parameters() {
        let tool = create_test_tool();

        // Missing file_path for metadata operation
        let input = create_test_input(
            "Extract metadata",
            json!({
                "operation": "metadata"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter 'file_path'"));

        // Missing output_path for thumbnail
        let input = create_test_input(
            "Generate thumbnail",
            json!({
                "operation": "thumbnail",
                "file_path": "/tmp/video.mp4"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("output_path is required"));
    }

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_tool();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "video_processor");
        assert!(metadata.description.contains("Video file processing"));

        let schema = tool.schema();
        assert_eq!(schema.name, "video_processor");
        assert_eq!(tool.category(), ToolCategory::Media);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);

        // Check parameters
        let params = &schema.parameters;
        assert!(params.iter().any(|p| p.name == "operation"));
        assert!(params.iter().any(|p| p.name == "file_path"));
        assert!(params.iter().any(|p| p.name == "output_path"));
        assert!(params.iter().any(|p| p.name == "timestamp_seconds"));
    }

    #[tokio::test]
    async fn test_default_operation() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mp4");

        fs::write(&file_path, b"dummy").unwrap();

        // No operation specified, should default to metadata
        let input = create_test_input(
            "Process video",
            json!({
                "file_path": file_path.to_str().unwrap()
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Video file"));
    }

    #[tokio::test]
    async fn test_empty_file_path() {
        let tool = create_test_tool();

        let input = create_test_input(
            "Detect format",
            json!({
                "operation": "detect",
                "file_path": ""
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_supported_formats() {
        let config = VideoProcessorConfig::default();
        assert!(config.supported_formats.contains(&VideoFormat::Mp4));
        assert!(config.supported_formats.contains(&VideoFormat::Mkv));
        assert!(config.supported_formats.contains(&VideoFormat::Webm));
        assert!(!config.supported_formats.contains(&VideoFormat::Unknown));
    }
}
