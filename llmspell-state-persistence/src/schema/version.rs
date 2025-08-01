// ABOUTME: Semantic versioning implementation for state schemas
// ABOUTME: Provides version comparison, parsing, and ordering for schema evolution

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VersionError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    #[error("Version component out of range: {0}")]
    ComponentOutOfRange(String),
}

/// Semantic version implementation following semver.org specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }

    pub fn with_pre_release(mut self, pre_release: String) -> Self {
        self.pre_release = Some(pre_release);
        self
    }

    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Major version must match for compatibility
        if self.major != other.major {
            return false;
        }

        // For same major version, newer minor/patch versions are compatible
        match self.minor.cmp(&other.minor) {
            Ordering::Greater => true,
            Ordering::Equal => self.patch >= other.patch,
            Ordering::Less => false,
        }
    }

    pub fn is_breaking_change_from(&self, other: &Self) -> bool {
        self.major > other.major
    }

    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    pub fn increment_patch(&mut self) {
        self.patch += 1;
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }

        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }

        Ok(())
    }
}

impl FromStr for SemanticVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Split on '+' for build metadata
        let (version_part, build) = if let Some(pos) = s.find('+') {
            (s[..pos].to_string(), Some(s[pos + 1..].to_string()))
        } else {
            (s.to_string(), None)
        };

        // Split on '-' for pre-release
        let (core_version, pre_release) = if let Some(pos) = version_part.find('-') {
            (
                version_part[..pos].to_string(),
                Some(version_part[pos + 1..].to_string()),
            )
        } else {
            (version_part, None)
        };

        // Parse core version (major.minor.patch)
        let parts: Vec<&str> = core_version.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(format!(
                "Expected format 'major.minor.patch', got: {}",
                s
            )));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| VersionError::ComponentOutOfRange(format!("major: {}", parts[0])))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| VersionError::ComponentOutOfRange(format!("minor: {}", parts[1])))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| VersionError::ComponentOutOfRange(format!("patch: {}", parts[2])))?;

        Ok(SemanticVersion {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major version first
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Then minor version
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Then patch version
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Pre-release versions have lower precedence than normal versions
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// Schema version wrapper with additional metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: SemanticVersion,
    pub created_at: std::time::SystemTime,
    pub description: Option<String>,
    pub breaking_changes: Vec<String>,
    pub deprecated_fields: Vec<String>,
}

impl SchemaVersion {
    pub fn new(version: SemanticVersion) -> Self {
        Self {
            version,
            created_at: std::time::SystemTime::now(),
            description: None,
            breaking_changes: Vec::new(),
            deprecated_fields: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn add_breaking_change(&mut self, change: String) {
        self.breaking_changes.push(change);
    }

    pub fn add_deprecated_field(&mut self, field: String) {
        self.deprecated_fields.push(field);
    }

    pub fn is_breaking_change_from(&self, other: &Self) -> bool {
        self.version.is_breaking_change_from(&other.version) || !self.breaking_changes.is_empty()
    }

    pub fn has_deprecated_fields(&self) -> bool {
        !self.deprecated_fields.is_empty()
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;

        if let Some(ref desc) = self.description {
            write!(f, " ({})", desc)?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "state")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_semantic_version_creation() {
        let version = SemanticVersion::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.build, None);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_version_parsing() {
        let version: SemanticVersion = "1.2.3".parse().unwrap();
        assert_eq!(version, SemanticVersion::new(1, 2, 3));

        let version_with_pre: SemanticVersion = "1.2.3-alpha.1".parse().unwrap();
        assert_eq!(version_with_pre.major, 1);
        assert_eq!(version_with_pre.minor, 2);
        assert_eq!(version_with_pre.patch, 3);
        assert_eq!(version_with_pre.pre_release, Some("alpha.1".to_string()));

        let version_with_build: SemanticVersion = "1.2.3+build.123".parse().unwrap();
        assert_eq!(version_with_build.build, Some("build.123".to_string()));

        let full_version: SemanticVersion = "1.2.3-beta.2+build.456".parse().unwrap();
        assert_eq!(full_version.pre_release, Some("beta.2".to_string()));
        assert_eq!(full_version.build, Some("build.456".to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_version_display() {
        let version = SemanticVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");

        let version_pre = SemanticVersion::new(1, 2, 3).with_pre_release("alpha.1".to_string());
        assert_eq!(version_pre.to_string(), "1.2.3-alpha.1");

        let version_build = SemanticVersion::new(1, 2, 3).with_build("build.123".to_string());
        assert_eq!(version_build.to_string(), "1.2.3+build.123");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_version_comparison() {
        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_0_1 = SemanticVersion::new(1, 0, 1);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        assert!(v1_0_1 > v1_0_0);
        assert!(v1_1_0 > v1_0_1);
        assert!(v2_0_0 > v1_1_0);

        // Pre-release versions
        let v1_0_0_alpha = SemanticVersion::new(1, 0, 0).with_pre_release("alpha".to_string());
        assert!(v1_0_0 > v1_0_0_alpha);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_compatibility_checking() {
        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_0_1 = SemanticVersion::new(1, 0, 1);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        assert!(v1_0_1.is_compatible_with(&v1_0_0));
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v1_0_1)); // Older version not compatible with newer
        assert!(!v2_0_0.is_compatible_with(&v1_0_0)); // Major version change
        assert!(!v1_0_0.is_compatible_with(&v2_0_0)); // Major version change
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_breaking_change_detection() {
        let v1_0_0 = SemanticVersion::new(1, 0, 0);
        let v1_1_0 = SemanticVersion::new(1, 1, 0);
        let v2_0_0 = SemanticVersion::new(2, 0, 0);

        assert!(!v1_1_0.is_breaking_change_from(&v1_0_0));
        assert!(v2_0_0.is_breaking_change_from(&v1_0_0));
        assert!(v2_0_0.is_breaking_change_from(&v1_1_0));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_version_increment() {
        let mut version = SemanticVersion::new(1, 2, 3);

        version.increment_patch();
        assert_eq!(version, SemanticVersion::new(1, 2, 4));

        version.increment_minor();
        assert_eq!(version, SemanticVersion::new(1, 3, 0));

        version.increment_major();
        assert_eq!(version, SemanticVersion::new(2, 0, 0));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_schema_version() {
        let semantic_version = SemanticVersion::new(1, 0, 0);
        let mut schema_version = SchemaVersion::new(semantic_version.clone())
            .with_description("Initial schema version".to_string());

        assert_eq!(schema_version.version, semantic_version);
        assert_eq!(
            schema_version.description,
            Some("Initial schema version".to_string())
        );
        assert!(!schema_version.has_deprecated_fields());

        schema_version.add_breaking_change("Removed field 'old_field'".to_string());
        schema_version.add_deprecated_field("legacy_field".to_string());

        assert_eq!(schema_version.breaking_changes.len(), 1);
        assert!(schema_version.has_deprecated_fields());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_invalid_version_parsing() {
        assert!("1.2".parse::<SemanticVersion>().is_err());
        assert!("1.2.3.4".parse::<SemanticVersion>().is_err());
        assert!("a.b.c".parse::<SemanticVersion>().is_err());
        assert!("".parse::<SemanticVersion>().is_err());
    }
}
