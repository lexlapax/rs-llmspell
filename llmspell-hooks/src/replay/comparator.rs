// ABOUTME: Hook result comparator for analyzing differences between original and replayed executions
// ABOUTME: Provides detailed comparison reports for debugging and what-if analysis

use crate::result::HookResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Result of comparing two hook executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Whether the results are identical
    pub identical: bool,
    /// Type of difference if not identical
    pub difference_type: Option<DifferenceType>,
    /// Detailed differences
    pub differences: Vec<Difference>,
    /// Similarity score (0.0 to 1.0)
    pub similarity_score: f64,
    /// Summary of the comparison
    pub summary: String,
}

/// Type of difference between results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifferenceType {
    /// Different result variants (Continue vs Modified, etc.)
    VariantMismatch,
    /// Same variant but different data
    DataMismatch,
    /// Different error types
    ErrorMismatch,
    /// Different cancellation reasons
    CancellationMismatch,
    /// Multiple differences
    Multiple,
}

/// A specific difference found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    /// Path to the difference (e.g., "data.field.subfield")
    pub path: String,
    /// Original value
    pub original: Option<Value>,
    /// Replayed value
    pub replayed: Option<Value>,
    /// Description of the difference
    pub description: String,
    /// Severity of the difference
    pub severity: DifferenceSeverity,
}

/// Severity of a difference
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DifferenceSeverity {
    /// Informational only
    Info,
    /// Minor difference
    Minor,
    /// Major difference
    Major,
    /// Critical difference
    Critical,
}

/// Hook result comparator
pub struct HookResultComparator {
    /// Configuration for comparison
    config: ComparatorConfig,
}

/// Comparator configuration
#[derive(Debug, Clone)]
pub struct ComparatorConfig {
    /// Whether to ignore timestamp differences
    pub ignore_timestamps: bool,
    /// Whether to ignore order in arrays
    pub ignore_array_order: bool,
    /// Fields to ignore in comparison
    pub ignore_fields: Vec<String>,
    /// Whether to perform deep comparison
    pub deep_compare: bool,
}

impl Default for ComparatorConfig {
    fn default() -> Self {
        Self {
            ignore_timestamps: true,
            ignore_array_order: false,
            ignore_fields: vec!["timestamp".to_string(), "duration".to_string()],
            deep_compare: true,
        }
    }
}

impl HookResultComparator {
    /// Create a new comparator with default config
    pub fn new() -> Self {
        Self {
            config: ComparatorConfig::default(),
        }
    }

    /// Create a comparator with custom config
    pub fn with_config(config: ComparatorConfig) -> Self {
        Self { config }
    }

    /// Compare two hook results
    pub fn compare(&self, original: &HookResult, replayed: &HookResult) -> ComparisonResult {
        let mut differences = Vec::new();
        let mut difference_type = None;

        // Compare result variants
        match (original, replayed) {
            (HookResult::Continue, HookResult::Continue) => {
                // Identical
            }
            (HookResult::Modified(orig_data), HookResult::Modified(repl_data)) => {
                self.compare_values("", orig_data, repl_data, &mut differences);
                if !differences.is_empty() {
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            (HookResult::Cancel(orig_reason), HookResult::Cancel(repl_reason)) => {
                if orig_reason != repl_reason {
                    differences.push(Difference {
                        path: "cancel_reason".to_string(),
                        original: Some(Value::String(orig_reason.clone())),
                        replayed: Some(Value::String(repl_reason.clone())),
                        description: "Cancellation reasons differ".to_string(),
                        severity: DifferenceSeverity::Major,
                    });
                    difference_type = Some(DifferenceType::CancellationMismatch);
                }
            }
            (HookResult::Redirect(orig_target), HookResult::Redirect(repl_target)) => {
                if orig_target != repl_target {
                    differences.push(Difference {
                        path: "redirect_target".to_string(),
                        original: Some(Value::String(orig_target.clone())),
                        replayed: Some(Value::String(repl_target.clone())),
                        description: "Redirect targets differ".to_string(),
                        severity: DifferenceSeverity::Major,
                    });
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            (HookResult::Replace(orig_data), HookResult::Replace(repl_data)) => {
                self.compare_values("", orig_data, repl_data, &mut differences);
                if !differences.is_empty() {
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            (
                HookResult::Retry {
                    delay: orig_delay,
                    max_attempts: orig_attempts,
                },
                HookResult::Retry {
                    delay: repl_delay,
                    max_attempts: repl_attempts,
                },
            ) => {
                if orig_delay != repl_delay {
                    differences.push(Difference {
                        path: "retry.delay".to_string(),
                        original: Some(Value::String(format!("{:?}", orig_delay))),
                        replayed: Some(Value::String(format!("{:?}", repl_delay))),
                        description: "Retry delays differ".to_string(),
                        severity: DifferenceSeverity::Major,
                    });
                }
                if orig_attempts != repl_attempts {
                    differences.push(Difference {
                        path: "retry.max_attempts".to_string(),
                        original: Some(Value::Number((*orig_attempts).into())),
                        replayed: Some(Value::Number((*repl_attempts).into())),
                        description: "Retry max attempts differ".to_string(),
                        severity: DifferenceSeverity::Major,
                    });
                }
                if !differences.is_empty() {
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            (
                HookResult::Fork {
                    parallel_operations: orig_results,
                },
                HookResult::Fork {
                    parallel_operations: repl_results,
                },
            ) => {
                if orig_results.len() != repl_results.len() {
                    differences.push(Difference {
                        path: "fork_count".to_string(),
                        original: Some(Value::Number(orig_results.len().into())),
                        replayed: Some(Value::Number(repl_results.len().into())),
                        description: "Fork result counts differ".to_string(),
                        severity: DifferenceSeverity::Major,
                    });
                } else {
                    // Compare operation structures
                    for (i, (orig_op, repl_op)) in
                        orig_results.iter().zip(repl_results.iter()).enumerate()
                    {
                        if orig_op.id != repl_op.id {
                            differences.push(Difference {
                                path: format!("fork[{}].id", i),
                                original: Some(Value::String(orig_op.id.clone())),
                                replayed: Some(Value::String(repl_op.id.clone())),
                                description: "Operation IDs differ".to_string(),
                                severity: DifferenceSeverity::Major,
                            });
                        }
                        if orig_op.operation_type != repl_op.operation_type {
                            differences.push(Difference {
                                path: format!("fork[{}].operation_type", i),
                                original: Some(Value::String(orig_op.operation_type.clone())),
                                replayed: Some(Value::String(repl_op.operation_type.clone())),
                                description: "Operation types differ".to_string(),
                                severity: DifferenceSeverity::Major,
                            });
                        }
                        self.compare_values(
                            &format!("fork[{}].parameters", i),
                            &orig_op.parameters,
                            &repl_op.parameters,
                            &mut differences,
                        );
                    }
                }
                if !differences.is_empty() {
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            (
                HookResult::Cache {
                    key: orig_key,
                    ttl: orig_ttl,
                },
                HookResult::Cache {
                    key: repl_key,
                    ttl: repl_ttl,
                },
            ) => {
                if orig_key != repl_key {
                    differences.push(Difference {
                        path: "cache.key".to_string(),
                        original: Some(Value::String(orig_key.clone())),
                        replayed: Some(Value::String(repl_key.clone())),
                        description: "Cache keys differ".to_string(),
                        severity: DifferenceSeverity::Major,
                    });
                }
                if orig_ttl != repl_ttl {
                    differences.push(Difference {
                        path: "cache.ttl".to_string(),
                        original: Some(Value::String(format!("{:?}", orig_ttl))),
                        replayed: Some(Value::String(format!("{:?}", repl_ttl))),
                        description: "Cache TTLs differ".to_string(),
                        severity: DifferenceSeverity::Minor,
                    });
                }
                if !differences.is_empty() {
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            (HookResult::Skipped(orig_reason), HookResult::Skipped(repl_reason)) => {
                if orig_reason != repl_reason {
                    differences.push(Difference {
                        path: "skip_reason".to_string(),
                        original: Some(Value::String(orig_reason.clone())),
                        replayed: Some(Value::String(repl_reason.clone())),
                        description: "Skip reasons differ".to_string(),
                        severity: DifferenceSeverity::Minor,
                    });
                    difference_type = Some(DifferenceType::DataMismatch);
                }
            }
            _ => {
                // Different variants
                differences.push(Difference {
                    path: "variant".to_string(),
                    original: Some(Value::String(format!("{:?}", original))),
                    replayed: Some(Value::String(format!("{:?}", replayed))),
                    description: "Result variants differ".to_string(),
                    severity: DifferenceSeverity::Critical,
                });
                difference_type = Some(DifferenceType::VariantMismatch);
            }
        }

        // Calculate similarity score
        let similarity_score = self.calculate_similarity(&differences);

        // Determine final difference type
        if differences.len() > 1 && difference_type.is_some() {
            difference_type = Some(DifferenceType::Multiple);
        }

        // Generate summary
        let summary = self.generate_summary(&differences, difference_type.as_ref());

        ComparisonResult {
            identical: differences.is_empty(),
            difference_type,
            differences,
            similarity_score,
            summary,
        }
    }

    /// Compare two JSON values recursively
    fn compare_values(
        &self,
        path: &str,
        original: &Value,
        replayed: &Value,
        differences: &mut Vec<Difference>,
    ) {
        match (original, replayed) {
            (Value::Object(orig_map), Value::Object(repl_map)) => {
                // Compare object fields
                let mut all_keys: Vec<_> = orig_map.keys().chain(repl_map.keys()).collect();
                all_keys.sort();
                all_keys.dedup();

                for key in all_keys {
                    let field_path = if path.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    // Check if field should be ignored
                    if self.config.ignore_fields.contains(&field_path) {
                        continue;
                    }

                    match (orig_map.get(key), repl_map.get(key)) {
                        (Some(orig_val), Some(repl_val)) => {
                            self.compare_values(&field_path, orig_val, repl_val, differences);
                        }
                        (Some(orig_val), None) => {
                            differences.push(Difference {
                                path: field_path,
                                original: Some(orig_val.clone()),
                                replayed: None,
                                description: "Field missing in replayed result".to_string(),
                                severity: DifferenceSeverity::Major,
                            });
                        }
                        (None, Some(repl_val)) => {
                            differences.push(Difference {
                                path: field_path,
                                original: None,
                                replayed: Some(repl_val.clone()),
                                description: "Field added in replayed result".to_string(),
                                severity: DifferenceSeverity::Major,
                            });
                        }
                        (None, None) => unreachable!(),
                    }
                }
            }
            (Value::Array(orig_arr), Value::Array(repl_arr)) => {
                if self.config.ignore_array_order {
                    // Compare arrays ignoring order
                    self.compare_arrays_unordered(path, orig_arr, repl_arr, differences);
                } else {
                    // Compare arrays with order
                    self.compare_arrays_ordered(path, orig_arr, repl_arr, differences);
                }
            }
            (orig, repl) if orig == repl => {
                // Values are equal
            }
            (orig, repl) => {
                // Values differ
                let severity = if path.contains("error") || path.contains("result") {
                    DifferenceSeverity::Critical
                } else {
                    DifferenceSeverity::Major
                };

                differences.push(Difference {
                    path: path.to_string(),
                    original: Some(orig.clone()),
                    replayed: Some(repl.clone()),
                    description: "Values differ".to_string(),
                    severity,
                });
            }
        }
    }

    /// Compare arrays preserving order
    fn compare_arrays_ordered(
        &self,
        path: &str,
        original: &[Value],
        replayed: &[Value],
        differences: &mut Vec<Difference>,
    ) {
        if original.len() != replayed.len() {
            differences.push(Difference {
                path: format!("{}.length", path),
                original: Some(Value::Number(original.len().into())),
                replayed: Some(Value::Number(replayed.len().into())),
                description: "Array lengths differ".to_string(),
                severity: DifferenceSeverity::Major,
            });
        }

        for (i, (orig, repl)) in original.iter().zip(replayed.iter()).enumerate() {
            let item_path = format!("{}[{}]", path, i);
            self.compare_values(&item_path, orig, repl, differences);
        }
    }

    /// Compare arrays ignoring order
    fn compare_arrays_unordered(
        &self,
        path: &str,
        original: &[Value],
        replayed: &[Value],
        differences: &mut Vec<Difference>,
    ) {
        let orig_counts = self.count_array_elements(original);
        let repl_counts = self.count_array_elements(replayed);

        for (value, &orig_count) in &orig_counts {
            let repl_count = repl_counts.get(value).copied().unwrap_or(0);
            if orig_count != repl_count {
                differences.push(Difference {
                    path: path.to_string(),
                    original: Some(Value::String(format!("{} occurrences", orig_count))),
                    replayed: Some(Value::String(format!("{} occurrences", repl_count))),
                    description: format!("Element count mismatch for {:?}", value),
                    severity: DifferenceSeverity::Major,
                });
            }
        }

        for (value, &repl_count) in &repl_counts {
            if !orig_counts.contains_key(value) {
                differences.push(Difference {
                    path: path.to_string(),
                    original: Some(Value::String("0 occurrences".to_string())),
                    replayed: Some(Value::String(format!("{} occurrences", repl_count))),
                    description: format!("New element in replayed: {:?}", value),
                    severity: DifferenceSeverity::Major,
                });
            }
        }
    }

    /// Count occurrences of each element in an array
    fn count_array_elements(&self, array: &[Value]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for value in array {
            let key = serde_json::to_string(value).unwrap_or_default();
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }

    /// Calculate similarity score based on differences
    fn calculate_similarity(&self, differences: &[Difference]) -> f64 {
        if differences.is_empty() {
            return 1.0;
        }

        let total_weight: f64 = differences
            .iter()
            .map(|d| match d.severity {
                DifferenceSeverity::Info => 0.1,
                DifferenceSeverity::Minor => 0.25,
                DifferenceSeverity::Major => 0.5,
                DifferenceSeverity::Critical => 1.0,
            })
            .sum();

        // Score decreases with more and more severe differences
        (100.0 - total_weight * 10.0).max(0.0) / 100.0
    }

    /// Generate a human-readable summary
    fn generate_summary(
        &self,
        differences: &[Difference],
        difference_type: Option<&DifferenceType>,
    ) -> String {
        if differences.is_empty() {
            return "Results are identical".to_string();
        }

        let critical_count = differences
            .iter()
            .filter(|d| d.severity == DifferenceSeverity::Critical)
            .count();
        let major_count = differences
            .iter()
            .filter(|d| d.severity == DifferenceSeverity::Major)
            .count();
        let minor_count = differences
            .iter()
            .filter(|d| d.severity == DifferenceSeverity::Minor)
            .count();

        let type_desc = match difference_type {
            Some(DifferenceType::VariantMismatch) => "different result types",
            Some(DifferenceType::DataMismatch) => "different data values",
            Some(DifferenceType::ErrorMismatch) => "different errors",
            Some(DifferenceType::CancellationMismatch) => "different cancellation reasons",
            Some(DifferenceType::Multiple) => "multiple differences",
            None => "unknown differences",
        };

        format!(
            "Found {} differences ({}): {} critical, {} major, {} minor",
            differences.len(),
            type_desc,
            critical_count,
            major_count,
            minor_count
        )
    }
}

impl Default for HookResultComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ComparisonResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary)?;
        if !self.identical {
            write!(f, " (similarity: {:.1}%)", self.similarity_score * 100.0)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_identical_results() {
        let comparator = HookResultComparator::new();
        let result = comparator.compare(&HookResult::Continue, &HookResult::Continue);

        assert!(result.identical);
        assert_eq!(result.similarity_score, 1.0);
        assert!(result.differences.is_empty());
    }
    #[test]
    fn test_variant_mismatch() {
        let comparator = HookResultComparator::new();
        let result = comparator.compare(
            &HookResult::Continue,
            &HookResult::Cancel("test".to_string()),
        );

        assert!(!result.identical);
        assert_eq!(
            result.difference_type,
            Some(DifferenceType::VariantMismatch)
        );
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].severity, DifferenceSeverity::Critical);
    }
    #[test]
    fn test_data_comparison() {
        let comparator = HookResultComparator::new();

        let orig_data = serde_json::json!({
            "field1": "value1",
            "field2": 42,
            "field3": {
                "nested": "data"
            }
        });

        let repl_data = serde_json::json!({
            "field1": "value1",
            "field2": 43,  // Different
            "field3": {
                "nested": "data"
            }
        });

        let result = comparator.compare(
            &HookResult::Modified(orig_data),
            &HookResult::Modified(repl_data),
        );

        assert!(!result.identical);
        assert_eq!(result.difference_type, Some(DifferenceType::DataMismatch));
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "field2");
    }
    #[test]
    fn test_ignore_fields() {
        let config = ComparatorConfig {
            ignore_fields: vec!["field2".to_string()],
            ..Default::default()
        };
        let comparator = HookResultComparator::with_config(config);

        let orig_data = serde_json::json!({
            "field1": "value1",
            "field2": 42,  // Will be ignored
        });

        let repl_data = serde_json::json!({
            "field1": "value1",
            "field2": 43,  // Different but ignored
        });

        let result = comparator.compare(
            &HookResult::Modified(orig_data),
            &HookResult::Modified(repl_data),
        );

        assert!(result.identical);
        assert_eq!(result.similarity_score, 1.0);
    }
}
