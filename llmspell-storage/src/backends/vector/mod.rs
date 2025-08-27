//! Vector storage backend implementations
//!
//! This module provides vector storage capabilities including:
//! - HNSW (Hierarchical Navigable Small World) indexing
//! - Dimension-aware routing for multi-provider support
//! - Metadata indexing for efficient filtering

pub mod dimension_router;
pub mod hnsw;
#[cfg(feature = "hnsw-real")]
pub mod hnsw_real;
pub mod metadata_index;

pub use dimension_router::{DimensionInfo, DimensionRouter};
pub use hnsw::HNSWVectorStorage;
#[cfg(feature = "hnsw-real")]
pub use hnsw_real::RealHNSWVectorStorage;
pub use metadata_index::{IndexType, MetadataIndex, MetadataQueryOptimizer, QueryStats};
