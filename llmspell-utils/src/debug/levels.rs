//! Debug level definitions and utilities
//!
//! Provides hierarchical debug levels that mirror standard logging levels
//! but are specifically for script debugging infrastructure.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Debug level hierarchy for script debugging
///
/// Levels are ordered from least to most verbose.
/// When a level is set, all messages at that level and below are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DebugLevel {
    /// No debug output at all
    Off = 0,
    /// Only errors are shown
    Error = 1,
    /// Errors and warnings
    Warn = 2,
    /// Errors, warnings, and informational messages
    Info = 3,
    /// All above plus debug messages
    Debug = 4,
    /// Everything including detailed trace information
    Trace = 5,
}

impl DebugLevel {
    /// Check if this level should be shown given the current filter level
    #[must_use]
    pub fn should_show(&self, filter: DebugLevel) -> bool {
        *self <= filter
    }

    /// Get the level as a numeric value for comparisons
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Convert from a numeric value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::Error),
            2 => Some(Self::Warn),
            3 => Some(Self::Info),
            4 => Some(Self::Debug),
            5 => Some(Self::Trace),
            _ => None,
        }
    }

    /// Get a colored representation for terminal output
    #[must_use]
    pub fn colored(&self) -> &'static str {
        match self {
            Self::Off => "",
            Self::Error => "\x1b[31mERROR\x1b[0m", // Red
            Self::Warn => "\x1b[33mWARN\x1b[0m",   // Yellow
            Self::Info => "\x1b[32mINFO\x1b[0m",   // Green
            Self::Debug => "\x1b[36mDEBUG\x1b[0m", // Cyan
            Self::Trace => "\x1b[35mTRACE\x1b[0m", // Magenta
        }
    }
}

impl Default for DebugLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl fmt::Display for DebugLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "OFF"),
            Self::Error => write!(f, "ERROR"),
            Self::Warn => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Trace => write!(f, "TRACE"),
        }
    }
}

impl FromStr for DebugLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" | "none" | "0" => Ok(Self::Off),
            "error" | "err" | "1" => Ok(Self::Error),
            "warn" | "warning" | "2" => Ok(Self::Warn),
            "info" | "3" => Ok(Self::Info),
            "debug" | "4" => Ok(Self::Debug),
            "trace" | "5" => Ok(Self::Trace),
            _ => Err(format!("Invalid debug level: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(DebugLevel::Error < DebugLevel::Warn);
        assert!(DebugLevel::Warn < DebugLevel::Info);
        assert!(DebugLevel::Info < DebugLevel::Debug);
        assert!(DebugLevel::Debug < DebugLevel::Trace);
    }

    #[test]
    fn test_should_show() {
        let filter = DebugLevel::Info;
        assert!(DebugLevel::Error.should_show(filter));
        assert!(DebugLevel::Warn.should_show(filter));
        assert!(DebugLevel::Info.should_show(filter));
        assert!(!DebugLevel::Debug.should_show(filter));
        assert!(!DebugLevel::Trace.should_show(filter));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(DebugLevel::from_str("debug").unwrap(), DebugLevel::Debug);
        assert_eq!(DebugLevel::from_str("TRACE").unwrap(), DebugLevel::Trace);
        assert_eq!(DebugLevel::from_str("3").unwrap(), DebugLevel::Info);
        assert!(DebugLevel::from_str("invalid").is_err());
    }
}
