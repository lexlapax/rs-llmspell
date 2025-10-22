//! Memory manager trait (to be implemented in Task 13.1.2)

use async_trait::async_trait;

use crate::error::Result;

/// Memory manager coordinates all memory subsystems
#[async_trait]
pub trait MemoryManager: Send + Sync {
    /// Access episodic memory
    fn episodic(&self) -> &dyn super::EpisodicMemory;

    /// Access semantic memory
    fn semantic(&self) -> &dyn super::SemanticMemory;

    /// Shutdown and cleanup resources
    async fn shutdown(&self) -> Result<()>;
}
