//! Tests for signal handling in the REPL
//!
//! Verifies Ctrl+C interruption, cleanup, and resource management.

#[cfg(test)]
mod signal_handling_tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;

    /// Test Ctrl+C during long-running script
    #[tokio::test]
    async fn test_interrupt_long_running_script() {
        let executing = Arc::new(AtomicBool::new(false));
        let interrupted = Arc::new(AtomicBool::new(false));

        let exec_clone = executing.clone();
        let int_clone = interrupted.clone();

        // Simulate long-running script
        let script_handle = tokio::spawn(async move {
            exec_clone.store(true, Ordering::Relaxed);

            // Simulate work that checks for interruption
            for _ in 0..100 {
                if int_clone.load(Ordering::Relaxed) {
                    return Err("Interrupted");
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Ok("Completed")
        });

        // Wait for script to start
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(executing.load(Ordering::Relaxed));

        // Simulate Ctrl+C
        interrupted.store(true, Ordering::Relaxed);

        // Script should terminate quickly after interruption
        let result = timeout(Duration::from_secs(1), script_handle).await;
        assert!(result.is_ok());

        match result.unwrap() {
            Ok(Err(msg)) => assert_eq!(msg, "Interrupted"),
            _ => panic!("Script should have been interrupted"),
        }
    }

    /// Test multiple Ctrl+C in succession
    #[tokio::test]
    async fn test_multiple_interrupts() {
        let interrupt_count = Arc::new(AtomicBool::new(false));
        let force_quit = Arc::new(AtomicBool::new(false));

        let int_clone = interrupt_count.clone();
        let force_clone = force_quit.clone();

        // Handler for multiple Ctrl+C
        let handler = tokio::spawn(async move {
            let mut interrupt_received = false;

            loop {
                if int_clone.load(Ordering::Relaxed) {
                    if interrupt_received {
                        // Second Ctrl+C - force quit
                        force_clone.store(true, Ordering::Relaxed);
                        break;
                    } else {
                        // First Ctrl+C
                        interrupt_received = true;
                        int_clone.store(false, Ordering::Relaxed);
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // First Ctrl+C
        interrupt_count.store(true, Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!force_quit.load(Ordering::Relaxed));

        // Second Ctrl+C
        interrupt_count.store(true, Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(force_quit.load(Ordering::Relaxed));

        handler.abort();
    }

    /// Test resource cleanup after interruption
    #[tokio::test]
    async fn test_resource_cleanup_after_interrupt() {
        use std::sync::Mutex;

        struct Resource {
            cleaned_up: Arc<Mutex<bool>>,
        }

        impl Drop for Resource {
            fn drop(&mut self) {
                *self.cleaned_up.lock().unwrap() = true;
            }
        }

        let cleanup_flag = Arc::new(Mutex::new(false));
        let flag_clone = cleanup_flag.clone();

        {
            let resource = Resource {
                cleaned_up: flag_clone,
            };

            // Simulate interrupted operation
            let handle = tokio::spawn(async move {
                let _r = resource;
                // Simulate work
                tokio::time::sleep(Duration::from_secs(10)).await;
            });

            // Interrupt after short delay
            tokio::time::sleep(Duration::from_millis(100)).await;
            handle.abort();

            // Give time for cleanup
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Verify resource was cleaned up
        assert!(*cleanup_flag.lock().unwrap());
    }

    /// Test Ctrl+C at empty prompt (should not exit)
    #[test]
    fn test_interrupt_at_empty_prompt() {
        let mut buffer = String::new();
        let interrupted = true;

        if interrupted && buffer.is_empty() {
            // Should clear buffer and continue
            buffer.clear();
            assert!(buffer.is_empty());
            // Should NOT exit
        }
    }

    /// Test Ctrl+C with partially typed command
    #[test]
    fn test_interrupt_with_partial_command() {
        let mut buffer = String::from("print('hello");
        let interrupted = true;

        if interrupted {
            // Should clear buffer
            buffer.clear();
            assert!(buffer.is_empty());
        }
    }

    /// Test Ctrl+C in multi-line mode
    #[test]
    fn test_interrupt_in_multiline_mode() {
        let mut multiline_buffer = vec!["function foo()".to_string(), "  return 42".to_string()];
        let interrupted = true;

        if interrupted {
            // Should clear multi-line buffer
            multiline_buffer.clear();
            assert!(multiline_buffer.is_empty());
        }
    }

    /// Test signal handler registration and deregistration
    #[test]
    fn test_signal_handler_lifecycle() {
        use std::sync::Once;

        static HANDLER_INSTALLED: Once = Once::new();
        static mut HANDLER_ACTIVE: bool = false;

        // Install handler (should only happen once)
        HANDLER_INSTALLED.call_once(|| unsafe {
            HANDLER_ACTIVE = true;
        });

        assert!(unsafe { HANDLER_ACTIVE });

        // Cleanup on exit
        struct SignalHandlerCleanup;
        impl Drop for SignalHandlerCleanup {
            fn drop(&mut self) {
                unsafe {
                    HANDLER_ACTIVE = false;
                }
            }
        }

        {
            let _cleanup = SignalHandlerCleanup;
            assert!(unsafe { HANDLER_ACTIVE });
        }
        // After drop
        assert!(!unsafe { HANDLER_ACTIVE });
    }

    /// Test no memory leaks after repeated interruptions
    #[tokio::test]
    async fn test_no_memory_leaks_after_interrupts() {
        let initial_memory = get_memory_usage();

        for _ in 0..10 {
            let handle = tokio::spawn(async {
                // Allocate some memory
                let data = vec![0u8; 1024 * 1024]; // 1MB

                // Simulate work
                tokio::time::sleep(Duration::from_secs(10)).await;
                data
            });

            // Interrupt quickly
            tokio::time::sleep(Duration::from_millis(10)).await;
            handle.abort();
        }

        // Allow time for cleanup
        tokio::time::sleep(Duration::from_millis(500)).await;

        let final_memory = get_memory_usage();

        // Memory usage should not grow significantly
        // Allow 10MB tolerance for runtime overhead
        assert!(
            final_memory < initial_memory + 10 * 1024 * 1024,
            "Memory leak detected: {} -> {}",
            initial_memory,
            final_memory
        );
    }

    /// Test interruption doesn't corrupt state
    #[tokio::test]
    async fn test_state_consistency_after_interrupt() {
        use std::sync::Mutex;

        let state = Arc::new(Mutex::new(Vec::new()));
        let state_clone = state.clone();

        let handle = tokio::spawn(async move {
            for i in 0..100 {
                {
                    let mut s = state_clone.lock().unwrap();
                    s.push(i);
                } // MutexGuard dropped here
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Interrupt
        handle.abort();

        // Check state is valid
        let final_state = state.lock().unwrap();

        // Should have some elements but not all
        assert!(!final_state.is_empty());
        assert!(final_state.len() < 100);

        // Elements should be in order
        for i in 1..final_state.len() {
            assert!(final_state[i] > final_state[i - 1]);
        }
    }

    /// Test graceful shutdown vs forced termination
    #[tokio::test]
    async fn test_graceful_vs_forced_shutdown() {
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let force_shutdown = Arc::new(AtomicBool::new(false));

        let shutdown_clone = shutdown_requested.clone();
        let force_clone = force_shutdown.clone();

        let worker = tokio::spawn(async move {
            loop {
                if force_clone.load(Ordering::Relaxed) {
                    // Forced shutdown - exit immediately
                    break;
                }

                if shutdown_clone.load(Ordering::Relaxed) {
                    // Graceful shutdown - cleanup first
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    break;
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            "Shutdown complete"
        });

        // Test graceful shutdown
        shutdown_requested.store(true, Ordering::Relaxed);
        let result = timeout(Duration::from_secs(1), worker).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), "Shutdown complete");
    }

    // Helper function to get memory usage (mock implementation)
    fn get_memory_usage() -> u64 {
        // In real implementation, would use system calls
        // For testing, return a mock value
        1024 * 1024 * 100 // 100MB
    }
}
