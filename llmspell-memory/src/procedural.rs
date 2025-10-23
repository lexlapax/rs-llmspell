//! Procedural memory implementations (learned patterns and skills)
//!
//! Placeholder for future implementation (Phase 13.6+)

use async_trait::async_trait;

use crate::error::Result;
use crate::traits::ProceduralMemory;

/// No-op procedural memory stub
///
/// Placeholder for future implementation.
/// Procedural memory will store learned patterns and skills.
pub struct NoopProceduralMemory;

#[async_trait]
impl ProceduralMemory for NoopProceduralMemory {
    async fn get_pattern(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn store_pattern(&self, _pattern_data: &str) -> Result<String> {
        Ok(String::new())
    }
}
