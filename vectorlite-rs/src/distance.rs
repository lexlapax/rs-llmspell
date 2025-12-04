//! Distance metrics for vector similarity

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported distance metrics for vector search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DistanceMetric {
    /// Euclidean distance (L2 norm)
    /// d(a, b) = sqrt(sum((a_i - b_i)^2))
    L2,

    /// Cosine distance (1 - cosine similarity)
    /// d(a, b) = 1 - (dot(a, b) / (||a|| * ||b||))
    #[default]
    Cosine,

    /// Negative inner product (for maximum inner product search)
    /// d(a, b) = -dot(a, b)
    InnerProduct,
}

impl FromStr for DistanceMetric {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "l2" | "euclidean" => Ok(Self::L2),
            "cosine" => Ok(Self::Cosine),
            "ip" | "inner_product" | "dot" => Ok(Self::InnerProduct),
            _ => Err(format!("Invalid distance metric: {s}")),
        }
    }
}

/// Calculate L2 (Euclidean) distance between two vectors
///
/// # Panics
///
/// Panics if vectors have different dimensions
pub fn distance_l2(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same dimension");
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

/// Calculate cosine distance between two vectors
///
/// Cosine distance = 1 - cosine similarity
/// Range: [0, 2] where 0 = identical direction, 2 = opposite direction
///
/// # Panics
///
/// Panics if vectors have different dimensions
pub fn distance_cosine(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // Maximum distance for zero vectors
    }

    let cosine_sim = dot_product / (norm_a * norm_b);
    1.0 - cosine_sim
}

/// Calculate negative inner product (for maximum inner product search)
///
/// Returns -dot(a, b) so that nearest neighbors have smallest distance
///
/// # Panics
///
/// Panics if vectors have different dimensions
pub fn distance_inner_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same dimension");
    -a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let dist = distance_l2(&a, &b);
        assert!((dist - 1.414).abs() < 0.01);

        // Same vector
        let dist = distance_l2(&a, &a);
        assert!(dist < 0.001);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let dist = distance_cosine(&a, &b);
        assert!((dist - 1.0).abs() < 0.01); // Orthogonal vectors

        // Same vector
        let dist = distance_cosine(&a, &a);
        assert!(dist < 0.001);

        // Opposite vectors
        let c = vec![-1.0, 0.0, 0.0];
        let dist = distance_cosine(&a, &c);
        assert!((dist - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_inner_product_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        // dot(a, b) = 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        let dist = distance_inner_product(&a, &b);
        assert!((dist + 32.0).abs() < 0.001);
    }

    #[test]
    fn test_metric_parsing() {
        assert_eq!(DistanceMetric::from_str("l2"), Ok(DistanceMetric::L2));
        assert_eq!(
            DistanceMetric::from_str("euclidean"),
            Ok(DistanceMetric::L2)
        );
        assert_eq!(
            DistanceMetric::from_str("cosine"),
            Ok(DistanceMetric::Cosine)
        );
        assert_eq!(
            DistanceMetric::from_str("ip"),
            Ok(DistanceMetric::InnerProduct)
        );
        assert_eq!(
            DistanceMetric::from_str("dot"),
            Ok(DistanceMetric::InnerProduct)
        );
        assert!(DistanceMetric::from_str("invalid").is_err());
    }

    #[test]
    fn test_default_metric() {
        let metric = DistanceMetric::default();
        assert_eq!(metric, DistanceMetric::Cosine);
    }
}
