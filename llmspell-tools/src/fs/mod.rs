//! ABOUTME: File system tools module for safe file operations
//! ABOUTME: Provides sandboxed file operations with security controls

pub mod file_operations;

pub use file_operations::{FileOperationsConfig, FileOperationsTool};
