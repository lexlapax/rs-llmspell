//! Vector storage backend implementations
//!
//! This module provides vector storage capabilities including:
//! - Metadata indexing for efficient filtering

pub mod metadata_index;

pub use metadata_index::{IndexType, MetadataIndex, MetadataQueryOptimizer, QueryStats};
