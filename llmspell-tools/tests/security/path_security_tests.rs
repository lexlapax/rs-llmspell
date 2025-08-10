//! ABOUTME: Comprehensive path security tests for file system access
//! ABOUTME: Tests for path traversal, symlinks, and directory access controls

use super::test_framework::*;
use serde_json::json;

/// Path traversal test vectors
#[must_use]
pub fn path_traversal_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "PATH_DOTDOT_UNIX".to_string(),
            description: "Unix-style directory traversal".to_string(),
            payload: json!({
                "path": "../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_DOTDOT_WINDOWS".to_string(),
            description: "Windows-style directory traversal".to_string(),
            payload: json!({
                "path": "..\\..\\..\\windows\\system32\\config\\sam",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_ABSOLUTE_UNIX".to_string(),
            description: "Absolute Unix path".to_string(),
            payload: json!({
                "path": "/etc/shadow",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_ABSOLUTE_WINDOWS".to_string(),
            description: "Absolute Windows path".to_string(),
            payload: json!({
                "path": "C:\\Windows\\System32\\drivers\\etc\\hosts",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_UNC_PATH".to_string(),
            description: "UNC network path".to_string(),
            payload: json!({
                "path": "\\\\attacker.com\\share\\payload.exe",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_FILE_URL".to_string(),
            description: "File URL scheme".to_string(),
            payload: json!({
                "path": "file:///etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Path encoding bypass attempts
#[must_use]
pub fn path_encoding_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "PATH_URL_ENCODED".to_string(),
            description: "URL encoded traversal".to_string(),
            payload: json!({
                "path": "..%2F..%2F..%2Fetc%2Fpasswd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_DOUBLE_URL_ENCODED".to_string(),
            description: "Double URL encoded traversal".to_string(),
            payload: json!({
                "path": "..%252F..%252F..%252Fetc%252Fpasswd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_UNICODE_ENCODED".to_string(),
            description: "Unicode encoded traversal".to_string(),
            payload: json!({
                "path": "..\\u002f..\\u002f..\\u002fetc\\u002fpasswd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_UTF8_OVERLONG".to_string(),
            description: "UTF-8 overlong encoding".to_string(),
            payload: json!({
                "path": "../\u{2f}../\u{2f}etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "PATH_NULL_BYTE".to_string(),
            description: "Null byte injection".to_string(),
            payload: json!({
                "path": "safe_file.txt\0../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Symlink attack vectors
#[must_use]
pub fn symlink_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SYMLINK_ESCAPE_JAIL".to_string(),
            description: "Symlink escaping jail".to_string(),
            payload: json!({
                "path": "symlink_to_etc_passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "SYMLINK_RECURSIVE".to_string(),
            description: "Recursive symlink".to_string(),
            payload: json!({
                "path": "link1/link2/link3/file",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "SYMLINK_TOCTOU".to_string(),
            description: "TOCTOU symlink attack".to_string(),
            payload: json!({
                "path": "changing_symlink",
                "operation": "write",
                "race_condition": true
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "SYMLINK_HARDLINK".to_string(),
            description: "Hardlink to sensitive file".to_string(),
            payload: json!({
                "path": "hardlink_to_shadow",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Directory traversal patterns
#[must_use]
pub fn directory_pattern_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "DIR_MIXED_SEPARATORS".to_string(),
            description: "Mixed path separators".to_string(),
            payload: json!({
                "path": "..\\../..\\../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "DIR_CURRENT_DIR".to_string(),
            description: "Current directory references".to_string(),
            payload: json!({
                "path": "./././../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "DIR_TRAILING_SEPARATOR".to_string(),
            description: "Trailing separator bypass".to_string(),
            payload: json!({
                "path": "../../../etc/passwd/",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "DIR_MULTIPLE_SLASHES".to_string(),
            description: "Multiple consecutive slashes".to_string(),
            payload: json!({
                "path": "///etc///passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "DIR_SPACE_BYPASS".to_string(),
            description: "Space character bypass".to_string(),
            payload: json!({
                "path": "../ ../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Special file access attempts
#[must_use]
pub fn special_file_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "SPECIAL_DEVICE_FILES".to_string(),
            description: "Device file access".to_string(),
            payload: json!({
                "path": "/dev/random",
                "operation": "read",
                "bytes": 1_000_000
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "SPECIAL_PROC_FILES".to_string(),
            description: "Proc filesystem access".to_string(),
            payload: json!({
                "path": "/proc/self/environ",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![
                TestCategory::PathTraversal,
                TestCategory::InformationDisclosure,
            ],
        },
        SecurityTestCase {
            name: "SPECIAL_NAMED_PIPE".to_string(),
            description: "Named pipe access".to_string(),
            payload: json!({
                "path": "/tmp/evil_pipe",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "SPECIAL_SOCKET_FILE".to_string(),
            description: "Socket file access".to_string(),
            payload: json!({
                "path": "/var/run/docker.sock",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Windows-specific path tests
#[must_use]
pub fn windows_path_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "WIN_ALTERNATE_STREAM".to_string(),
            description: "Alternate data stream".to_string(),
            payload: json!({
                "path": "file.txt:hidden:$DATA",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "WIN_SHORT_NAME".to_string(),
            description: "8.3 short name".to_string(),
            payload: json!({
                "path": "PROGRA~1\\sensitive.dat",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "WIN_RESERVED_NAME".to_string(),
            description: "Reserved device name".to_string(),
            payload: json!({
                "path": "CON",
                "operation": "write",
                "input": "test"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "WIN_DRIVE_LETTER".to_string(),
            description: "Drive letter traversal".to_string(),
            payload: json!({
                "path": "D:\\sensitive\\data.txt",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Archive path extraction tests
#[must_use]
pub fn archive_path_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "ARCHIVE_ABSOLUTE_PATH".to_string(),
            description: "Archive with absolute paths".to_string(),
            payload: json!({
                "path": "malicious.zip",
                "operation": "extract",
                "target_path": "/tmp/safe",
                "contains": "/etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "ARCHIVE_TRAVERSAL_PATH".to_string(),
            description: "Archive with traversal paths".to_string(),
            payload: json!({
                "path": "malicious.tar",
                "operation": "extract",
                "target_path": "/tmp/safe",
                "contains": "../../etc/passwd"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "ARCHIVE_SYMLINK_ESCAPE".to_string(),
            description: "Archive with escaping symlinks".to_string(),
            payload: json!({
                "path": "symlink_archive.zip",
                "operation": "extract",
                "target_path": "/tmp/safe"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Permission bypass attempts
#[must_use]
pub fn permission_bypass_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "PERM_READONLY_WRITE".to_string(),
            description: "Write to read-only file".to_string(),
            payload: json!({
                "path": "/etc/hosts",
                "operation": "write",
                "input": "127.0.0.1 evil.com"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal, TestCategory::Authorization],
        },
        SecurityTestCase {
            name: "PERM_EXECUTE_SCRIPT".to_string(),
            description: "Execute non-executable file".to_string(),
            payload: json!({
                "path": "/tmp/script.sh",
                "operation": "execute"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal, TestCategory::Authorization],
        },
        SecurityTestCase {
            name: "PERM_HIDDEN_FILE".to_string(),
            description: "Access hidden file".to_string(),
            payload: json!({
                "path": ".ssh/id_rsa",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![
                TestCategory::PathTraversal,
                TestCategory::InformationDisclosure,
            ],
        },
    ]
}

/// Case sensitivity tests
#[must_use]
pub fn case_sensitivity_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "CASE_WINDOWS_BYPASS".to_string(),
            description: "Case variation on Windows".to_string(),
            payload: json!({
                "path": "C:\\WiNdOwS\\SyStEm32\\config\\SAM",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "CASE_EXTENSION_BYPASS".to_string(),
            description: "Extension case bypass".to_string(),
            payload: json!({
                "path": "script.PHP",
                "operation": "execute"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Path normalization bypass tests
#[must_use]
pub fn path_normalization_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "NORM_DOUBLE_DOTS".to_string(),
            description: "Double dot normalization".to_string(),
            payload: json!({
                "path": "safe/../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "NORM_UNICODE_NORMALIZE".to_string(),
            description: "Unicode normalization bypass".to_string(),
            payload: json!({
                "path": "café/../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "NORM_RELATIVE_ABSOLUTE".to_string(),
            description: "Relative to absolute conversion".to_string(),
            payload: json!({
                "path": "./../../../../../../../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Race condition path tests
#[must_use]
pub fn race_condition_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "RACE_CHECK_USE".to_string(),
            description: "Check-then-use race condition".to_string(),
            payload: json!({
                "path": "temp_file",
                "operation": "check_then_write",
                "race_window": 100
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "RACE_SYMLINK_SWAP".to_string(),
            description: "Symlink swap during operation".to_string(),
            payload: json!({
                "path": "swapping_link",
                "operation": "multi_access",
                "swap_timing": "between_checks"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Path length and boundary tests
#[must_use]
pub fn path_boundary_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "BOUNDARY_MAX_PATH".to_string(),
            description: "Maximum path length".to_string(),
            payload: json!({
                "path": "a".repeat(4096),
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal, TestCategory::DoS],
        },
        SecurityTestCase {
            name: "BOUNDARY_EMPTY_PATH".to_string(),
            description: "Empty path string".to_string(),
            payload: json!({
                "path": "",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::PathTraversal],
        },
        SecurityTestCase {
            name: "BOUNDARY_UNICODE_PATH".to_string(),
            description: "Unicode characters in path".to_string(),
            payload: json!({
                "path": "文件名/../../etc/passwd",
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::PathTraversal],
        },
    ]
}

/// Create all path security test cases
#[must_use]
pub fn all_path_security_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(path_traversal_tests());
    tests.extend(path_encoding_tests());
    tests.extend(symlink_tests());
    tests.extend(directory_pattern_tests());
    tests.extend(special_file_tests());
    tests.extend(windows_path_tests());
    tests.extend(archive_path_tests());
    tests.extend(permission_bypass_tests());
    tests.extend(case_sensitivity_tests());
    tests.extend(path_normalization_tests());
    tests.extend(race_condition_tests());
    tests.extend(path_boundary_tests());
    tests
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_path_security_test_creation() {
        let tests = all_path_security_tests();
        assert!(!tests.is_empty());

        // Verify we have tests from each category
        let categories: Vec<String> = tests
            .iter()
            .map(|t| t.name.split('_').next().unwrap_or(""))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();

        assert!(categories.contains(&"PATH".to_string()));
        assert!(categories.contains(&"SYMLINK".to_string()));
        assert!(categories.contains(&"DIR".to_string()));
        assert!(categories.contains(&"SPECIAL".to_string()));
        assert!(categories.contains(&"WIN".to_string()));
        assert!(categories.contains(&"ARCHIVE".to_string()));
        assert!(categories.contains(&"PERM".to_string()));
        assert!(categories.contains(&"CASE".to_string()));
        assert!(categories.contains(&"NORM".to_string()));
        assert!(categories.contains(&"RACE".to_string()));
        assert!(categories.contains(&"BOUNDARY".to_string()));
    }
    #[test]
    fn test_severity_distribution() {
        let tests = all_path_security_tests();
        let mut severity_counts = std::collections::HashMap::new();

        for test in &tests {
            *severity_counts.entry(&test.severity).or_insert(0) += 1;
        }

        // Ensure we have tests at different severity levels
        assert!(severity_counts.get(&Severity::Critical).unwrap_or(&0) > &0);
        assert!(severity_counts.get(&Severity::High).unwrap_or(&0) > &0);
        assert!(severity_counts.get(&Severity::Medium).unwrap_or(&0) > &0);
    }
    #[test]
    fn test_expected_behavior() {
        let tests = all_path_security_tests();

        // All path security tests should reject malicious paths
        for test in &tests {
            assert_eq!(
                test.expected_behavior,
                ExpectedBehavior::Reject,
                "Test {} should reject malicious path",
                test.name
            );
        }
    }
}
