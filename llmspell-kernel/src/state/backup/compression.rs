// ABOUTME: Backup compression implementation with multiple algorithm support
// ABOUTME: Targets >70% compression ratio for typical state data

use super::super::config::CompressionType;
use crate::state::StateError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tracing::debug;

// CompressionType is imported from config module

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionLevel(u32);

impl CompressionLevel {
    /// Create new compression level (1-9)
    ///
    /// # Errors
    ///
    /// Returns `StateError::ValidationError` if:
    /// - Level is not between 1 and 9 inclusive
    pub fn new(level: u32) -> Result<Self, StateError> {
        if !(1..=9).contains(&level) {
            return Err(StateError::validation_error(
                "Compression level must be between 1 and 9".to_string(),
            ));
        }
        Ok(Self(level))
    }

    /// Get as u32
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Fast compression (level 1)
    pub fn fast() -> Self {
        Self(1)
    }

    /// Default compression (level 3)
    pub fn default_level() -> Self {
        Self(3)
    }

    /// Best compression (level 9)
    pub fn best() -> Self {
        Self(9)
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::default_level()
    }
}

/// Backup compression handler
pub struct BackupCompression {
    compression_type: CompressionType,
    compression_level: CompressionLevel,
}

impl BackupCompression {
    /// Create new compression handler
    pub fn new(compression_type: CompressionType, level: CompressionLevel) -> Self {
        Self {
            compression_type,
            compression_level: level,
        }
    }

    /// Compress data
    ///
    /// # Errors
    ///
    /// Returns `StateError` if:
    /// - Compression algorithm fails to compress the data
    /// - Output buffer allocation fails
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>, StateError> {
        let start_size = data.len();
        debug!(
            "Compressing {} bytes with {} level {}",
            start_size, self.compression_type, self.compression_level.0
        );

        let compressed = match self.compression_type {
            CompressionType::None => data.to_vec(),
            CompressionType::Gzip => self.compress_gzip(data)?,
            CompressionType::Zstd => self.compress_zstd(data)?,
            CompressionType::Lz4 => self.compress_lz4(data)?,
            CompressionType::Brotli => self.compress_brotli(data)?,
        };

        let compressed_size = compressed.len();
        let ratio = if start_size > 0 && compressed_size < start_size {
            #[allow(clippy::cast_precision_loss)]
            let size_diff = (start_size - compressed_size) as f64;
            #[allow(clippy::cast_precision_loss)]
            let start_size_f64 = start_size as f64;
            (size_diff / start_size_f64) * 100.0
        } else {
            0.0
        };

        debug!(
            "Compression complete: {} -> {} bytes ({:.1}% reduction)",
            start_size, compressed_size, ratio
        );

        Ok(compressed)
    }

    /// Decompress data
    ///
    /// # Errors
    ///
    /// Returns `StateError` if:
    /// - Data is corrupted or invalid
    /// - Wrong compression algorithm for the data
    /// - Decompression buffer allocation fails
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, StateError> {
        debug!(
            "Decompressing {} bytes with {}",
            data.len(),
            self.compression_type
        );

        let decompressed = match self.compression_type {
            CompressionType::None => data.to_vec(),
            CompressionType::Gzip => Self::decompress_gzip(data)?,
            CompressionType::Zstd => Self::decompress_zstd(data)?,
            CompressionType::Lz4 => Self::decompress_lz4(data)?,
            CompressionType::Brotli => Self::decompress_brotli(data)?,
        };

        debug!("Decompression complete: {} bytes", decompressed.len());
        Ok(decompressed)
    }

    /// Compress with gzip
    fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>, StateError> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.compression_level.0));
        encoder
            .write_all(data)
            .map_err(|e| StateError::storage(format!("Compression error: {e}")))?;
        encoder
            .finish()
            .map_err(|e| StateError::storage(format!("Compression error: {e}")))
    }

    /// Decompress gzip
    fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, StateError> {
        use flate2::read::GzDecoder;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| StateError::storage(format!("Decompression error: {e}")))?;
        Ok(decompressed)
    }

    /// Compress with zstd
    fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>, StateError> {
        #[allow(clippy::cast_possible_wrap)]
        let level_i32 = self.compression_level.0 as i32;
        zstd::encode_all(data, level_i32)
            .map_err(|e| StateError::storage(format!("Compression error: {e}")))
    }

    /// Decompress zstd
    fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>, StateError> {
        zstd::decode_all(data).map_err(|e| StateError::storage(format!("Decompression error: {e}")))
    }

    /// Compress with lz4
    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, StateError> {
        // LZ4 doesn't use traditional compression levels, but we can map them
        let acceleration = match self.compression_level.0 {
            1..=3 => 1, // Fast
            4..=6 => 3, // Default
            7..=9 => 9, // Best
            _ => 3,
        };

        lz4::block::compress(
            data,
            Some(lz4::block::CompressionMode::HIGHCOMPRESSION(acceleration)),
            true,
        )
        .map_err(|e| StateError::storage(format!("Compression error: {e}")))
    }

    /// Decompress lz4
    fn decompress_lz4(data: &[u8]) -> Result<Vec<u8>, StateError> {
        lz4::block::decompress(data, None)
            .map_err(|e| StateError::storage(format!("Decompression error: {e}")))
    }

    /// Compress with brotli
    fn compress_brotli(&self, data: &[u8]) -> Result<Vec<u8>, StateError> {
        let mut output = Vec::new();
        let params = brotli::enc::BrotliEncoderParams {
            #[allow(clippy::cast_possible_wrap)]
            quality: self.compression_level.0 as i32,
            ..Default::default()
        };

        brotli::BrotliCompress(&mut std::io::Cursor::new(data), &mut output, &params)
            .map_err(|e| StateError::storage(format!("Compression error: {e}")))?;

        Ok(output)
    }

    /// Decompress brotli
    fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, StateError> {
        let mut output = Vec::new();
        brotli::BrotliDecompress(&mut std::io::Cursor::new(data), &mut output)
            .map_err(|e| StateError::storage(format!("Decompression error: {e}")))?;

        Ok(output)
    }

    /// Analyze compression efficiency for data
    pub fn analyze_compression(&self, data: &[u8]) -> CompressionAnalysis {
        let start_time = std::time::Instant::now();
        let original_size = data.len();

        let compressed = match self.compress(data) {
            Ok(c) => c,
            Err(_) => {
                return CompressionAnalysis {
                    original_size,
                    compressed_size: original_size,
                    compression_ratio: 0.0,
                    compression_time_ms: 0,
                    estimated_decompression_time_ms: 0,
                    algorithm: self.compression_type,
                    level: self.compression_level,
                    is_compressible: false,
                };
            }
        };

        let compression_time = start_time.elapsed();
        let compressed_size = compressed.len();

        // Test decompression time
        let decompress_start = std::time::Instant::now();
        let _ = self.decompress(&compressed);
        let decompression_time = decompress_start.elapsed();

        let compression_ratio = if original_size > 0 && compressed_size < original_size {
            #[allow(clippy::cast_precision_loss)]
            let size_diff = (original_size - compressed_size) as f64;
            #[allow(clippy::cast_precision_loss)]
            let original_size_f64 = original_size as f64;
            (size_diff / original_size_f64) * 100.0
        } else {
            0.0
        };

        CompressionAnalysis {
            original_size,
            compressed_size,
            compression_ratio,
            #[allow(clippy::cast_possible_truncation)]
            compression_time_ms: compression_time.as_millis() as u64,
            #[allow(clippy::cast_possible_truncation)]
            estimated_decompression_time_ms: decompression_time.as_millis() as u64,
            algorithm: self.compression_type,
            level: self.compression_level,
            is_compressible: compression_ratio > 10.0, // Consider >10% reduction as compressible
        }
    }
}

/// Compression analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionAnalysis {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub compression_time_ms: u64,
    pub estimated_decompression_time_ms: u64,
    pub algorithm: CompressionType,
    pub level: CompressionLevel,
    pub is_compressible: bool,
}

/// Find optimal compression settings for data
///
/// # Errors
///
/// Returns `StateError` if:
/// - All compression algorithms fail to compress the data
/// - Time measurement fails
pub fn find_optimal_compression(
    data: &[u8],
    max_time_ms: u64,
) -> Result<(CompressionType, CompressionLevel), StateError> {
    let algorithms = vec![
        CompressionType::Lz4,    // Fastest
        CompressionType::Zstd,   // Balanced
        CompressionType::Gzip,   // Compatible
        CompressionType::Brotli, // Best ratio
    ];

    let levels = vec![
        CompressionLevel::fast(),
        CompressionLevel::default(),
        CompressionLevel::best(),
    ];

    let mut best_ratio = 0.0;
    let mut best_config = (CompressionType::Zstd, CompressionLevel::default());

    for algorithm in algorithms {
        for level in &levels {
            let compressor = BackupCompression::new(algorithm, *level);
            let analysis = compressor.analyze_compression(data);

            // Skip if too slow
            if analysis.compression_time_ms > max_time_ms {
                continue;
            }

            // Update best if better ratio
            if analysis.compression_ratio > best_ratio {
                best_ratio = analysis.compression_ratio;
                best_config = (algorithm, *level);
            }
        }
    }

    debug!(
        "Optimal compression: {} level {} ({:.1}% reduction)",
        best_config.0, best_config.1 .0, best_ratio
    );

    Ok(best_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compression_level_validation() {
        assert!(CompressionLevel::new(0).is_err());
        assert!(CompressionLevel::new(5).is_ok());
        assert!(CompressionLevel::new(10).is_err());
    }
    #[test]
    fn test_compression_type_extension() {
        assert_eq!(CompressionType::None.extension(), "");
        assert_eq!(CompressionType::Gzip.extension(), ".gz");
        assert_eq!(CompressionType::Zstd.extension(), ".zst");
        assert_eq!(CompressionType::Lz4.extension(), ".lz4");
        assert_eq!(CompressionType::Brotli.extension(), ".br");
    }
    #[test]
    fn test_compression_roundtrip() {
        let data = b"Hello, World! This is test data for compression.".repeat(100);
        let compressor = BackupCompression::new(CompressionType::Zstd, CompressionLevel::default());

        let compressed = compressor.compress(&data).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
    #[test]
    fn test_all_compression_algorithms() {
        let data = b"Test data for all compression algorithms".repeat(50);
        let algorithms = vec![
            CompressionType::None,
            CompressionType::Gzip,
            CompressionType::Zstd,
            CompressionType::Lz4,
            CompressionType::Brotli,
        ];

        for algorithm in algorithms {
            let compressor = BackupCompression::new(algorithm, CompressionLevel::default());
            let compressed = compressor.compress(&data).unwrap();
            let decompressed = compressor.decompress(&compressed).unwrap();
            assert_eq!(decompressed, data, "Failed for {:?}", algorithm);
        }
    }
    #[test]
    fn test_compression_analysis() {
        let data = b"Highly compressible data ".repeat(100);
        let compressor = BackupCompression::new(CompressionType::Zstd, CompressionLevel::default());

        let analysis = compressor.analyze_compression(&data);
        assert!(analysis.is_compressible);
        assert!(analysis.compression_ratio > 50.0); // Should achieve >50% compression
        assert!(analysis.compressed_size < analysis.original_size);
    }
    #[test]
    fn test_compression_edge_cases() {
        // Test 1: Empty data
        let empty_data = b"";
        let compressor = BackupCompression::new(CompressionType::Zstd, CompressionLevel::default());
        let compressed = compressor.compress(empty_data).unwrap();
        assert!(!compressed.is_empty()); // Even empty data has compression headers
        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, empty_data);

        // Test 2: Small incompressible data (may expand due to compression headers)
        let small_data = b"xyz123";
        let compressed_small = compressor.compress(small_data).unwrap();
        // This might be larger than original due to compression overhead
        let decompressed_small = compressor.decompress(&compressed_small).unwrap();
        assert_eq!(decompressed_small, small_data);

        // Test 3: Already compressed/random data
        let random_data = vec![0xFF, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE];
        let compressed_random = compressor.compress(&random_data).unwrap();
        let decompressed_random = compressor.decompress(&compressed_random).unwrap();
        assert_eq!(decompressed_random, random_data);
    }
    #[test]
    fn test_compression_ratio_calculation() {
        // Test that compression ratio calculation doesn't panic
        let compressor = BackupCompression::new(CompressionType::Gzip, CompressionLevel::default());

        // Case 1: Data that compresses well
        let good_data = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec();
        let analysis1 = compressor.analyze_compression(&good_data);
        assert!(analysis1.compression_ratio > 0.0);

        // Case 2: Data that doesn't compress (might expand)
        let bad_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let analysis2 = compressor.analyze_compression(&bad_data);
        // Should not panic even if compression ratio is 0 or negative
        assert!(analysis2.compression_ratio >= 0.0);

        // Case 3: Empty data
        let empty_data = vec![];
        let analysis3 = compressor.analyze_compression(&empty_data);
        assert_eq!(analysis3.compression_ratio, 0.0);
    }
}
