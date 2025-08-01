//! ABOUTME: Archive handling tool for ZIP/TAR/GZ formats with security controls
//! ABOUTME: Provides safe extraction and creation of archives with resource limits

// Helper macro to convert IO errors
macro_rules! io_err {
    ($e:expr, $ctx:expr) => {
        $e.map_err(|e| LLMSpellError::Storage {
            message: $ctx.to_string(),
            operation: None,
            source: Some(Box::new(e)),
        })
    };
}
use async_trait::async_trait;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_security::sandbox::FileSandbox;
use llmspell_utils::{extract_parameters, extract_required_string, response::ResponseBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tracing::warn;
use zip::write::{FileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

/// Configuration for archive operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveHandlerConfig {
    /// Maximum size of files to extract (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// Maximum total size of extracted content (in bytes)
    #[serde(default = "default_max_total_size")]
    pub max_total_size: u64,

    /// Maximum number of files to extract
    #[serde(default = "default_max_files")]
    pub max_files: usize,

    /// Maximum extraction depth (for nested archives)
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,

    /// Default compression level (0-9)
    #[serde(default = "default_compression_level")]
    pub compression_level: u32,

    /// Whether to preserve file permissions
    #[serde(default = "default_preserve_permissions")]
    pub preserve_permissions: bool,
}

fn default_max_file_size() -> u64 {
    100 * 1024 * 1024
} // 100MB
fn default_max_total_size() -> u64 {
    1024 * 1024 * 1024
} // 1GB
fn default_max_files() -> usize {
    10000
}
fn default_max_depth() -> u32 {
    3
}
fn default_compression_level() -> u32 {
    6
}
fn default_preserve_permissions() -> bool {
    false
}

impl Default for ArchiveHandlerConfig {
    fn default() -> Self {
        Self {
            max_file_size: default_max_file_size(),
            max_total_size: default_max_total_size(),
            max_files: default_max_files(),
            max_depth: default_max_depth(),
            compression_level: default_compression_level(),
            preserve_permissions: default_preserve_permissions(),
        }
    }
}

/// Archive handler tool for safe archive operations
pub struct ArchiveHandlerTool {
    metadata: ComponentMetadata,
    config: ArchiveHandlerConfig,
    file_sandbox: Option<FileSandbox>,
}

impl ArchiveHandlerTool {
    /// Create a new archive handler with default configuration
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "archive-handler-tool".to_string(),
                "Handle ZIP, TAR, and GZ archives with security controls".to_string(),
            ),
            config: ArchiveHandlerConfig::default(),
            file_sandbox: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ArchiveHandlerConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "archive-handler-tool".to_string(),
                "Handle ZIP, TAR, and GZ archives with security controls".to_string(),
            ),
            config,
            file_sandbox: None,
        }
    }

    /// Set file sandbox for security
    pub fn with_sandbox(mut self, sandbox: FileSandbox) -> Self {
        self.file_sandbox = Some(sandbox);
        self
    }

    /// Detect archive format from file extension
    fn detect_format(path: &Path) -> Result<ArchiveFormat> {
        let ext =
            path.extension()
                .and_then(|e| e.to_str())
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Cannot determine archive format from path".to_string(),
                    field: Some("path".to_string()),
                })?;

        match ext.to_lowercase().as_str() {
            "zip" => Ok(ArchiveFormat::Zip),
            "tar" => Ok(ArchiveFormat::Tar),
            "gz" | "tgz" => {
                // Check if it's a .tar.gz
                if path.to_string_lossy().ends_with(".tar.gz") {
                    Ok(ArchiveFormat::TarGz)
                } else {
                    Ok(ArchiveFormat::Gz)
                }
            }
            _ => Err(LLMSpellError::Validation {
                message: format!("Unsupported archive format: {}", ext),
                field: Some("archive_path".to_string()),
            }),
        }
    }

    /// Check if path is safe (no path traversal)
    fn is_safe_path(path: &Path) -> bool {
        let mut depth = 0;
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => depth -= 1,
                std::path::Component::Normal(_) => depth += 1,
                _ => {}
            }
            if depth < 0 {
                return false;
            }
        }
        true
    }

    /// Extract archive
    async fn extract_archive(&self, params: &Value) -> Result<Value> {
        let archive_path = params.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            LLMSpellError::Validation {
                message: "Missing 'path' parameter".to_string(),
                field: Some("path".to_string()),
            }
        })?;

        let output_dir = params
            .get("target_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'target_path' parameter".to_string(),
                field: Some("target_path".to_string()),
            })?;

        let archive_path = PathBuf::from(archive_path);
        let output_dir = PathBuf::from(output_dir);

        // Check sandbox permissions
        if let Some(sandbox) = &self.file_sandbox {
            // Validate paths through sandbox
            sandbox
                .validate_path(&archive_path)
                .map_err(|e| LLMSpellError::Security {
                    message: format!("Archive path validation failed: {}", e),
                    violation_type: Some("file_access".to_string()),
                })?;
            sandbox
                .validate_path(&output_dir)
                .map_err(|e| LLMSpellError::Security {
                    message: format!("Output directory validation failed: {}", e),
                    violation_type: Some("file_access".to_string()),
                })?;
        }

        // Create output directory
        fs::create_dir_all(&output_dir).map_err(|e| LLMSpellError::Storage {
            message: format!(
                "Failed to create output directory: {}",
                output_dir.display()
            ),
            operation: Some("create_dir".to_string()),
            source: Some(Box::new(e)),
        })?;

        let format = Self::detect_format(&archive_path)?;
        let mut extracted_files = Vec::new();
        let mut total_size = 0u64;

        match format {
            ArchiveFormat::Zip => {
                self.extract_zip(
                    &archive_path,
                    &output_dir,
                    &mut extracted_files,
                    &mut total_size,
                )?;
            }
            ArchiveFormat::Tar => {
                self.extract_tar(
                    &archive_path,
                    &output_dir,
                    &mut extracted_files,
                    &mut total_size,
                )?;
            }
            ArchiveFormat::TarGz => {
                self.extract_tar_gz(
                    &archive_path,
                    &output_dir,
                    &mut extracted_files,
                    &mut total_size,
                )?;
            }
            ArchiveFormat::Gz => {
                self.extract_gz(
                    &archive_path,
                    &output_dir,
                    &mut extracted_files,
                    &mut total_size,
                )?;
            }
        }

        Ok(ResponseBuilder::success("extract")
            .with_message(format!(
                "Extracted {} files ({} bytes) to {}",
                extracted_files.len(),
                total_size,
                output_dir.display()
            ))
            .with_result(json!({
                "extracted_files": extracted_files,
                "total_size": total_size,
                "output_dir": output_dir.to_string_lossy().to_string()
            }))
            .build())
    }

    /// Extract ZIP archive
    fn extract_zip(
        &self,
        archive_path: &Path,
        output_dir: &Path,
        extracted_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::open(archive_path),
            format!("Failed to open archive: {}", archive_path.display())
        )?;
        let mut archive = ZipArchive::new(file).map_err(|e| LLMSpellError::Validation {
            message: format!("Invalid ZIP archive: {}", e),
            field: Some("archive_path".to_string()),
        })?;

        for i in 0..archive.len() {
            if extracted_files.len() >= self.config.max_files {
                warn!("Reached maximum file limit of {}", self.config.max_files);
                break;
            }

            let mut file = archive.by_index(i).map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to read file at index {}", i),
                operation: Some("read_zip_entry".to_string()),
                source: Some(Box::new(e)),
            })?;
            let outpath = match file.enclosed_name() {
                Some(path) => output_dir.join(path),
                None => continue,
            };

            // Security check
            if !Self::is_safe_path(&outpath) {
                warn!("Skipping unsafe path: {:?}", file.name());
                continue;
            }

            // Size check
            let file_size = file.size();
            if file_size > self.config.max_file_size {
                warn!("Skipping file {} - exceeds size limit", file.name());
                continue;
            }

            if *total_size + file_size > self.config.max_total_size {
                warn!("Reached total size limit");
                break;
            }

            if file.is_dir() {
                io_err!(
                    fs::create_dir_all(&outpath),
                    format!("Failed to create directory: {}", outpath.display())
                )?;
            } else {
                if let Some(p) = outpath.parent() {
                    io_err!(
                        fs::create_dir_all(p),
                        format!("Failed to create parent directory: {}", p.display())
                    )?;
                }
                let mut outfile = io_err!(
                    File::create(&outpath),
                    format!("Failed to create output file: {}", outpath.display())
                )?;
                io_err!(
                    io::copy(&mut file, &mut outfile),
                    format!("Failed to extract file: {}", outpath.display())
                )?;

                // Set permissions if requested
                #[cfg(unix)]
                if self.config.preserve_permissions {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        io_err!(
                            fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)),
                            "Failed to set file permissions"
                        )?;
                    }
                }

                extracted_files.push(outpath.to_string_lossy().to_string());
                *total_size += file_size;
            }
        }

        Ok(())
    }

    /// Extract TAR archive
    fn extract_tar(
        &self,
        archive_path: &Path,
        output_dir: &Path,
        extracted_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::open(archive_path),
            format!("Failed to open archive: {}", archive_path.display())
        )?;
        let mut archive = Archive::new(file);

        for entry in archive.entries().map_err(|e| LLMSpellError::Storage {
            message: "Failed to read TAR entries".to_string(),
            operation: Some("read_tar_entries".to_string()),
            source: Some(Box::new(e)),
        })? {
            if extracted_files.len() >= self.config.max_files {
                warn!("Reached maximum file limit of {}", self.config.max_files);
                break;
            }

            let mut entry = entry.map_err(|e| LLMSpellError::Storage {
                message: "Failed to read TAR entry".to_string(),
                operation: Some("read_tar_entry".to_string()),
                source: Some(Box::new(e)),
            })?;
            let path = entry.path().map_err(|e| LLMSpellError::Storage {
                message: "Failed to read TAR entry path".to_string(),
                operation: Some("read_tar_path".to_string()),
                source: Some(Box::new(e)),
            })?;
            let outpath = output_dir.join(&path);

            // Security check
            if !Self::is_safe_path(&outpath) {
                warn!("Skipping unsafe path: {:?}", path);
                continue;
            }

            // Size check
            let file_size = entry.size();
            if file_size > self.config.max_file_size {
                warn!("Skipping file {:?} - exceeds size limit", path);
                continue;
            }

            if *total_size + file_size > self.config.max_total_size {
                warn!("Reached total size limit");
                break;
            }

            io_err!(
                entry.unpack(&outpath),
                format!("Failed to extract file: {}", outpath.display())
            )?;

            if entry.header().entry_type().is_file() {
                extracted_files.push(outpath.to_string_lossy().to_string());
                *total_size += file_size;
            }
        }

        Ok(())
    }

    /// Extract TAR.GZ archive
    fn extract_tar_gz(
        &self,
        archive_path: &Path,
        output_dir: &Path,
        extracted_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::open(archive_path),
            format!("Failed to open archive: {}", archive_path.display())
        )?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        for entry in archive.entries().map_err(|e| LLMSpellError::Storage {
            message: "Failed to read TAR.GZ entries".to_string(),
            operation: Some("read_targz_entries".to_string()),
            source: Some(Box::new(e)),
        })? {
            if extracted_files.len() >= self.config.max_files {
                warn!("Reached maximum file limit of {}", self.config.max_files);
                break;
            }

            let mut entry = entry.map_err(|e| LLMSpellError::Storage {
                message: "Failed to read TAR.GZ entry".to_string(),
                operation: Some("read_targz_entry".to_string()),
                source: Some(Box::new(e)),
            })?;
            let path = entry.path().map_err(|e| LLMSpellError::Storage {
                message: "Failed to read TAR.GZ entry path".to_string(),
                operation: Some("read_targz_path".to_string()),
                source: Some(Box::new(e)),
            })?;
            let outpath = output_dir.join(&path);

            // Security check
            if !Self::is_safe_path(&outpath) {
                warn!("Skipping unsafe path: {:?}", path);
                continue;
            }

            // Size check
            let file_size = entry.size();
            if file_size > self.config.max_file_size {
                warn!("Skipping file {:?} - exceeds size limit", path);
                continue;
            }

            if *total_size + file_size > self.config.max_total_size {
                warn!("Reached total size limit");
                break;
            }

            io_err!(
                entry.unpack(&outpath),
                format!("Failed to extract file: {}", outpath.display())
            )?;

            if entry.header().entry_type().is_file() {
                extracted_files.push(outpath.to_string_lossy().to_string());
                *total_size += file_size;
            }
        }

        Ok(())
    }

    /// Extract GZ file (single file compression)
    fn extract_gz(
        &self,
        archive_path: &Path,
        output_dir: &Path,
        extracted_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::open(archive_path),
            format!("Failed to open archive: {}", archive_path.display())
        )?;
        let mut decoder = GzDecoder::new(file);

        // Determine output filename
        let stem = archive_path
            .file_stem()
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Cannot determine output filename".to_string(),
                field: Some("archive_path".to_string()),
            })?;
        let outpath = output_dir.join(stem);

        // Create output file
        let mut outfile = io_err!(
            File::create(&outpath),
            format!("Failed to create output file: {}", outpath.display())
        )?;
        let bytes_written = io_err!(
            io::copy(&mut decoder, &mut outfile),
            format!("Failed to extract GZ file: {}", outpath.display())
        )?;

        if bytes_written > self.config.max_file_size {
            // Remove the file if it's too large
            fs::remove_file(&outpath).map_err(|e| LLMSpellError::Storage {
                message: "Failed to remove oversized file".to_string(),
                operation: Some("remove_file".to_string()),
                source: Some(Box::new(e)),
            })?;
            return Err(LLMSpellError::Validation {
                message: "Extracted file exceeds size limit".to_string(),
                field: Some("archive_path".to_string()),
            });
        }

        extracted_files.push(outpath.to_string_lossy().to_string());
        *total_size += bytes_written;

        Ok(())
    }

    /// Create archive
    async fn create_archive(&self, params: &Value) -> Result<Value> {
        eprintln!("DEBUG create_archive: params = {:?}", params);
        let archive_path = params.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            LLMSpellError::Validation {
                message: "Missing 'path' parameter".to_string(),
                field: Some("path".to_string()),
            }
        })?;

        let files = params
            .get("input")
            .and_then(|v| v.as_array())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'input' parameter".to_string(),
                field: Some("input".to_string()),
            })?;

        let archive_path = PathBuf::from(archive_path);

        // Check sandbox permissions
        if let Some(sandbox) = &self.file_sandbox {
            sandbox
                .validate_path(&archive_path)
                .map_err(|e| LLMSpellError::Security {
                    message: format!("Archive path validation failed: {}", e),
                    violation_type: Some("file_access".to_string()),
                })?;
            for file in files {
                if let Some(path) = file.as_str() {
                    sandbox.validate_path(Path::new(path)).map_err(|e| {
                        LLMSpellError::Security {
                            message: format!("Input file path validation failed: {}", e),
                            violation_type: Some("file_access".to_string()),
                        }
                    })?;
                }
            }
        }

        let format = Self::detect_format(&archive_path)?;
        let mut archived_files = Vec::new();
        let mut total_size = 0u64;

        match format {
            ArchiveFormat::Zip => {
                self.create_zip(&archive_path, files, &mut archived_files, &mut total_size)?;
            }
            ArchiveFormat::Tar => {
                self.create_tar(&archive_path, files, &mut archived_files, &mut total_size)?;
            }
            ArchiveFormat::TarGz => {
                self.create_tar_gz(&archive_path, files, &mut archived_files, &mut total_size)?;
            }
            ArchiveFormat::Gz => {
                if files.len() != 1 {
                    return Err(LLMSpellError::Validation {
                        message: "GZ format can only compress a single file".to_string(),
                        field: Some("files".to_string()),
                    });
                }
                self.create_gz(&archive_path, files, &mut archived_files, &mut total_size)?;
            }
        }

        Ok(ResponseBuilder::success("create")
            .with_message(format!(
                "Created archive with {} files ({} bytes)",
                archived_files.len(),
                total_size
            ))
            .with_result(json!({
                "archive_path": archive_path.to_string_lossy().to_string(),
                "archived_files": archived_files,
                "total_size": total_size,
                "compression_level": self.config.compression_level
            }))
            .build())
    }

    /// Create ZIP archive
    fn create_zip(
        &self,
        archive_path: &Path,
        files: &[Value],
        archived_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::create(archive_path),
            format!("Failed to create archive: {}", archive_path.display())
        )?;
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

        for file_value in files {
            if let Some(file_path) = file_value.as_str() {
                let path = Path::new(&file_path);
                if !path.exists() {
                    warn!("File not found: {}", file_path);
                    continue;
                }

                if path.is_file() {
                    let metadata = io_err!(
                        fs::metadata(path),
                        format!("Failed to read file metadata: {}", path.display())
                    )?;
                    if metadata.len() > self.config.max_file_size {
                        warn!("Skipping file {} - exceeds size limit", file_path);
                        continue;
                    }

                    let file_name = path
                        .file_name()
                        .ok_or_else(|| LLMSpellError::Validation {
                            message: "Invalid file name".to_string(),
                            field: Some("files".to_string()),
                        })?
                        .to_string_lossy();

                    zip.start_file(file_name.as_ref(), options).map_err(|e| {
                        LLMSpellError::Storage {
                            message: format!("Failed to start ZIP entry: {}", file_name),
                            operation: Some("zip_start_file".to_string()),
                            source: Some(Box::new(e)),
                        }
                    })?;
                    let mut f = io_err!(
                        File::open(path),
                        format!("Failed to open file: {}", path.display())
                    )?;
                    io_err!(
                        io::copy(&mut f, &mut zip),
                        format!("Failed to add file to ZIP: {}", path.display())
                    )?;

                    archived_files.push(file_path.to_string());
                    *total_size += metadata.len();
                }
            }
        }

        zip.finish().map_err(|e| LLMSpellError::Storage {
            message: "Failed to finalize ZIP archive".to_string(),
            operation: Some("zip_finish".to_string()),
            source: Some(Box::new(e)),
        })?;
        Ok(())
    }

    /// Create TAR archive
    fn create_tar(
        &self,
        archive_path: &Path,
        files: &[Value],
        archived_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::create(archive_path),
            format!("Failed to create archive: {}", archive_path.display())
        )?;
        let mut builder = Builder::new(file);

        for file_value in files {
            if let Some(file_path) = file_value.as_str() {
                let path = Path::new(&file_path);
                if !path.exists() {
                    warn!("File not found: {}", file_path);
                    continue;
                }

                if path.is_file() {
                    let metadata = io_err!(
                        fs::metadata(path),
                        format!("Failed to read file metadata: {}", path.display())
                    )?;
                    if metadata.len() > self.config.max_file_size {
                        warn!("Skipping file {} - exceeds size limit", file_path);
                        continue;
                    }

                    let file_name = path.file_name().ok_or_else(|| LLMSpellError::Validation {
                        message: "Invalid file name".to_string(),
                        field: Some("files".to_string()),
                    })?;

                    let mut f = io_err!(
                        File::open(path),
                        format!("Failed to open file: {}", path.display())
                    )?;
                    builder
                        .append_file(file_name, &mut f)
                        .map_err(|e| LLMSpellError::Storage {
                            message: format!("Failed to add file to TAR: {}", path.display()),
                            operation: Some("tar_append_file".to_string()),
                            source: Some(Box::new(e)),
                        })?;

                    archived_files.push(file_path.to_string());
                    *total_size += metadata.len();
                }
            }
        }

        builder.finish().map_err(|e| LLMSpellError::Storage {
            message: "Failed to finalize TAR archive".to_string(),
            operation: Some("tar_finish".to_string()),
            source: Some(Box::new(e)),
        })?;
        Ok(())
    }

    /// Create TAR.GZ archive
    fn create_tar_gz(
        &self,
        archive_path: &Path,
        files: &[Value],
        archived_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        let file = io_err!(
            File::create(archive_path),
            format!("Failed to create archive: {}", archive_path.display())
        )?;
        let encoder = GzEncoder::new(file, Compression::new(self.config.compression_level));
        let mut builder = Builder::new(encoder);

        for file_value in files {
            if let Some(file_path) = file_value.as_str() {
                let path = Path::new(&file_path);
                if !path.exists() {
                    warn!("File not found: {}", file_path);
                    continue;
                }

                if path.is_file() {
                    let metadata = io_err!(
                        fs::metadata(path),
                        format!("Failed to read file metadata: {}", path.display())
                    )?;
                    if metadata.len() > self.config.max_file_size {
                        warn!("Skipping file {} - exceeds size limit", file_path);
                        continue;
                    }

                    let file_name = path.file_name().ok_or_else(|| LLMSpellError::Validation {
                        message: "Invalid file name".to_string(),
                        field: Some("files".to_string()),
                    })?;

                    let mut f = io_err!(
                        File::open(path),
                        format!("Failed to open file: {}", path.display())
                    )?;
                    builder
                        .append_file(file_name, &mut f)
                        .map_err(|e| LLMSpellError::Storage {
                            message: format!("Failed to add file to TAR: {}", path.display()),
                            operation: Some("tar_append_file".to_string()),
                            source: Some(Box::new(e)),
                        })?;

                    archived_files.push(file_path.to_string());
                    *total_size += metadata.len();
                }
            }
        }

        builder.finish().map_err(|e| LLMSpellError::Storage {
            message: "Failed to finalize TAR archive".to_string(),
            operation: Some("tar_finish".to_string()),
            source: Some(Box::new(e)),
        })?;
        Ok(())
    }

    /// Create GZ file (single file compression)
    fn create_gz(
        &self,
        archive_path: &Path,
        files: &[Value],
        archived_files: &mut Vec<String>,
        total_size: &mut u64,
    ) -> Result<()> {
        if files.len() != 1 {
            return Err(LLMSpellError::Validation {
                message: "GZ format can only compress a single file".to_string(),
                field: Some("files".to_string()),
            });
        }

        if let Some(file_path) = files[0].as_str() {
            let path = Path::new(&file_path);
            if !path.exists() {
                return Err(LLMSpellError::Validation {
                    message: format!("File not found: {}", file_path),
                    field: Some("files".to_string()),
                });
            }

            if !path.is_file() {
                return Err(LLMSpellError::Validation {
                    message: "Can only compress files, not directories".to_string(),
                    field: Some("files".to_string()),
                });
            }

            let metadata = io_err!(
                fs::metadata(path),
                format!("Failed to read file metadata: {}", path.display())
            )?;
            if metadata.len() > self.config.max_file_size {
                return Err(LLMSpellError::Validation {
                    message: "File exceeds size limit".to_string(),
                    field: Some("files".to_string()),
                });
            }

            let file = io_err!(
                File::create(archive_path),
                format!("Failed to create archive: {}", archive_path.display())
            )?;
            let mut encoder = GzEncoder::new(file, Compression::new(self.config.compression_level));

            let mut input = io_err!(
                File::open(path),
                format!("Failed to open file: {}", path.display())
            )?;
            io_err!(
                io::copy(&mut input, &mut encoder),
                format!("Failed to compress file: {}", path.display())
            )?;
            encoder.finish().map_err(|e| LLMSpellError::Storage {
                message: "Failed to finalize GZ archive".to_string(),
                operation: Some("gz_finish".to_string()),
                source: Some(Box::new(e)),
            })?;

            archived_files.push(file_path.to_string());
            *total_size += metadata.len();
        }

        Ok(())
    }

    /// List archive contents
    async fn list_archive(&self, params: &Value) -> Result<Value> {
        let archive_path = params.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            LLMSpellError::Validation {
                message: "Missing 'path' parameter".to_string(),
                field: Some("path".to_string()),
            }
        })?;

        let archive_path = PathBuf::from(archive_path);

        // Check sandbox permissions
        if let Some(sandbox) = &self.file_sandbox {
            sandbox
                .validate_path(&archive_path)
                .map_err(|e| LLMSpellError::Security {
                    message: format!("Archive path validation failed: {}", e),
                    violation_type: Some("file_access".to_string()),
                })?;
        }

        let format = Self::detect_format(&archive_path)?;
        let mut files = Vec::new();

        match format {
            ArchiveFormat::Zip => {
                let file = io_err!(
                    File::open(&archive_path),
                    format!("Failed to open archive: {}", archive_path.display())
                )?;
                let mut archive = ZipArchive::new(file).map_err(|e| LLMSpellError::Validation {
                    message: format!("Invalid ZIP archive: {}", e),
                    field: Some("path".to_string()),
                })?;

                for i in 0..archive.len() {
                    let file = archive.by_index(i).map_err(|e| LLMSpellError::Storage {
                        message: format!("Failed to read file at index {}", i),
                        operation: Some("zip_list_entry".to_string()),
                        source: Some(Box::new(e)),
                    })?;
                    files.push(serde_json::json!({
                        "name": file.name(),
                        "size": file.size(),
                        "compressed_size": file.compressed_size(),
                        "is_dir": file.is_dir()
                    }));
                }
            }
            ArchiveFormat::Tar => {
                let file = io_err!(
                    File::open(&archive_path),
                    format!("Failed to open archive: {}", archive_path.display())
                )?;
                let mut archive = Archive::new(file);

                for entry in archive.entries().map_err(|e| LLMSpellError::Storage {
                    message: "Failed to read TAR entries".to_string(),
                    operation: Some("tar_list_entries".to_string()),
                    source: Some(Box::new(e)),
                })? {
                    let entry = entry.map_err(|e| LLMSpellError::Storage {
                        message: "Failed to read TAR entry".to_string(),
                        operation: Some("tar_list_entry".to_string()),
                        source: Some(Box::new(e)),
                    })?;
                    let path = entry.path().map_err(|e| LLMSpellError::Storage {
                        message: "Failed to read TAR entry path".to_string(),
                        operation: Some("tar_list_path".to_string()),
                        source: Some(Box::new(e)),
                    })?;
                    let header = entry.header();

                    files.push(serde_json::json!({
                        "name": path.to_string_lossy().to_string(),
                        "size": header.size().map_err(|e| LLMSpellError::Storage {
                            message: "Failed to read TAR entry size".to_string(),
                            operation: Some("tar_entry_size".to_string()),
                            source: Some(Box::new(e)),
                        })?,
                        "is_dir": header.entry_type().is_dir()
                    }));
                }
            }
            ArchiveFormat::TarGz => {
                let file = io_err!(
                    File::open(&archive_path),
                    format!("Failed to open archive: {}", archive_path.display())
                )?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);

                for entry in archive.entries().map_err(|e| LLMSpellError::Storage {
                    message: "Failed to read TAR.GZ entries".to_string(),
                    operation: Some("targz_list_entries".to_string()),
                    source: Some(Box::new(e)),
                })? {
                    let entry = entry.map_err(|e| LLMSpellError::Storage {
                        message: "Failed to read TAR.GZ entry".to_string(),
                        operation: Some("targz_list_entry".to_string()),
                        source: Some(Box::new(e)),
                    })?;
                    let path = entry.path().map_err(|e| LLMSpellError::Storage {
                        message: "Failed to read TAR.GZ entry path".to_string(),
                        operation: Some("targz_list_path".to_string()),
                        source: Some(Box::new(e)),
                    })?;
                    let header = entry.header();

                    files.push(serde_json::json!({
                        "name": path.to_string_lossy().to_string(),
                        "size": header.size().map_err(|e| LLMSpellError::Storage {
                            message: "Failed to read TAR.GZ entry size".to_string(),
                            operation: Some("targz_entry_size".to_string()),
                            source: Some(Box::new(e)),
                        })?,
                        "is_dir": header.entry_type().is_dir()
                    }));
                }
            }
            ArchiveFormat::Gz => {
                // GZ files don't store original filename, just indicate it's compressed
                let metadata = io_err!(
                    fs::metadata(&archive_path),
                    format!(
                        "Failed to read archive metadata: {}",
                        archive_path.display()
                    )
                )?;
                files.push(serde_json::json!({
                    "name": "<compressed file>",
                    "compressed_size": metadata.len()
                }));
            }
        }

        Ok(ResponseBuilder::success("list")
            .with_message(format!(
                "Listed {} files in {} archive",
                files.len(),
                format
            ))
            .with_result(json!({
                "format": format.to_string(),
                "files": files,
                "file_count": files.len()
            }))
            .build())
    }
}

/// Archive format enumeration
#[derive(Debug, Clone, Copy)]
enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    Gz,
}

impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveFormat::Zip => write!(f, "ZIP"),
            ArchiveFormat::Tar => write!(f, "TAR"),
            ArchiveFormat::TarGz => write!(f, "TAR.GZ"),
            ArchiveFormat::Gz => write!(f, "GZ"),
        }
    }
}

#[async_trait]
impl BaseAgent for ArchiveHandlerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> llmspell_core::Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;
        let operation = extract_required_string(params, "operation")?;

        let response = match operation {
            "extract" => self.extract_archive(params).await?,
            "create" => self.create_archive(params).await?,
            "list" => self.list_archive(params).await?,
            _ => {
                return Err(LLMSpellError::Validation {
                    message: format!("Unknown operation: {}", operation),
                    field: Some("operation".to_string()),
                })
            }
        };

        // Extract the message for text output
        let output_text = response
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Operation completed")
            .to_string();

        // Create metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata
            .extra
            .insert("operation".to_string(), operation.into());
        metadata.extra.insert("response".to_string(), response);

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> llmspell_core::Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> llmspell_core::Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Archive handler error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for ArchiveHandlerTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Filesystem
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "archive-handler".to_string(),
            description: "Handle ZIP, TAR, and GZ archives with security controls".to_string(),
            parameters: vec![
                ParameterDef {
                    name: "operation".to_string(),
                    description: "Archive operation: extract, create, or list".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "path".to_string(),
                    description: "Path to archive file".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "target_path".to_string(),
                    description: "Directory to extract to (for extract operation)".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "input".to_string(),
                    description: "Files to add to archive (for create operation)".to_string(),
                    param_type: ParameterType::Array,
                    required: false,
                    default: None,
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }
}

impl Default for ArchiveHandlerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_detect_format() {
        assert!(matches!(
            ArchiveHandlerTool::detect_format(Path::new("test.zip")).unwrap(),
            ArchiveFormat::Zip
        ));
        assert!(matches!(
            ArchiveHandlerTool::detect_format(Path::new("test.tar")).unwrap(),
            ArchiveFormat::Tar
        ));
        assert!(matches!(
            ArchiveHandlerTool::detect_format(Path::new("test.tar.gz")).unwrap(),
            ArchiveFormat::TarGz
        ));
        assert!(matches!(
            ArchiveHandlerTool::detect_format(Path::new("test.gz")).unwrap(),
            ArchiveFormat::Gz
        ));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_safe_path() {
        assert!(ArchiveHandlerTool::is_safe_path(Path::new("file.txt")));
        assert!(ArchiveHandlerTool::is_safe_path(Path::new("dir/file.txt")));
        assert!(!ArchiveHandlerTool::is_safe_path(Path::new("../file.txt")));
        assert!(!ArchiveHandlerTool::is_safe_path(Path::new(
            "dir/../../file.txt"
        )));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_create_and_extract_zip() {
        let temp_dir = TempDir::new().unwrap();
        let tool = ArchiveHandlerTool::new();

        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "Hello").unwrap();
        fs::write(&file2, "World").unwrap();

        // Create archive
        let archive_path = temp_dir.path().join("test.zip");
        let create_params = serde_json::json!({
            "operation": "create",
            "path": archive_path.to_str().unwrap(),
            "input": [file1.to_str().unwrap(), file2.to_str().unwrap()]
        });

        let input = AgentInput::text("").with_parameter("parameters", create_params);
        let _result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(archive_path.exists());

        // Extract archive
        let extract_dir = temp_dir.path().join("extracted");
        let extract_params = serde_json::json!({
            "operation": "extract",
            "path": archive_path.to_str().unwrap(),
            "target_path": extract_dir.to_str().unwrap()
        });

        let input = AgentInput::text("").with_parameter("parameters", extract_params);
        let _result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(extract_dir.join("file1.txt").exists());
        assert!(extract_dir.join("file2.txt").exists());
    }
}
