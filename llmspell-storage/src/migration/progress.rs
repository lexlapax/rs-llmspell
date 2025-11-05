//! ABOUTME: Migration progress reporting
//! ABOUTME: Real-time progress tracking with percentage and ETA

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Migration progress for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationProgress {
    /// Component being migrated
    pub component: String,

    /// Current record count processed
    pub current: usize,

    /// Total records to migrate
    pub total: usize,

    /// Progress percentage (0-100)
    pub percentage: f64,

    /// Estimated time remaining
    pub eta: Option<Duration>,

    /// Migration start time
    pub started_at: DateTime<Utc>,
}

impl MigrationProgress {
    /// Create new progress tracker
    pub fn new(component: String, total: usize) -> Self {
        Self {
            component,
            current: 0,
            total,
            percentage: 0.0,
            eta: None,
            started_at: Utc::now(),
        }
    }

    /// Update progress with current count
    pub fn update(&mut self, current: usize) {
        self.current = current;
        self.percentage = if self.total > 0 {
            (current as f64 / self.total as f64) * 100.0
        } else {
            100.0
        };

        // Calculate ETA based on elapsed time and remaining work
        if current > 0 && current < self.total {
            let elapsed = Utc::now().signed_duration_since(self.started_at);
            let rate = current as f64 / elapsed.num_seconds() as f64;
            let remaining = self.total - current;
            let eta_seconds = (remaining as f64 / rate) as i64;
            self.eta = Some(Duration::seconds(eta_seconds));
        } else if current >= self.total {
            self.eta = Some(Duration::seconds(0));
        }
    }

    /// Format progress as human-readable string
    pub fn format(&self) -> String {
        let eta_str = self
            .eta
            .map(|d| {
                let secs = d.num_seconds();
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m", secs / 60)
                } else {
                    format!("{}h", secs / 3600)
                }
            })
            .unwrap_or_else(|| "calculating...".to_string());

        format!(
            "{}: {}/{} ({:.1}%) - ETA: {}",
            self.component, self.current, self.total, self.percentage, eta_str
        )
    }
}

/// Final migration report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Migration success status
    pub success: bool,

    /// Components migrated
    pub components: Vec<String>,

    /// Source record count (total across all components)
    pub source_count: usize,

    /// Target record count (total across all components)
    pub target_count: usize,

    /// Migration duration
    pub duration: Duration,

    /// Records per second (throughput)
    pub records_per_second: f64,

    /// Validation results
    pub validation_results: Vec<String>,

    /// Errors encountered (if any)
    pub errors: Vec<String>,
}

impl MigrationReport {
    /// Create new migration report
    pub fn new(
        success: bool,
        components: Vec<String>,
        source_count: usize,
        target_count: usize,
        duration: Duration,
    ) -> Self {
        let records_per_second = if duration.num_seconds() > 0 {
            source_count as f64 / duration.num_seconds() as f64
        } else {
            0.0
        };

        Self {
            success,
            components,
            source_count,
            target_count,
            duration,
            records_per_second,
            validation_results: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add validation result
    pub fn add_validation(&mut self, result: String) {
        self.validation_results.push(result);
    }

    /// Add error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Format report as human-readable string
    pub fn format(&self) -> String {
        let status = if self.success { "SUCCESS" } else { "FAILED" };
        let mut output = format!("Migration {}\n\n", status);

        output.push_str(&format!("Components: {:?}\n", self.components));
        output.push_str(&format!("Source Records: {}\n", self.source_count));
        output.push_str(&format!("Target Records: {}\n", self.target_count));
        output.push_str(&format!("Duration: {}s\n", self.duration.num_seconds()));
        output.push_str(&format!("Throughput: {:.2} records/sec\n", self.records_per_second));

        if !self.validation_results.is_empty() {
            output.push_str("\nValidation Results:\n");
            for result in &self.validation_results {
                output.push_str(&format!("  - {}\n", result));
            }
        }

        if !self.errors.is_empty() {
            output.push_str("\nErrors:\n");
            for error in &self.errors {
                output.push_str(&format!("  - {}\n", error));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracking() {
        let mut progress = MigrationProgress::new("agent_state".to_string(), 1000);
        assert_eq!(progress.current, 0);
        assert_eq!(progress.total, 1000);
        assert_eq!(progress.percentage, 0.0);

        progress.update(500);
        assert_eq!(progress.current, 500);
        assert_eq!(progress.percentage, 50.0);

        progress.update(1000);
        assert_eq!(progress.current, 1000);
        assert_eq!(progress.percentage, 100.0);
    }

    #[test]
    fn test_progress_formatting() {
        let mut progress = MigrationProgress::new("agent_state".to_string(), 1000);
        progress.update(250);
        let formatted = progress.format();
        assert!(formatted.contains("agent_state"));
        assert!(formatted.contains("250/1000"));
        assert!(formatted.contains("25.0%"));
    }

    #[test]
    fn test_migration_report() {
        let report = MigrationReport::new(
            true,
            vec!["agent_state".to_string()],
            1000,
            1000,
            Duration::seconds(60),
        );

        assert!(report.success);
        assert_eq!(report.source_count, 1000);
        assert_eq!(report.target_count, 1000);
        assert!((report.records_per_second - 16.67).abs() < 0.1); // 1000/60 ≈ 16.67
    }
}
