// ABOUTME: Time and date utilities for parsing, formatting, and manipulation
// ABOUTME: Provides timezone conversion, date arithmetic, and format parsing functions

//! Time and date utilities
//!
//! This module provides utilities for working with dates and times including:
//! - Parsing dates from multiple formats
//! - Timezone conversion with DST handling
//! - Date arithmetic operations
//! - Formatting dates in various standards

use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, Utc, Weekday};
use chrono_tz::Tz;
use std::str::FromStr;

/// Common date formats for parsing
pub const DATE_FORMATS: &[&str] = &[
    // ISO 8601 formats
    "%Y-%m-%dT%H:%M:%S%.fZ",
    "%Y-%m-%dT%H:%M:%SZ",
    "%Y-%m-%dT%H:%M:%S%z",
    "%Y-%m-%dT%H:%M:%S%:z",
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d",
    // RFC formats
    "%a, %d %b %Y %H:%M:%S GMT",
    "%a, %d %b %Y %H:%M:%S %z",
    // Common formats
    "%Y/%m/%d %H:%M:%S",
    "%Y/%m/%d",
    "%d/%m/%Y %H:%M:%S",
    "%d/%m/%Y",
    "%m/%d/%Y %H:%M:%S",
    "%m/%d/%Y",
    "%d-%m-%Y %H:%M:%S",
    "%d-%m-%Y",
    "%d.%m.%Y %H:%M:%S",
    "%d.%m.%Y",
    // Human readable
    "%B %d, %Y %H:%M:%S",
    "%B %d, %Y",
    "%b %d, %Y %H:%M:%S",
    "%b %d, %Y",
];

/// Time utilities error type
#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    /// Failed to parse date/time
    #[error("Failed to parse date/time: {0}")]
    ParseError(String),

    /// Invalid timezone
    #[error("Invalid timezone: {0}")]
    InvalidTimezone(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type for time operations
pub type TimeResult<T> = Result<T, TimeError>;

/// Parse a date/time string trying multiple formats
///
/// # Errors
///
/// Returns `TimeError::ParseError` if the input cannot be parsed as any known format
pub fn parse_datetime(input: &str) -> TimeResult<DateTime<Utc>> {
    // Try parsing as timestamp first
    if let Ok(timestamp) = input.parse::<i64>() {
        if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
            return Ok(dt);
        }
    }

    // Try parsing with timezone info
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(dt) = DateTime::parse_from_rfc2822(input) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try common formats
    for format in DATE_FORMATS {
        if let Ok(dt) = DateTime::parse_from_str(input, format) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try as naive datetime assuming UTC
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, format) {
            return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
        }

        // Try as naive date (no time component)
        if let Ok(date) = chrono::NaiveDate::parse_from_str(input, format) {
            if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
            }
        }
    }

    Err(TimeError::ParseError(format!(
        "Could not parse '{input}' as a date/time"
    )))
}

/// Convert datetime to specified timezone
///
/// # Errors
///
/// Returns `TimeError::InvalidTimezone` if the timezone name is not valid
pub fn convert_timezone(dt: &DateTime<Utc>, tz_name: &str) -> TimeResult<DateTime<Tz>> {
    let tz = Tz::from_str(tz_name).map_err(|_| TimeError::InvalidTimezone(tz_name.to_string()))?;
    Ok(dt.with_timezone(&tz))
}

/// Get current time in UTC
#[must_use]
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Get current time in local timezone
#[must_use]
pub fn now_local() -> DateTime<Local> {
    Local::now()
}

/// Format datetime in specified format
#[must_use]
pub fn format_datetime(dt: &DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

/// Add duration to datetime
///
/// # Errors
///
/// Returns `TimeError::InvalidOperation` if the unit is unknown or arithmetic overflows
pub fn add_duration(dt: &DateTime<Utc>, amount: i64, unit: &str) -> TimeResult<DateTime<Utc>> {
    let duration = match unit.to_lowercase().as_str() {
        "second" | "seconds" | "sec" | "secs" | "s" => Duration::seconds(amount),
        "minute" | "minutes" | "min" | "mins" | "m" => Duration::minutes(amount),
        "hour" | "hours" | "hr" | "hrs" | "h" => Duration::hours(amount),
        "day" | "days" | "d" => Duration::days(amount),
        "week" | "weeks" | "w" => Duration::weeks(amount),
        _ => {
            return Err(TimeError::InvalidOperation(format!(
                "Unknown time unit: {unit}"
            )))
        }
    };

    dt.checked_add_signed(duration)
        .ok_or_else(|| TimeError::InvalidOperation("Date arithmetic overflow".to_string()))
}

/// Subtract duration from datetime
///
/// # Errors
///
/// Returns `TimeError::InvalidOperation` if the unit is unknown or arithmetic overflows
pub fn subtract_duration(dt: &DateTime<Utc>, amount: i64, unit: &str) -> TimeResult<DateTime<Utc>> {
    add_duration(dt, -amount, unit)
}

/// Calculate duration between two datetimes
#[must_use]
pub fn duration_between(start: &DateTime<Utc>, end: &DateTime<Utc>) -> Duration {
    end.signed_duration_since(start)
}

/// Get start of day for a datetime
#[must_use]
pub fn start_of_day(dt: &DateTime<Utc>) -> DateTime<Utc> {
    dt.date_naive()
        .and_hms_opt(0, 0, 0)
        .map_or(*dt, |nd| DateTime::from_naive_utc_and_offset(nd, Utc))
}

/// Get end of day for a datetime
#[must_use]
pub fn end_of_day(dt: &DateTime<Utc>) -> DateTime<Utc> {
    dt.date_naive()
        .and_hms_nano_opt(23, 59, 59, 999_999_999)
        .map_or(*dt, |nd| DateTime::from_naive_utc_and_offset(nd, Utc))
}

/// Get weekday name
#[must_use]
pub fn weekday_name(dt: &DateTime<Utc>) -> &'static str {
    match dt.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

/// Check if year is leap year
#[must_use]
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get days in month
#[must_use]
pub fn days_in_month(year: i32, month: u32) -> Option<u32> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}

/// Format duration in human readable format
#[must_use]
pub fn format_duration(duration: &Duration) -> String {
    let total_seconds = duration.num_seconds().abs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days} day{}", if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!("{hours} hour{}", if hours == 1 { "" } else { "s" }));
    }
    if minutes > 0 {
        parts.push(format!(
            "{minutes} minute{}",
            if minutes == 1 { "" } else { "s" }
        ));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!(
            "{seconds} second{}",
            if seconds == 1 { "" } else { "s" }
        ));
    }

    if duration.num_seconds() < 0 {
        format!("{} ago", parts.join(", "))
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_parse_datetime_formats() {
        let test_dates = vec![
            "2024-01-15T10:30:00Z",
            "2024-01-15 10:30:00",
            "2024-01-15",
            "2024/01/15",
            "15/01/2024",
            "01/15/2024",
            "15-01-2024",
            "15.01.2024",
            "January 15, 2024",
            "Jan 15, 2024",
        ];

        for date_str in test_dates {
            let result = parse_datetime(date_str);
            assert!(result.is_ok(), "Failed to parse: {date_str}");
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_parse_timestamp() {
        let timestamp = 1_705_315_800; // 2024-01-15 10:30:00 UTC
        let result = parse_datetime(&timestamp.to_string()).unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 15);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_timezone_conversion() {
        let utc_time = parse_datetime("2024-01-15T10:30:00Z").unwrap();
        let ny_time = convert_timezone(&utc_time, "America/New_York").unwrap();
        let tokyo_time = convert_timezone(&utc_time, "Asia/Tokyo").unwrap();

        // NYC is UTC-5 in January
        assert_eq!(ny_time.hour(), 5);

        // Tokyo is UTC+9
        assert_eq!(tokyo_time.hour(), 19);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_date_arithmetic() {
        let dt = parse_datetime("2024-01-15T10:30:00Z").unwrap();

        let plus_1_day = add_duration(&dt, 1, "day").unwrap();
        assert_eq!(plus_1_day.day(), 16);

        let plus_2_hours = add_duration(&dt, 2, "hours").unwrap();
        assert_eq!(plus_2_hours.hour(), 12);

        let minus_30_mins = subtract_duration(&dt, 30, "minutes").unwrap();
        assert_eq!(minus_30_mins.hour(), 10);
        assert_eq!(minus_30_mins.minute(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), Some(29)); // Leap year
        assert_eq!(days_in_month(2023, 2), Some(28)); // Non-leap year
        assert_eq!(days_in_month(2024, 1), Some(31));
        assert_eq!(days_in_month(2024, 4), Some(30));
        assert_eq!(days_in_month(2024, 13), None); // Invalid month
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_format_duration() {
        let duration = Duration::seconds(3665); // 1 hour, 1 minute, 5 seconds
        assert_eq!(format_duration(&duration), "1 hour, 1 minute, 5 seconds");

        let duration = Duration::seconds(-3600); // 1 hour ago
        assert_eq!(format_duration(&duration), "1 hour ago");

        let duration = Duration::seconds(86400 * 2 + 3600 * 3); // 2 days, 3 hours
        assert_eq!(format_duration(&duration), "2 days, 3 hours");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_start_end_of_day() {
        let dt = parse_datetime("2024-01-15T14:30:45Z").unwrap();

        let start = start_of_day(&dt);
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);

        let end = end_of_day(&dt);
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
    }
}
