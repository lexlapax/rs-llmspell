//! Virtual table helper functions

use crate::distance::DistanceMetric;

use rusqlite::Result as SqliteResult;
use std::str::from_utf8;

/// Parse dimension parameter from CREATE VIRTUAL TABLE args
///
/// Example: CREATE VIRTUAL TABLE x USING vectorlite(dimension=768, ...)
///
/// # Errors
///
/// Returns error if dimension parameter is missing or invalid
pub fn parse_dimension(args: &[&[u8]]) -> SqliteResult<usize> {
    for arg in args.iter().skip(3) {
        // Skip module, db, table names
        if let Ok(s) = from_utf8(arg) {
            if let Some(dim_str) = s.strip_prefix("dimension=") {
                if let Ok(dim) = dim_str.parse::<usize>() {
                    if dim > 0 {
                        return Ok(dim);
                    }
                }
                return Err(rusqlite::Error::ModuleError(format!(
                    "Invalid dimension {dim_str}, must be > 0"
                )));
            }
        }
    }

    Err(rusqlite::Error::ModuleError(
        "Missing required parameter: dimension".to_string(),
    ))
}

/// Parse distance metric parameter from CREATE VIRTUAL TABLE args
///
/// Example: CREATE VIRTUAL TABLE x USING vectorlite(metric='cosine', ...)
///
/// Defaults to Cosine if not specified
pub fn parse_metric(args: &[&[u8]]) -> SqliteResult<DistanceMetric> {
    use std::str::FromStr;

    for arg in args.iter().skip(3) {
        if let Ok(s) = from_utf8(arg) {
            if let Some(metric_str) = s.strip_prefix("metric=") {
                // Remove quotes if present
                let metric_str = metric_str.trim_matches('\'').trim_matches('"');
                match DistanceMetric::from_str(metric_str) {
                    Ok(metric) => return Ok(metric),
                    Err(_) => {
                        return Err(rusqlite::Error::ModuleError(format!(
                            "Invalid metric '{metric_str}', must be l2/cosine/ip"
                        )))
                    }
                }
            }
        }
    }

    Ok(DistanceMetric::default())
}

/// Parse max_elements parameter from CREATE VIRTUAL TABLE args
///
/// Example: CREATE VIRTUAL TABLE x USING vectorlite(max_elements=100000, ...)
///
/// Returns None if not specified (caller should use default)
pub fn parse_max_elements(args: &[&[u8]]) -> Option<usize> {
    for arg in args.iter().skip(3) {
        if let Ok(s) = from_utf8(arg) {
            if let Some(max_str) = s.strip_prefix("max_elements=") {
                if let Ok(max) = max_str.parse::<usize>() {
                    return Some(max);
                }
            }
        }
    }
    None
}

/// Parse ef_construction parameter from CREATE VIRTUAL TABLE args
///
/// Example: CREATE VIRTUAL TABLE x USING vectorlite(ef_construction=200, ...)
///
/// Returns None if not specified (caller should use default)
pub fn parse_ef_construction(args: &[&[u8]]) -> Option<usize> {
    for arg in args.iter().skip(3) {
        if let Ok(s) = from_utf8(arg) {
            if let Some(ef_str) = s.strip_prefix("ef_construction=") {
                if let Ok(ef) = ef_str.parse::<usize>() {
                    return Some(ef);
                }
            }
        }
    }
    None
}

/// Parse m parameter from CREATE VIRTUAL TABLE args
///
/// Example: CREATE VIRTUAL TABLE x USING vectorlite(m=16, ...)
///
/// Returns None if not specified (caller should use default)
pub fn parse_m(args: &[&[u8]]) -> Option<usize> {
    for arg in args.iter().skip(3) {
        if let Ok(s) = from_utf8(arg) {
            if let Some(m_str) = s.strip_prefix("m=") {
                if let Ok(m) = m_str.parse::<usize>() {
                    return Some(m);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dimension() {
        let args: &[&[u8]] = &[b"vectorlite", b"main", b"test_table", b"dimension=768"];
        assert_eq!(parse_dimension(args).unwrap(), 768);

        let args: &[&[u8]] = &[b"vectorlite", b"main", b"test_table", b"dimension=1536"];
        assert_eq!(parse_dimension(args).unwrap(), 1536);

        // Test dimension=0 (should fail)
        let args: &[&[u8]] = &[b"vectorlite", b"main", b"test_table", b"dimension=0"];
        assert!(parse_dimension(args).is_err());

        let args: &[&[u8]] = &[b"vectorlite", b"main", b"test_table"];
        assert!(parse_dimension(args).is_err());
    }

    #[test]
    fn test_parse_metric() {
        let args: &[&[u8]] = &[
            b"vectorlite",
            b"main",
            b"test_table",
            b"dimension=768",
            b"metric='cosine'",
        ];
        assert_eq!(parse_metric(args).unwrap(), DistanceMetric::Cosine);

        let args: &[&[u8]] = &[
            b"vectorlite",
            b"main",
            b"test_table",
            b"dimension=768",
            b"metric=l2",
        ];
        assert_eq!(parse_metric(args).unwrap(), DistanceMetric::L2);

        // Default to cosine if not specified
        let args: &[&[u8]] = &[b"vectorlite", b"main", b"test_table", b"dimension=768"];
        assert_eq!(parse_metric(args).unwrap(), DistanceMetric::Cosine);
    }

    #[test]
    fn test_parse_parameters() {
        let args: &[&[u8]] = &[
            b"vectorlite",
            b"main",
            b"test_table",
            b"dimension=768",
            b"max_elements=50000",
            b"ef_construction=100",
            b"m=32",
        ];

        assert_eq!(parse_max_elements(args), Some(50000));
        assert_eq!(parse_ef_construction(args), Some(100));
        assert_eq!(parse_m(args), Some(32));
    }

    #[test]
    fn test_parse_parameters_defaults() {
        let args: &[&[u8]] = &[b"vectorlite", b"main", b"test_table", b"dimension=768"];

        assert_eq!(parse_max_elements(args), None);
        assert_eq!(parse_ef_construction(args), None);
        assert_eq!(parse_m(args), None);
    }
}
