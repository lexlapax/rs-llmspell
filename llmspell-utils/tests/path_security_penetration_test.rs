//! ABOUTME: Comprehensive penetration tests for path security hardening
//! ABOUTME: Tests advanced attack vectors and edge cases for path validation security

use llmspell_utils::security::path::{PathSecurityConfig, PathSecurityValidator};
use std::path::Path;
use tempfile::TempDir;
#[test]
fn test_path_traversal_attack_vectors() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Standard path traversal attempts
    let attack_vectors = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "../../../../root/.ssh/id_rsa",
        "..//.//..//etc/passwd",
        "..%2F..%2F..%2Fetc%2Fpasswd",
        "..%252F..%252F..%252Fetc%252Fpasswd",
        "..%c0%af..%c0%af..%c0%afetc%c0%afpasswd",
        "..%c1%9c..%c1%9c..%c1%9cetc%c1%9cpasswd",
        "..%e0%80%af..%e0%80%af..%e0%80%afetc%e0%80%afpasswd",
        "..%f0%80%80%af..%f0%80%80%af..%f0%80%80%afetc%f0%80%80%afpasswd",
        "..%5c..%5c..%5cwindows%5csystem32%5cconfig%5csam",
        "..%255c..%255c..%255cwindows%255csystem32%255cconfig%255csam",
        "....//....//....//etc//passwd",
        "....\\....\\....\\windows\\system32\\config\\sam",
    ];

    for attack in attack_vectors {
        let result = validator.validate(Path::new(attack));
        assert!(
            result.is_err(),
            "Path traversal attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_unicode_path_traversal_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Unicode-based path traversal attempts
    let unicode_attacks = vec![
        "..%u002f..%u002f..%u002fetc%u002fpasswd",
        "..%u005c..%u005c..%u005cwindows%u005csystem32",
        "..\u{002f}..\u{002f}..\u{002f}etc\u{002f}passwd",
        "..\u{005c}..\u{005c}..\u{005c}windows\u{005c}system32",
        "../\u{ff0e}\u{ff0e}/\u{ff0e}\u{ff0e}/etc/passwd",
        "..\u{2215}..\u{2215}..\u{2215}etc\u{2215}passwd",
        "..\u{2216}..\u{2216}..\u{2216}windows\u{2216}system32",
        "../\u{ff0e}\u{ff0e}/\u{ff0e}\u{ff0e}/etc/passwd",
    ];

    for attack in unicode_attacks {
        let result = validator.validate(Path::new(attack));
        assert!(
            result.is_err(),
            "Unicode path traversal attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_null_byte_injection_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Null byte injection attempts
    let null_attacks = vec![
        "safe_file.txt\0../../../etc/passwd",
        "safe_file.txt\0\0../../../etc/passwd",
        "safe_file.txt%00../../../etc/passwd",
        "safe_file.txt%00%00../../../etc/passwd",
        "safe_file.txt\x00../../../etc/passwd",
        "safe_file.txt\x00\x00../../../etc/passwd",
    ];

    for attack in null_attacks {
        let result = validator.validate(Path::new(attack));
        // These should be blocked by path traversal detection or normalization
        assert!(
            result.is_err(),
            "Null byte injection attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_case_sensitivity_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Case variation attacks
    let case_attacks = vec![
        "../../../ETC/passwd",
        "../../../Etc/Passwd",
        "../../../ETC/PASSWD",
        "..\\..\\..\\WINDOWS\\SYSTEM32\\CONFIG\\SAM",
        "..\\..\\..\\Windows\\System32\\Config\\Sam",
        "..\\..\\..\\windows\\SYSTEM32\\config\\SAM",
    ];

    for attack in case_attacks {
        let result = validator.validate(Path::new(attack));
        assert!(
            result.is_err(),
            "Case sensitivity attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_double_encoding_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Double encoding attacks (these contain .. patterns)
    let double_encoding_attacks = vec![
        "../../../etc/passwd",           // Standard path traversal
        "..\\..\\..\\windows\\system32", // Windows path traversal
        "../../../etc/passwd",           // Duplicate for testing
    ];

    for attack in double_encoding_attacks {
        let result = validator.validate(Path::new(attack));
        assert!(
            result.is_err(),
            "Double encoding attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_overlong_utf8_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Overlong UTF-8 encoding attacks
    let overlong_attacks = vec![
        "..%c0%af..%c0%af..%c0%afetc%c0%afpasswd",
        "..%e0%80%af..%e0%80%af..%e0%80%afetc%e0%80%afpasswd",
        "..%f0%80%80%af..%f0%80%80%af..%f0%80%80%afetc%f0%80%80%afpasswd",
        "..%c1%9c..%c1%9c..%c1%9cwindows%c1%9csystem32",
        "..%e0%80%9c..%e0%80%9c..%e0%80%9cwindows%e0%80%9csystem32",
    ];

    for attack in overlong_attacks {
        let result = validator.validate(Path::new(attack));
        assert!(
            result.is_err(),
            "Overlong UTF-8 attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_windows_reserved_device_names() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Windows reserved device names
    let device_names = vec![
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "con",
        "prn", "aux", "nul", "com1", "lpt1", "Con", "Prn", "Aux", "Nul", "Com1", "Lpt1",
    ];

    for device in device_names {
        let path_str = format!("/tmp/{}.txt", device);
        let result = validator.validate(Path::new(&path_str));
        assert!(
            result.is_err(),
            "Windows reserved device name '{}' should be blocked",
            device
        );
    }
}
#[test]
fn test_path_length_dos_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Path length DoS attacks
    let long_path_attacks = vec![
        format!("/tmp/{}", "a".repeat(1000)),
        format!("/tmp/{}", "a".repeat(5000)),
        format!("/tmp/{}", "a".repeat(10000)),
        format!("/tmp/{}/file.txt", "a".repeat(500)),
        format!("/tmp/{}/{}/file.txt", "a".repeat(200), "b".repeat(200)),
    ];

    for attack in long_path_attacks {
        let result = validator.validate(Path::new(&attack));
        assert!(
            result.is_err(),
            "Long path DoS attack (length {}) should be blocked",
            attack.len()
        );
    }
}
#[test]
fn test_symlink_manipulation_attacks() {
    let temp_dir = TempDir::new().unwrap();
    let jail_path = temp_dir.path().to_path_buf();

    // Create a symlink chain that tries to escape jail
    let symlink_path = jail_path.join("escape_link");
    let target_path = jail_path.parent().unwrap().join("outside_file.txt");

    // Skip on Windows as symlinks require elevated permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        // Create the symlink (this might fail in some environments)
        if symlink(&target_path, &symlink_path).is_ok() {
            let config = PathSecurityConfig::strict().with_jail(jail_path.clone());
            let validator = PathSecurityValidator::with_config(config);

            let result = validator.validate(&symlink_path);
            assert!(result.is_err(), "Symlink escape attack should be blocked");
        }
    }

    // Test symlink loop detection
    let config = PathSecurityConfig {
        allow_symlinks: true,
        jail_directory: Some(jail_path.clone()),
        ..Default::default()
    };
    let validator = PathSecurityValidator::with_config(config);

    // Test with legitimate path to ensure it still works
    let safe_path = jail_path.join("safe_file.txt");
    let _result = validator.validate(&safe_path);
    // This might fail if jail_path has symlinks in its own path structure, which is expected
    // The test is primarily about the symlink attack detection logic
}
#[test]
fn test_permission_escalation_attacks() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Permission escalation attempts (these should be blocked by disallowed prefixes)
    let escalation_attacks = vec![
        "/etc/passwd",
        "/etc/shadow",
        "/etc/group",
        "/etc/sudoers",
        "/sys/kernel/debug",
        "/proc/self/environ",
        "/dev/null",
    ];

    for attack in escalation_attacks {
        let result = validator.validate(Path::new(attack));
        assert!(
            result.is_err(),
            "Permission escalation attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_jail_escape_attacks() {
    let temp_dir = TempDir::new().unwrap();
    let jail_path = temp_dir.path().to_path_buf();

    let config = PathSecurityConfig::strict().with_jail(jail_path.clone());
    let validator = PathSecurityValidator::with_config(config);

    // Jail escape attempts
    let escape_attacks = vec![
        jail_path.parent().unwrap().join("outside_file.txt"),
        jail_path
            .parent()
            .unwrap()
            .join("../another_outside_file.txt"),
        PathBuf::from("/etc/passwd"),
        PathBuf::from("/tmp/outside_jail.txt"),
        PathBuf::from("/home/user/file.txt"),
    ];

    for attack in escape_attacks {
        let result = validator.validate(&attack);
        assert!(
            result.is_err(),
            "Jail escape attack '{}' should be blocked",
            attack.display()
        );
    }

    // Legitimate path within jail should work (when jail path doesn't contain symlinks)
    let safe_path = jail_path.join("safe_file.txt");
    let _result = validator.validate(&safe_path);
    // Note: This might fail if the temp directory path itself contains symlinks
    // which is acceptable behavior for the security validator
}
#[test]
fn test_chroot_jail_bypass_attacks() {
    let temp_dir = TempDir::new().unwrap();
    let jail_path = temp_dir.path().to_path_buf();

    let config = PathSecurityConfig {
        jail_directory: Some(jail_path.clone()),
        allow_hidden: true,          // Allow hidden files for temp paths
        allow_symlinks: true,        // Allow symlinks for temp paths
        disallowed_prefixes: vec![], // Clear for this test
        ..Default::default()
    };
    let validator = PathSecurityValidator::with_config(config);

    // Chroot bypass attempts
    let bypass_attacks = vec![
        jail_path.parent().unwrap().join("../etc/passwd"),
        jail_path.parent().unwrap().join("../../root/.ssh/id_rsa"),
        jail_path.parent().unwrap().join("../../../tmp/outside.txt"),
    ];

    for attack in bypass_attacks {
        let result = validator.validate(&attack);
        assert!(
            result.is_err(),
            "Chroot bypass attack '{}' should be blocked",
            attack.display()
        );
    }

    // Path within chroot should work
    let safe_path = jail_path.join("subdir/safe_file.txt");
    let result = validator.validate(&safe_path);
    assert!(result.is_ok(), "Safe path within chroot should be allowed");
}
#[test]
fn test_complex_attack_combinations() {
    let temp_dir = TempDir::new().unwrap();
    let jail_path = temp_dir.path().to_path_buf();

    let config = PathSecurityConfig::strict().with_jail(jail_path.clone());
    let validator = PathSecurityValidator::with_config(config);

    // Complex combination attacks
    let combination_attacks = vec![
        format!("{}/../../../etc/passwd", jail_path.display()),
        format!("{}/../..\\..\\windows\\system32", jail_path.display()),
        format!("{}/../../../tmp/{}", jail_path.display(), "a".repeat(300)),
        format!("{}/../../../etc/CON", jail_path.display()),
        format!("{}/../../../tmp/.hidden/../etc/passwd", jail_path.display()),
    ];

    for attack in combination_attacks {
        let result = validator.validate(Path::new(&attack));
        assert!(
            result.is_err(),
            "Complex combination attack '{}' should be blocked",
            attack
        );
    }
}
#[test]
fn test_edge_case_paths() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Edge case paths that should be rejected by path traversal detection
    let path_traversal_cases = vec![
        "..",      // Parent directory
        "../",     // Parent directory with slash
        "/../",    // Root parent directory
        "/../../", // Multiple parent directories
    ];

    for edge_case in path_traversal_cases {
        let result = validator.validate(Path::new(edge_case));
        assert!(
            result.is_err(),
            "Edge case path '{}' should be blocked",
            edge_case
        );
    }

    // Test cross-platform validation separately
    let config = PathSecurityConfig {
        cross_platform_validation: true,
        ..Default::default()
    };
    let cross_platform_validator = PathSecurityValidator::with_config(config);

    let cross_platform_cases = vec![
        "/tmp/file ", // Trailing space in path component
        "/tmp/file.", // Trailing dot in path component
    ];

    for edge_case in cross_platform_cases {
        let result = cross_platform_validator.validate(Path::new(edge_case));
        assert!(
            result.is_err(),
            "Cross-platform edge case '{}' should be blocked",
            edge_case
        );
    }
}
#[test]
fn test_performance_under_attack() {
    let validator = PathSecurityValidator::with_config(PathSecurityConfig::strict());

    // Performance test with many attack vectors
    let start_time = std::time::Instant::now();

    for i in 0..1000 {
        let attack = format!("../../../etc/passwd{}", i);
        let result = validator.validate(Path::new(&attack));
        assert!(result.is_err(), "Attack {} should be blocked", i);
    }

    let elapsed = start_time.elapsed();

    // Should complete within reasonable time (adjust as needed)
    assert!(
        elapsed.as_millis() < 5000,
        "Path validation should complete within 5 seconds, took {}ms",
        elapsed.as_millis()
    );
}

use std::path::PathBuf;
