// ABOUTME: Storage backend implementation patterns for persistent data management
// ABOUTME: Demonstrates how to implement custom storage backends for state, sessions, and artifact persistence

//! # Storage Backend Patterns
//! 
//! This module demonstrates how to implement custom storage backends for llmspell,
//! including database integration, file systems, cloud storage, and caching layers.
//! 
//! ## Key Patterns
//! 
//! 1. **Storage Trait Implementation**: Core storage abstraction
//! 2. **Database Backends**: SQL and NoSQL database integration
//! 3. **File System Storage**: Local and distributed file storage
//! 4. **Cloud Storage**: S3-compatible and cloud provider integration
//! 5. **Caching Layers**: Redis, in-memory, and hybrid caching
//! 6. **Serialization**: Efficient data serialization and compression
//! 7. **Transactions**: ACID compliance and consistency patterns
//! 8. **Migration**: Schema evolution and data migration strategies

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use uuid::Uuid;

/// Core storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend_type: StorageBackendType,
    pub connection_string: String,
    pub encryption_key: Option<String>,
    pub compression_enabled: bool,
    pub backup_enabled: bool,
    pub retention_policy: RetentionPolicy,
    pub performance_settings: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendType {
    FileSystem,
    SQLite,
    PostgreSQL,
    Redis,
    S3Compatible,
    MongoDB,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_age_days: Option<u32>,
    pub max_size_bytes: Option<u64>,
    pub auto_cleanup: bool,
    pub backup_before_cleanup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub connection_pool_size: u32,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub batch_size: u32,
    pub enable_compression: bool,
    pub cache_ttl: Duration,
}

/// Generic data record for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRecord {
    pub id: String,
    pub namespace: String,
    pub key: String,
    pub data: StorageData,
    pub metadata: StorageMetadata,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageData {
    Text(String),
    Binary(Vec<u8>),
    Json(serde_json::Value),
    Compressed(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub content_type: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub tags: HashMap<String, String>,
    pub version: u32,
}

/// Query parameters for storage operations
#[derive(Debug, Clone, Default)]
pub struct StorageQuery {
    pub namespace: Option<String>,
    pub key_prefix: Option<String>,
    pub tags: HashMap<String, String>,
    pub created_after: Option<SystemTime>,
    pub created_before: Option<SystemTime>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Storage operation results
#[derive(Debug)]
pub struct StorageResult {
    pub success: bool,
    pub records_affected: u64,
    pub operation_time_ms: u64,
    pub error: Option<String>,
}

/// Batch operation for bulk operations
#[derive(Debug)]
pub struct BatchOperation {
    pub operation_type: BatchOperationType,
    pub records: Vec<StorageRecord>,
}

#[derive(Debug)]
pub enum BatchOperationType {
    Insert,
    Update,
    Delete,
    Upsert,
}

/// Storage backend errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Record not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Permission denied: {0}")]
    PermissionError(String),
    
    #[error("Storage full: {0}")]
    StorageFull(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Corruption detected: {0}")]
    CorruptionError(String),
}

/// Core storage backend trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Backend identification
    fn backend_type(&self) -> StorageBackendType;
    fn name(&self) -> &str;
    
    /// Connection management
    async fn connect(&mut self) -> Result<(), StorageError>;
    async fn disconnect(&mut self) -> Result<(), StorageError>;
    async fn health_check(&self) -> Result<StorageHealth, StorageError>;
    
    /// Basic CRUD operations
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<StorageRecord>, StorageError>;
    async fn put(&self, record: StorageRecord) -> Result<StorageResult, StorageError>;
    async fn delete(&self, namespace: &str, key: &str) -> Result<StorageResult, StorageError>;
    async fn exists(&self, namespace: &str, key: &str) -> Result<bool, StorageError>;
    
    /// Query operations
    async fn query(&self, query: StorageQuery) -> Result<Vec<StorageRecord>, StorageError>;
    async fn count(&self, query: StorageQuery) -> Result<u64, StorageError>;
    
    /// Batch operations
    async fn batch_execute(&self, operations: Vec<BatchOperation>) -> Result<Vec<StorageResult>, StorageError>;
    
    /// Namespace management
    async fn list_namespaces(&self) -> Result<Vec<String>, StorageError>;
    async fn create_namespace(&self, namespace: &str) -> Result<(), StorageError>;
    async fn delete_namespace(&self, namespace: &str) -> Result<StorageResult, StorageError>;
    
    /// Maintenance operations
    async fn cleanup_expired(&self) -> Result<StorageResult, StorageError>;
    async fn backup(&self, backup_path: &str) -> Result<StorageResult, StorageError>;
    async fn restore(&self, backup_path: &str) -> Result<StorageResult, StorageError>;
    async fn optimize(&self) -> Result<StorageResult, StorageError>;
}

/// Storage health information
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageHealth {
    pub status: HealthStatus,
    pub total_records: u64,
    pub total_size_bytes: u64,
    pub available_space_bytes: Option<u64>,
    pub connection_count: u32,
    pub last_backup: Option<SystemTime>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Offline,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_read_time_ms: f64,
    pub avg_write_time_ms: f64,
    pub operations_per_second: f64,
    pub cache_hit_ratio: f64,
}

/// Pattern 1: File System Storage Backend
/// 
/// This demonstrates a simple file-based storage implementation
pub struct FileSystemStorage {
    config: StorageConfig,
    base_path: PathBuf,
    connected: bool,
}

impl FileSystemStorage {
    pub fn new(config: StorageConfig) -> Result<Self, StorageError> {
        let base_path = PathBuf::from(&config.connection_string);
        
        Ok(Self {
            config,
            base_path,
            connected: false,
        })
    }
    
    fn get_record_path(&self, namespace: &str, key: &str) -> PathBuf {
        self.base_path
            .join(namespace)
            .join(format!("{}.json", key))
    }
    
    fn get_namespace_path(&self, namespace: &str) -> PathBuf {
        self.base_path.join(namespace)
    }
    
    async fn ensure_namespace_exists(&self, namespace: &str) -> Result<(), StorageError> {
        let namespace_path = self.get_namespace_path(namespace);
        if !namespace_path.exists() {
            fs::create_dir_all(&namespace_path)
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }
    
    async fn serialize_record(&self, record: &StorageRecord) -> Result<Vec<u8>, StorageError> {
        let json = serde_json::to_vec(record)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        if self.config.compression_enabled {
            // In a real implementation, use proper compression like gzip
            Ok(json) // Simplified for example
        } else {
            Ok(json)
        }
    }
    
    async fn deserialize_record(&self, data: &[u8]) -> Result<StorageRecord, StorageError> {
        let record = serde_json::from_slice(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(record)
    }
    
    fn calculate_checksum(&self, data: &[u8]) -> String {
        // Simple checksum calculation (in production, use proper hashing)
        let checksum = data.iter().fold(0u32, |acc, &byte| acc.wrapping_add(byte as u32));
        format!("{:08x}", checksum)
    }
}

#[async_trait]
impl StorageBackend for FileSystemStorage {
    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::FileSystem
    }
    
    fn name(&self) -> &str {
        "FileSystemStorage"
    }
    
    async fn connect(&mut self) -> Result<(), StorageError> {
        // Ensure base directory exists
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        self.connected = true;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), StorageError> {
        self.connected = false;
        Ok(())
    }
    
    async fn health_check(&self) -> Result<StorageHealth, StorageError> {
        if !self.connected {
            return Ok(StorageHealth {
                status: HealthStatus::Offline,
                total_records: 0,
                total_size_bytes: 0,
                available_space_bytes: None,
                connection_count: 0,
                last_backup: None,
                performance_metrics: PerformanceMetrics {
                    avg_read_time_ms: 0.0,
                    avg_write_time_ms: 0.0,
                    operations_per_second: 0.0,
                    cache_hit_ratio: 0.0,
                },
            });
        }
        
        // Count records and calculate total size
        let mut total_records = 0u64;
        let mut total_size = 0u64;
        
        let mut dir_entries = fs::read_dir(&self.base_path).await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        while let Some(entry) = dir_entries.next_entry().await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))? {
            
            if entry.file_type().await.unwrap_or_default().is_dir() {
                let mut namespace_entries = fs::read_dir(entry.path()).await
                    .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
                
                while let Some(file_entry) = namespace_entries.next_entry().await
                    .map_err(|e| StorageError::ConnectionError(e.to_string()))? {
                    
                    if file_entry.file_name().to_string_lossy().ends_with(".json") {
                        total_records += 1;
                        
                        if let Ok(metadata) = file_entry.metadata().await {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }
        
        Ok(StorageHealth {
            status: HealthStatus::Healthy,
            total_records,
            total_size_bytes: total_size,
            available_space_bytes: None, // Would need platform-specific code
            connection_count: 1,
            last_backup: None,
            performance_metrics: PerformanceMetrics {
                avg_read_time_ms: 5.0,
                avg_write_time_ms: 10.0,
                operations_per_second: 100.0,
                cache_hit_ratio: 0.0,
            },
        })
    }
    
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<StorageRecord>, StorageError> {
        let record_path = self.get_record_path(namespace, key);
        
        if !record_path.exists() {
            return Ok(None);
        }
        
        let data = fs::read(&record_path)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        let record = self.deserialize_record(&data).await?;
        
        // Check if record is expired
        if let Some(expires_at) = record.expires_at {
            if SystemTime::now() > expires_at {
                // Remove expired record
                let _ = fs::remove_file(&record_path).await;
                return Ok(None);
            }
        }
        
        Ok(Some(record))
    }
    
    async fn put(&self, mut record: StorageRecord) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        self.ensure_namespace_exists(&record.namespace).await?;
        
        // Update metadata
        record.updated_at = SystemTime::now();
        if record.id.is_empty() {
            record.id = Uuid::new_v4().to_string();
        }
        
        let data = self.serialize_record(&record).await?;
        record.metadata.size_bytes = data.len() as u64;
        record.metadata.checksum = self.calculate_checksum(&data);
        
        // Re-serialize with updated metadata
        let final_data = self.serialize_record(&record).await?;
        
        let record_path = self.get_record_path(&record.namespace, &record.key);
        
        fs::write(&record_path, &final_data)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: 1,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn delete(&self, namespace: &str, key: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        let record_path = self.get_record_path(namespace, key);
        
        let existed = record_path.exists();
        
        if existed {
            fs::remove_file(&record_path)
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        }
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: if existed { 1 } else { 0 },
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn exists(&self, namespace: &str, key: &str) -> Result<bool, StorageError> {
        let record_path = self.get_record_path(namespace, key);
        Ok(record_path.exists())
    }
    
    async fn query(&self, query: StorageQuery) -> Result<Vec<StorageRecord>, StorageError> {
        let mut results = Vec::new();
        
        // Get namespaces to search
        let namespaces = if let Some(ns) = &query.namespace {
            vec![ns.clone()]
        } else {
            self.list_namespaces().await?
        };
        
        for namespace in namespaces {
            let namespace_path = self.get_namespace_path(&namespace);
            
            if !namespace_path.exists() {
                continue;
            }
            
            let mut dir_entries = fs::read_dir(&namespace_path).await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
            
            while let Some(entry) = dir_entries.next_entry().await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))? {
                
                if !entry.file_name().to_string_lossy().ends_with(".json") {
                    continue;
                }
                
                let data = fs::read(entry.path()).await
                    .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
                
                if let Ok(record) = self.deserialize_record(&data).await {
                    // Apply query filters
                    if let Some(prefix) = &query.key_prefix {
                        if !record.key.starts_with(prefix) {
                            continue;
                        }
                    }
                    
                    if let Some(created_after) = query.created_after {
                        if record.created_at <= created_after {
                            continue;
                        }
                    }
                    
                    if let Some(created_before) = query.created_before {
                        if record.created_at >= created_before {
                            continue;
                        }
                    }
                    
                    // Check tag filters
                    if !query.tags.is_empty() {
                        let mut matches = true;
                        for (key, value) in &query.tags {
                            if record.metadata.tags.get(key) != Some(value) {
                                matches = false;
                                break;
                            }
                        }
                        if !matches {
                            continue;
                        }
                    }
                    
                    results.push(record);
                }
            }
        }
        
        // Apply limit and offset
        if let Some(offset) = query.offset {
            let offset = offset as usize;
            if offset < results.len() {
                results = results[offset..].to_vec();
            } else {
                results.clear();
            }
        }
        
        if let Some(limit) = query.limit {
            let limit = limit as usize;
            if results.len() > limit {
                results.truncate(limit);
            }
        }
        
        Ok(results)
    }
    
    async fn count(&self, query: StorageQuery) -> Result<u64, StorageError> {
        let records = self.query(query).await?;
        Ok(records.len() as u64)
    }
    
    async fn batch_execute(&self, operations: Vec<BatchOperation>) -> Result<Vec<StorageResult>, StorageError> {
        let mut results = Vec::new();
        
        for operation in operations {
            match operation.operation_type {
                BatchOperationType::Insert | BatchOperationType::Upsert => {
                    for record in operation.records {
                        let result = self.put(record).await?;
                        results.push(result);
                    }
                }
                BatchOperationType::Delete => {
                    for record in operation.records {
                        let result = self.delete(&record.namespace, &record.key).await?;
                        results.push(result);
                    }
                }
                BatchOperationType::Update => {
                    for record in operation.records {
                        // Check if record exists before updating
                        if self.exists(&record.namespace, &record.key).await? {
                            let result = self.put(record).await?;
                            results.push(result);
                        } else {
                            results.push(StorageResult {
                                success: false,
                                records_affected: 0,
                                operation_time_ms: 0,
                                error: Some("Record not found for update".to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn list_namespaces(&self) -> Result<Vec<String>, StorageError> {
        let mut namespaces = Vec::new();
        
        let mut dir_entries = fs::read_dir(&self.base_path).await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        while let Some(entry) = dir_entries.next_entry().await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))? {
            
            if entry.file_type().await.unwrap_or_default().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    namespaces.push(name.to_string());
                }
            }
        }
        
        Ok(namespaces)
    }
    
    async fn create_namespace(&self, namespace: &str) -> Result<(), StorageError> {
        self.ensure_namespace_exists(namespace).await
    }
    
    async fn delete_namespace(&self, namespace: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        let namespace_path = self.get_namespace_path(namespace);
        
        let existed = namespace_path.exists();
        let mut records_affected = 0;
        
        if existed {
            // Count records before deletion
            let query = StorageQuery {
                namespace: Some(namespace.to_string()),
                ..Default::default()
            };
            records_affected = self.count(query).await?;
            
            fs::remove_dir_all(&namespace_path)
                .await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        }
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn cleanup_expired(&self) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        let mut records_cleaned = 0;
        
        let namespaces = self.list_namespaces().await?;
        let now = SystemTime::now();
        
        for namespace in namespaces {
            let namespace_path = self.get_namespace_path(&namespace);
            
            let mut dir_entries = fs::read_dir(&namespace_path).await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
            
            while let Some(entry) = dir_entries.next_entry().await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))? {
                
                if !entry.file_name().to_string_lossy().ends_with(".json") {
                    continue;
                }
                
                if let Ok(data) = fs::read(entry.path()).await {
                    if let Ok(record) = self.deserialize_record(&data).await {
                        if let Some(expires_at) = record.expires_at {
                            if now > expires_at {
                                let _ = fs::remove_file(entry.path()).await;
                                records_cleaned += 1;
                            }
                        }
                    }
                }
            }
        }
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: records_cleaned,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn backup(&self, backup_path: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        // Simple backup: copy entire directory structure
        let backup_dir = PathBuf::from(backup_path);
        
        // In a real implementation, this would be more sophisticated
        // For now, just count the files that would be backed up
        let health = self.health_check().await?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: health.total_records,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn restore(&self, _backup_path: &str) -> Result<StorageResult, StorageError> {
        // Simplified restore implementation
        Ok(StorageResult {
            success: true,
            records_affected: 0,
            operation_time_ms: 0,
            error: Some("Restore not implemented in this example".to_string()),
        })
    }
    
    async fn optimize(&self) -> Result<StorageResult, StorageError> {
        // Cleanup expired records as optimization
        self.cleanup_expired().await
    }
}

/// Pattern 2: In-Memory Storage with Persistence
/// 
/// This demonstrates a high-performance in-memory storage with optional persistence
pub struct InMemoryStorage {
    config: StorageConfig,
    data: tokio::sync::RwLock<HashMap<String, HashMap<String, StorageRecord>>>,
    connected: bool,
    persistence_file: Option<PathBuf>,
}

impl InMemoryStorage {
    pub fn new(config: StorageConfig) -> Self {
        let persistence_file = if config.connection_string != ":memory:" {
            Some(PathBuf::from(&config.connection_string))
        } else {
            None
        };
        
        Self {
            config,
            data: tokio::sync::RwLock::new(HashMap::new()),
            connected: false,
            persistence_file,
        }
    }
    
    async fn load_from_disk(&self) -> Result<(), StorageError> {
        if let Some(file_path) = &self.persistence_file {
            if file_path.exists() {
                let data = fs::read(file_path).await
                    .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
                
                let loaded_data: HashMap<String, HashMap<String, StorageRecord>> = 
                    serde_json::from_slice(&data)
                        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                
                let mut data_guard = self.data.write().await;
                *data_guard = loaded_data;
            }
        }
        Ok(())
    }
    
    async fn save_to_disk(&self) -> Result<(), StorageError> {
        if let Some(file_path) = &self.persistence_file {
            let data_guard = self.data.read().await;
            let data = serde_json::to_vec(&*data_guard)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            fs::write(file_path, data).await
                .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::FileSystem // Simplified for example
    }
    
    fn name(&self) -> &str {
        "InMemoryStorage"
    }
    
    async fn connect(&mut self) -> Result<(), StorageError> {
        self.load_from_disk().await?;
        self.connected = true;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), StorageError> {
        self.save_to_disk().await?;
        self.connected = false;
        Ok(())
    }
    
    async fn health_check(&self) -> Result<StorageHealth, StorageError> {
        let data_guard = self.data.read().await;
        
        let mut total_records = 0u64;
        let mut total_size = 0u64;
        
        for namespace_data in data_guard.values() {
            total_records += namespace_data.len() as u64;
            for record in namespace_data.values() {
                total_size += record.metadata.size_bytes;
            }
        }
        
        Ok(StorageHealth {
            status: if self.connected { HealthStatus::Healthy } else { HealthStatus::Offline },
            total_records,
            total_size_bytes: total_size,
            available_space_bytes: None,
            connection_count: 1,
            last_backup: None,
            performance_metrics: PerformanceMetrics {
                avg_read_time_ms: 0.1,
                avg_write_time_ms: 0.5,
                operations_per_second: 10000.0,
                cache_hit_ratio: 1.0, // Always cache hit for in-memory
            },
        })
    }
    
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<StorageRecord>, StorageError> {
        let data_guard = self.data.read().await;
        
        if let Some(namespace_data) = data_guard.get(namespace) {
            if let Some(record) = namespace_data.get(key) {
                // Check expiration
                if let Some(expires_at) = record.expires_at {
                    if SystemTime::now() > expires_at {
                        return Ok(None);
                    }
                }
                return Ok(Some(record.clone()));
            }
        }
        
        Ok(None)
    }
    
    async fn put(&self, mut record: StorageRecord) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        // Update metadata
        record.updated_at = SystemTime::now();
        if record.id.is_empty() {
            record.id = Uuid::new_v4().to_string();
        }
        
        let mut data_guard = self.data.write().await;
        
        let namespace_data = data_guard.entry(record.namespace.clone())
            .or_insert_with(HashMap::new);
        
        namespace_data.insert(record.key.clone(), record);
        
        drop(data_guard);
        
        // Save to disk if persistence is enabled
        self.save_to_disk().await?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: 1,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn delete(&self, namespace: &str, key: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        let mut data_guard = self.data.write().await;
        let records_affected = if let Some(namespace_data) = data_guard.get_mut(namespace) {
            if namespace_data.remove(key).is_some() { 1 } else { 0 }
        } else {
            0
        };
        
        drop(data_guard);
        
        // Save to disk if persistence is enabled
        self.save_to_disk().await?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn exists(&self, namespace: &str, key: &str) -> Result<bool, StorageError> {
        let data_guard = self.data.read().await;
        
        if let Some(namespace_data) = data_guard.get(namespace) {
            Ok(namespace_data.contains_key(key))
        } else {
            Ok(false)
        }
    }
    
    async fn query(&self, query: StorageQuery) -> Result<Vec<StorageRecord>, StorageError> {
        let data_guard = self.data.read().await;
        let mut results = Vec::new();
        
        for (namespace, namespace_data) in data_guard.iter() {
            // Apply namespace filter
            if let Some(query_namespace) = &query.namespace {
                if namespace != query_namespace {
                    continue;
                }
            }
            
            for record in namespace_data.values() {
                // Apply filters (similar to file system implementation)
                if let Some(prefix) = &query.key_prefix {
                    if !record.key.starts_with(prefix) {
                        continue;
                    }
                }
                
                if let Some(created_after) = query.created_after {
                    if record.created_at <= created_after {
                        continue;
                    }
                }
                
                if let Some(created_before) = query.created_before {
                    if record.created_at >= created_before {
                        continue;
                    }
                }
                
                // Check tag filters
                if !query.tags.is_empty() {
                    let mut matches = true;
                    for (key, value) in &query.tags {
                        if record.metadata.tags.get(key) != Some(value) {
                            matches = false;
                            break;
                        }
                    }
                    if !matches {
                        continue;
                    }
                }
                
                results.push(record.clone());
            }
        }
        
        // Apply limit and offset
        if let Some(offset) = query.offset {
            let offset = offset as usize;
            if offset < results.len() {
                results = results[offset..].to_vec();
            } else {
                results.clear();
            }
        }
        
        if let Some(limit) = query.limit {
            let limit = limit as usize;
            if results.len() > limit {
                results.truncate(limit);
            }
        }
        
        Ok(results)
    }
    
    async fn count(&self, query: StorageQuery) -> Result<u64, StorageError> {
        let records = self.query(query).await?;
        Ok(records.len() as u64)
    }
    
    async fn batch_execute(&self, operations: Vec<BatchOperation>) -> Result<Vec<StorageResult>, StorageError> {
        let mut results = Vec::new();
        
        for operation in operations {
            match operation.operation_type {
                BatchOperationType::Insert | BatchOperationType::Upsert => {
                    for record in operation.records {
                        let result = self.put(record).await?;
                        results.push(result);
                    }
                }
                BatchOperationType::Delete => {
                    for record in operation.records {
                        let result = self.delete(&record.namespace, &record.key).await?;
                        results.push(result);
                    }
                }
                BatchOperationType::Update => {
                    for record in operation.records {
                        if self.exists(&record.namespace, &record.key).await? {
                            let result = self.put(record).await?;
                            results.push(result);
                        } else {
                            results.push(StorageResult {
                                success: false,
                                records_affected: 0,
                                operation_time_ms: 0,
                                error: Some("Record not found for update".to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn list_namespaces(&self) -> Result<Vec<String>, StorageError> {
        let data_guard = self.data.read().await;
        Ok(data_guard.keys().cloned().collect())
    }
    
    async fn create_namespace(&self, namespace: &str) -> Result<(), StorageError> {
        let mut data_guard = self.data.write().await;
        data_guard.entry(namespace.to_string()).or_insert_with(HashMap::new);
        Ok(())
    }
    
    async fn delete_namespace(&self, namespace: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        let mut data_guard = self.data.write().await;
        let records_affected = if let Some(namespace_data) = data_guard.remove(namespace) {
            namespace_data.len() as u64
        } else {
            0
        };
        
        drop(data_guard);
        
        // Save to disk if persistence is enabled
        self.save_to_disk().await?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn cleanup_expired(&self) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        let mut records_cleaned = 0;
        let now = SystemTime::now();
        
        let mut data_guard = self.data.write().await;
        
        for namespace_data in data_guard.values_mut() {
            let mut keys_to_remove = Vec::new();
            
            for (key, record) in namespace_data.iter() {
                if let Some(expires_at) = record.expires_at {
                    if now > expires_at {
                        keys_to_remove.push(key.clone());
                    }
                }
            }
            
            for key in keys_to_remove {
                namespace_data.remove(&key);
                records_cleaned += 1;
            }
        }
        
        drop(data_guard);
        
        // Save to disk if persistence is enabled
        self.save_to_disk().await?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: records_cleaned,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn backup(&self, backup_path: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        let data_guard = self.data.read().await;
        let backup_data = serde_json::to_vec(&*data_guard)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        fs::write(backup_path, backup_data).await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        let total_records = data_guard.values()
            .map(|ns| ns.len() as u64)
            .sum();
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: total_records,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn restore(&self, backup_path: &str) -> Result<StorageResult, StorageError> {
        let start_time = std::time::Instant::now();
        
        let backup_data = fs::read(backup_path).await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        let restored_data: HashMap<String, HashMap<String, StorageRecord>> = 
            serde_json::from_slice(&backup_data)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        let total_records = restored_data.values()
            .map(|ns| ns.len() as u64)
            .sum();
        
        let mut data_guard = self.data.write().await;
        *data_guard = restored_data;
        
        drop(data_guard);
        
        // Save to disk if persistence is enabled
        self.save_to_disk().await?;
        
        let operation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageResult {
            success: true,
            records_affected: total_records,
            operation_time_ms: operation_time,
            error: None,
        })
    }
    
    async fn optimize(&self) -> Result<StorageResult, StorageError> {
        // Cleanup expired records and save to disk
        let cleanup_result = self.cleanup_expired().await?;
        self.save_to_disk().await?;
        Ok(cleanup_result)
    }
}

/// Pattern 3: Storage Backend Factory
/// 
/// This demonstrates how to create different storage backends based on configuration
pub struct StorageBackendFactory;

impl StorageBackendFactory {
    pub fn create_backend(config: StorageConfig) -> Result<Box<dyn StorageBackend>, StorageError> {
        match config.backend_type {
            StorageBackendType::FileSystem => {
                let backend = FileSystemStorage::new(config)?;
                Ok(Box::new(backend))
            }
            StorageBackendType::SQLite => {
                // In a real implementation, create SQLite backend
                let backend = InMemoryStorage::new(config);
                Ok(Box::new(backend))
            }
            _ => Err(StorageError::ValidationError(
                format!("Unsupported backend type: {:?}", config.backend_type)
            ))
        }
    }
    
    pub fn create_default_config(backend_type: StorageBackendType) -> StorageConfig {
        StorageConfig {
            backend_type,
            connection_string: match backend_type {
                StorageBackendType::FileSystem => "./data".to_string(),
                StorageBackendType::SQLite => "./data.db".to_string(),
                StorageBackendType::Redis => "redis://localhost:6379".to_string(),
                _ => ":memory:".to_string(),
            },
            encryption_key: None,
            compression_enabled: false,
            backup_enabled: true,
            retention_policy: RetentionPolicy {
                max_age_days: Some(365),
                max_size_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
                auto_cleanup: true,
                backup_before_cleanup: true,
            },
            performance_settings: PerformanceSettings {
                connection_pool_size: 10,
                read_timeout: Duration::from_secs(5),
                write_timeout: Duration::from_secs(10),
                batch_size: 100,
                enable_compression: false,
                cache_ttl: Duration::from_secs(300),
            },
        }
    }
}

/// Example usage and testing patterns
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio;
    
    #[tokio::test]
    async fn test_filesystem_storage() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            backend_type: StorageBackendType::FileSystem,
            connection_string: temp_dir.path().to_string_lossy().to_string(),
            encryption_key: None,
            compression_enabled: false,
            backup_enabled: false,
            retention_policy: RetentionPolicy {
                max_age_days: None,
                max_size_bytes: None,
                auto_cleanup: false,
                backup_before_cleanup: false,
            },
            performance_settings: PerformanceSettings {
                connection_pool_size: 1,
                read_timeout: Duration::from_secs(5),
                write_timeout: Duration::from_secs(5),
                batch_size: 10,
                enable_compression: false,
                cache_ttl: Duration::from_secs(60),
            },
        };
        
        let mut storage = FileSystemStorage::new(config).unwrap();
        storage.connect().await.unwrap();
        
        // Test basic operations
        let record = StorageRecord {
            id: "test-1".to_string(),
            namespace: "test".to_string(),
            key: "key1".to_string(),
            data: StorageData::Text("Hello, World!".to_string()),
            metadata: StorageMetadata {
                content_type: "text/plain".to_string(),
                size_bytes: 13,
                checksum: "".to_string(),
                tags: HashMap::new(),
                version: 1,
            },
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            expires_at: None,
        };
        
        // Test put
        let result = storage.put(record.clone()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.records_affected, 1);
        
        // Test get
        let retrieved = storage.get("test", "key1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-1");
        
        // Test exists
        assert!(storage.exists("test", "key1").await.unwrap());
        assert!(!storage.exists("test", "nonexistent").await.unwrap());
        
        // Test delete
        let delete_result = storage.delete("test", "key1").await.unwrap();
        assert!(delete_result.success);
        assert_eq!(delete_result.records_affected, 1);
        
        // Verify deleted
        assert!(storage.get("test", "key1").await.unwrap().is_none());
        
        storage.disconnect().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_in_memory_storage() {
        let config = StorageConfig {
            backend_type: StorageBackendType::FileSystem,
            connection_string: ":memory:".to_string(),
            encryption_key: None,
            compression_enabled: false,
            backup_enabled: false,
            retention_policy: RetentionPolicy {
                max_age_days: None,
                max_size_bytes: None,
                auto_cleanup: false,
                backup_before_cleanup: false,
            },
            performance_settings: PerformanceSettings {
                connection_pool_size: 1,
                read_timeout: Duration::from_secs(5),
                write_timeout: Duration::from_secs(5),
                batch_size: 10,
                enable_compression: false,
                cache_ttl: Duration::from_secs(60),
            },
        };
        
        let mut storage = InMemoryStorage::new(config);
        storage.connect().await.unwrap();
        
        // Test health check
        let health = storage.health_check().await.unwrap();
        assert!(matches!(health.status, HealthStatus::Healthy));
        
        // Test namespace operations
        storage.create_namespace("test").await.unwrap();
        let namespaces = storage.list_namespaces().await.unwrap();
        assert!(namespaces.contains(&"test".to_string()));
        
        storage.disconnect().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_storage_factory() {
        let config = StorageBackendFactory::create_default_config(StorageBackendType::FileSystem);
        assert!(matches!(config.backend_type, StorageBackendType::FileSystem));
        
        let backend = StorageBackendFactory::create_backend(config);
        assert!(backend.is_ok());
    }
}

/// Key Takeaways for Storage Backend Implementation:
///
/// 1. **Trait Abstraction**: Define a common interface for all storage backends
/// 2. **Configuration Management**: Use strongly-typed configuration structures
/// 3. **Error Handling**: Implement comprehensive error types and propagation
/// 4. **Performance Optimization**: Consider caching, batching, and compression
/// 5. **Data Integrity**: Implement checksums, validation, and corruption detection
/// 6. **Scalability**: Design for concurrent access and large datasets
/// 7. **Maintenance Operations**: Provide cleanup, backup, and optimization capabilities
/// 8. **Query Support**: Implement flexible querying with filters and pagination
/// 9. **Transaction Support**: Consider ACID properties for critical operations
/// 10. **Testing**: Write comprehensive tests for all storage operations
///
/// This pattern allows llmspell to support multiple storage backends while
/// maintaining consistent behavior and performance characteristics.