//! Storage backend implementations

use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use super::types::{DebugState, ExecutionState, SessionState};

/// Storage backend enum that provides multiple persistence options
#[derive(Clone)]
pub enum StorageBackend {
    /// In-memory storage (no persistence)
    Memory(Box<MemoryBackend>),
    /// Sled embedded database storage
    Sled(SledBackend),
    /// Vector storage backend
    Vector(VectorBackend),
}

impl fmt::Debug for StorageBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory(_) => write!(f, "Memory"),
            Self::Sled(_) => write!(f, "Sled"),
            Self::Vector(_) => write!(f, "Vector"),
        }
    }
}

impl StorageBackend {
    /// Store execution state
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails
    pub fn store_execution(&mut self, state: &ExecutionState) -> Result<()> {
        match self {
            Self::Memory(backend) => {
                backend.store_execution(state);
                Ok(())
            }
            Self::Sled(backend) => backend.store_execution(state),
            Self::Vector(backend) => backend.store_execution(state),
        }
    }

    /// Load execution state
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails
    pub fn load_execution(&self) -> Result<Option<ExecutionState>> {
        match self {
            Self::Memory(backend) => Ok(backend.load_execution()),
            Self::Sled(backend) => backend.load_execution(),
            Self::Vector(backend) => backend.load_execution(),
        }
    }

    /// Store session state
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails
    pub fn store_session(&mut self, state: &SessionState) -> Result<()> {
        match self {
            Self::Memory(backend) => {
                backend.store_session(state);
                Ok(())
            }
            Self::Sled(backend) => backend.store_session(state),
            Self::Vector(backend) => backend.store_session(state),
        }
    }

    /// Load session state
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails
    pub fn load_session(&self) -> Result<Option<SessionState>> {
        match self {
            Self::Memory(backend) => Ok(backend.load_session()),
            Self::Sled(backend) => backend.load_session(),
            Self::Vector(backend) => backend.load_session(),
        }
    }

    /// Store debug state
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails
    pub fn store_debug(&mut self, state: &DebugState) -> Result<()> {
        match self {
            Self::Memory(backend) => {
                backend.store_debug(state);
                Ok(())
            }
            Self::Sled(backend) => backend.store_debug(state),
            Self::Vector(backend) => backend.store_debug(state),
        }
    }

    /// Load debug state
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails
    pub fn load_debug(&self) -> Result<Option<DebugState>> {
        match self {
            Self::Memory(backend) => Ok(backend.load_debug()),
            Self::Sled(backend) => backend.load_debug(),
            Self::Vector(backend) => backend.load_debug(),
        }
    }

    /// Clear all stored state
    ///
    /// # Errors
    ///
    /// Returns an error if clearing fails
    pub fn clear(&mut self) -> Result<()> {
        match self {
            Self::Memory(backend) => {
                backend.clear();
                Ok(())
            }
            Self::Sled(backend) => backend.clear(),
            Self::Vector(backend) => {
                backend.clear();
                Ok(())
            }
        }
    }
}

/// In-memory storage backend (no persistence)
#[derive(Clone, Default)]
pub struct MemoryBackend {
    execution: Option<ExecutionState>,
    session: Option<SessionState>,
    debug: Option<DebugState>,
}

impl MemoryBackend {
    /// Create a new memory backend
    pub fn new() -> Self {
        Self::default()
    }

    fn store_execution(&mut self, state: &ExecutionState) {
        self.execution = Some(state.clone());
    }

    fn load_execution(&self) -> Option<ExecutionState> {
        self.execution.clone()
    }

    fn store_session(&mut self, state: &SessionState) {
        self.session = Some(state.clone());
    }

    fn load_session(&self) -> Option<SessionState> {
        self.session.clone()
    }

    fn store_debug(&mut self, state: &DebugState) {
        self.debug = Some(state.clone());
    }

    fn load_debug(&self) -> Option<DebugState> {
        self.debug.clone()
    }

    fn clear(&mut self) {
        self.execution = None;
        self.session = None;
        self.debug = None;
    }
}

/// Sled embedded database backend for persistent storage
#[derive(Clone)]
pub struct SledBackend {
    db: sled::Db,
}

impl SledBackend {
    /// Create a new Sled backend
    ///
    /// # Errors
    ///
    /// Returns an error if database creation fails
    pub fn new(path: PathBuf) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// Create a temporary Sled backend (for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if database creation fails
    pub fn temporary() -> Result<Self> {
        let db = sled::Config::new().temporary(true).open()?;
        Ok(Self { db })
    }

    fn store_execution(&mut self, state: &ExecutionState) -> Result<()> {
        let serialized = serde_json::to_vec(state)?;
        self.db.insert(b"execution_state", serialized)?;
        self.db.flush()?;
        Ok(())
    }

    fn load_execution(&self) -> Result<Option<ExecutionState>> {
        if let Some(data) = self.db.get(b"execution_state")? {
            let state = serde_json::from_slice(&data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    fn store_session(&mut self, state: &SessionState) -> Result<()> {
        let serialized = serde_json::to_vec(state)?;
        self.db.insert(b"session_state", serialized)?;
        self.db.flush()?;
        Ok(())
    }

    fn load_session(&self) -> Result<Option<SessionState>> {
        if let Some(data) = self.db.get(b"session_state")? {
            let state = serde_json::from_slice(&data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    fn store_debug(&mut self, state: &DebugState) -> Result<()> {
        let serialized = serde_json::to_vec(state)?;
        self.db.insert(b"debug_state", serialized)?;
        self.db.flush()?;
        Ok(())
    }

    fn load_debug(&self) -> Result<Option<DebugState>> {
        if let Some(data) = self.db.get(b"debug_state")? {
            let state = serde_json::from_slice(&data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    fn clear(&mut self) -> Result<()> {
        self.db.remove(b"execution_state")?;
        self.db.remove(b"session_state")?;
        self.db.remove(b"debug_state")?;
        self.db.flush()?;
        Ok(())
    }
}

/// Vector storage backend for advanced querying
#[derive(Clone)]
pub struct VectorBackend {
    /// Storage for state vectors
    vectors: HashMap<String, Vec<f32>>,
    /// Storage for state data
    data: HashMap<String, Vec<u8>>,
}

impl VectorBackend {
    /// Create a new vector backend
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            data: HashMap::new(),
        }
    }

    fn store_execution(&mut self, state: &ExecutionState) -> Result<()> {
        let serialized = serde_json::to_vec(state)?;
        self.data.insert("execution_state".to_string(), serialized);

        // Generate embedding vector (simplified - in real implementation would use embedding model)
        let vector = Self::generate_embedding(state);
        self.vectors.insert("execution_state".to_string(), vector);

        Ok(())
    }

    fn load_execution(&self) -> Result<Option<ExecutionState>> {
        if let Some(data) = self.data.get("execution_state") {
            let state = serde_json::from_slice(data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    fn store_session(&mut self, state: &SessionState) -> Result<()> {
        let serialized = serde_json::to_vec(state)?;
        self.data.insert("session_state".to_string(), serialized);

        // Generate embedding vector
        let vector = Self::generate_session_embedding(state);
        self.vectors.insert("session_state".to_string(), vector);

        Ok(())
    }

    fn load_session(&self) -> Result<Option<SessionState>> {
        if let Some(data) = self.data.get("session_state") {
            let state = serde_json::from_slice(data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    fn store_debug(&mut self, state: &DebugState) -> Result<()> {
        let serialized = serde_json::to_vec(state)?;
        self.data.insert("debug_state".to_string(), serialized);

        // Generate embedding vector
        let vector = Self::generate_debug_embedding(state);
        self.vectors.insert("debug_state".to_string(), vector);

        Ok(())
    }

    fn load_debug(&self) -> Result<Option<DebugState>> {
        if let Some(data) = self.data.get("debug_state") {
            let state = serde_json::from_slice(data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    fn clear(&mut self) {
        self.data.clear();
        self.vectors.clear();
    }

    /// Generate embedding for execution state (simplified)
    #[allow(clippy::cast_precision_loss)]
    fn generate_embedding(state: &ExecutionState) -> Vec<f32> {
        // This is a simplified embedding - in real implementation would use a proper embedding model
        vec![
            state.execution_count as f32,
            if state.status == super::types::ExecutionStatus::Running {
                1.0
            } else {
                0.0
            },
            state.variables.len() as f32,
            state.history.len() as f32,
            state.total_execution_time.as_secs_f32(),
        ]
    }

    /// Generate embedding for session state (simplified)
    #[allow(clippy::cast_precision_loss)]
    fn generate_session_embedding(state: &SessionState) -> Vec<f32> {
        vec![
            if state.session_id.is_some() { 1.0 } else { 0.0 },
            state.breakpoints.len() as f32,
            state.artifacts.len() as f32,
            state.resources.memory_bytes as f32 / 1_000_000.0, // MB
            state.resources.api_calls as f32,
        ]
    }

    /// Generate embedding for debug state (simplified)
    #[allow(clippy::cast_precision_loss)]
    fn generate_debug_embedding(state: &DebugState) -> Vec<f32> {
        vec![
            if state.enabled { 1.0 } else { 0.0 },
            state.stack_frames.len() as f32,
            state.variables.len() as f32,
            state.watches.len() as f32,
            state.step_count as f32,
        ]
    }
}

impl Default for VectorBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_backend() {
        let mut backend = MemoryBackend::new();
        let mut exec_state = ExecutionState::default();
        exec_state.increment_counter();

        backend.store_execution(&exec_state);
        let loaded = backend.load_execution().unwrap();
        assert_eq!(loaded.execution_count, 1);
    }

    #[test]
    fn test_sled_backend() {
        let mut backend = SledBackend::temporary().unwrap();
        let mut session_state = SessionState::default();
        session_state.set_id("test-session");

        backend.store_session(&session_state).unwrap();
        let loaded = backend.load_session().unwrap().unwrap();
        assert_eq!(loaded.session_id, Some("test-session".to_string()));
    }

    #[test]
    fn test_vector_backend() {
        let mut backend = VectorBackend::new();
        let debug_state = DebugState::default();

        backend.store_debug(&debug_state).unwrap();
        let loaded = backend.load_debug().unwrap().unwrap();
        assert!(!loaded.enabled);
    }
}
