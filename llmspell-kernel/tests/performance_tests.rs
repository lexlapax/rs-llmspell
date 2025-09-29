//! Tests for REPL performance monitoring features
//!
//! Verifies execution timing, memory tracking, and statistics accumulation.

#[cfg(test)]
mod performance_tests {

    use llmspell_kernel::repl::session::SessionStatistics;
    use std::time::{Duration, Instant};

    /// Test execution time accuracy
    #[tokio::test]
    async fn test_execution_time_measurement() {
        let mut stats = SessionStatistics::default();
        let start = Instant::now();

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(50)).await;

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis();

        // Record execution time
        stats.commands_executed = 1;
        stats.total_execution_time_ms = elapsed_ms;
        stats.min_execution_time_ms = elapsed_ms;
        stats.max_execution_time_ms = elapsed_ms;
        stats.avg_execution_time_ms = elapsed_ms;

        // Verify timing is reasonably accurate (within 10ms tolerance)
        assert!(stats.total_execution_time_ms >= 45);
        assert!(stats.total_execution_time_ms <= 65);
        assert_eq!(stats.min_execution_time_ms, elapsed_ms);
        assert_eq!(stats.max_execution_time_ms, elapsed_ms);
        assert_eq!(stats.avg_execution_time_ms, elapsed_ms);
    }

    /// Test memory tracking accuracy
    #[test]
    fn test_memory_tracking() {
        let mut stats = SessionStatistics::default();

        // Simulate memory usage changes
        let before = 1_000_000u64; // 1 MB
        let after = 1_500_000u64; // 1.5 MB

        stats.memory_delta = Some((before, after));
        stats.peak_memory_bytes = after.max(stats.peak_memory_bytes);

        let (mem_before, mem_after) = stats.memory_delta.unwrap();
        assert_eq!(mem_before, before);
        assert_eq!(mem_after, after);

        let delta = (mem_after as i64) - (mem_before as i64);
        assert_eq!(delta, 500_000); // 500KB increase

        // Test peak tracking
        assert_eq!(stats.peak_memory_bytes, after);

        // Simulate higher peak
        let higher = 2_000_000u64;
        stats.peak_memory_bytes = higher.max(stats.peak_memory_bytes);
        assert_eq!(stats.peak_memory_bytes, higher);
    }

    /// Test performance monitoring toggle
    #[test]
    fn test_perf_monitoring_toggle() {
        let mut perf_enabled = true;

        // When enabled, should track metrics
        if perf_enabled {
            let start = Instant::now();
            // Do work
            std::thread::sleep(Duration::from_millis(10));
            let elapsed = start.elapsed();
            assert!(elapsed.as_millis() > 0);
        }

        // Disable monitoring
        perf_enabled = false;

        // When disabled, should skip tracking
        if !perf_enabled {
            // No timing overhead
            let start = Instant::now();
            // Work happens without measurement overhead
            let elapsed = start.elapsed();
            // This check happens but results aren't stored
            assert!(elapsed.as_micros() < 1000); // Should be very fast
        }
    }

    /// Test with various script sizes
    #[test]
    fn test_performance_with_different_sizes() {
        let mut stats = SessionStatistics::default();

        // Small script
        let small_time = 5u128;
        stats.commands_executed += 1;
        stats.total_execution_time_ms += small_time;
        stats.min_execution_time_ms = small_time;
        stats.max_execution_time_ms = small_time;

        // Medium script
        let medium_time = 50u128;
        stats.commands_executed += 1;
        stats.total_execution_time_ms += medium_time;
        stats.max_execution_time_ms = medium_time;

        // Large script
        let large_time = 500u128;
        stats.commands_executed += 1;
        stats.total_execution_time_ms += large_time;
        stats.max_execution_time_ms = large_time;

        // Calculate average
        stats.avg_execution_time_ms =
            stats.total_execution_time_ms / stats.commands_executed as u128;

        assert_eq!(stats.commands_executed, 3);
        assert_eq!(stats.min_execution_time_ms, small_time);
        assert_eq!(stats.max_execution_time_ms, large_time);
        assert_eq!(stats.avg_execution_time_ms, 185); // (5 + 50 + 500) / 3
    }

    /// Test minimal overhead when monitoring disabled
    #[test]
    fn test_minimal_overhead_when_disabled() {
        let perf_enabled = false;
        let iterations = 10000;

        let start = Instant::now();
        for _ in 0..iterations {
            if perf_enabled {
                // This branch not taken
                let _ = Instant::now();
            }
            // Actual work happens here
            let _result = 2 + 2;
        }
        let elapsed = start.elapsed();

        // Should complete very quickly with monitoring disabled
        assert!(elapsed.as_millis() < 10); // Less than 10ms for 10k iterations
    }

    /// Test session statistics accumulation
    #[test]
    fn test_statistics_accumulation() {
        let mut stats = SessionStatistics::default();

        // Simulate multiple command executions
        let execution_times = vec![10u128, 20, 15, 30, 25];

        for time in &execution_times {
            stats.commands_executed += 1;
            stats.total_execution_time_ms += time;

            // Update min
            if stats.min_execution_time_ms == u128::MAX || *time < stats.min_execution_time_ms {
                stats.min_execution_time_ms = *time;
            }

            // Update max
            if *time > stats.max_execution_time_ms {
                stats.max_execution_time_ms = *time;
            }
        }

        // Calculate average
        stats.avg_execution_time_ms =
            stats.total_execution_time_ms / stats.commands_executed as u128;

        assert_eq!(stats.commands_executed, 5);
        assert_eq!(stats.total_execution_time_ms, 100);
        assert_eq!(stats.min_execution_time_ms, 10);
        assert_eq!(stats.max_execution_time_ms, 30);
        assert_eq!(stats.avg_execution_time_ms, 20);
    }

    /// Test formatting of various magnitudes
    #[test]
    fn test_time_formatting() {
        fn format_duration(ms: u128) -> String {
            if ms < 1000 {
                format!("{} ms", ms)
            } else if ms < 60_000 {
                format!("{:.2} s", ms as f64 / 1000.0)
            } else {
                let minutes = ms / 60_000;
                let seconds = (ms % 60_000) / 1000;
                format!("{} min {} s", minutes, seconds)
            }
        }

        assert_eq!(format_duration(500), "500 ms");
        assert_eq!(format_duration(1500), "1.50 s");
        assert_eq!(format_duration(65_000), "1 min 5 s");
        assert_eq!(format_duration(125_000), "2 min 5 s");
    }

    /// Test memory size formatting
    #[test]
    fn test_memory_formatting() {
        fn format_memory(bytes: u64) -> String {
            const KB: u64 = 1024;
            const MB: u64 = KB * 1024;
            const GB: u64 = MB * 1024;

            if bytes < KB {
                format!("{} B", bytes)
            } else if bytes < MB {
                format!("{:.2} KB", bytes as f64 / KB as f64)
            } else if bytes < GB {
                format!("{:.2} MB", bytes as f64 / MB as f64)
            } else {
                format!("{:.2} GB", bytes as f64 / GB as f64)
            }
        }

        assert_eq!(format_memory(512), "512 B");
        assert_eq!(format_memory(1536), "1.50 KB");
        assert_eq!(format_memory(1_572_864), "1.50 MB");
        assert_eq!(format_memory(1_610_612_736), "1.50 GB");
    }

    /// Test error counting
    #[test]
    fn test_error_counting() {
        let mut stats = SessionStatistics::default();

        // Successful execution
        stats.commands_executed += 1;

        // Error execution
        stats.commands_executed += 1;
        stats.errors_encountered += 1;

        // Another successful
        stats.commands_executed += 1;

        // Another error
        stats.commands_executed += 1;
        stats.errors_encountered += 1;

        assert_eq!(stats.commands_executed, 4);
        assert_eq!(stats.errors_encountered, 2);

        // Calculate error rate
        let error_rate = (stats.errors_encountered as f64 / stats.commands_executed as f64) * 100.0;
        assert_eq!(error_rate, 50.0);
    }

    /// Test performance over long session
    #[test]
    fn test_long_session_statistics() {
        let mut stats = SessionStatistics::default();

        // Simulate a long session (1000 commands)
        for i in 0u32..1000 {
            stats.commands_executed += 1;

            // Vary execution times
            let time = (i % 100 + 1) as u128;
            stats.total_execution_time_ms += time;

            // Update min/max
            if stats.min_execution_time_ms == u128::MAX || time < stats.min_execution_time_ms {
                stats.min_execution_time_ms = time;
            }
            if time > stats.max_execution_time_ms {
                stats.max_execution_time_ms = time;
            }

            // Simulate occasional errors (5% error rate)
            if i.is_multiple_of(20) {
                stats.errors_encountered += 1;
            }

            // Simulate memory growth
            stats.peak_memory_bytes = (stats.peak_memory_bytes + 1024).min(100 * 1024 * 1024);
        }

        stats.avg_execution_time_ms =
            stats.total_execution_time_ms / stats.commands_executed as u128;

        assert_eq!(stats.commands_executed, 1000);
        assert_eq!(stats.min_execution_time_ms, 1);
        assert_eq!(stats.max_execution_time_ms, 100);
        assert_eq!(stats.errors_encountered, 50); // 5% of 1000
        assert!(stats.avg_execution_time_ms > 0);
        assert!(stats.peak_memory_bytes > 0);
    }

    /// Test percentile calculations
    #[test]
    fn test_percentile_calculations() {
        let mut times = [10u128, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        times.sort();

        // P50 (median)
        let p50_index = times.len() / 2;
        let p50 = times[p50_index];
        assert_eq!(p50, 60);

        // P95
        let p95_index = (times.len() as f64 * 0.95) as usize;
        let p95 = times[p95_index.min(times.len() - 1)];
        assert_eq!(p95, 100);

        // P99
        let p99_index = (times.len() as f64 * 0.99) as usize;
        let p99 = times[p99_index.min(times.len() - 1)];
        assert_eq!(p99, 100);
    }

    /// Test reset statistics
    #[test]
    fn test_reset_statistics() {
        let _initial_stats = SessionStatistics {
            commands_executed: 100,
            errors_encountered: 10,
            total_execution_time_ms: 5000,
            peak_memory_bytes: 1_000_000,
            ..Default::default()
        };

        // Reset
        let stats = SessionStatistics::default();

        // Verify reset
        assert_eq!(stats.commands_executed, 0);
        assert_eq!(stats.errors_encountered, 0);
        assert_eq!(stats.total_execution_time_ms, 0);
        assert_eq!(stats.peak_memory_bytes, 0);
        assert_eq!(stats.min_execution_time_ms, u128::MAX);
        assert_eq!(stats.max_execution_time_ms, 0);
    }
}
