//! ABOUTME: Simple port allocation utilities for kernel tests
//! ABOUTME: Provides unique port allocation to prevent test conflicts

//! Kernel test helpers for port allocation.
//!
//! Provides simple utilities to allocate unique ports for kernel tests
//! to prevent conflicts when running tests in parallel.

use anyhow::Result;
use std::sync::atomic::{AtomicU16, Ordering};

/// Simple atomic counter for unique kernel IDs
static KERNEL_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Simple atomic counter for port allocation
/// Kernels use 5 consecutive ports, so we jump by 10 to avoid conflicts
static PORT_COUNTER: AtomicU16 = AtomicU16::new(20000);

/// Allocate a unique port for kernel testing
///
/// Returns a base port. The kernel will use this port and the next 4 ports
/// (base, base+1, base+2, base+3, base+4) for its ZMQ channels.
pub async fn allocate_kernel_port() -> Result<u16> {
    let port = PORT_COUNTER.fetch_add(10, Ordering::SeqCst);
    if port > 30000 {
        // Reset to start if we go too high
        PORT_COUNTER.store(20000, Ordering::SeqCst);
        Ok(20000)
    } else {
        Ok(port)
    }
}

/// Generate a unique kernel ID for testing
pub fn generate_test_kernel_id() -> String {
    let counter = KERNEL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test-kernel-{}", counter)
}

/// Create a unique test configuration with allocated port
///
/// Returns a tuple of (kernel_id, port) that are guaranteed to be unique
/// for this test run.
pub async fn create_test_kernel_config() -> Result<(String, u16)> {
    let kernel_id = generate_test_kernel_id();
    let port = allocate_kernel_port().await?;
    Ok((kernel_id, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_port_allocation() {
        let port1 = allocate_kernel_port().await.unwrap();
        let port2 = allocate_kernel_port().await.unwrap();

        // Should get different ports with 10-port spacing
        assert!(port2 >= port1 + 10);

        // Ports should be in expected range
        assert!((20000..=30000).contains(&port1));
        assert!((20000..=30000).contains(&port2));
    }

    #[test]
    fn test_kernel_id_generation() {
        let id1 = generate_test_kernel_id();
        let id2 = generate_test_kernel_id();

        // Should get different IDs
        assert_ne!(id1, id2);

        // Should have expected format
        assert!(id1.starts_with("test-kernel-"));
        assert!(id2.starts_with("test-kernel-"));
    }
}
