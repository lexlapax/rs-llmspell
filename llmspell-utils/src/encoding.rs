// ABOUTME: Encoding and hashing utilities for llmspell
// ABOUTME: Provides hash calculation, base64 encoding, and other encoding functions

//! Encoding and hashing utilities
//!
//! This module provides various encoding and hashing functions including:
//! - Hash calculation (MD5, SHA-1, SHA-256, SHA-512)
//! - Base64 encoding/decoding
//! - Hex encoding/decoding

use base64::{engine::general_purpose, Engine as _};
use chardetng::EncodingDetector;
use encoding_rs::UTF_8;
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

/// Text encoding types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextEncoding {
    /// UTF-8 encoding
    Utf8,
    /// UTF-16 Little Endian
    Utf16Le,
    /// UTF-16 Big Endian
    Utf16Be,
    /// Windows-1252 (Western European)
    Windows1252,
    /// ISO-8859-1 (Latin-1)
    Iso88591,
    /// ASCII encoding
    Ascii,
    /// Unknown/other encoding
    Unknown,
}

impl std::fmt::Display for TextEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextEncoding::Utf8 => write!(f, "UTF-8"),
            TextEncoding::Utf16Le => write!(f, "UTF-16LE"),
            TextEncoding::Utf16Be => write!(f, "UTF-16BE"),
            TextEncoding::Windows1252 => write!(f, "Windows-1252"),
            TextEncoding::Iso88591 => write!(f, "ISO-8859-1"),
            TextEncoding::Ascii => write!(f, "ASCII"),
            TextEncoding::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Line ending types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// Unix line ending (LF)
    Lf,
    /// Windows line ending (CRLF)
    Crlf,
    /// Classic Mac line ending (CR)
    Cr,
    /// Mixed line endings
    Mixed,
}

impl std::fmt::Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineEnding::Lf => write!(f, "LF"),
            LineEnding::Crlf => write!(f, "CRLF"),
            LineEnding::Cr => write!(f, "CR"),
            LineEnding::Mixed => write!(f, "Mixed"),
        }
    }
}

/// Detect the encoding of text data
#[must_use]
pub fn detect_text_encoding(data: &[u8]) -> TextEncoding {
    // Check for BOM first
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return TextEncoding::Utf8;
    }
    if data.starts_with(&[0xFF, 0xFE]) {
        return TextEncoding::Utf16Le;
    }
    if data.starts_with(&[0xFE, 0xFF]) {
        return TextEncoding::Utf16Be;
    }

    // Use chardet for detection
    let mut detector = EncodingDetector::new();
    detector.feed(data, true);
    let encoding = detector.guess(None, true);

    match encoding.name() {
        "UTF-8" => TextEncoding::Utf8,
        "UTF-16LE" => TextEncoding::Utf16Le,
        "UTF-16BE" => TextEncoding::Utf16Be,
        "windows-1252" => TextEncoding::Windows1252,
        "ISO-8859-1" => TextEncoding::Iso88591,
        "ASCII" => TextEncoding::Ascii,
        _ => TextEncoding::Unknown,
    }
}

/// Convert text from one encoding to another
///
/// # Errors
///
/// Returns an error if the source encoding is not supported or conversion fails
pub fn convert_text_encoding(
    data: &[u8],
    from_encoding: TextEncoding,
    to_encoding: TextEncoding,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Convert from source encoding to UTF-8 string
    let text = decode_text(data, from_encoding)?;

    // Convert from UTF-8 string to target encoding
    encode_text(&text, to_encoding)
}

/// Decode text data to UTF-8 string
///
/// # Errors
///
/// Returns an error if the encoding is not supported or decoding fails
pub fn decode_text(
    data: &[u8],
    encoding: TextEncoding,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let encoding_rs = match encoding {
        TextEncoding::Utf8 => UTF_8,
        TextEncoding::Utf16Le => encoding_rs::UTF_16LE,
        TextEncoding::Utf16Be => encoding_rs::UTF_16BE,
        TextEncoding::Windows1252 | TextEncoding::Iso88591 => encoding_rs::WINDOWS_1252,
        TextEncoding::Ascii => encoding_rs::UTF_8, // ASCII is subset of UTF-8
        TextEncoding::Unknown => return Err("Unknown encoding cannot be decoded".into()),
    };

    let (text, _, had_errors) = encoding_rs.decode(data);
    if had_errors && encoding != TextEncoding::Unknown {
        return Err("Decoding errors encountered".into());
    }

    Ok(text.into_owned())
}

/// Encode UTF-8 string to specified encoding
///
/// # Errors
///
/// Returns an error if the encoding is not supported or encoding fails
pub fn encode_text(
    text: &str,
    encoding: TextEncoding,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let encoding_rs = match encoding {
        TextEncoding::Utf8 => UTF_8,
        TextEncoding::Utf16Le => encoding_rs::UTF_16LE,
        TextEncoding::Utf16Be => encoding_rs::UTF_16BE,
        TextEncoding::Windows1252 | TextEncoding::Iso88591 => encoding_rs::WINDOWS_1252,
        TextEncoding::Ascii => encoding_rs::UTF_8, // ASCII is subset of UTF-8
        TextEncoding::Unknown => return Err("Cannot encode to unknown encoding".into()),
    };

    let (encoded, _, had_errors) = encoding_rs.encode(text);
    if had_errors {
        return Err("Encoding errors encountered".into());
    }

    Ok(encoded.into_owned())
}

/// Detect line ending type in text
#[must_use]
pub fn detect_line_ending(text: &str) -> LineEnding {
    let crlf_count = text.matches("\r\n").count();
    let lf_count = text.matches('\n').count() - crlf_count; // Subtract CRLF instances
    let cr_count = text.matches('\r').count() - crlf_count; // Subtract CRLF instances

    let total = crlf_count + lf_count + cr_count;
    if total == 0 {
        return LineEnding::Lf; // Default to LF if no line endings found
    }

    // Check if mixed
    let mut types_found = 0;
    if crlf_count > 0 {
        types_found += 1;
    }
    if lf_count > 0 {
        types_found += 1;
    }
    if cr_count > 0 {
        types_found += 1;
    }

    if types_found > 1 {
        return LineEnding::Mixed;
    }

    // Return the dominant type
    if crlf_count > 0 {
        LineEnding::Crlf
    } else if lf_count > 0 {
        LineEnding::Lf
    } else {
        LineEnding::Cr
    }
}

/// Convert line endings in text
#[must_use]
pub fn convert_line_endings(text: &str, target: LineEnding) -> String {
    let target_ending = match target {
        LineEnding::Lf => "\n",
        LineEnding::Crlf => "\r\n",
        LineEnding::Cr => "\r",
        LineEnding::Mixed => return text.to_string(), // No conversion for mixed
    };

    // First normalize all line endings to LF
    let normalized = text
        .replace("\r\n", "\n") // CRLF -> LF
        .replace('\r', "\n"); // CR -> LF

    // Then convert to target
    if target_ending == "\n" {
        normalized
    } else {
        normalized.replace('\n', target_ending)
    }
}

/// Convert tabs to spaces
#[must_use]
pub fn tabs_to_spaces(text: &str, tab_size: usize) -> String {
    let spaces = " ".repeat(tab_size);
    text.replace('\t', &spaces)
}

/// Convert spaces to tabs (converts groups of spaces equal to `tab_size`)
#[must_use]
pub fn spaces_to_tabs(text: &str, tab_size: usize) -> String {
    if tab_size == 0 {
        return text.to_string();
    }

    let spaces = " ".repeat(tab_size);
    text.replace(&spaces, "\t")
}

/// Remove BOM (Byte Order Mark) from data if present
#[must_use]
pub fn remove_bom(data: &[u8]) -> &[u8] {
    // UTF-8 BOM
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return &data[3..];
    }
    // UTF-16LE BOM
    if data.starts_with(&[0xFF, 0xFE]) {
        return &data[2..];
    }
    // UTF-16BE BOM
    if data.starts_with(&[0xFE, 0xFF]) {
        return &data[2..];
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hex_conversion() {
        let data = vec![0xFF, 0x00, 0xAB, 0xCD];
        let hex = to_hex_string(&data);
        assert_eq!(hex, "ff00abcd");

        let parsed = from_hex_string(&hex).unwrap();
        assert_eq!(parsed, data);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_text_encoding_detection() {
        // UTF-8 with BOM
        let utf8_bom = [0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
        assert_eq!(detect_text_encoding(&utf8_bom), TextEncoding::Utf8);

        // UTF-16LE with BOM
        let utf16le_bom = [0xFF, 0xFE, b'H', 0x00, b'e', 0x00];
        assert_eq!(detect_text_encoding(&utf16le_bom), TextEncoding::Utf16Le);

        // UTF-16BE with BOM
        let utf16be_bom_data = [0xFE, 0xFF, 0x00, b'H', 0x00, b'e'];
        assert_eq!(
            detect_text_encoding(&utf16be_bom_data),
            TextEncoding::Utf16Be
        );

        // Plain ASCII/UTF-8
        let ascii = b"Hello World";
        let detected = detect_text_encoding(ascii);
        assert!(matches!(detected, TextEncoding::Utf8 | TextEncoding::Ascii));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_text_encoding_conversion() {
        // Test simple ASCII conversion
        let ascii_text = "Hello World";
        let ascii_bytes = ascii_text.as_bytes();

        // UTF-8 to UTF-8 (identity)
        let converted =
            convert_text_encoding(ascii_bytes, TextEncoding::Utf8, TextEncoding::Utf8).unwrap();
        assert_eq!(converted, ascii_bytes);

        // Test encoding to different formats (one-way tests to avoid BOM issues)
        let utf16le = convert_text_encoding(ascii_bytes, TextEncoding::Utf8, TextEncoding::Utf16Le);
        assert!(utf16le.is_ok());
        assert!(!utf16le.unwrap().is_empty());

        let utf16be_result =
            convert_text_encoding(ascii_bytes, TextEncoding::Utf8, TextEncoding::Utf16Be);
        assert!(utf16be_result.is_ok());
        assert!(!utf16be_result.unwrap().is_empty());

        // Test basic Windows-1252 conversion for basic ASCII
        let ascii_only = b"Hello"; // Pure ASCII should work fine
        let windows_utf8 =
            convert_text_encoding(ascii_only, TextEncoding::Windows1252, TextEncoding::Utf8)
                .unwrap();
        let windows_back =
            convert_text_encoding(&windows_utf8, TextEncoding::Utf8, TextEncoding::Windows1252)
                .unwrap();
        assert_eq!(windows_back, ascii_only);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_line_ending_detection() {
        assert_eq!(detect_line_ending("line1\nline2\n"), LineEnding::Lf);
        assert_eq!(detect_line_ending("line1\r\nline2\r\n"), LineEnding::Crlf);
        assert_eq!(detect_line_ending("line1\rline2\r"), LineEnding::Cr);
        assert_eq!(
            detect_line_ending("line1\nline2\r\nline3"),
            LineEnding::Mixed
        );
        assert_eq!(detect_line_ending("no line endings"), LineEnding::Lf); // Default
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_line_ending_conversion() {
        let text = "line1\r\nline2\nline3\r";

        // Convert to LF
        let lf = convert_line_endings(text, LineEnding::Lf);
        assert_eq!(lf, "line1\nline2\nline3\n");

        // Convert to CRLF
        let crlf = convert_line_endings(text, LineEnding::Crlf);
        assert_eq!(crlf, "line1\r\nline2\r\nline3\r\n");

        // Convert to CR
        let cr = convert_line_endings(text, LineEnding::Cr);
        assert_eq!(cr, "line1\rline2\rline3\r");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_tabs_spaces_conversion() {
        let text_with_tabs = "line1\tindented\ttext";
        let text_with_spaces = "line1    indented    text";

        // Tabs to spaces
        let spaces = tabs_to_spaces(text_with_tabs, 4);
        assert_eq!(spaces, text_with_spaces);

        // Spaces to tabs
        let tabs = spaces_to_tabs(text_with_spaces, 4);
        assert_eq!(tabs, text_with_tabs);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_bom_removal() {
        // UTF-8 BOM
        let utf8_with_bom = [0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
        let without_bom = remove_bom(&utf8_with_bom);
        assert_eq!(without_bom, b"Hello");

        // UTF-16LE BOM
        let utf16le_with_bom = [0xFF, 0xFE, b'H', 0x00, b'e', 0x00];
        let without_bom = remove_bom(&utf16le_with_bom);
        assert_eq!(without_bom, &[b'H', 0x00, b'e', 0x00]);

        // No BOM
        let no_bom = b"Hello";
        let result = remove_bom(no_bom);
        assert_eq!(result, no_bom);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_decode_encode_text() {
        let text = "Hello, World!";
        let utf8_bytes = text.as_bytes();

        // Decode UTF-8
        let decoded = decode_text(utf8_bytes, TextEncoding::Utf8).unwrap();
        assert_eq!(decoded, text);

        // Encode to UTF-8
        let encoded = encode_text(text, TextEncoding::Utf8).unwrap();
        assert_eq!(encoded, utf8_bytes);
    }
}
