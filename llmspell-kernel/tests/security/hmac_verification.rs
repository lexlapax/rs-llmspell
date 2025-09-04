//! Security tests for HMAC signature verification
//! Critical security tests for message authentication

use anyhow::Result;
use llmspell_kernel::jupyter::wire::WireProtocol;

/// Test HMAC calculation consistency
#[tokio::test]
async fn test_hmac_calculation_consistency() -> Result<()> {
    let wire_protocol = WireProtocol::new("test-signing-key".to_string());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let hmac1 = wire_protocol.calculate_hmac(&parts);
    let hmac2 = wire_protocol.calculate_hmac(&parts);

    assert_eq!(hmac1, hmac2, "HMAC calculation should be deterministic");
    assert!(!hmac1.is_empty(), "HMAC should not be empty");
    Ok(())
}

/// Test HMAC verification with correct signature
#[tokio::test]
async fn test_hmac_verification_valid() -> Result<()> {
    let wire_protocol = WireProtocol::new("test-signing-key".to_string());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let expected_hmac = wire_protocol.calculate_hmac(&parts);
    let is_valid = wire_protocol.verify_hmac_signature(&expected_hmac, &expected_hmac);

    assert!(is_valid, "Valid HMAC signature should verify successfully");
    Ok(())
}

/// Test HMAC verification with invalid signature
#[tokio::test]
async fn test_hmac_verification_invalid() -> Result<()> {
    let wire_protocol = WireProtocol::new("test-signing-key".to_string());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let expected_hmac = wire_protocol.calculate_hmac(&parts);
    let invalid_hmac = b"invalid-hmac-signature";

    let is_valid = wire_protocol.verify_hmac_signature(invalid_hmac, &expected_hmac);
    assert!(!is_valid, "Invalid HMAC signature should fail verification");
    Ok(())
}

/// Test HMAC verification with different signing keys
#[tokio::test]
async fn test_hmac_different_keys() -> Result<()> {
    let wire_protocol1 = WireProtocol::new("key1".to_string());
    let wire_protocol2 = WireProtocol::new("key2".to_string());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let hmac1 = wire_protocol1.calculate_hmac(&parts);
    let hmac2 = wire_protocol2.calculate_hmac(&parts);

    assert_ne!(
        hmac1, hmac2,
        "Different keys should produce different HMACs"
    );

    // Verify cross-validation fails
    let is_valid = wire_protocol1.verify_hmac_signature(&hmac2, &hmac1);
    assert!(
        !is_valid,
        "HMAC from different key should fail verification"
    );
    Ok(())
}

/// Test HMAC verification with modified content
#[tokio::test]
async fn test_hmac_modified_content() -> Result<()> {
    let wire_protocol = WireProtocol::new("test-signing-key".to_string());

    let original_parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"original_content".as_slice(),
    ];

    let modified_parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"modified_content".as_slice(),
    ];

    let original_hmac = wire_protocol.calculate_hmac(&original_parts);
    let modified_hmac = wire_protocol.calculate_hmac(&modified_parts);

    assert_ne!(
        original_hmac, modified_hmac,
        "Modified content should produce different HMAC"
    );

    // Verify original HMAC fails with modified content
    let is_valid = wire_protocol.verify_hmac_signature(&original_hmac, &modified_hmac);
    assert!(!is_valid, "Original HMAC should fail with modified content");
    Ok(())
}

/// Test constant-time comparison behavior
#[tokio::test]
async fn test_constant_time_comparison() -> Result<()> {
    let wire_protocol = WireProtocol::new("test-signing-key".to_string());

    // Test equal length strings
    let sig1 = b"1234567890abcdef";
    let sig2 = b"1234567890abcdef";
    let sig3 = b"fedcba0987654321";

    assert!(
        wire_protocol.verify_hmac_signature(sig1, sig2),
        "Equal signatures should verify"
    );
    assert!(
        !wire_protocol.verify_hmac_signature(sig1, sig3),
        "Different signatures should fail"
    );

    // Test different length strings (should fail fast)
    let short_sig = b"short";
    let long_sig = b"much_longer_signature";

    assert!(
        !wire_protocol.verify_hmac_signature(short_sig, long_sig),
        "Different length signatures should fail"
    );
    assert!(
        !wire_protocol.verify_hmac_signature(long_sig, short_sig),
        "Different length signatures should fail"
    );
    Ok(())
}

/// Test HMAC with empty signing key
#[tokio::test]
async fn test_hmac_empty_key() -> Result<()> {
    let wire_protocol = WireProtocol::new(String::new());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let hmac = wire_protocol.calculate_hmac(&parts);
    // Empty key should still produce a valid HMAC (though not secure)
    assert!(
        !hmac.is_empty(),
        "HMAC with empty key should still be calculated"
    );
    Ok(())
}

/// Test HMAC with special characters in key
#[tokio::test]
async fn test_hmac_special_chars_key() -> Result<()> {
    let wire_protocol = WireProtocol::new("key-with-special!@#$%^&*()_+chars".to_string());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let hmac = wire_protocol.calculate_hmac(&parts);
    assert!(
        !hmac.is_empty(),
        "HMAC with special characters key should work"
    );

    // Verify it's consistent
    let hmac2 = wire_protocol.calculate_hmac(&parts);
    assert_eq!(
        hmac, hmac2,
        "HMAC with special characters should be consistent"
    );
    Ok(())
}

/// Test HMAC with binary data in message parts
#[tokio::test]
async fn test_hmac_binary_data() -> Result<()> {
    let wire_protocol = WireProtocol::new("binary-test-key".to_string());

    let binary_parts = [
        &[0x00, 0x01, 0x02, 0x03][..], // Binary header
        &[0xFF, 0xFE, 0xFD, 0xFC][..], // Binary parent_header
        &[0x80, 0x90, 0xA0, 0xB0][..], // Binary metadata
        &[0x10, 0x20, 0x30, 0x40][..], // Binary content
    ];

    let hmac = wire_protocol.calculate_hmac(&binary_parts);
    assert!(!hmac.is_empty(), "HMAC should handle binary data");

    // Verify consistency with binary data
    let hmac2 = wire_protocol.calculate_hmac(&binary_parts);
    assert_eq!(hmac, hmac2, "HMAC with binary data should be consistent");
    Ok(())
}

/// Test HMAC verification timing resistance (basic test)
#[tokio::test]
async fn test_hmac_timing_resistance() -> Result<()> {
    let wire_protocol = WireProtocol::new("timing-test-key".to_string());

    let parts = [
        b"header".as_slice(),
        b"parent_header".as_slice(),
        b"metadata".as_slice(),
        b"content".as_slice(),
    ];

    let correct_hmac = wire_protocol.calculate_hmac(&parts);

    // Test with signatures that differ in first byte vs last byte
    let mut early_diff = correct_hmac.clone();
    if !early_diff.is_empty() {
        early_diff[0] = early_diff[0].wrapping_add(1);
    }

    let mut late_diff = correct_hmac.clone();
    if !late_diff.is_empty() {
        let len = late_diff.len();
        late_diff[len - 1] = late_diff[len - 1].wrapping_add(1);
    }

    // Both should fail in constant time (we can't easily test timing here)
    assert!(!wire_protocol.verify_hmac_signature(&early_diff, &correct_hmac));
    assert!(!wire_protocol.verify_hmac_signature(&late_diff, &correct_hmac));
    Ok(())
}

/// Test HMAC with very large message parts
#[tokio::test]
async fn test_hmac_large_data() -> Result<()> {
    let wire_protocol = WireProtocol::new("large-data-key".to_string());

    // Create large message parts (1KB each)
    let large_data = vec![0xAB; 1024];
    let parts = [
        large_data.as_slice(),
        large_data.as_slice(),
        large_data.as_slice(),
        large_data.as_slice(),
    ];

    let hmac = wire_protocol.calculate_hmac(&parts);
    assert!(!hmac.is_empty(), "HMAC should handle large data");

    // Verify it's different from small data HMAC
    let small_parts = [
        b"a".as_slice(),
        b"b".as_slice(),
        b"c".as_slice(),
        b"d".as_slice(),
    ];
    let small_hmac = wire_protocol.calculate_hmac(&small_parts);
    assert_ne!(
        hmac, small_hmac,
        "Large data HMAC should differ from small data HMAC"
    );
    Ok(())
}
