// ABOUTME: Encoding and hashing utilities for llmspell
// ABOUTME: Provides hash calculation, base64 encoding, and other encoding functions

//! Encoding and hashing utilities
//!
//! This module provides various encoding and hashing functions including:
//! - Hash calculation (MD5, SHA-1, SHA-256, SHA-512)
//! - Base64 encoding/decoding
//! - Hex encoding/decoding

use base64::{engine::general_purpose, Engine as _};
use md5;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Supported hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    /// MD5 hash algorithm (128-bit)
    Md5,
    /// SHA-1 hash algorithm (160-bit)
    Sha1,
    /// SHA-256 hash algorithm (256-bit)
    Sha256,
    /// SHA-512 hash algorithm (512-bit)
    Sha512,
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Md5 => write!(f, "MD5"),
            HashAlgorithm::Sha1 => write!(f, "SHA-1"),
            HashAlgorithm::Sha256 => write!(f, "SHA-256"),
            HashAlgorithm::Sha512 => write!(f, "SHA-512"),
        }
    }
}

/// Calculate hash of data using specified algorithm
#[must_use]
pub fn hash_data(data: &[u8], algorithm: HashAlgorithm) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Md5 => md5::compute(data).0.to_vec(),
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
    }
}

/// Calculate hash of a string
#[must_use]
pub fn hash_string(text: &str, algorithm: HashAlgorithm) -> Vec<u8> {
    hash_data(text.as_bytes(), algorithm)
}

/// Calculate hash of a file with streaming (memory efficient)
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read
pub fn hash_file<P: AsRef<Path>>(path: P, algorithm: HashAlgorithm) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 8192]; // 8KB buffer

    match algorithm {
        HashAlgorithm::Md5 => {
            let mut hasher = md5::Context::new();
            loop {
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.consume(&buffer[..bytes_read]);
            }
            Ok(hasher.compute().0.to_vec())
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            loop {
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            loop {
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            loop {
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(hasher.finalize().to_vec())
        }
    }
}

/// Convert bytes to hexadecimal string
#[must_use]
pub fn to_hex_string(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}

/// Parse hexadecimal string to bytes
///
/// # Errors
///
/// Returns an error if the hex string is invalid
pub fn from_hex_string(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex)
}

/// Verify if data matches a given hash
#[must_use]
pub fn verify_hash(data: &[u8], expected_hash: &[u8], algorithm: HashAlgorithm) -> bool {
    let calculated = hash_data(data, algorithm);
    calculated == expected_hash
}

/// Base64 encode data
#[must_use]
pub fn base64_encode(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Base64 decode data
///
/// # Errors
///
/// Returns an error if the base64 string is invalid
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(encoded)
}

/// Base64 encode (URL-safe variant)
#[must_use]
pub fn base64_encode_url_safe(data: &[u8]) -> String {
    general_purpose::URL_SAFE.encode(data)
}

/// Base64 decode (URL-safe variant)
///
/// # Errors
///
/// Returns an error if the base64 string is invalid
pub fn base64_decode_url_safe(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::URL_SAFE.decode(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        let text = "Hello, World!";

        // Test MD5
        let md5_hash = hash_string(text, HashAlgorithm::Md5);
        let md5_hex = to_hex_string(&md5_hash);
        assert_eq!(md5_hex, "65a8e27d8879283831b664bd8b7f0ad4");

        // Test SHA-256
        let sha256_hash = hash_string(text, HashAlgorithm::Sha256);
        let sha256_hex = to_hex_string(&sha256_hash);
        assert_eq!(
            sha256_hex,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_verify_hash() {
        let data = b"test data";
        let hash = hash_data(data, HashAlgorithm::Sha256);

        assert!(verify_hash(data, &hash, HashAlgorithm::Sha256));
        assert!(!verify_hash(
            b"different data",
            &hash,
            HashAlgorithm::Sha256
        ));
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"Hello, Base64!";

        // Standard encoding
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);

        // URL-safe encoding
        let url_encoded = base64_encode_url_safe(data);
        let url_decoded = base64_decode_url_safe(&url_encoded).unwrap();
        assert_eq!(url_decoded, data);
    }

    #[test]
    fn test_hex_conversion() {
        let data = vec![0xFF, 0x00, 0xAB, 0xCD];
        let hex = to_hex_string(&data);
        assert_eq!(hex, "ff00abcd");

        let parsed = from_hex_string(&hex).unwrap();
        assert_eq!(parsed, data);
    }
}
