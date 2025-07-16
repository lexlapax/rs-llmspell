// ABOUTME: Memory usage tracking for preventing resource exhaustion attacks
// ABOUTME: Monitors memory allocations during expression evaluation

//! Memory tracking utilities for `DoS` prevention
//!
//! This module provides utilities to track and limit memory usage
//! during expression evaluation to prevent resource exhaustion attacks.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Memory tracker for monitoring allocations
#[derive(Debug, Clone)]
pub struct MemoryTracker {
    /// Current memory usage in bytes
    current_usage: Arc<AtomicUsize>,
    /// Maximum allowed memory in bytes
    max_memory: usize,
}

impl MemoryTracker {
    /// Create a new memory tracker
    #[must_use]
    pub fn new(max_memory: usize) -> Self {
        Self {
            current_usage: Arc::new(AtomicUsize::new(0)),
            max_memory,
        }
    }

    /// Track memory allocation
    ///
    /// # Errors
    ///
    /// Returns an error if the allocation would exceed the memory limit
    pub fn allocate(&self, bytes: usize) -> Result<(), String> {
        let current = self.current_usage.fetch_add(bytes, Ordering::SeqCst);
        let new_usage = current + bytes;

        if new_usage > self.max_memory {
            // Rollback the allocation
            self.current_usage.fetch_sub(bytes, Ordering::SeqCst);
            return Err(format!(
                "Memory limit exceeded: {} + {} > {} bytes",
                current, bytes, self.max_memory
            ));
        }

        Ok(())
    }

    /// Track memory deallocation
    pub fn deallocate(&self, bytes: usize) {
        self.current_usage.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Get current memory usage
    #[must_use]
    pub fn current_usage(&self) -> usize {
        self.current_usage.load(Ordering::SeqCst)
    }

    /// Reset memory tracking
    pub fn reset(&self) {
        self.current_usage.store(0, Ordering::SeqCst);
    }

    /// Check if allocation would exceed limit
    #[must_use]
    pub fn would_exceed(&self, bytes: usize) -> bool {
        let current = self.current_usage.load(Ordering::SeqCst);
        current + bytes > self.max_memory
    }
}

/// Guard for automatic memory deallocation
pub struct MemoryGuard {
    tracker: MemoryTracker,
    bytes: usize,
}

impl MemoryGuard {
    /// Create a new memory guard
    ///
    /// # Errors
    ///
    /// Returns an error if the allocation fails
    pub fn new(tracker: MemoryTracker, bytes: usize) -> Result<Self, String> {
        tracker.allocate(bytes)?;
        Ok(Self { tracker, bytes })
    }
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        self.tracker.deallocate(self.bytes);
    }
}

/// Scoped memory tracking
pub struct ScopedMemoryTracker {
    parent: Option<MemoryTracker>,
    local_tracker: MemoryTracker,
}

impl ScopedMemoryTracker {
    /// Create a new scoped tracker
    #[must_use]
    pub fn new(max_memory: usize) -> Self {
        Self {
            parent: None,
            local_tracker: MemoryTracker::new(max_memory),
        }
    }

    /// Create a child scope with its own limit
    #[must_use]
    pub fn child(&self, max_memory: usize) -> Self {
        Self {
            parent: Some(self.local_tracker.clone()),
            local_tracker: MemoryTracker::new(max_memory),
        }
    }

    /// Allocate memory in this scope
    ///
    /// # Errors
    ///
    /// Returns an error if the allocation would exceed the memory limit in this scope or parent scope
    pub fn allocate(&self, bytes: usize) -> Result<MemoryGuard, String> {
        // Check parent scope first if it exists
        if let Some(ref parent) = self.parent {
            if parent.would_exceed(bytes) {
                return Err("Parent scope memory limit would be exceeded".to_string());
            }
        }

        MemoryGuard::new(self.local_tracker.clone(), bytes)
    }

    /// Get current usage in this scope
    #[must_use]
    pub fn current_usage(&self) -> usize {
        self.local_tracker.current_usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracking() {
        let tracker = MemoryTracker::new(1000);

        // Test allocation
        assert!(tracker.allocate(500).is_ok());
        assert_eq!(tracker.current_usage(), 500);

        // Test deallocation
        tracker.deallocate(200);
        assert_eq!(tracker.current_usage(), 300);

        // Test exceeding limit
        assert!(tracker.allocate(800).is_err());
        assert_eq!(tracker.current_usage(), 300); // Should not change

        // Test reset
        tracker.reset();
        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_memory_guard() {
        let tracker = MemoryTracker::new(1000);

        {
            let _guard = MemoryGuard::new(tracker.clone(), 500).unwrap();
            assert_eq!(tracker.current_usage(), 500);
        }

        // Guard should deallocate on drop
        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_scoped_tracking() {
        let root = ScopedMemoryTracker::new(1000);
        let child = root.child(500);

        let _guard1 = root.allocate(400).unwrap();
        assert_eq!(root.current_usage(), 400);

        let _guard2 = child.allocate(300).unwrap();
        assert_eq!(child.current_usage(), 300);

        // Child should not affect parent's local usage
        assert_eq!(root.current_usage(), 400);
    }
}
