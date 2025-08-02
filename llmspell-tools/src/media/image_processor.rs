// ABOUTME: Image processing tool for format conversion, resizing, cropping, and metadata extraction
// ABOUTME: Provides basic image file operations without external dependencies

// TODO: Phase 3+ - Add advanced image processing functionality:
// - Integrate image crate for full format support
// - Implement actual format conversion (PNG, JPEG, WebP, GIF, BMP)
// - Add advanced operations (blur, sharpen, filters, color adjustments)
// - Extract full EXIF metadata from JPEG files
// - Support image compression with quality settings
// - Add batch processing for multiple images
// - Implement smart cropping with face/object detection

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::SandboxContext;
use llmspell_utils::{
    extract_optional_bool, extract_optional_string, extract_optional_u64, extract_parameters,
    extract_required_string, response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Image format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Bmp,
    Tiff,
    Ico,
    Unknown,
}

impl ImageFormat {
    /// Detect format from file extension
    #[must_use]
    pub fn from_extension(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase)
            .as_deref()
        {
            Some("png") => Self::Png,
            Some("jpg" | "jpeg") => Self::Jpeg,
            Some("gif") => Self::Gif,
            Some("webp") => Self::Webp,
            Some("bmp") => Self::Bmp,
            Some("tiff" | "tif") => Self::Tiff,
            Some("ico") => Self::Ico,
            _ => Self::Unknown,
        }
    }

    /// Get MIME type for the format
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Ico => "image/x-icon",
            Self::Unknown => "application/octet-stream",
        }
    }
}

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    /// Calculate aspect ratio
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.width) / f64::from(self.height)
    }

    /// Get orientation
    #[must_use]
    pub fn orientation(&self) -> &'static str {
        match self.aspect_ratio() {
            r if r > 1.2 => "landscape",
            r if r < 0.8 => "portrait",
            _ => "square",
        }
    }
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Image format
    pub format: ImageFormat,
    /// Image dimensions
    pub dimensions: Option<ImageDimensions>,
    /// Color mode (RGB, RGBA, Grayscale, etc.)
    pub color_mode: Option<String>,
    /// Bit depth per channel
    pub bit_depth: Option<u8>,
    /// File size in bytes
    pub file_size: u64,
    /// DPI if available
    pub dpi: Option<(u32, u32)>,
    /// EXIF data (if available)
    pub exif: Option<HashMap<String, String>>,
}

/// Image processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProcessorConfig {
    /// Maximum file size to process (in bytes)
    pub max_file_size: u64,
    /// Maximum image dimensions
    pub max_width: u32,
    pub max_height: u32,
    /// Whether to preserve metadata during operations
    pub preserve_metadata: bool,
    /// Supported formats for conversion
    pub supported_formats: Vec<ImageFormat>,
    /// Default JPEG quality (0-100)
    pub jpeg_quality: u8,
    /// Default PNG compression level (0-9)
    pub png_compression: u8,
}

impl Default for ImageProcessorConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_width: 10000,
            max_height: 10000,
            preserve_metadata: true,
            supported_formats: vec![
                ImageFormat::Png,
                ImageFormat::Jpeg,
                ImageFormat::Gif,
                ImageFormat::Webp,
            ],
            jpeg_quality: 85,
            png_compression: 6,
        }
    }
}

/// Image processor tool for format conversion and basic operations
#[derive(Clone)]
pub struct ImageProcessorTool {
    metadata: ComponentMetadata,
    config: ImageProcessorConfig,
    #[allow(dead_code)] // Reserved for future sandbox integration
    sandbox_context: Option<Arc<SandboxContext>>,
}

impl ImageProcessorTool {
    /// Create a new image processor tool
    #[must_use]
    pub fn new(config: ImageProcessorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "image_processor".to_string(),
                "Image file processing for format conversion, resizing, and metadata extraction"
                    .to_string(),
            ),
            config,
            sandbox_context: None,
        }
    }

    /// Create a new image processor tool with sandbox context
    #[must_use]
    pub fn with_sandbox(
        config: ImageProcessorConfig,
        sandbox_context: Arc<SandboxContext>,
    ) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "image_processor".to_string(),
                "Image file processing for format conversion, resizing, and metadata extraction"
                    .to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
        }
    }

    /// Detect image format from file
    #[allow(clippy::unused_async)]
    async fn detect_format(&self, file_path: &Path) -> LLMResult<ImageFormat> {
        // First try extension-based detection
        let format = ImageFormat::from_extension(file_path);

        if format != ImageFormat::Unknown {
            debug!("Detected image format from extension: {:?}", format);
            return Ok(format);
        }

        // TODO: Phase 3+ - Implement magic number detection by reading file headers
        // PNG: 89 50 4E 47 0D 0A 1A 0A
        // JPEG: FF D8 FF
        // GIF: 47 49 46 38 37/39 61
        // WebP: 52 49 46 46 xx xx xx xx 57 45 42 50

        debug!("Could not detect image format for: {:?}", file_path);
        Ok(ImageFormat::Unknown)
    }

    /// Extract metadata from image file
    async fn extract_metadata(&self, file_path: &Path) -> LLMResult<ImageMetadata> {
        // Get file size
        let file_metadata = std::fs::metadata(file_path).map_err(|e| LLMSpellError::Tool {
            message: format!("Failed to read file metadata: {e}"),
            tool_name: Some("image_processor".to_string()),
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
                tool_name: Some("image_processor".to_string()),
                source: None,
            });
        }

        // Detect format
        let format = self.detect_format(file_path).await?;

        // Create basic metadata
        let metadata = ImageMetadata {
            format,
            dimensions: None,
            color_mode: None,
            bit_depth: None,
            file_size,
            dpi: None,
            exif: None,
        };

        // TODO: Phase 3+ - Extract actual image metadata:
        // - Use image crate to decode headers and get dimensions
        // - Extract color mode (RGB, RGBA, Grayscale, Indexed)
        // - Get bit depth information
        // - Extract DPI/PPI information
        // - Parse EXIF data from JPEG files
        // - Extract color profile information

        Ok(metadata)
    }

    /// Resize image
    #[allow(clippy::unused_async)]
    async fn resize_image(
        &self,
        _input_path: &Path,
        _output_path: &Path,
        _width: Option<u32>,
        _height: Option<u32>,
        _maintain_aspect_ratio: bool,
    ) -> LLMResult<()> {
        // TODO: Phase 3+ - Implement actual image resizing
        // This would require image decoding and encoding capabilities

        Err(LLMSpellError::Tool {
            message: "Image resizing is not implemented in this basic version. Image processing capabilities will be added in Phase 3+".to_string(),
            tool_name: Some("image_processor".to_string()),
            source: None,
        })
    }

    /// Convert image format
    #[allow(clippy::unused_async)]
    async fn convert_format(
        &self,
        _input_path: &Path,
        _output_path: &Path,
        target_format: ImageFormat,
    ) -> LLMResult<()> {
        // Check if conversion is supported
        if !self.config.supported_formats.contains(&target_format) {
            return Err(LLMSpellError::Tool {
                message: format!("Conversion to {target_format:?} format is not supported"),
                tool_name: Some("image_processor".to_string()),
                source: None,
            });
        }

        // TODO: Phase 3+ - Implement actual format conversion
        // This would require full image codec support

        Err(LLMSpellError::Tool {
            message: "Image format conversion is not implemented in this basic version. Full image processing will be added in Phase 3+".to_string(),
            tool_name: Some("image_processor".to_string()),
            source: None,
        })
    }

    /// Crop image
    #[allow(clippy::unused_async)]
    async fn crop_image(
        &self,
        _input_path: &Path,
        _output_path: &Path,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> LLMResult<()> {
        // TODO: Phase 3+ - Implement actual image cropping

        Err(LLMSpellError::Tool {
            message: "Image cropping is not implemented in this basic version. Image processing capabilities will be added in Phase 3+".to_string(),
            tool_name: Some("image_processor".to_string()),
            source: None,
        })
    }

    /// Rotate image
    #[allow(clippy::unused_async)]
    async fn rotate_image(
        &self,
        _input_path: &Path,
        _output_path: &Path,
        _degrees: i32,
    ) -> LLMResult<()> {
        // TODO: Phase 3+ - Implement actual image rotation
        // Support 90, 180, 270 degree rotations

        Err(LLMSpellError::Tool {
            message: "Image rotation is not implemented in this basic version. Image processing capabilities will be added in Phase 3+".to_string(),
            tool_name: Some("image_processor".to_string()),
            source: None,
        })
    }

    /// Generate thumbnail
    #[allow(clippy::unused_async)]
    async fn generate_thumbnail(
        &self,
        _input_path: &Path,
        _output_path: &Path,
        _max_width: u32,
        _max_height: u32,
    ) -> LLMResult<()> {
        // TODO: Phase 3+ - Implement actual thumbnail generation
        // This would resize while maintaining aspect ratio

        Err(LLMSpellError::Tool {
            message: "Thumbnail generation is not implemented in this basic version. Image processing capabilities will be added in Phase 3+".to_string(),
            tool_name: Some("image_processor".to_string()),
            source: None,
        })
    }

    /// Validate processing parameters
    #[allow(clippy::unused_async)]
    async fn validate_parameters(&self, params: &serde_json::Value) -> LLMResult<()> {
        // Validate operation
        if let Some(operation) = extract_optional_string(params, "operation") {
            match operation {
                "detect" | "metadata" | "resize" | "convert" | "crop" | "rotate" | "thumbnail" => {}
                _ => {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "Invalid operation: {operation}. Supported operations: detect, metadata, resize, convert, crop, rotate, thumbnail"
                        ),
                        field: Some("operation".to_string()),
                    });
                }
            }
        }

        // Validate file paths
        if let Some(file_path) = extract_optional_string(params, "file_path") {
            if file_path.is_empty() {
                return Err(LLMSpellError::Validation {
                    message: "File path cannot be empty".to_string(),
                    field: Some("file_path".to_string()),
                });
            }
        }

        // Validate resize parameters
        if extract_optional_string(params, "operation") == Some("resize")
            && params.get("width").is_none()
            && params.get("height").is_none()
        {
            return Err(LLMSpellError::Validation {
                message: "At least one of width or height must be specified for resize".to_string(),
                field: Some("dimensions".to_string()),
            });
        }

        // Validate crop parameters
        if extract_optional_string(params, "operation") == Some("crop") {
            for field in ["x", "y", "width", "height"] {
                if params.get(field).is_none() {
                    return Err(LLMSpellError::Validation {
                        message: format!("{field} is required for crop operation"),
                        field: Some(field.to_string()),
                    });
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BaseAgent for ImageProcessorTool {
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

                let response = ResponseBuilder::success("detect")
                    .with_message(format!("Detected image format: {format:?}"))
                    .with_result(json!({
                        "file_path": file_path,
                        "format": format,
                        "mime_type": format.mime_type(),
                        "supported": format != ImageFormat::Unknown && self.config.supported_formats.contains(&format)
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "metadata" => {
                let file_path = extract_required_string(params, "file_path")?;

                let path = Path::new(file_path);
                let metadata = self.extract_metadata(path).await?;

                let mut message = format!("Image file: {:?} format", metadata.format);

                if let Some(dims) = &metadata.dimensions {
                    use std::fmt::Write;
                    let _ = write!(
                        message,
                        ", {}x{} pixels ({})",
                        dims.width,
                        dims.height,
                        dims.orientation()
                    );
                }

                use std::fmt::Write;
                let _ = write!(message, ", Size: {} bytes", metadata.file_size);

                let response = ResponseBuilder::success("metadata")
                    .with_message(message)
                    .with_result(json!({
                        "file_path": file_path,
                        "metadata": metadata
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "resize" => {
                let input_path = extract_required_string(params, "source_path")?;
                let output_path = extract_required_string(params, "target_path")?;
                let width = extract_optional_u64(params, "width").map(|w| w as u32);
                let height = extract_optional_u64(params, "height").map(|h| h as u32);
                let maintain_aspect_ratio =
                    extract_optional_bool(params, "maintain_aspect_ratio").unwrap_or(true);

                self.resize_image(
                    Path::new(input_path),
                    Path::new(output_path),
                    width,
                    height,
                    maintain_aspect_ratio,
                )
                .await?;

                let response = ResponseBuilder::success("resize")
                    .with_message(format!("Resized image from {input_path} to {output_path}"))
                    .with_result(json!({
                        "source_path": input_path,
                        "target_path": output_path,
                        "width": width,
                        "height": height,
                        "maintain_aspect_ratio": maintain_aspect_ratio
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "convert" => {
                let input_path = extract_required_string(params, "source_path")?;
                let output_path = extract_required_string(params, "target_path")?;
                let target_format = extract_optional_string(params, "target_format")
                    .map(|s| match s.to_lowercase().as_str() {
                        "png" => ImageFormat::Png,
                        "jpeg" | "jpg" => ImageFormat::Jpeg,
                        "gif" => ImageFormat::Gif,
                        "webp" => ImageFormat::Webp,
                        _ => ImageFormat::Unknown,
                    })
                    .unwrap_or_else(|| ImageFormat::from_extension(Path::new(output_path)));

                self.convert_format(
                    Path::new(input_path),
                    Path::new(output_path),
                    target_format.clone(),
                )
                .await?;

                let response = ResponseBuilder::success("convert")
                    .with_message(format!(
                        "Converted image from {input_path} to {output_path} ({target_format:?} format)"
                    ))
                    .with_result(json!({
                        "source_path": input_path,
                        "target_path": output_path,
                        "target_format": target_format
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "crop" => {
                let input_path = extract_required_string(params, "source_path")?;
                let output_path = extract_required_string(params, "target_path")?;

                let x = params
                    .get("x")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "x is required".to_string(),
                        field: Some("x".to_string()),
                    })? as u32;

                let y = params
                    .get("y")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "y is required".to_string(),
                        field: Some("y".to_string()),
                    })? as u32;

                let width = params
                    .get("width")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "width is required".to_string(),
                        field: Some("width".to_string()),
                    })? as u32;

                let height = params
                    .get("height")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "height is required".to_string(),
                        field: Some("height".to_string()),
                    })? as u32;

                self.crop_image(
                    Path::new(input_path),
                    Path::new(output_path),
                    x,
                    y,
                    width,
                    height,
                )
                .await?;

                let response = ResponseBuilder::success("crop")
                    .with_message(format!(
                        "Cropped image from {input_path} to {output_path} ({width}x{height} at {x}, {y})"
                    ))
                    .with_result(json!({
                        "source_path": input_path,
                        "target_path": output_path,
                        "x": x,
                        "y": y,
                        "width": width,
                        "height": height
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "rotate" => {
                let input_path = extract_required_string(params, "source_path")?;
                let output_path = extract_required_string(params, "target_path")?;
                let degrees = params
                    .get("degrees")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(90) as i32;

                self.rotate_image(Path::new(input_path), Path::new(output_path), degrees)
                    .await?;

                let response = ResponseBuilder::success("rotate")
                    .with_message(format!(
                        "Rotated image {degrees} degrees from {input_path} to {output_path}"
                    ))
                    .with_result(json!({
                        "source_path": input_path,
                        "target_path": output_path,
                        "degrees": degrees
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "thumbnail" => {
                let input_path = extract_required_string(params, "source_path")?;
                let output_path = extract_required_string(params, "target_path")?;
                let max_width = extract_optional_u64(params, "max_width").map_or(200, |w| w as u32);
                let max_height =
                    extract_optional_u64(params, "max_height").map_or(200, |h| h as u32);

                self.generate_thumbnail(
                    Path::new(input_path),
                    Path::new(output_path),
                    max_width,
                    max_height,
                )
                .await?;

                let response = ResponseBuilder::success("thumbnail")
                    .with_message(format!(
                        "Generated thumbnail from {input_path} to {output_path} (max {max_width}x{max_height})"
                    ))
                    .with_result(json!({
                        "source_path": input_path,
                        "target_path": output_path,
                        "max_width": max_width,
                        "max_height": max_height
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
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
        Ok(AgentOutput::text(format!("Image processor error: {error}")))
    }
}

#[async_trait]
impl Tool for ImageProcessorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Media
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "image_processor".to_string(),
            "Process image files for format conversion, resizing, and metadata extraction"
                .to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description:
                "Operation to perform: detect, metadata, resize, convert, crop, rotate, thumbnail"
                    .to_string(),
            required: false,
            default: Some(json!("metadata")),
        })
        .with_parameter(ParameterDef {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the image file (for detect and metadata operations)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "source_path".to_string(),
            param_type: ParameterType::String,
            description: "Source image file path (for processing operations)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "target_path".to_string(),
            param_type: ParameterType::String,
            description: "Target image file path (for processing operations)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "width".to_string(),
            param_type: ParameterType::Number,
            description: "Target width for resize/crop operations".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "height".to_string(),
            param_type: ParameterType::Number,
            description: "Target height for resize/crop operations".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "x".to_string(),
            param_type: ParameterType::Number,
            description: "X coordinate for crop operation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "y".to_string(),
            param_type: ParameterType::Number,
            description: "Y coordinate for crop operation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "degrees".to_string(),
            param_type: ParameterType::Number,
            description: "Rotation degrees (90, 180, 270)".to_string(),
            required: false,
            default: Some(json!(90)),
        })
        .with_parameter(ParameterDef {
            name: "maintain_aspect_ratio".to_string(),
            param_type: ParameterType::Boolean,
            description: "Whether to maintain aspect ratio during resize".to_string(),
            required: false,
            default: Some(json!(true)),
        })
        .with_parameter(ParameterDef {
            name: "target_format".to_string(),
            param_type: ParameterType::String,
            description: "Target image format for conversion: png, jpeg, gif, webp".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "max_width".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum width for thumbnail generation".to_string(),
            required: false,
            default: Some(json!(200)),
        })
        .with_parameter(ParameterDef {
            name: "max_height".to_string(),
            param_type: ParameterType::Number,
            description: "Maximum height for thumbnail generation".to_string(),
            required: false,
            default: Some(json!(200)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_image_processor() -> ImageProcessorTool {
        let config = ImageProcessorConfig::default();
        ImageProcessorTool::new(config)
    }
    #[tokio::test]
    async fn test_format_detection_by_extension() {
        let tool = create_test_image_processor();
        let temp_dir = TempDir::new().unwrap();

        // Test various extensions
        let test_cases = vec![
            ("test.png", ImageFormat::Png),
            ("test.jpg", ImageFormat::Jpeg),
            ("test.jpeg", ImageFormat::Jpeg),
            ("test.gif", ImageFormat::Gif),
            ("test.webp", ImageFormat::Webp),
            ("test.bmp", ImageFormat::Bmp),
            ("test.tiff", ImageFormat::Tiff),
            ("test.ico", ImageFormat::Ico),
            ("test.unknown", ImageFormat::Unknown),
        ];

        for (filename, expected_format) in test_cases {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, b"dummy").unwrap();

            let format = tool.detect_format(&file_path).await.unwrap();
            assert_eq!(format, expected_format);
        }
    }
    #[tokio::test]
    async fn test_mime_types() {
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Gif.mime_type(), "image/gif");
        assert_eq!(ImageFormat::Webp.mime_type(), "image/webp");
        assert_eq!(ImageFormat::Unknown.mime_type(), "application/octet-stream");
    }
    #[tokio::test]
    async fn test_image_dimensions() {
        let landscape = ImageDimensions {
            width: 1920,
            height: 1080,
        };
        assert!((landscape.aspect_ratio() - 1.78).abs() < 0.01);
        assert_eq!(landscape.orientation(), "landscape");

        let portrait = ImageDimensions {
            width: 1080,
            height: 1920,
        };
        assert!((portrait.aspect_ratio() - 0.56).abs() < 0.01);
        assert_eq!(portrait.orientation(), "portrait");

        let square = ImageDimensions {
            width: 1000,
            height: 1000,
        };
        assert_eq!(square.aspect_ratio(), 1.0);
        assert_eq!(square.orientation(), "square");
    }
    #[tokio::test]
    async fn test_metadata_extraction() {
        let tool = create_test_image_processor();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("image.png");

        fs::write(&file_path, b"dummy png content").unwrap();

        let input = create_test_tool_input(vec![("operation", "metadata")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Image file"));
        assert!(result.text.contains("Png"));
        assert!(result.text.contains("Size: 17 bytes"));
    }
    #[tokio::test]
    async fn test_format_detection_operation() {
        let tool = create_test_image_processor();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.jpeg");

        fs::write(&file_path, b"dummy").unwrap();

        let input = create_test_tool_input(vec![("operation", "detect")]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Detected image format: Jpeg"));
    }
    #[tokio::test]
    async fn test_file_size_limit() {
        let config = ImageProcessorConfig {
            max_file_size: 10, // Very small limit
            ..Default::default()
        };
        let tool = ImageProcessorTool::new(config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.png");

        // Create a file larger than the limit
        fs::write(&file_path, vec![0u8; 100]).unwrap();

        let input = create_test_tool_input(vec![("operation", "metadata")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }
    #[tokio::test]
    async fn test_resize_not_implemented() {
        let tool = create_test_image_processor();
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.png");
        let output_path = temp_dir.path().join("output.png");

        fs::write(&input_path, b"dummy").unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "resize"),
            ("source_path", "input_path.to_str().unwrap()"),
            ("width", "100"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not implemented"));
    }
    #[tokio::test]
    async fn test_convert_not_implemented() {
        let tool = create_test_image_processor();
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("input.png");
        let output_path = temp_dir.path().join("output.jpg");

        fs::write(&input_path, b"dummy").unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "convert"),
            ("source_path", "input_path.to_str().unwrap()"),
            ("target_format", "jpeg"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not implemented"));
    }
    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = create_test_image_processor();

        let input = create_test_tool_input(vec![("operation", "invalid")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid operation"));
    }
    #[tokio::test]
    async fn test_missing_required_parameters() {
        let tool = create_test_image_processor();

        // Missing file_path for metadata operation
        let input = create_test_tool_input(vec![("operation", "metadata")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter 'file_path'"));

        // Missing width and height for resize
        let input = create_test_tool_input(vec![
            ("operation", "resize"),
            ("source_path", "/tmp/input.png"),
            ("target_path", "/tmp/output.png"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one of width or height"));
    }
    #[tokio::test]
    async fn test_crop_parameter_validation() {
        let tool = create_test_image_processor();

        // Missing crop parameters
        let input = create_test_tool_input(vec![
            ("operation", "crop"),
            ("source_path", "/tmp/input.png"),
            ("target_path", "/tmp/output.png"),
            ("x", "0"),
            ("y", "0"),
            ("width", "100"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("height is required"));
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_image_processor();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "image_processor");
        assert!(metadata.description.contains("Image file processing"));

        let schema = tool.schema();
        assert_eq!(schema.name, "image_processor");
        assert_eq!(tool.category(), ToolCategory::Media);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);

        // Check parameters
        let params = &schema.parameters;
        assert!(params.iter().any(|p| p.name == "operation"));
        assert!(params.iter().any(|p| p.name == "file_path"));
        assert!(params.iter().any(|p| p.name == "width"));
        assert!(params.iter().any(|p| p.name == "height"));
        assert!(params.iter().any(|p| p.name == "degrees"));
    }
    #[tokio::test]
    async fn test_default_operation() {
        let tool = create_test_image_processor();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");

        fs::write(&file_path, b"dummy").unwrap();

        // No operation specified, should default to metadata
        let input = create_test_tool_input(vec![
            ,
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Image file"));
    }
    #[tokio::test]
    async fn test_empty_file_path() {
        let tool = create_test_image_processor();

        let input = create_test_tool_input(vec![("operation", "detect")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }
    #[tokio::test]
    async fn test_supported_formats() {
        let config = ImageProcessorConfig::default();
        assert!(config.supported_formats.contains(&ImageFormat::Png));
        assert!(config.supported_formats.contains(&ImageFormat::Jpeg));
        assert!(config.supported_formats.contains(&ImageFormat::Webp));
        assert!(!config.supported_formats.contains(&ImageFormat::Unknown));
    }
}
