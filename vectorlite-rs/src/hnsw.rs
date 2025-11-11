//! HNSW index wrapper for vector search

use crate::distance::DistanceMetric;
use crate::error::{Error, Result};
use hnsw_rs::{
    hnsw::{Hnsw, Neighbour},
    prelude::{DistCosine, DistDot, DistL2},
};
use rusqlite::Result as SqliteResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Enum to hold different HNSW index types based on distance metric
enum HnswIndexType {
    Cosine(Hnsw<'static, f32, DistCosine>),
    L2(Hnsw<'static, f32, DistL2>),
    InnerProduct(Hnsw<'static, f32, DistDot>),
}

impl HnswIndexType {
    /// Search for k nearest neighbors
    fn search(&self, query: &[f32], k: usize, ef: usize) -> Vec<Neighbour> {
        match self {
            Self::Cosine(hnsw) => hnsw.search(query, k, ef),
            Self::L2(hnsw) => hnsw.search(query, k, ef),
            Self::InnerProduct(hnsw) => hnsw.search(query, k, ef),
        }
    }

    /// Insert vector into index
    fn insert(&self, vector: &[f32], point_id: usize) {
        match self {
            Self::Cosine(hnsw) => hnsw.insert((vector, point_id)),
            Self::L2(hnsw) => hnsw.insert((vector, point_id)),
            Self::InnerProduct(hnsw) => hnsw.insert((vector, point_id)),
        }
    }
}

/// Serializable persistence data for HNSW index
///
/// Contains all data needed to rebuild the index from scratch.
/// Does not serialize the HNSW graph itself (hnsw_rs doesn't support serde),
/// instead stores raw vectors and rebuilds the graph on load.
#[derive(Serialize, Deserialize)]
struct HnswPersistence {
    /// Vector dimension
    dimension: usize,
    /// Distance metric
    metric: DistanceMetric,
    /// Maximum number of elements
    max_elements: usize,
    /// HNSW parameters
    m: usize,
    ef_construction: usize,
    /// All vectors (rowid -> vector)
    vectors: HashMap<i64, Vec<f32>>,
}

/// HNSW index for approximate nearest neighbor search
///
/// This wrapper provides thread-safe access to hnsw_rs indices with
/// support for incremental inserts and K-NN queries.
///
/// # Persistence
///
/// The index can be serialized to MessagePack format for persistence.
/// Since hnsw_rs doesn't support serde, we store the raw vectors and
/// rebuild the HNSW graph on deserialization.
#[derive(Clone)]
pub struct HnswIndex {
    /// Vector dimension
    dimension: usize,
    /// Distance metric
    metric: DistanceMetric,
    /// Maximum number of elements
    max_elements: usize,
    /// HNSW index (thread-safe)
    index: Arc<RwLock<Option<HnswIndexType>>>,
    /// Vector storage (rowid -> vector)
    vectors: Arc<RwLock<HashMap<i64, Vec<f32>>>>,
    /// HNSW parameters
    m: usize,
    ef_construction: usize,
}

impl HnswIndex {
    /// Create new HNSW index
    ///
    /// # Parameters
    ///
    /// - `dimension`: Vector dimension (384, 768, 1536, 3072)
    /// - `max_elements`: Maximum number of vectors
    /// - `m`: Number of bi-directional links per node (default: 16)
    /// - `ef_construction`: Size of dynamic candidate list (default: 200)
    /// - `metric`: Distance metric
    ///
    /// # Errors
    ///
    /// Returns error if parameters are invalid
    pub fn new(
        dimension: usize,
        max_elements: usize,
        m: usize,
        ef_construction: usize,
        metric: DistanceMetric,
    ) -> SqliteResult<Self> {
        // Validate parameters
        if !matches!(dimension, 384 | 768 | 1536 | 3072) {
            return Err(rusqlite::Error::ModuleError(format!(
                "Invalid dimension {dimension}, must be 384/768/1536/3072"
            )));
        }

        if max_elements == 0 {
            return Err(rusqlite::Error::ModuleError(
                "max_elements must be > 0".to_string(),
            ));
        }

        if m == 0 {
            return Err(rusqlite::Error::ModuleError("m must be > 0".to_string()));
        }

        if ef_construction == 0 {
            return Err(rusqlite::Error::ModuleError(
                "ef_construction must be > 0".to_string(),
            ));
        }

        Ok(Self {
            dimension,
            metric,
            max_elements,
            index: Arc::new(RwLock::new(None)),
            vectors: Arc::new(RwLock::new(HashMap::new())),
            m,
            ef_construction,
        })
    }

    /// Initialize the HNSW index
    ///
    /// This must be called before any insertions. It creates the hnsw_rs index
    /// with the configured parameters.
    ///
    /// # Errors
    ///
    /// Returns error if index is already initialized or parameters are invalid
    fn initialize(&self) -> Result<()> {
        let mut index = self
            .index
            .write()
            .map_err(|e| Error::Hnsw(format!("Failed to acquire write lock: {e}")))?;

        if index.is_some() {
            return Err(Error::Hnsw("Index already initialized".to_string()));
        }

        // Calculate nb_layers based on max_elements
        let nb_layers = 16.min((self.max_elements as f32).ln() as usize).max(1);

        // Create hnsw_rs index based on metric
        let hnsw = match self.metric {
            DistanceMetric::Cosine => {
                let hnsw = Hnsw::new(
                    self.m,
                    self.max_elements,
                    nb_layers,
                    self.ef_construction,
                    DistCosine,
                );
                HnswIndexType::Cosine(hnsw)
            }
            DistanceMetric::L2 => {
                let hnsw = Hnsw::new(
                    self.m,
                    self.max_elements,
                    nb_layers,
                    self.ef_construction,
                    DistL2,
                );
                HnswIndexType::L2(hnsw)
            }
            DistanceMetric::InnerProduct => {
                let hnsw = Hnsw::new(
                    self.m,
                    self.max_elements,
                    nb_layers,
                    self.ef_construction,
                    DistDot,
                );
                HnswIndexType::InnerProduct(hnsw)
            }
        };

        *index = Some(hnsw);
        Ok(())
    }

    /// Insert vector into index
    ///
    /// # Parameters
    ///
    /// - `rowid`: SQLite row ID
    /// - `vector`: Vector to insert (must have correct dimension)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Vector dimension doesn't match
    /// - Index is not initialized
    /// - Insertion fails
    pub fn insert(&self, rowid: i64, vector: Vec<f32>) -> Result<()> {
        // Validate dimension
        if vector.len() != self.dimension {
            return Err(Error::InvalidDimension {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        // Initialize index if needed
        {
            let index = self
                .index
                .read()
                .map_err(|e| Error::Hnsw(format!("Failed to acquire read lock: {e}")))?;

            if index.is_none() {
                drop(index);
                self.initialize()?;
            }
        }

        // Store vector
        let mut vectors = self
            .vectors
            .write()
            .map_err(|e| Error::Hnsw(format!("Failed to acquire write lock: {e}")))?;
        vectors.insert(rowid, vector.clone());

        // Insert into HNSW index
        let index = self
            .index
            .read()
            .map_err(|e| Error::Hnsw(format!("Failed to acquire read lock: {e}")))?;

        if let Some(ref hnsw) = *index {
            // hnsw_rs insert API: insert(&vector, point_id)
            // Use rowid as point_id for mapping
            hnsw.insert(vector.as_slice(), rowid as usize);
        } else {
            return Err(Error::IndexNotInitialized);
        }

        Ok(())
    }

    /// Search for K nearest neighbors
    ///
    /// # Parameters
    ///
    /// - `query`: Query vector
    /// - `k`: Number of nearest neighbors to return
    /// - `ef_search`: Size of dynamic candidate list (higher = more accurate, slower)
    ///
    /// # Returns
    ///
    /// Vector of (rowid, distance) pairs sorted by distance (ascending)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Query dimension doesn't match
    /// - Index is not initialized
    /// - Search fails
    pub fn search(&self, query: &[f32], k: usize, ef_search: usize) -> Result<Vec<(i64, f32)>> {
        // Validate dimension
        if query.len() != self.dimension {
            return Err(Error::InvalidDimension {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        let index = self
            .index
            .read()
            .map_err(|e| Error::Hnsw(format!("Failed to acquire read lock: {e}")))?;

        let hnsw = index.as_ref().ok_or(Error::IndexNotInitialized)?;

        // hnsw_rs search API: search(query, k, ef_search)
        let neighbors = hnsw.search(query, k, ef_search);

        // Convert results: hnsw_rs returns Vec<Neighbour>
        // Neighbour has d_id: DataId (usize) and distance: f32
        let results: Vec<(i64, f32)> = neighbors
            .iter()
            .map(|n| (n.d_id as i64, n.distance))
            .collect();

        Ok(results)
    }

    /// Get vector by rowid
    ///
    /// # Errors
    ///
    /// Returns error if rowid not found
    pub fn get_vector(&self, rowid: i64) -> Result<Vec<f32>> {
        let vectors = self
            .vectors
            .read()
            .map_err(|e| Error::Hnsw(format!("Failed to acquire read lock: {e}")))?;

        vectors
            .get(&rowid)
            .cloned()
            .ok_or(Error::VectorNotFound(rowid))
    }

    /// Get number of vectors in index
    pub fn len(&self) -> usize {
        self.vectors.read().map(|v| v.len()).unwrap_or(0)
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Serialize index to MessagePack bytes
    ///
    /// Serializes the raw vectors and metadata. The HNSW graph is not
    /// serialized (hnsw_rs doesn't support it) and will be rebuilt on
    /// deserialization.
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        let vectors = self
            .vectors
            .read()
            .map_err(|e| Error::Hnsw(format!("Failed to acquire read lock: {e}")))?;

        let persistence = HnswPersistence {
            dimension: self.dimension,
            metric: self.metric,
            max_elements: self.max_elements,
            m: self.m,
            ef_construction: self.ef_construction,
            vectors: vectors.clone(),
        };

        rmp_serde::to_vec(&persistence)
            .map_err(|e| Error::Hnsw(format!("Failed to serialize index: {e}")))
    }

    /// Deserialize index from MessagePack bytes
    ///
    /// Deserializes the raw vectors and rebuilds the HNSW graph from scratch.
    ///
    /// # Errors
    ///
    /// Returns error if deserialization or index building fails
    pub fn from_msgpack(data: &[u8]) -> Result<Self> {
        let persistence: HnswPersistence = rmp_serde::from_slice(data)
            .map_err(|e| Error::Hnsw(format!("Failed to deserialize index: {e}")))?;

        // Create new index with deserialized parameters
        let index = Self::new(
            persistence.dimension,
            persistence.max_elements,
            persistence.m,
            persistence.ef_construction,
            persistence.metric,
        )?;

        // Insert all vectors to rebuild HNSW graph
        for (rowid, vector) in persistence.vectors {
            index.insert(rowid, vector)?;
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_create() {
        let index = HnswIndex::new(768, 1000, 16, 200, DistanceMetric::Cosine);
        assert!(index.is_ok());

        let index = index.unwrap();
        assert_eq!(index.dimension, 768);
        assert!(index.is_empty());
    }

    #[test]
    fn test_hnsw_invalid_dimension() {
        let index = HnswIndex::new(512, 1000, 16, 200, DistanceMetric::Cosine);
        assert!(index.is_err());
    }

    #[test]
    fn test_hnsw_insert_and_search() -> Result<()> {
        let index = HnswIndex::new(384, 1000, 16, 200, DistanceMetric::L2)?;

        // Insert test vectors
        let v1 = vec![1.0; 384];
        let v2 = vec![2.0; 384];
        let v3 = vec![3.0; 384];

        index.insert(1, v1)?;
        index.insert(2, v2)?;
        index.insert(3, v3)?;

        assert_eq!(index.len(), 3);

        // Search
        let query = vec![1.5; 384];
        let results = index.search(&query, 2, 100)?;

        assert_eq!(results.len(), 2);
        // Closest should be v2 (all 2.0), then v1 (all 1.0)
        assert_eq!(results[0].0, 2);

        Ok(())
    }

    #[test]
    fn test_hnsw_dimension_mismatch() -> Result<()> {
        let index = HnswIndex::new(384, 1000, 16, 200, DistanceMetric::L2)?;

        let wrong_dim = vec![1.0; 768];
        let result = index.insert(1, wrong_dim);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_hnsw_serialization() -> Result<()> {
        let index = HnswIndex::new(768, 1000, 16, 200, DistanceMetric::Cosine)?;

        // Insert test vectors
        let v1 = vec![1.0; 768];
        let v2 = vec![2.0; 768];
        let v3 = vec![3.0; 768];

        index.insert(1, v1)?;
        index.insert(2, v2)?;
        index.insert(3, v3)?;

        // Serialize
        let serialized = index.to_msgpack()?;
        assert!(!serialized.is_empty());

        // Deserialize
        let loaded = HnswIndex::from_msgpack(&serialized)?;
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded.dimension, 768);

        // Verify search works
        let query = vec![1.5; 768];
        let results = loaded.search(&query, 2, 100)?;
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[test]
    fn test_hnsw_get_vector() -> Result<()> {
        let index = HnswIndex::new(384, 1000, 16, 200, DistanceMetric::L2)?;

        let v1 = vec![1.0; 384];
        index.insert(1, v1.clone())?;

        let retrieved = index.get_vector(1)?;
        assert_eq!(retrieved, v1);

        let not_found = index.get_vector(999);
        assert!(not_found.is_err());

        Ok(())
    }
}
