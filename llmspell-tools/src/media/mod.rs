//! ABOUTME: Media processing tools module for audio, video, and image file operations
//! ABOUTME: Provides format detection, metadata extraction, and basic media transformations

pub mod audio_processor;
pub mod image_processor;
pub mod video_processor;

pub use audio_processor::{AudioMetadata, AudioProcessorConfig, AudioProcessorTool};
pub use image_processor::{ImageMetadata, ImageProcessorConfig, ImageProcessorTool};
pub use video_processor::{VideoMetadata, VideoProcessorConfig, VideoProcessorTool};
