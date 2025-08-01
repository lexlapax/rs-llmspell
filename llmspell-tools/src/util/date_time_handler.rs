// ABOUTME: Date and time manipulation tool for parsing, converting, and calculating dates
// ABOUTME: Provides timezone conversion, date arithmetic, and multiple format support

//! Date and time handler tool
//!
//! This tool provides comprehensive date and time manipulation including:
//! - Parsing dates from multiple formats
//! - Timezone conversion with DST handling
//! - Date arithmetic operations
//! - Current date/time retrieval
//! - Date formatting

use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{tool_error, validation_error},
    params::{
        extract_optional_i64, extract_optional_string, extract_parameters, extract_required_string,
        extract_string_with_default,
    },
    response::ResponseBuilder,
    time::{
        add_duration, convert_timezone, days_in_month, duration_between, end_of_day,
        format_datetime, format_duration, is_leap_year, now_local, now_utc, parse_datetime,
        start_of_day, subtract_duration, weekday_name, DATE_FORMATS,
    },
};
use serde_json::{json, Value};

/// Date/time handler tool
#[derive(Debug, Clone)]
pub struct DateTimeHandlerTool {
    /// Tool metadata
    metadata: ComponentMetadata,
}

impl Default for DateTimeHandlerTool {
    fn default() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "datetime-handler".to_string(),
                "Date and time manipulation tool with parsing, timezone conversion, and arithmetic"
                    .to_string(),
            ),
        }
    }
}

impl DateTimeHandlerTool {
    /// Create a new date/time handler tool
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Process date/time operation
    async fn process_operation(&self, params: &Value) -> Result<Value> {
        let operation = extract_string_with_default(params, "operation", "parse");

        match operation {
            "parse" => {
                let input = extract_required_string(params, "input")?;

                let dt = parse_datetime(input).map_err(|e| {
                    tool_error(
                        format!("Failed to parse date: {}", e),
                        Some(self.metadata.name.clone()),
                    )
                })?;

                let output_format =
                    extract_string_with_default(params, "format", "%Y-%m-%dT%H:%M:%S%.fZ");

                let response = ResponseBuilder::success("parse")
                    .with_message("Date parsed successfully")
                    .with_result(json!({
                        "input": input,
                        "parsed": {
                            "utc": format_datetime(&dt, output_format),
                            "timestamp": dt.timestamp(),
                            "year": dt.year(),
                            "month": dt.month(),
                            "day": dt.day(),
                            "hour": dt.hour(),
                            "minute": dt.minute(),
                            "second": dt.second(),
                            "weekday": weekday_name(&dt),
                        },
                        "format_used": output_format
                    }))
                    .build();
                Ok(response)
            }
            "now" => {
                let timezone = extract_optional_string(params, "timezone");
                let format = extract_string_with_default(params, "format", "%Y-%m-%dT%H:%M:%S%.fZ");

                let (dt_str, tz_name) = if let Some(tz) = timezone {
                    let utc_now = now_utc();
                    let tz_time =
                        convert_timezone(&utc_now, tz).map_err(|e| LLMSpellError::Tool {
                            message: format!("Invalid timezone: {}", e),
                            tool_name: Some(self.metadata.name.clone()),
                            source: None,
                        })?;
                    (tz_time.format(format).to_string(), tz.to_string())
                } else {
                    let local = now_local();
                    (local.format(format).to_string(), "local".to_string())
                };

                let response = ResponseBuilder::success("now")
                    .with_message("Current time retrieved")
                    .with_result(json!({
                        "timezone": tz_name,
                        "datetime": dt_str,
                        "format": format
                    }))
                    .build();
                Ok(response)
            }
            "convert_timezone" => {
                let input = extract_required_string(params, "input")?;

                let target_tz = params
                    .get("target_timezone")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Missing 'target_timezone' parameter".to_string(),
                        field: Some("target_timezone".to_string()),
                    })?;

                let dt = parse_datetime(input).map_err(|e| {
                    tool_error(
                        format!("Failed to parse date: {}", e),
                        Some(self.metadata.name.clone()),
                    )
                })?;

                let converted =
                    convert_timezone(&dt, target_tz).map_err(|e| LLMSpellError::Tool {
                        message: format!("Failed to convert timezone: {}", e),
                        tool_name: Some(self.metadata.name.clone()),
                        source: None,
                    })?;

                let format = params
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("%Y-%m-%d %H:%M:%S %Z");

                let response = ResponseBuilder::success("convert_timezone")
                    .with_message("Timezone converted successfully")
                    .with_result(json!({
                        "input": input,
                        "target_timezone": target_tz,
                        "original": format_datetime(&dt, format),
                        "converted": converted.format(format).to_string(),
                    }))
                    .build();
                Ok(response)
            }
            "add" | "subtract" => {
                let input = extract_required_string(params, "input")?;

                let amount = extract_optional_i64(params, "amount").ok_or_else(|| {
                    validation_error(
                        "Missing or invalid 'amount' parameter",
                        Some("amount".to_string()),
                    )
                })?;

                let unit = extract_required_string(params, "unit")?;

                let dt = parse_datetime(input).map_err(|e| {
                    tool_error(
                        format!("Failed to parse date: {}", e),
                        Some(self.metadata.name.clone()),
                    )
                })?;

                let result = if operation == "add" {
                    add_duration(&dt, amount, unit)
                } else {
                    subtract_duration(&dt, amount, unit)
                }
                .map_err(|e| {
                    tool_error(
                        format!("Failed to {} duration: {}", operation, e),
                        Some(self.metadata.name.clone()),
                    )
                })?;

                let format = extract_string_with_default(params, "format", "%Y-%m-%dT%H:%M:%S%.fZ");

                let response = ResponseBuilder::success(operation)
                    .with_message(format!("{} operation completed", operation))
                    .with_result(json!({
                        "input": input,
                        "amount": amount,
                        "unit": unit,
                        "original": format_datetime(&dt, format),
                        "result": format_datetime(&result, format),
                    }))
                    .build();
                Ok(response)
            }
            "difference" => {
                let start = params
                    .get("start")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Missing 'start' parameter".to_string(),
                        field: Some("start".to_string()),
                    })?;

                let end = params.get("end").and_then(|v| v.as_str()).ok_or_else(|| {
                    LLMSpellError::Validation {
                        message: "Missing 'end' parameter".to_string(),
                        field: Some("end".to_string()),
                    }
                })?;

                let start_dt = parse_datetime(start).map_err(|e| LLMSpellError::Tool {
                    message: format!("Failed to parse start date: {}", e),
                    tool_name: Some(self.metadata.name.clone()),
                    source: None,
                })?;

                let end_dt = parse_datetime(end).map_err(|e| LLMSpellError::Tool {
                    message: format!("Failed to parse end date: {}", e),
                    tool_name: Some(self.metadata.name.clone()),
                    source: None,
                })?;

                let duration = duration_between(&start_dt, &end_dt);

                let response = ResponseBuilder::success("difference")
                    .with_message("Date difference calculated")
                    .with_result(json!({
                        "start": start,
                        "end": end,
                        "difference": {
                            "total_seconds": duration.num_seconds(),
                            "days": duration.num_days(),
                            "hours": duration.num_hours(),
                            "minutes": duration.num_minutes(),
                            "human_readable": format_duration(&duration),
                        }
                    }))
                    .build();
                Ok(response)
            }
            "info" => {
                let input = extract_required_string(params, "input")?;

                let dt = parse_datetime(input).map_err(|e| {
                    tool_error(
                        format!("Failed to parse date: {}", e),
                        Some(self.metadata.name.clone()),
                    )
                })?;

                let year = dt.year();
                let month = dt.month();
                let day_of_year = dt.ordinal();
                let week_of_year = dt.iso_week().week();

                let response = ResponseBuilder::success("info")
                    .with_message("Date information retrieved")
                    .with_result(json!({
                        "input": input,
                        "info": {
                            "year": year,
                            "month": month,
                            "day": dt.day(),
                            "hour": dt.hour(),
                            "minute": dt.minute(),
                            "second": dt.second(),
                            "weekday": weekday_name(&dt),
                            "day_of_year": day_of_year,
                            "week_of_year": week_of_year,
                            "is_leap_year": is_leap_year(year),
                            "days_in_month": days_in_month(year, month),
                            "start_of_day": format_datetime(&start_of_day(&dt), "%Y-%m-%dT%H:%M:%S%.fZ"),
                            "end_of_day": format_datetime(&end_of_day(&dt), "%Y-%m-%dT%H:%M:%S%.fZ"),
                        }
                    }))
                    .build();
                Ok(response)
            }
            "formats" => {
                // Show available date formats
                let response = ResponseBuilder::success("formats")
                    .with_message("Available date formats")
                    .with_result(json!({
                        "available_formats": DATE_FORMATS,
                        "example_formats": {
                            "ISO8601": "%Y-%m-%dT%H:%M:%S%.fZ",
                            "RFC3339": "%Y-%m-%d %H:%M:%S%:z",
                            "RFC2822": "%a, %d %b %Y %H:%M:%S %z",
                            "Unix timestamp": "timestamp",
                            "Human readable": "%B %d, %Y at %I:%M %p",
                            "Date only": "%Y-%m-%d",
                            "Time only": "%H:%M:%S",
                        }
                    }))
                    .build();
                Ok(response)
            }
            _ => Err(validation_error(
                format!("Unknown operation: {}", operation),
                Some("operation".to_string()),
            )),
        }
    }
}

#[async_trait]
impl BaseAgent for DateTimeHandlerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Process the operation
        let result = self.process_operation(params).await?;

        // Return the result as JSON formatted text
        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&result).unwrap(),
        ))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(validation_error(
                "Input prompt cannot be empty",
                Some("prompt".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Date/time operation error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for DateTimeHandlerTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "datetime_handler".to_string(),
            "Date and time manipulation tool with parsing, timezone conversion, and arithmetic"
                .to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: parse, now, convert_timezone, add, subtract, difference, info, formats".to_string(),
            required: true,
            default: Some(json!("parse")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input date/time string (for parse, convert_timezone, add, subtract, info operations)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "timezone".to_string(),
            param_type: ParameterType::String,
            description: "Timezone name (e.g., 'America/New_York', 'Asia/Tokyo') for 'now' operation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "target_timezone".to_string(),
            param_type: ParameterType::String,
            description: "Target timezone for conversion".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "amount".to_string(),
            param_type: ParameterType::Number,
            description: "Amount to add or subtract".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "unit".to_string(),
            param_type: ParameterType::String,
            description: "Time unit: seconds, minutes, hours, days, weeks".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "start".to_string(),
            param_type: ParameterType::String,
            description: "Start date/time for difference calculation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "end".to_string(),
            param_type: ParameterType::String,
            description: "End date/time for difference calculation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "format".to_string(),
            param_type: ParameterType::String,
            description: "Output format string (strftime format)".to_string(),
            required: false,
            default: Some(json!("%Y-%m-%dT%H:%M:%S%.fZ")),
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::strict()
            .with_memory_limit(10 * 1024 * 1024) // 10MB
            .with_cpu_limit(1000) // 1 second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_parse_operation() {
        let tool = DateTimeHandlerTool::new();

        let input = AgentInput::text("parse date").with_parameter(
            "parameters",
            json!({
                "operation": "parse",
                "input": "2024-01-15T10:30:00Z"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["parsed"]["year"], 2024);
        assert_eq!(output["result"]["parsed"]["month"], 1);
        assert_eq!(output["result"]["parsed"]["day"], 15);
        assert_eq!(output["result"]["parsed"]["hour"], 10);
        assert_eq!(output["result"]["parsed"]["minute"], 30);
        assert_eq!(output["result"]["parsed"]["weekday"], "Monday");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_now_operation() {
        let tool = DateTimeHandlerTool::new();

        let input = AgentInput::text("get current time").with_parameter(
            "parameters",
            json!({
                "operation": "now",
                "timezone": "UTC"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["timezone"], "UTC");
        assert!(output["result"]["datetime"].is_string());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timezone_conversion() {
        let tool = DateTimeHandlerTool::new();

        let input = AgentInput::text("convert timezone").with_parameter(
            "parameters",
            json!({
                "operation": "convert_timezone",
                "input": "2024-01-15T10:30:00Z",
                "target_timezone": "America/New_York"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["target_timezone"], "America/New_York");
        assert!(output["result"]["converted"]
            .as_str()
            .unwrap()
            .contains("EST"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_date_arithmetic() {
        let tool = DateTimeHandlerTool::new();

        // Test add operation
        let input = AgentInput::text("add days").with_parameter(
            "parameters",
            json!({
                "operation": "add",
                "input": "2024-01-15T10:30:00Z",
                "amount": 5,
                "unit": "days"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert!(output["result"]["result"]
            .as_str()
            .unwrap()
            .contains("2024-01-20T10:30:00"));

        // Test subtract operation
        let input = AgentInput::text("subtract hours").with_parameter(
            "parameters",
            json!({
                "operation": "subtract",
                "input": "2024-01-15T10:30:00Z",
                "amount": 2,
                "unit": "hours"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert!(output["result"]["result"]
            .as_str()
            .unwrap()
            .contains("2024-01-15T08:30:00"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_date_difference() {
        let tool = DateTimeHandlerTool::new();

        let input = AgentInput::text("calculate difference").with_parameter(
            "parameters",
            json!({
                "operation": "difference",
                "start": "2024-01-15T10:30:00Z",
                "end": "2024-01-20T15:45:00Z"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["difference"]["days"], 5);
        assert!(output["result"]["difference"]["human_readable"]
            .as_str()
            .unwrap()
            .contains("5 days"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_date_info() {
        let tool = DateTimeHandlerTool::new();

        let input = AgentInput::text("get date info").with_parameter(
            "parameters",
            json!({
                "operation": "info",
                "input": "2024-02-29T12:00:00Z" // Leap year date
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["info"]["is_leap_year"], true);
        assert_eq!(output["result"]["info"]["days_in_month"], 29);
        assert_eq!(output["result"]["info"]["weekday"], "Thursday");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = DateTimeHandlerTool::new();

        assert_eq!(tool.metadata().name, "datetime-handler");
        assert!(tool
            .metadata()
            .description
            .contains("Date and time manipulation"));
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
    }
}
