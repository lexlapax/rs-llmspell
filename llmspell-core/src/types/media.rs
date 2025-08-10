//! ABOUTME: Multimodal content types for text, image, audio, video, and binary data
//! ABOUTME: Provides MediaContent enum, format types, metadata structures, and validation helpers

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum file size limits for different media types (in bytes)
pub const MAX_IMAGE_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const MAX_AUDIO_SIZE: usize = 500 * 1024 * 1024; // 500MB
pub const MAX_VIDEO_SIZE: usize = 5 * 1024 * 1024 * 1024; // 5GB
pub const MAX_BINARY_SIZE: usize = 1024 * 1024 * 1024; // 1GB

/// Multimodal content that can be processed by agents
///
/// # Examples
///
/// ```
/// use llmspell_core::types::{MediaContent, ImageFormat, ImageMetadata, ColorSpace};
///
/// // Text content
/// let text = MediaContent::Text("Hello world".to_string());
///
/// // Image content with metadata
/// let image = MediaContent::Image {
///     data: vec![0xFF, 0xD8, 0xFF], // JPEG header
///     format: ImageFormat::Jpeg,
///     metadata: ImageMetadata {
///         width: 1920,
///         height: 1080,
///         color_space: ColorSpace::RGB,
///         has_transparency: false,
///         dpi: Some(72),
///     },
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MediaContent {
    /// Plain text content
    Text(String),

    /// Image data with format and metadata
    Image {
        /// Raw image bytes
        data: Vec<u8>,
        /// Image format
        format: ImageFormat,
        /// Image metadata
        metadata: ImageMetadata,
    },

    /// Audio data with format and metadata
    Audio {
        /// Raw audio bytes
        data: Vec<u8>,
        /// Audio format
        format: AudioFormat,
        /// Audio metadata
        metadata: AudioMetadata,
    },

    /// Video data with format and metadata
    Video {
        /// Raw video bytes
        data: Vec<u8>,
        /// Video format
        format: VideoFormat,
        /// Video metadata
        metadata: VideoMetadata,
    },

    /// Generic binary data
    Binary {
        /// Raw binary bytes
        data: Vec<u8>,
        /// MIME type if known
        mime_type: Option<String>,
        /// Original filename if available
        filename: Option<String>,
    },
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
    /// WebP format
    Webp,
    /// GIF format
    Gif,
    /// SVG format
    Svg,
    /// TIFF format
    Tiff,
}

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    /// MP3 format
    Mp3,
    /// WAV format
    Wav,
    /// FLAC format
    Flac,
    /// OGG format
    Ogg,
    /// M4A format
    M4a,
}

/// Supported video formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoFormat {
    /// MP4 format
    Mp4,
    /// WebM format
    Webm,
    /// AVI format
    Avi,
    /// MOV format
    Mov,
    /// MKV format
    Mkv,
}

/// Color space information for images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorSpace {
    /// RGB color space
    RGB,
    /// RGBA color space (with alpha)
    RGBA,
    /// Grayscale
    Grayscale,
    /// CMYK color space
    CMYK,
}

/// Metadata for image content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Color space
    pub color_space: ColorSpace,
    /// Whether image has transparency
    pub has_transparency: bool,
    /// Dots per inch (if available)
    pub dpi: Option<u32>,
}

/// Metadata for audio content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioMetadata {
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo, etc.)
    pub channels: u8,
    /// Bitrate in bits per second
    pub bitrate: Option<u32>,
}

/// Metadata for video content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Frames per second
    pub fps: f32,
    /// Video codec name
    pub codec: Option<String>,
}

/// Media types for capability detection
///
/// Used to indicate which media types an agent or tool can handle.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::MediaType;
///
/// let supported_types = vec![
///     MediaType::Text,
///     MediaType::Image,
/// ];
///
/// // Check if audio is supported
/// if supported_types.contains(&MediaType::Audio) {
///     println!("Audio processing available");
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    /// Text content
    Text,
    /// Image content
    Image,
    /// Audio content
    Audio,
    /// Video content
    Video,
    /// Binary content
    Binary,
}

// Display implementations

impl fmt::Display for MediaContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaContent::Text(text) => {
                write!(f, "Text({} chars)", text.len())
            }
            MediaContent::Image {
                format, metadata, ..
            } => {
                write!(
                    f,
                    "Image({:?}, {}x{})",
                    format, metadata.width, metadata.height
                )
            }
            MediaContent::Audio {
                format, metadata, ..
            } => {
                write!(f, "Audio({:?}, {}ms)", format, metadata.duration_ms)
            }
            MediaContent::Video {
                format, metadata, ..
            } => {
                write!(
                    f,
                    "Video({:?}, {}x{}, {}ms)",
                    format, metadata.width, metadata.height, metadata.duration_ms
                )
            }
            MediaContent::Binary {
                mime_type,
                filename,
                data,
            } => {
                write!(f, "Binary({} bytes", data.len())?;
                if let Some(mime) = mime_type {
                    write!(f, ", {mime}")?;
                }
                if let Some(name) = filename {
                    write!(f, ", {name}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "PNG"),
            ImageFormat::Jpeg => write!(f, "JPEG"),
            ImageFormat::Webp => write!(f, "WebP"),
            ImageFormat::Gif => write!(f, "GIF"),
            ImageFormat::Svg => write!(f, "SVG"),
            ImageFormat::Tiff => write!(f, "TIFF"),
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Flac => write!(f, "FLAC"),
            AudioFormat::Ogg => write!(f, "OGG"),
            AudioFormat::M4a => write!(f, "M4A"),
        }
    }
}

impl fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoFormat::Mp4 => write!(f, "MP4"),
            VideoFormat::Webm => write!(f, "WebM"),
            VideoFormat::Avi => write!(f, "AVI"),
            VideoFormat::Mov => write!(f, "MOV"),
            VideoFormat::Mkv => write!(f, "MKV"),
        }
    }
}

impl fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorSpace::RGB => write!(f, "RGB"),
            ColorSpace::RGBA => write!(f, "RGBA"),
            ColorSpace::Grayscale => write!(f, "Grayscale"),
            ColorSpace::CMYK => write!(f, "CMYK"),
        }
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaType::Text => write!(f, "Text"),
            MediaType::Image => write!(f, "Image"),
            MediaType::Audio => write!(f, "Audio"),
            MediaType::Video => write!(f, "Video"),
            MediaType::Binary => write!(f, "Binary"),
        }
    }
}

// Helper implementations

impl MediaContent {
    /// Get the size of the content in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            MediaContent::Text(text) => text.len(),
            MediaContent::Image { data, .. } => data.len(),
            MediaContent::Audio { data, .. } => data.len(),
            MediaContent::Video { data, .. } => data.len(),
            MediaContent::Binary { data, .. } => data.len(),
        }
    }

    /// Get the media type of this content
    pub fn media_type(&self) -> MediaType {
        match self {
            MediaContent::Text(_) => MediaType::Text,
            MediaContent::Image { .. } => MediaType::Image,
            MediaContent::Audio { .. } => MediaType::Audio,
            MediaContent::Video { .. } => MediaType::Video,
            MediaContent::Binary { .. } => MediaType::Binary,
        }
    }

    /// Validate size constraints
    pub fn validate_size(&self) -> Result<(), String> {
        let size = self.size_bytes();
        let (max_size, type_name) = match self {
            MediaContent::Text(_) => return Ok(()), // No size limit for text
            MediaContent::Image { .. } => (MAX_IMAGE_SIZE, "image"),
            MediaContent::Audio { .. } => (MAX_AUDIO_SIZE, "audio"),
            MediaContent::Video { .. } => (MAX_VIDEO_SIZE, "video"),
            MediaContent::Binary { .. } => (MAX_BINARY_SIZE, "binary"),
        };

        if size > max_size {
            Err(format!(
                "{type_name} size {size} bytes exceeds maximum {max_size} bytes"
            ))
        } else {
            Ok(())
        }
    }
}

impl ImageFormat {
    /// Get MIME type for the image format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Webp => "image/webp",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Svg => "image/svg+xml",
            ImageFormat::Tiff => "image/tiff",
        }
    }

    /// Get common file extensions for the format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            ImageFormat::Png => &["png"],
            ImageFormat::Jpeg => &["jpg", "jpeg"],
            ImageFormat::Webp => &["webp"],
            ImageFormat::Gif => &["gif"],
            ImageFormat::Svg => &["svg"],
            ImageFormat::Tiff => &["tif", "tiff"],
        }
    }
}

impl AudioFormat {
    /// Get MIME type for the audio format
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::M4a => "audio/mp4",
        }
    }

    /// Get common file extensions for the format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            AudioFormat::Mp3 => &["mp3"],
            AudioFormat::Wav => &["wav"],
            AudioFormat::Flac => &["flac"],
            AudioFormat::Ogg => &["ogg", "oga"],
            AudioFormat::M4a => &["m4a"],
        }
    }
}

impl VideoFormat {
    /// Get MIME type for the video format
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "video/mp4",
            VideoFormat::Webm => "video/webm",
            VideoFormat::Avi => "video/x-msvideo",
            VideoFormat::Mov => "video/quicktime",
            VideoFormat::Mkv => "video/x-matroska",
        }
    }

    /// Get common file extensions for the format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            VideoFormat::Mp4 => &["mp4"],
            VideoFormat::Webm => &["webm"],
            VideoFormat::Avi => &["avi"],
            VideoFormat::Mov => &["mov"],
            VideoFormat::Mkv => &["mkv"],
        }
    }
}

// Type conversion utilities

impl TryFrom<&str> for ImageFormat {
    type Error = String;

    fn try_from(ext: &str) -> Result<Self, Self::Error> {
        match ext.to_lowercase().as_str() {
            "png" => Ok(ImageFormat::Png),
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "webp" => Ok(ImageFormat::Webp),
            "gif" => Ok(ImageFormat::Gif),
            "svg" => Ok(ImageFormat::Svg),
            "tif" | "tiff" => Ok(ImageFormat::Tiff),
            _ => Err(format!("Unknown image format: {ext}")),
        }
    }
}

impl TryFrom<&str> for AudioFormat {
    type Error = String;

    fn try_from(ext: &str) -> Result<Self, Self::Error> {
        match ext.to_lowercase().as_str() {
            "mp3" => Ok(AudioFormat::Mp3),
            "wav" => Ok(AudioFormat::Wav),
            "flac" => Ok(AudioFormat::Flac),
            "ogg" | "oga" => Ok(AudioFormat::Ogg),
            "m4a" => Ok(AudioFormat::M4a),
            _ => Err(format!("Unknown audio format: {ext}")),
        }
    }
}

impl TryFrom<&str> for VideoFormat {
    type Error = String;

    fn try_from(ext: &str) -> Result<Self, Self::Error> {
        match ext.to_lowercase().as_str() {
            "mp4" => Ok(VideoFormat::Mp4),
            "webm" => Ok(VideoFormat::Webm),
            "avi" => Ok(VideoFormat::Avi),
            "mov" => Ok(VideoFormat::Mov),
            "mkv" => Ok(VideoFormat::Mkv),
            _ => Err(format!("Unknown video format: {ext}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_media_content_text() {
        let content = MediaContent::Text("Hello, world!".to_string());
        assert_eq!(content.size_bytes(), 13);
        assert_eq!(content.media_type(), MediaType::Text);
        assert!(content.validate_size().is_ok());
        assert_eq!(format!("{}", content), "Text(13 chars)");
    }
    #[test]
    fn test_media_content_image() {
        let content = MediaContent::Image {
            data: vec![0xFF, 0xD8, 0xFF],
            format: ImageFormat::Jpeg,
            metadata: ImageMetadata {
                width: 1920,
                height: 1080,
                color_space: ColorSpace::RGB,
                has_transparency: false,
                dpi: Some(72),
            },
        };

        assert_eq!(content.size_bytes(), 3);
        assert_eq!(content.media_type(), MediaType::Image);
        assert!(content.validate_size().is_ok());
        assert_eq!(format!("{}", content), "Image(Jpeg, 1920x1080)");
    }
    #[test]
    fn test_media_content_audio() {
        let content = MediaContent::Audio {
            data: vec![0x00; 1024],
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_ms: 180000,
                sample_rate: 44100,
                channels: 2,
                bitrate: Some(320000),
            },
        };

        assert_eq!(content.size_bytes(), 1024);
        assert_eq!(content.media_type(), MediaType::Audio);
        assert!(content.validate_size().is_ok());
        assert_eq!(format!("{}", content), "Audio(Mp3, 180000ms)");
    }
    #[test]
    fn test_media_content_video() {
        let content = MediaContent::Video {
            data: vec![0x00; 2048],
            format: VideoFormat::Mp4,
            metadata: VideoMetadata {
                duration_ms: 60000,
                width: 1920,
                height: 1080,
                fps: 30.0,
                codec: Some("h264".to_string()),
            },
        };

        assert_eq!(content.size_bytes(), 2048);
        assert_eq!(content.media_type(), MediaType::Video);
        assert!(content.validate_size().is_ok());
        assert_eq!(format!("{}", content), "Video(Mp4, 1920x1080, 60000ms)");
    }
    #[test]
    fn test_media_content_binary() {
        let content = MediaContent::Binary {
            data: vec![0x00; 512],
            mime_type: Some("application/pdf".to_string()),
            filename: Some("document.pdf".to_string()),
        };

        assert_eq!(content.size_bytes(), 512);
        assert_eq!(content.media_type(), MediaType::Binary);
        assert!(content.validate_size().is_ok());
        assert_eq!(
            format!("{}", content),
            "Binary(512 bytes, application/pdf, document.pdf)"
        );
    }
    #[test]
    fn test_size_validation() {
        // Test oversized image
        let oversized_image = MediaContent::Image {
            data: vec![0x00; MAX_IMAGE_SIZE + 1],
            format: ImageFormat::Png,
            metadata: ImageMetadata {
                width: 10000,
                height: 10000,
                color_space: ColorSpace::RGBA,
                has_transparency: true,
                dpi: None,
            },
        };

        assert!(oversized_image.validate_size().is_err());
    }
    #[test]
    fn test_image_format_conversions() {
        assert_eq!(ImageFormat::try_from("png").unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::try_from("JPG").unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::try_from("jpeg").unwrap(), ImageFormat::Jpeg);
        assert!(ImageFormat::try_from("unknown").is_err());

        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Jpeg.extensions(), &["jpg", "jpeg"]);
    }
    #[test]
    fn test_audio_format_conversions() {
        assert_eq!(AudioFormat::try_from("mp3").unwrap(), AudioFormat::Mp3);
        assert_eq!(AudioFormat::try_from("WAV").unwrap(), AudioFormat::Wav);
        assert!(AudioFormat::try_from("unknown").is_err());

        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Ogg.extensions(), &["ogg", "oga"]);
    }
    #[test]
    fn test_video_format_conversions() {
        assert_eq!(VideoFormat::try_from("mp4").unwrap(), VideoFormat::Mp4);
        assert_eq!(VideoFormat::try_from("WEBM").unwrap(), VideoFormat::Webm);
        assert!(VideoFormat::try_from("unknown").is_err());

        assert_eq!(VideoFormat::Mp4.mime_type(), "video/mp4");
        assert_eq!(VideoFormat::Mkv.extensions(), &["mkv"]);
    }
    #[test]
    fn test_display_implementations() {
        assert_eq!(format!("{}", ImageFormat::Png), "PNG");
        assert_eq!(format!("{}", AudioFormat::Mp3), "MP3");
        assert_eq!(format!("{}", VideoFormat::Mp4), "MP4");
        assert_eq!(format!("{}", ColorSpace::RGB), "RGB");
        assert_eq!(format!("{}", MediaType::Image), "Image");
    }
    #[test]
    fn test_serialization() {
        let metadata = ImageMetadata {
            width: 1920,
            height: 1080,
            color_space: ColorSpace::RGB,
            has_transparency: false,
            dpi: Some(72),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ImageMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata, deserialized);
    }
    #[test]
    fn test_media_content_serialization() {
        let content = MediaContent::Image {
            data: vec![0xFF, 0xD8],
            format: ImageFormat::Jpeg,
            metadata: ImageMetadata {
                width: 800,
                height: 600,
                color_space: ColorSpace::RGB,
                has_transparency: false,
                dpi: None,
            },
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: MediaContent = serde_json::from_str(&json).unwrap();

        match deserialized {
            MediaContent::Image {
                format, metadata, ..
            } => {
                assert_eq!(format, ImageFormat::Jpeg);
                assert_eq!(metadata.width, 800);
                assert_eq!(metadata.height, 600);
            }
            _ => panic!("Expected image content"),
        }
    }
}
