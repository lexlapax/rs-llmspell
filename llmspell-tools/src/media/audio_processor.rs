// ABOUTME: Audio processing tool for format detection, metadata extraction, and basic conversions
// ABOUTME: Supports common audio formats including WAV, MP3, and provides duration/bitrate information

// TODO: Phase 3+ - Add advanced audio processing functionality:
// - Integrate symphonia or rodio for full codec support
// - Implement actual format conversion (MP3, FLAC, OGG encoding/decoding)
// - Add audio resampling and bit depth conversion
// - Extract full metadata tags (ID3, Vorbis comments, etc.)
// - Add waveform generation and audio analysis features
// - Support streaming processing for large files

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
    extract_optional_string, extract_parameters, extract_required_string, response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Audio format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Mp3,
    Flac,
    Ogg,
    M4a,
    Unknown,
}

impl AudioFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("wav") => Self::Wav,
            Some("mp3") => Self::Mp3,
            Some("flac") => Self::Flac,
            Some("ogg") => Self::Ogg,
            Some("m4a") => Self::M4a,
            _ => Self::Unknown,
        }
    }
}

/// Audio metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    /// Audio format
    pub format: AudioFormat,
    /// Duration in seconds
    pub duration_seconds: Option<f64>,
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    /// Number of channels
    pub channels: Option<u8>,
    /// Bit rate in bits per second
    pub bit_rate: Option<u32>,
    /// Title metadata
    pub title: Option<String>,
    /// Artist metadata
    pub artist: Option<String>,
    /// Album metadata
    pub album: Option<String>,
    /// Year metadata
    pub year: Option<u32>,
    /// File size in bytes
    pub file_size: u64,
}

/// Audio processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProcessorConfig {
    /// Maximum file size to process (in bytes)
    pub max_file_size: u64,
    /// Whether to extract metadata tags
    pub extract_tags: bool,
    /// Whether to analyze audio properties
    pub analyze_properties: bool,
    /// Supported formats for conversion
    pub supported_formats: Vec<AudioFormat>,
    /// Conversion quality (0-10, where 10 is highest)
    pub conversion_quality: u8,
}

impl Default for AudioProcessorConfig {
    fn default() -> Self {
        Self {
            max_file_size: 500 * 1024 * 1024, // 500MB
            extract_tags: true,
            analyze_properties: true,
            supported_formats: vec![AudioFormat::Wav, AudioFormat::Mp3],
            conversion_quality: 8,
        }
    }
}

/// Audio processor tool for format detection and metadata extraction
#[derive(Clone)]
pub struct AudioProcessorTool {
    metadata: ComponentMetadata,
    config: AudioProcessorConfig,
    #[allow(dead_code)] // Reserved for future sandbox integration
    sandbox_context: Option<Arc<SandboxContext>>,
}

impl AudioProcessorTool {
    /// Create a new audio processor tool
    pub fn new(config: AudioProcessorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "audio_processor".to_string(),
                "Audio file processing for format detection, metadata extraction, and conversions"
                    .to_string(),
            ),
            config,
            sandbox_context: None,
        }
    }

    /// Create a new audio processor tool with sandbox context
    pub fn with_sandbox(
        config: AudioProcessorConfig,
        sandbox_context: Arc<SandboxContext>,
    ) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "audio_processor".to_string(),
                "Audio file processing for format detection, metadata extraction, and conversions"
                    .to_string(),
            ),
            config,
            sandbox_context: Some(sandbox_context),
        }
    }

    /// Detect audio format from file
    async fn detect_format(&self, file_path: &Path) -> LLMResult<AudioFormat> {
        // First try extension-based detection
        let format = AudioFormat::from_extension(file_path);

        if format != AudioFormat::Unknown {
            debug!("Detected audio format from extension: {:?}", format);
            return Ok(format);
        }

        // For unknown formats, we would need to read file headers
        // For now, we'll return Unknown for unsupported formats
        debug!("Could not detect audio format for: {:?}", file_path);
        Ok(AudioFormat::Unknown)
    }

    /// Extract metadata from audio file
    async fn extract_metadata(&self, file_path: &Path) -> LLMResult<AudioMetadata> {
        // Get file size
        let file_metadata = std::fs::metadata(file_path).map_err(|e| LLMSpellError::Tool {
            message: format!("Failed to read file metadata: {}", e),
            tool_name: Some("audio_processor".to_string()),
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
                tool_name: Some("audio_processor".to_string()),
                source: None,
            });
        }

        // Detect format
        let format = self.detect_format(file_path).await?;

        // Create basic metadata
        let mut metadata = AudioMetadata {
            format: format.clone(),
            duration_seconds: None,
            sample_rate: None,
            channels: None,
            bit_rate: None,
            title: None,
            artist: None,
            album: None,
            year: None,
            file_size,
        };

        // For WAV files, we can extract some basic information
        if format == AudioFormat::Wav && self.config.analyze_properties {
            if let Ok(wav_info) = self.analyze_wav_file(file_path).await {
                metadata.sample_rate = Some(wav_info.sample_rate);
                metadata.channels = Some(wav_info.channels);
                metadata.duration_seconds = wav_info.duration_seconds;
                metadata.bit_rate = wav_info.bit_rate;
            }
        }

        // TODO: Phase 3+ - Extract metadata for other formats:
        // - MP3: ID3v1/ID3v2 tags (title, artist, album, year, genre, cover art)
        // - FLAC: Vorbis comments and embedded pictures
        // - OGG: Vorbis comments
        // - M4A: iTunes metadata atoms
        // This will require symphonia or similar library for proper codec support

        // Extract file name as default title if no metadata
        if metadata.title.is_none() && self.config.extract_tags {
            if let Some(stem) = file_path.file_stem() {
                metadata.title = Some(stem.to_string_lossy().to_string());
            }
        }

        Ok(metadata)
    }

    /// Analyze WAV file structure
    async fn analyze_wav_file(&self, file_path: &Path) -> LLMResult<WavInfo> {
        use std::fs::File;
        use std::io::{Read, Seek, SeekFrom};

        let mut file = File::open(file_path).map_err(|e| LLMSpellError::Tool {
            message: format!("Failed to open WAV file: {}", e),
            tool_name: Some("audio_processor".to_string()),
            source: None,
        })?;

        // Read RIFF header
        let mut riff_header = [0u8; 12];
        file.read_exact(&mut riff_header)
            .map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to read RIFF header: {}", e),
                tool_name: Some("audio_processor".to_string()),
                source: None,
            })?;

        // Verify RIFF and WAVE markers
        if &riff_header[0..4] != b"RIFF" || &riff_header[8..12] != b"WAVE" {
            return Err(LLMSpellError::Tool {
                message: "Invalid WAV file format".to_string(),
                tool_name: Some("audio_processor".to_string()),
                source: None,
            });
        }

        // Find fmt chunk
        let mut chunk_header = [0u8; 8];
        loop {
            if file.read_exact(&mut chunk_header).is_err() {
                break;
            }

            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]);

            if &chunk_header[0..4] == b"fmt " {
                // Read format chunk
                let mut fmt_data = vec![0u8; 16.min(chunk_size as usize)];
                file.read_exact(&mut fmt_data)
                    .map_err(|e| LLMSpellError::Tool {
                        message: format!("Failed to read fmt chunk: {}", e),
                        tool_name: Some("audio_processor".to_string()),
                        source: None,
                    })?;

                let channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]) as u8;
                let sample_rate =
                    u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
                let byte_rate =
                    u32::from_le_bytes([fmt_data[8], fmt_data[9], fmt_data[10], fmt_data[11]]);
                let bits_per_sample = if fmt_data.len() >= 16 {
                    u16::from_le_bytes([fmt_data[14], fmt_data[15]])
                } else {
                    16
                };

                // Find data chunk for duration calculation
                file.seek(SeekFrom::Start(12)).ok();
                let mut data_size = 0u32;

                loop {
                    if file.read_exact(&mut chunk_header).is_err() {
                        break;
                    }

                    let size = u32::from_le_bytes([
                        chunk_header[4],
                        chunk_header[5],
                        chunk_header[6],
                        chunk_header[7],
                    ]);

                    if &chunk_header[0..4] == b"data" {
                        data_size = size;
                        break;
                    }

                    // Skip this chunk
                    if file.seek(SeekFrom::Current(size as i64)).is_err() {
                        break;
                    }
                }

                let duration_seconds = if byte_rate > 0 {
                    Some(data_size as f64 / byte_rate as f64)
                } else {
                    None
                };

                let bit_rate = byte_rate.checked_mul(8);

                return Ok(WavInfo {
                    sample_rate,
                    channels,
                    bits_per_sample,
                    duration_seconds,
                    bit_rate,
                });
            }

            // Skip to next chunk
            if file.seek(SeekFrom::Current(chunk_size as i64)).is_err() {
                break;
            }
        }

        Err(LLMSpellError::Tool {
            message: "Could not find fmt chunk in WAV file".to_string(),
            tool_name: Some("audio_processor".to_string()),
            source: None,
        })
    }

    /// Convert audio file to another format
    async fn convert_audio(
        &self,
        source_path: &Path,
        target_path: &Path,
        target_format: AudioFormat,
    ) -> LLMResult<()> {
        // Check if conversion is supported
        if !self.config.supported_formats.contains(&target_format) {
            return Err(LLMSpellError::Tool {
                message: format!("Conversion to {:?} format is not supported", target_format),
                tool_name: Some("audio_processor".to_string()),
                source: None,
            });
        }

        // TODO: Phase 3+ - Implement full audio format conversion using symphonia/rodio
        // For this basic implementation, we'll only support WAV to WAV copying
        // Future versions will support:
        // - MP3 encoding/decoding with configurable bitrates
        // - FLAC lossless compression
        // - OGG Vorbis encoding
        // - M4A/AAC support
        // - Automatic format detection from file contents (not just extension)

        let source_format = self.detect_format(source_path).await?;

        if source_format == AudioFormat::Wav && target_format == AudioFormat::Wav {
            // Simple file copy for same format
            std::fs::copy(source_path, target_path).map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to copy audio file: {}", e),
                tool_name: Some("audio_processor".to_string()),
                source: None,
            })?;

            info!(
                "Copied WAV file from {:?} to {:?}",
                source_path, target_path
            );
            Ok(())
        } else {
            Err(LLMSpellError::Tool {
                message: format!(
                    "Conversion from {:?} to {:?} is not implemented in this basic version. Advanced audio processing will be added in Phase 3+",
                    source_format, target_format
                ),
                tool_name: Some("audio_processor".to_string()),
                source: None,
            })
        }
    }

    /// Validate processing parameters
    async fn validate_parameters(&self, params: &serde_json::Value) -> LLMResult<()> {
        // Validate operation
        if let Some(operation) = extract_optional_string(params, "operation") {
            match operation {
                "detect" | "metadata" | "convert" => {}
                _ => {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "Invalid operation: {}. Supported operations: detect, metadata, convert",
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
        } else if extract_optional_string(params, "operation") != Some("convert") {
            return Err(LLMSpellError::Validation {
                message: "file_path is required for this operation".to_string(),
                field: Some("file_path".to_string()),
            });
        }

        // Validate conversion parameters
        if extract_optional_string(params, "operation") == Some("convert") {
            if params.get("source_path").is_none() {
                return Err(LLMSpellError::Validation {
                    message: "source_path is required for convert operation".to_string(),
                    field: Some("source_path".to_string()),
                });
            }
            if params.get("target_path").is_none() {
                return Err(LLMSpellError::Validation {
                    message: "target_path is required for convert operation".to_string(),
                    field: Some("target_path".to_string()),
                });
            }
        }

        Ok(())
    }
}

/// WAV file information
#[derive(Debug)]
struct WavInfo {
    sample_rate: u32,
    channels: u8,
    #[allow(dead_code)]
    bits_per_sample: u16,
    duration_seconds: Option<f64>,
    bit_rate: Option<u32>,
}

#[async_trait]
impl BaseAgent for AudioProcessorTool {
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
                    .with_message(format!("Detected audio format: {:?}", format))
                    .with_result(json!({
                        "file_path": file_path,
                        "format": format,
                        "supported": format != AudioFormat::Unknown
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "metadata" => {
                let file_path = extract_required_string(params, "file_path")?;

                let path = Path::new(file_path);
                let metadata = self.extract_metadata(path).await?;

                let mut message = format!(
                    "Audio file: {} ({:?})",
                    metadata.title.as_deref().unwrap_or("Unknown"),
                    metadata.format
                );

                if let Some(duration) = metadata.duration_seconds {
                    message.push_str(&format!(", Duration: {:.1}s", duration));
                }

                if let Some(sample_rate) = metadata.sample_rate {
                    message.push_str(&format!(", Sample rate: {}Hz", sample_rate));
                }

                let response = ResponseBuilder::success("metadata")
                    .with_message(message)
                    .with_result(json!({
                        "file_path": file_path,
                        "metadata": metadata
                    }))
                    .build();

                Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
            }

            "convert" => {
                let source_path = extract_required_string(params, "source_path")?;
                let target_path = extract_required_string(params, "target_path")?;

                let target_format = extract_optional_string(params, "target_format")
                    .map(|s| match s.to_lowercase().as_str() {
                        "wav" => AudioFormat::Wav,
                        "mp3" => AudioFormat::Mp3,
                        _ => AudioFormat::Unknown,
                    })
                    .unwrap_or_else(|| AudioFormat::from_extension(Path::new(target_path)));

                let input = Path::new(source_path);
                let output = Path::new(target_path);

                self.convert_audio(input, output, target_format.clone())
                    .await?;

                let response = ResponseBuilder::success("convert")
                    .with_message(format!(
                        "Converted audio from {} to {} ({:?} format)",
                        source_path, target_path, target_format
                    ))
                    .with_result(json!({
                        "source_path": source_path,
                        "target_path": target_path,
                        "target_format": target_format,
                        "success": true
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
        Ok(AgentOutput::text(format!(
            "Audio processor error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for AudioProcessorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Media
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "audio_processor".to_string(),
            "Process audio files for format detection, metadata extraction, and conversions"
                .to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: detect, metadata, convert".to_string(),
            required: false,
            default: Some(json!("metadata")),
        })
        .with_parameter(ParameterDef {
            name: "file_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to the audio file (for detect and metadata operations)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "source_path".to_string(),
            param_type: ParameterType::String,
            description: "Source audio file path (for convert operation)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "target_path".to_string(),
            param_type: ParameterType::String,
            description: "Target audio file path (for convert operation)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "target_format".to_string(),
            param_type: ParameterType::String,
            description: "Target audio format for conversion: wav, mp3".to_string(),
            required: false,
            default: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tool() -> AudioProcessorTool {
        let config = AudioProcessorConfig::default();
        AudioProcessorTool::new(config)
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

    fn create_test_wav_file(path: &Path) -> std::io::Result<()> {
        use std::io::Write;

        let mut file = fs::File::create(path)?;

        // RIFF header
        file.write_all(b"RIFF")?;
        file.write_all(&[36, 0, 0, 0])?; // File size - 8
        file.write_all(b"WAVE")?;

        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&[16, 0, 0, 0])?; // Chunk size
        file.write_all(&[1, 0])?; // Audio format (PCM)
        file.write_all(&[2, 0])?; // Channels (stereo)
        file.write_all(&[68, 172, 0, 0])?; // Sample rate (44100)
        file.write_all(&[16, 177, 2, 0])?; // Byte rate
        file.write_all(&[4, 0])?; // Block align
        file.write_all(&[16, 0])?; // Bits per sample

        // data chunk
        file.write_all(b"data")?;
        file.write_all(&[0, 0, 0, 0])?; // Data size (empty)

        Ok(())
    }

    #[tokio::test]
    async fn test_format_detection_by_extension() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();

        // Test various extensions
        let test_cases = vec![
            ("test.wav", AudioFormat::Wav),
            ("test.mp3", AudioFormat::Mp3),
            ("test.flac", AudioFormat::Flac),
            ("test.ogg", AudioFormat::Ogg),
            ("test.m4a", AudioFormat::M4a),
            ("test.unknown", AudioFormat::Unknown),
        ];

        for (filename, expected_format) in test_cases {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, b"dummy").unwrap();

            let format = tool.detect_format(&file_path).await.unwrap();
            assert_eq!(format, expected_format);
        }
    }

    #[tokio::test]
    async fn test_wav_file_analysis() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let wav_path = temp_dir.path().join("test.wav");

        create_test_wav_file(&wav_path).unwrap();

        let metadata = tool.extract_metadata(&wav_path).await.unwrap();

        assert_eq!(metadata.format, AudioFormat::Wav);
        assert_eq!(metadata.sample_rate, Some(44100));
        assert_eq!(metadata.channels, Some(2));
        assert_eq!(metadata.bit_rate, Some(44100 * 2 * 16)); // 1411200 bps
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("audio.mp3");

        fs::write(&file_path, b"dummy mp3 content").unwrap();

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

        assert!(result.text.contains("Audio file"));
        assert!(result.text.contains("Mp3"));
    }

    #[tokio::test]
    async fn test_format_detection_operation() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.wav");

        create_test_wav_file(&file_path).unwrap();

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

        assert!(result.text.contains("Detected audio format: Wav"));
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let mut config = AudioProcessorConfig::default();
        config.max_file_size = 10; // Very small limit
        let tool = AudioProcessorTool::new(config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.wav");

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
    async fn test_wav_to_wav_conversion() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("input.wav");
        let target_path = temp_dir.path().join("output.wav");

        create_test_wav_file(&source_path).unwrap();

        let input = create_test_input(
            "Convert audio",
            json!({
                "operation": "convert",
                "source_path": source_path.to_str().unwrap(),
                "target_path": target_path.to_str().unwrap(),
                "target_format": "wav"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Converted audio"));
        assert!(target_path.exists());
    }

    #[tokio::test]
    async fn test_unsupported_conversion() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("input.wav");
        let target_path = temp_dir.path().join("output.flac");

        create_test_wav_file(&source_path).unwrap();

        let input = create_test_input(
            "Convert audio",
            json!({
                "operation": "convert",
                "source_path": source_path.to_str().unwrap(),
                "target_path": target_path.to_str().unwrap(),
                "target_format": "flac"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not supported"));
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
            .contains("file_path is required"));
    }

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = create_test_tool();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "audio_processor");
        assert!(metadata.description.contains("Audio file processing"));

        let schema = tool.schema();
        assert_eq!(schema.name, "audio_processor");
        assert_eq!(tool.category(), ToolCategory::Media);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);

        // Check parameters
        let params = &schema.parameters;
        assert!(params.iter().any(|p| p.name == "operation"));
        assert!(params.iter().any(|p| p.name == "file_path"));
        assert!(params.iter().any(|p| p.name == "source_path"));
        assert!(params.iter().any(|p| p.name == "target_path"));
        assert!(params.iter().any(|p| p.name == "target_format"));
    }

    #[tokio::test]
    async fn test_default_operation() {
        let tool = create_test_tool();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.wav");

        create_test_wav_file(&file_path).unwrap();

        // No operation specified, should default to metadata
        let input = create_test_input(
            "Process audio",
            json!({
                "file_path": file_path.to_str().unwrap()
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert!(result.text.contains("Audio file"));
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
    async fn test_file_not_found() {
        let tool = create_test_tool();

        let input = create_test_input(
            "Extract metadata",
            json!({
                "operation": "metadata",
                "file_path": "/non/existent/file.wav"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
}
