//! ABOUTME: Migration validation with checksum verification
//! ABOUTME: Pre-flight, backup, and post-migration validation

use super::plan::MigrationPlan;
use super::traits::{MigrationSource, MigrationTarget};
use anyhow::Result;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Migration validator for pre-flight and post-migration checks
pub struct MigrationValidator {
    source: Arc<dyn MigrationSource>,
    target: Arc<dyn MigrationTarget>,
}

impl MigrationValidator {
    /// Create new migration validator
    pub fn new(source: Arc<dyn MigrationSource>, target: Arc<dyn MigrationTarget>) -> Self {
        Self { source, target }
    }

    /// Run pre-flight validation checks
    pub async fn pre_flight(&self, plan: &MigrationPlan) -> Result<PreFlightReport> {
        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 1. Source connectivity and component validation
        for component in &plan.components {
            match self.source.count(&component.name).await {
                Ok(count) => {
                    checks.push(format!(
                        "Source {}: {} records found",
                        component.name, count
                    ));

                    // Warn if no records to migrate
                    if count == 0 {
                        warnings.push(format!(
                            "Component {} has no records to migrate",
                            component.name
                        ));
                    }

                    // Warn if count differs significantly from estimate
                    if component.estimated_count > 0 {
                        let diff_pct = ((count as i64 - component.estimated_count as i64).abs()
                            as f64
                            / component.estimated_count as f64)
                            * 100.0;
                        if diff_pct > 10.0 {
                            warnings.push(format!(
                                "Component {} count differs from estimate by {:.1}% (estimated: {}, actual: {})",
                                component.name, diff_pct, component.estimated_count, count
                            ));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Source {} connectivity failed: {}",
                        component.name, e
                    ));
                }
            }
        }

        // 2. Target connectivity validation
        for component in &plan.components {
            match self.target.count(&component.name).await {
                Ok(count) => {
                    checks.push(format!(
                        "Target {}: {} existing records",
                        component.name, count
                    ));

                    // Warn if target already has data
                    if count > 0 {
                        warnings.push(format!(
                            "Target {} already contains {} records - migration will append/overwrite",
                            component.name, count
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Target {} connectivity failed: {}",
                        component.name, e
                    ));
                }
            }
        }

        // 3. Validation rules check
        if plan.validation.checksum_sample_percent == 0 {
            warnings.push("Checksum validation is disabled (sample percentage is 0)".to_string());
        } else if plan.validation.checksum_sample_percent < 10 {
            warnings.push(format!(
                "Low checksum sample percentage ({}%) - consider increasing for better validation",
                plan.validation.checksum_sample_percent
            ));
        }

        let success = errors.is_empty();

        Ok(PreFlightReport {
            success,
            checks,
            warnings,
            errors,
        })
    }

    /// Validate migration for a component
    pub async fn validate(&self, component: &str) -> Result<ValidationReport> {
        // Count validation
        let source_count = self.source.count(component).await?;
        let target_count = self.target.count(component).await?;
        let count_match = source_count == target_count;

        if !count_match {
            return Ok(ValidationReport {
                component: component.to_string(),
                source_count,
                target_count,
                count_match,
                checksums_validated: 0,
                checksum_mismatches: Vec::new(),
                full_comparison: false,
                success: false,
            });
        }

        // Checksum validation for random sample (10%)
        let sample_size = (source_count as f64 * 0.1).ceil() as usize;
        let checksum_report = if source_count > 0 && sample_size > 0 {
            self.validate_checksums(component, sample_size).await?
        } else {
            ChecksumReport {
                validated: 0,
                mismatches: Vec::new(),
            }
        };

        let success = count_match && checksum_report.mismatches.is_empty();

        Ok(ValidationReport {
            component: component.to_string(),
            source_count,
            target_count,
            count_match,
            checksums_validated: checksum_report.validated,
            checksum_mismatches: checksum_report.mismatches,
            full_comparison: false,
            success,
        })
    }

    /// Validate checksums for random sample of records
    async fn validate_checksums(
        &self,
        component: &str,
        sample_size: usize,
    ) -> Result<ChecksumReport> {
        // Get all keys for component
        let all_keys = self.source.list_keys(component).await?;

        // Random sample
        let mut rng = rand::thread_rng();
        let sampled_keys: Vec<_> = all_keys
            .choose_multiple(&mut rng, sample_size.min(all_keys.len()))
            .cloned()
            .collect();

        let mut validated = 0;
        let mut mismatches = Vec::new();

        for key in sampled_keys {
            // Get values from source and target
            let source_value = self.source.get_value(component, &key).await?;
            let target_value = self.target.get_value(component, &key).await?;

            // Compare values
            match (source_value, target_value) {
                (Some(src), Some(tgt)) => {
                    // Compute checksums
                    let src_checksum = compute_checksum(&src);
                    let tgt_checksum = compute_checksum(&tgt);

                    if src_checksum != tgt_checksum {
                        mismatches.push(format!(
                            "{} (source: {}, target: {})",
                            key, src_checksum, tgt_checksum
                        ));
                    }
                    validated += 1;
                }
                (Some(_), None) => {
                    // Key exists in source but not in target
                    mismatches.push(format!("{} (missing in target)", key));
                }
                (None, Some(_)) => {
                    // Key exists in target but not in source (shouldn't happen)
                    mismatches.push(format!("{} (unexpected in target)", key));
                }
                (None, None) => {
                    // Key doesn't exist in either (shouldn't happen since we got it from source)
                    mismatches.push(format!("{} (missing in both)", key));
                }
            }
        }

        Ok(ChecksumReport {
            validated,
            mismatches,
        })
    }
}

/// Compute SHA-256 checksum of value
fn compute_checksum(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}

/// Pre-flight validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreFlightReport {
    /// Overall success status
    pub success: bool,

    /// Validation checks performed
    pub checks: Vec<String>,

    /// Warnings encountered
    pub warnings: Vec<String>,

    /// Errors encountered
    pub errors: Vec<String>,
}

impl PreFlightReport {
    /// Format summary for display
    pub fn summary(&self) -> String {
        if self.success {
            "Pre-flight validation passed".to_string()
        } else {
            format!("Pre-flight validation failed: {}", self.errors.join(", "))
        }
    }
}

/// Validation report for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Component validated
    pub component: String,

    /// Source record count
    pub source_count: usize,

    /// Target record count
    pub target_count: usize,

    /// Whether counts match
    pub count_match: bool,

    /// Number of checksums validated
    pub checksums_validated: usize,

    /// Keys with checksum mismatches
    pub checksum_mismatches: Vec<String>,

    /// Whether full data comparison was performed
    pub full_comparison: bool,

    /// Overall success status
    pub success: bool,
}

impl ValidationReport {
    /// Format summary for display
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "{}: {} records validated, {} checksums verified",
                self.component, self.source_count, self.checksums_validated
            )
        } else if !self.count_match {
            format!(
                "{}: Count mismatch (source: {}, target: {})",
                self.component, self.source_count, self.target_count
            )
        } else {
            format!(
                "{}: {} checksum mismatches found",
                self.component,
                self.checksum_mismatches.len()
            )
        }
    }
}

/// Checksum validation report
#[derive(Debug, Clone)]
pub struct ChecksumReport {
    /// Number of checksums validated
    pub validated: usize,

    /// Keys with mismatches
    pub mismatches: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_computation() {
        let data = b"test data";
        let checksum = compute_checksum(data);
        assert_eq!(checksum.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_validation_report_summary() {
        let report = ValidationReport {
            component: "agent_state".to_string(),
            source_count: 1000,
            target_count: 1000,
            count_match: true,
            checksums_validated: 100,
            checksum_mismatches: Vec::new(),
            full_comparison: false,
            success: true,
        };

        let summary = report.summary();
        assert!(summary.contains("agent_state"));
        assert!(summary.contains("1000 records"));
        assert!(summary.contains("100 checksums"));
    }

    #[test]
    fn test_pre_flight_report_summary() {
        let report = PreFlightReport {
            success: true,
            checks: vec!["Connectivity OK".to_string()],
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        assert_eq!(report.summary(), "Pre-flight validation passed");

        let failed_report = PreFlightReport {
            success: false,
            checks: Vec::new(),
            warnings: Vec::new(),
            errors: vec!["No disk space".to_string()],
        };

        assert!(failed_report.summary().contains("failed"));
        assert!(failed_report.summary().contains("No disk space"));
    }
}
