//! Dynamic dimension handling for multiple embedding models

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for dimension handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionConfig {
    /// Default dimensions when not specified
    pub default_dimensions: usize,

    /// Supported dimension values for dynamic models
    pub supported_dimensions: Vec<usize>,

    /// Whether to allow dimension reduction (Matryoshka)
    pub allow_reduction: bool,

    /// Whether to allow dimension expansion (padding)
    pub allow_expansion: bool,

    /// Maximum dimensions to support
    pub max_dimensions: usize,
}

impl Default for DimensionConfig {
    fn default() -> Self {
        Self {
            default_dimensions: 1536,
            supported_dimensions: vec![256, 384, 512, 768, 1024, 1536, 2048, 3072, 4096],
            allow_reduction: true,
            allow_expansion: false,
            max_dimensions: 4096,
        }
    }
}

/// Maps embeddings between different dimensions
#[derive(Debug)]
pub struct DimensionMapper {
    config: DimensionConfig,
    dimension_indices: HashMap<usize, usize>,
}

impl DimensionMapper {
    /// Create a new dimension mapper
    #[must_use]
    pub fn new(config: DimensionConfig) -> Self {
        let mut dimension_indices = HashMap::new();
        for (idx, &dim) in config.supported_dimensions.iter().enumerate() {
            dimension_indices.insert(dim, idx);
        }

        Self {
            config,
            dimension_indices,
        }
    }

    /// Check if dimensions are compatible for storage together
    #[must_use]
    pub const fn are_compatible(&self, dims1: usize, dims2: usize) -> bool {
        dims1 == dims2
            || (self.config.allow_reduction && dims1 > dims2 && dims1 % dims2 == 0)
            || (self.config.allow_expansion && dims2 > dims1 && dims2 % dims1 == 0)
    }

    /// Reduce dimensions using Matryoshka representation learning
    ///
    /// This truncates the vector to the first `target_dims` dimensions,
    /// which works for models trained with Matryoshka representation learning.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension reduction is not allowed
    /// - The vector is smaller than target dimensions
    pub fn reduce_dimensions(&self, vector: &[f32], target_dims: usize) -> Result<Vec<f32>> {
        if !self.config.allow_reduction {
            anyhow::bail!("Dimension reduction not allowed");
        }

        if vector.len() < target_dims {
            anyhow::bail!(
                "Cannot reduce {} dimensions to {} (vector too small)",
                vector.len(),
                target_dims
            );
        }

        Ok(vector[..target_dims].to_vec())
    }

    /// Expand dimensions by padding with zeros
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension expansion is not allowed
    /// - The vector is larger than target dimensions
    pub fn expand_dimensions(&self, vector: &[f32], target_dims: usize) -> Result<Vec<f32>> {
        if !self.config.allow_expansion {
            anyhow::bail!("Dimension expansion not allowed");
        }

        if vector.len() > target_dims {
            anyhow::bail!(
                "Cannot expand {} dimensions to {} (vector too large)",
                vector.len(),
                target_dims
            );
        }

        let mut expanded = vector.to_vec();
        expanded.resize(target_dims, 0.0);
        Ok(expanded)
    }

    /// Normalize vector dimensions to a supported size
    ///
    /// # Errors
    ///
    /// Returns an error if no compatible dimension can be found
    pub fn normalize_dimensions(&self, vector: &[f32]) -> Result<Vec<f32>> {
        let current_dims = vector.len();

        // Check if already a supported dimension
        if self.dimension_indices.contains_key(&current_dims) {
            return Ok(vector.to_vec());
        }

        // Find closest supported dimension
        let target_dims = self.find_closest_dimension(current_dims)?;

        if target_dims < current_dims {
            self.reduce_dimensions(vector, target_dims)
        } else {
            self.expand_dimensions(vector, target_dims)
        }
    }

    /// Find the closest supported dimension
    ///
    /// # Errors
    ///
    /// Returns an error if no compatible dimension is found
    fn find_closest_dimension(&self, dims: usize) -> Result<usize> {
        // Try reduction first (preferred)
        if self.config.allow_reduction {
            for &supported_dim in self.config.supported_dimensions.iter().rev() {
                if supported_dim <= dims && (dims % supported_dim == 0 || supported_dim == dims) {
                    return Ok(supported_dim);
                }
            }
        }

        // Try expansion if allowed
        if self.config.allow_expansion {
            for &supported_dim in &self.config.supported_dimensions {
                if supported_dim >= dims {
                    return Ok(supported_dim);
                }
            }
        }

        anyhow::bail!(
            "No compatible dimension found for {} (supported: {:?})",
            dims,
            self.config.supported_dimensions
        )
    }

    /// Get dimension bucket index for routing
    #[must_use]
    pub fn get_dimension_bucket(&self, dims: usize) -> Option<usize> {
        self.dimension_indices.get(&dims).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_compatibility() {
        let config = DimensionConfig::default();
        let mapper = DimensionMapper::new(config);

        assert!(mapper.are_compatible(1536, 1536));
        assert!(mapper.are_compatible(1536, 768));
        assert!(!mapper.are_compatible(1536, 1000));
    }

    #[test]
    fn test_dimension_reduction() {
        let config = DimensionConfig::default();
        let mapper = DimensionMapper::new(config);

        let vector = vec![1.0; 1536];
        let reduced = mapper.reduce_dimensions(&vector, 768).unwrap();
        assert_eq!(reduced.len(), 768);
        assert!((reduced[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dimension_expansion() {
        let config = DimensionConfig {
            allow_expansion: true,
            ..Default::default()
        };
        let mapper = DimensionMapper::new(config);

        let vector = vec![1.0; 768];
        let expanded = mapper.expand_dimensions(&vector, 1536).unwrap();
        assert_eq!(expanded.len(), 1536);
        assert!((expanded[0] - 1.0).abs() < f32::EPSILON);
        assert!((expanded[768]).abs() < f32::EPSILON);
    }
}
