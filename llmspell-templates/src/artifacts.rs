//! Artifact management for template outputs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Template execution artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact filename
    pub filename: String,

    /// Artifact content
    pub content: String,

    /// Artifact MIME type
    pub mime_type: String,

    /// Artifact metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(
        filename: impl Into<String>,
        content: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content: content.into(),
            mime_type: mime_type.into(),
            metadata: HashMap::new(),
        }
    }

    /// Create a markdown artifact
    pub fn markdown(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(filename, content, "text/markdown")
    }

    /// Create a JSON artifact
    pub fn json(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(filename, content, "application/json")
    }

    /// Create a plain text artifact
    pub fn text(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(filename, content, "text/plain")
    }

    /// Create an HTML artifact
    pub fn html(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(filename, content, "text/html")
    }

    /// Add metadata to artifact
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Write artifact to file
    pub fn write_to_file(&self, base_path: &std::path::Path) -> std::io::Result<PathBuf> {
        let file_path = base_path.join(&self.filename);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&file_path, &self.content)?;
        Ok(file_path)
    }

    /// Get artifact size in bytes
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Check if artifact is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Collection of artifacts from template execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactCollection {
    /// Artifacts
    pub artifacts: Vec<Artifact>,
}

impl ArtifactCollection {
    /// Create a new artifact collection
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
        }
    }

    /// Add an artifact to the collection
    pub fn add(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Get artifact by filename
    pub fn get(&self, filename: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.filename == filename)
    }

    /// Get all artifacts
    pub fn all(&self) -> &[Artifact] {
        &self.artifacts
    }

    /// Get number of artifacts
    pub fn count(&self) -> usize {
        self.artifacts.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    /// Write all artifacts to directory
    pub fn write_all(&self, base_path: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for artifact in &self.artifacts {
            let path = artifact.write_to_file(base_path)?;
            paths.push(path);
        }
        Ok(paths)
    }

    /// Get total size of all artifacts
    pub fn total_size(&self) -> usize {
        self.artifacts.iter().map(|a| a.size()).sum()
    }
}

impl From<Vec<Artifact>> for ArtifactCollection {
    fn from(artifacts: Vec<Artifact>) -> Self {
        Self { artifacts }
    }
}

impl From<ArtifactCollection> for Vec<Artifact> {
    fn from(collection: ArtifactCollection) -> Self {
        collection.artifacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact::markdown("test.md", "# Hello");
        assert_eq!(artifact.filename, "test.md");
        assert_eq!(artifact.content, "# Hello");
        assert_eq!(artifact.mime_type, "text/markdown");
    }

    #[test]
    fn test_artifact_with_metadata() {
        let artifact = Artifact::json("data.json", "{}")
            .with_metadata("author", serde_json::json!("test"))
            .with_metadata("version", serde_json::json!(1));

        assert_eq!(artifact.metadata.len(), 2);
        assert_eq!(
            artifact.metadata.get("author").unwrap(),
            &serde_json::json!("test")
        );
    }

    #[test]
    fn test_artifact_write_to_file() {
        let dir = tempdir().unwrap();
        let artifact = Artifact::markdown("test.md", "# Hello World");

        let path = artifact.write_to_file(dir.path()).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "# Hello World");
    }

    #[test]
    fn test_artifact_size() {
        let artifact = Artifact::text("test.txt", "Hello World");
        assert_eq!(artifact.size(), 11);
    }

    #[test]
    fn test_artifact_collection() {
        let mut collection = ArtifactCollection::new();
        assert!(collection.is_empty());

        collection.add(Artifact::markdown("test1.md", "Content 1"));
        collection.add(Artifact::markdown("test2.md", "Content 2"));

        assert_eq!(collection.count(), 2);
        assert!(!collection.is_empty());

        let artifact = collection.get("test1.md").unwrap();
        assert_eq!(artifact.content, "Content 1");
    }

    #[test]
    fn test_artifact_collection_write_all() {
        let dir = tempdir().unwrap();
        let mut collection = ArtifactCollection::new();
        collection.add(Artifact::markdown("test1.md", "Content 1"));
        collection.add(Artifact::markdown("test2.md", "Content 2"));

        let paths = collection.write_all(dir.path()).unwrap();
        assert_eq!(paths.len(), 2);

        for path in paths {
            assert!(path.exists());
        }
    }

    #[test]
    fn test_artifact_collection_total_size() {
        let mut collection = ArtifactCollection::new();
        collection.add(Artifact::text("test1.txt", "Hello")); // 5 bytes
        collection.add(Artifact::text("test2.txt", "World")); // 5 bytes

        assert_eq!(collection.total_size(), 10);
    }
}
