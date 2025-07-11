//! ABOUTME: CSV analysis tool with streaming support, encoding detection, and multiple export formats
//! ABOUTME: Provides memory-efficient CSV processing with Parquet and Excel export capabilities

use arrow::array::{ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chardetng::EncodingDetector;
use csv::{ReaderBuilder, Writer};
use encoding_rs::Encoding;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::{
    extract_optional_string, extract_parameters, extract_required_string, response::ResponseBuilder,
};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use xlsxwriter::Workbook;

/// CSV analysis operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsvOperation {
    /// Analyze CSV structure and statistics
    Analyze,
    /// Convert CSV to another format
    Convert,
    /// Filter rows based on conditions
    Filter,
    /// Transform CSV data
    Transform,
    /// Validate CSV structure
    Validate,
    /// Sample rows from CSV
    Sample,
}

impl std::fmt::Display for CsvOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvOperation::Analyze => write!(f, "analyze"),
            CsvOperation::Convert => write!(f, "convert"),
            CsvOperation::Filter => write!(f, "filter"),
            CsvOperation::Transform => write!(f, "transform"),
            CsvOperation::Validate => write!(f, "validate"),
            CsvOperation::Sample => write!(f, "sample"),
        }
    }
}

impl std::str::FromStr for CsvOperation {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "analyze" => Ok(CsvOperation::Analyze),
            "convert" => Ok(CsvOperation::Convert),
            "filter" => Ok(CsvOperation::Filter),
            "transform" => Ok(CsvOperation::Transform),
            "validate" => Ok(CsvOperation::Validate),
            "sample" => Ok(CsvOperation::Sample),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown CSV operation: {}", s),
                field: Some("operation".to_string()),
            }),
        }
    }
}

/// CSV analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvAnalyzerConfig {
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Enable automatic encoding detection
    pub auto_detect_encoding: bool,
    /// Default delimiter
    pub default_delimiter: u8,
    /// Enable header detection
    pub detect_headers: bool,
    /// Maximum rows to analyze for statistics
    pub max_analysis_rows: usize,
    /// Sample size for type inference
    pub type_inference_sample_size: usize,
    /// Chunk size for streaming operations
    pub stream_chunk_size: usize,
}

impl Default for CsvAnalyzerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 500 * 1024 * 1024, // 500MB
            auto_detect_encoding: true,
            default_delimiter: b',',
            detect_headers: true,
            max_analysis_rows: 10000,
            type_inference_sample_size: 1000,
            stream_chunk_size: 1000, // Process 1000 rows at a time
        }
    }
}

/// Column data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnType {
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    String,
    Mixed,
}

/// Column statistics for streaming analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingColumnStats {
    pub name: String,
    pub data_type: ColumnType,
    pub null_count: usize,
    pub unique_values: HashMap<String, usize>, // Track unique values with counts
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub sum: Option<f64>,         // For mean calculation
    pub sum_squares: Option<f64>, // For std dev calculation
    pub count: usize,
    pub sample_values: Vec<String>,
}

impl StreamingColumnStats {
    fn new(name: String) -> Self {
        Self {
            name,
            data_type: ColumnType::String,
            null_count: 0,
            unique_values: HashMap::new(),
            min_value: None,
            max_value: None,
            sum: None,
            sum_squares: None,
            count: 0,
            sample_values: Vec::new(),
        }
    }

    fn update(&mut self, value: &str) {
        if value.trim().is_empty() {
            self.null_count += 1;
            return;
        }

        self.count += 1;

        // Update unique values (limit to prevent memory explosion)
        if self.unique_values.len() < 10000 {
            *self.unique_values.entry(value.to_string()).or_insert(0) += 1;
        }

        // Update min/max
        match (&self.min_value, &self.max_value) {
            (None, None) => {
                self.min_value = Some(value.to_string());
                self.max_value = Some(value.to_string());
            }
            (Some(min), Some(max)) => {
                if value < min.as_str() {
                    self.min_value = Some(value.to_string());
                }
                if value > max.as_str() {
                    self.max_value = Some(value.to_string());
                }
            }
            _ => {}
        }

        // Update numeric stats if applicable
        if let Ok(num) = value.parse::<f64>() {
            self.sum = Some(self.sum.unwrap_or(0.0) + num);
            self.sum_squares = Some(self.sum_squares.unwrap_or(0.0) + num * num);
        }

        // Collect samples
        if self.sample_values.len() < 5 {
            self.sample_values.push(value.to_string());
        }
    }

    fn finalize(&self) -> ColumnStats {
        let unique_count = self.unique_values.len();

        let (mean, std_dev) = if let (Some(sum), Some(sum_squares)) = (self.sum, self.sum_squares) {
            if self.count > 0 {
                let mean = sum / self.count as f64;
                let variance = (sum_squares / self.count as f64) - (mean * mean);
                let std_dev = if variance > 0.0 { variance.sqrt() } else { 0.0 };
                (Some(mean), Some(std_dev))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        ColumnStats {
            name: self.name.clone(),
            data_type: self.data_type,
            null_count: self.null_count,
            unique_count,
            min_value: self.min_value.clone(),
            max_value: self.max_value.clone(),
            mean,
            median: None, // Would require storing all values
            std_dev,
            sample_values: self.sample_values.clone(),
        }
    }
}

/// Column statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub name: String,
    pub data_type: ColumnType,
    pub null_count: usize,
    pub unique_count: usize,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std_dev: Option<f64>,
    pub sample_values: Vec<String>,
}

/// CSV analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvAnalysisResult {
    pub row_count: usize,
    pub column_count: usize,
    pub encoding: String,
    pub delimiter: char,
    pub has_headers: bool,
    pub columns: Vec<ColumnStats>,
    pub file_size_bytes: usize,
    pub parse_errors: Vec<String>,
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    JsonLines,
    Parquet,
    Excel,
    Tsv,
}

impl std::str::FromStr for ExportFormat {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "jsonlines" | "jsonl" => Ok(ExportFormat::JsonLines),
            "parquet" => Ok(ExportFormat::Parquet),
            "excel" | "xlsx" => Ok(ExportFormat::Excel),
            "tsv" => Ok(ExportFormat::Tsv),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown export format: {}", s),
                field: Some("format".to_string()),
            }),
        }
    }
}

/// CSV analyzer tool
pub struct CsvAnalyzerTool {
    metadata: ComponentMetadata,
    config: CsvAnalyzerConfig,
}

impl CsvAnalyzerTool {
    pub fn new(config: CsvAnalyzerConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "csv-analyzer-tool".to_string(),
                "Analyze and process CSV files with streaming support".to_string(),
            ),
            config,
        }
    }

    /// Parse parameters from input
    fn parse_parameters(&self, params: &Value) -> Result<(CsvOperation, String, Option<Value>)> {
        let operation_str = extract_optional_string(params, "operation").unwrap_or("analyze");
        let operation: CsvOperation = operation_str.parse()?;

        let content = extract_required_string(params, "input")?.to_string();

        let options = params.get("options").cloned();

        Ok((operation, content, options))
    }

    /// Detect encoding of CSV content
    fn detect_encoding(&self, content: &[u8]) -> &'static Encoding {
        if !self.config.auto_detect_encoding {
            return encoding_rs::UTF_8;
        }

        let mut detector = EncodingDetector::new();
        detector.feed(content, true);
        detector.guess(None, true)
    }

    /// Detect delimiter from content sample
    fn detect_delimiter(&self, content: &str) -> Result<u8> {
        let sample = content.lines().take(10).collect::<Vec<_>>().join("\n");
        let delimiters = [b',', b';', b'\t', b'|'];
        let mut delimiter_scores = HashMap::new();

        for &delim in &delimiters {
            let mut reader = ReaderBuilder::new()
                .delimiter(delim)
                .has_headers(false)
                .from_reader(sample.as_bytes());

            let mut field_counts = Vec::new();
            for record in reader.records().take(5).flatten() {
                field_counts.push(record.len());
            }

            if field_counts.len() > 1 {
                let all_same = field_counts.windows(2).all(|w| w[0] == w[1]);
                let avg_fields =
                    field_counts.iter().sum::<usize>() as f64 / field_counts.len() as f64;

                if all_same && avg_fields > 1.0 {
                    delimiter_scores.insert(delim, avg_fields);
                }
            }
        }

        delimiter_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(delim, _)| delim)
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Could not detect CSV delimiter".to_string(),
                field: Some("delimiter".to_string()),
            })
    }

    /// Infer column data type from values
    fn infer_column_type(&self, values: &[&str]) -> ColumnType {
        let mut type_counts: HashMap<ColumnType, usize> = HashMap::new();

        for value in values.iter().take(self.config.type_inference_sample_size) {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }

            let detected_type = if trimmed.parse::<i64>().is_ok() {
                ColumnType::Integer
            } else if trimmed.parse::<f64>().is_ok() {
                ColumnType::Float
            } else if matches!(
                trimmed.to_lowercase().as_str(),
                "true" | "false" | "yes" | "no" | "y" | "n" | "1" | "0"
            ) {
                ColumnType::Boolean
            } else if chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").is_ok() {
                ColumnType::Date
            } else if chrono::DateTime::parse_from_rfc3339(trimmed).is_ok() {
                ColumnType::DateTime
            } else {
                ColumnType::String
            };

            *type_counts.entry(detected_type).or_insert(0) += 1;
        }

        // Return the most common type
        type_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(typ, _)| typ)
            .unwrap_or(ColumnType::String)
    }

    /// Analyze CSV content with streaming
    async fn analyze_csv_streaming(&self, content: &str) -> Result<CsvAnalysisResult> {
        debug!("Analyzing CSV content with streaming");

        let content_bytes = content.as_bytes();
        let encoding = self.detect_encoding(content_bytes);

        // Convert to UTF-8 if needed
        let (utf8_content, _, _) = encoding.decode(content_bytes);

        // Try to detect delimiter
        let delimiter = self.detect_delimiter(&utf8_content)?;

        // Create CSV reader
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(utf8_content.as_bytes());

        // Get headers
        let headers = reader.headers().map_err(Self::csv_error)?.clone();
        let column_count = headers.len();

        // Initialize streaming column stats
        let mut column_stats: Vec<StreamingColumnStats> = headers
            .iter()
            .enumerate()
            .map(|(idx, header)| {
                let name = if header.is_empty() {
                    format!("Column_{}", idx + 1)
                } else {
                    header.to_string()
                };
                StreamingColumnStats::new(name)
            })
            .collect();

        let mut row_count = 0;
        let mut parse_errors = Vec::new();
        let mut type_samples: Vec<Vec<String>> = vec![Vec::new(); column_count];

        // Process rows in chunks
        for (idx, result) in reader.records().enumerate() {
            if idx >= self.config.max_analysis_rows {
                break;
            }

            match result {
                Ok(record) => {
                    for (col_idx, field) in record.iter().enumerate() {
                        if col_idx < column_count {
                            column_stats[col_idx].update(field);

                            // Collect samples for type inference
                            if type_samples[col_idx].len() < self.config.type_inference_sample_size
                            {
                                type_samples[col_idx].push(field.to_string());
                            }
                        }
                    }
                    row_count += 1;
                }
                Err(e) => {
                    parse_errors.push(format!("Row {}: {}", idx + 2, e));
                    if parse_errors.len() >= 10 {
                        parse_errors.push("... more errors omitted".to_string());
                        break;
                    }
                }
            }
        }

        // Infer types and finalize stats
        for (col_idx, stats) in column_stats.iter_mut().enumerate() {
            let value_refs: Vec<&str> = type_samples[col_idx].iter().map(|s| s.as_str()).collect();
            stats.data_type = self.infer_column_type(&value_refs);
        }

        let columns: Vec<ColumnStats> = column_stats.iter().map(|s| s.finalize()).collect();

        Ok(CsvAnalysisResult {
            row_count,
            column_count,
            encoding: encoding.name().to_string(),
            delimiter: delimiter as char,
            has_headers: self.config.detect_headers,
            columns,
            file_size_bytes: content_bytes.len(),
            parse_errors,
        })
    }

    /// Convert CSV to Parquet format
    async fn convert_to_parquet(&self, content: &str) -> Result<Vec<u8>> {
        debug!("Converting CSV to Parquet");

        let delimiter = self.detect_delimiter(content)?;
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

        let headers = reader.headers().map_err(Self::csv_error)?.clone();

        // First pass: detect types
        let mut type_samples: Vec<Vec<String>> = vec![Vec::new(); headers.len()];
        for record in reader
            .records()
            .take(self.config.type_inference_sample_size)
            .flatten()
        {
            for (idx, field) in record.iter().enumerate() {
                if idx < headers.len() {
                    type_samples[idx].push(field.to_string());
                }
            }
        }

        // Infer column types
        let mut fields = Vec::new();
        let mut column_types = Vec::new();

        for (idx, header) in headers.iter().enumerate() {
            let value_refs: Vec<&str> = type_samples[idx].iter().map(|s| s.as_str()).collect();
            let col_type = self.infer_column_type(&value_refs);
            column_types.push(col_type);

            let arrow_type = match col_type {
                ColumnType::Integer => DataType::Int64,
                ColumnType::Float => DataType::Float64,
                ColumnType::Boolean => DataType::Boolean,
                _ => DataType::Utf8,
            };

            fields.push(Field::new(header, arrow_type, true));
        }

        let schema = Arc::new(Schema::new(fields));

        // Second pass: convert data
        let mut output = Vec::new();
        let props = WriterProperties::builder().build();
        let mut writer =
            ArrowWriter::try_new(&mut output, schema.clone(), Some(props)).map_err(|e| {
                LLMSpellError::Internal {
                    message: format!("Failed to create Parquet writer: {}", e),
                    source: None,
                }
            })?;

        // Reset reader
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(content.as_bytes());
        reader.headers().map_err(Self::csv_error)?; // Skip headers

        // Process in chunks
        let mut chunk_data: Vec<Vec<String>> = vec![Vec::new(); headers.len()];
        let mut chunk_size = 0;

        for record in reader.records().flatten() {
            for (idx, field) in record.iter().enumerate() {
                if idx < headers.len() {
                    chunk_data[idx].push(field.to_string());
                }
            }
            chunk_size += 1;

            if chunk_size >= self.config.stream_chunk_size {
                write_parquet_chunk(&mut writer, &schema, &column_types, &chunk_data)?;
                chunk_data = vec![Vec::new(); headers.len()];
                chunk_size = 0;
            }
        }

        // Write remaining data
        if chunk_size > 0 {
            write_parquet_chunk(&mut writer, &schema, &column_types, &chunk_data)?;
        }

        writer.close().map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to close Parquet writer: {}", e),
            source: None,
        })?;

        Ok(output)
    }

    /// Convert CSV to Excel format
    async fn convert_to_excel(&self, content: &str) -> Result<Vec<u8>> {
        debug!("Converting CSV to Excel");

        let delimiter = self.detect_delimiter(content)?;
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

        let headers = reader.headers().map_err(Self::csv_error)?.clone();

        // Create a temporary file for the Excel output
        let temp_path = std::env::temp_dir().join(format!(
            "csv_export_{}.xlsx",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        {
            let workbook =
                Workbook::new(temp_path.to_str().unwrap()).map_err(|e| LLMSpellError::Tool {
                    message: format!("Failed to create Excel workbook: {}", e),
                    tool_name: Some("csv_analyzer".to_string()),
                    source: None,
                })?;

            let mut worksheet = workbook
                .add_worksheet(None)
                .map_err(|e| LLMSpellError::Tool {
                    message: format!("Failed to add worksheet: {}", e),
                    tool_name: Some("csv_analyzer".to_string()),
                    source: None,
                })?;

            // Write headers
            for (col, header) in headers.iter().enumerate() {
                worksheet
                    .write_string(0, col as u16, header, None)
                    .map_err(|e| LLMSpellError::Tool {
                        message: format!("Failed to write header: {}", e),
                        tool_name: Some("csv_analyzer".to_string()),
                        source: None,
                    })?;
            }

            // Write data
            let mut row_idx = 1;
            for record in reader.records().flatten() {
                for (col_idx, field) in record.iter().enumerate() {
                    // Try to write as number first
                    if let Ok(num) = field.parse::<f64>() {
                        worksheet
                            .write_number(row_idx, col_idx as u16, num, None)
                            .map_err(|e| LLMSpellError::Tool {
                                message: format!("Failed to write number: {}", e),
                                tool_name: Some("csv_analyzer".to_string()),
                                source: None,
                            })?;
                    } else {
                        worksheet
                            .write_string(row_idx, col_idx as u16, field, None)
                            .map_err(|e| LLMSpellError::Tool {
                                message: format!("Failed to write string: {}", e),
                                tool_name: Some("csv_analyzer".to_string()),
                                source: None,
                            })?;
                    }
                }
                row_idx += 1;
            }

            workbook.close().map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to close workbook: {}", e),
                tool_name: Some("csv_analyzer".to_string()),
                source: None,
            })?;
        }

        // Read the file back into memory
        let output = tokio::fs::read(&temp_path)
            .await
            .map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to read Excel file: {}", e),
                tool_name: Some("csv_analyzer".to_string()),
                source: None,
            })?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_path).await;

        Ok(output)
    }

    /// Convert CSV to another format with streaming support
    async fn convert_csv_streaming(&self, content: &str, format: ExportFormat) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Parquet => self.convert_to_parquet(content).await,
            ExportFormat::Excel => self.convert_to_excel(content).await,
            ExportFormat::Json | ExportFormat::JsonLines | ExportFormat::Tsv => {
                // These formats return strings, so convert to bytes
                let result = self.convert_csv_text_format(content, format).await?;
                Ok(result.into_bytes())
            }
        }
    }

    /// Convert CSV to text-based formats
    async fn convert_csv_text_format(&self, content: &str, format: ExportFormat) -> Result<String> {
        debug!("Converting CSV to {:?}", format);

        let delimiter = self.detect_delimiter(content)?;
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

        match format {
            ExportFormat::Json => {
                let headers = reader.headers().map_err(Self::csv_error)?.clone();
                let mut records = Vec::new();

                for result in reader.records() {
                    let record = result.map_err(Self::csv_error)?;
                    let mut row_map = serde_json::Map::new();

                    for (idx, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(idx) {
                            // Try to parse as number
                            let value = if let Ok(num) = field.parse::<f64>() {
                                Value::Number(serde_json::Number::from_f64(num).unwrap())
                            } else if let Ok(bool_val) = field.parse::<bool>() {
                                Value::Bool(bool_val)
                            } else {
                                Value::String(field.to_string())
                            };
                            row_map.insert(header.to_string(), value);
                        }
                    }

                    records.push(Value::Object(row_map));
                }

                serde_json::to_string_pretty(&records).map_err(|e| LLMSpellError::Internal {
                    message: format!("Failed to serialize to JSON: {}", e),
                    source: None,
                })
            }
            ExportFormat::JsonLines => {
                let headers = reader.headers().map_err(Self::csv_error)?.clone();
                let mut output = String::new();

                for result in reader.records() {
                    let record = result.map_err(Self::csv_error)?;
                    let mut row_map = serde_json::Map::new();

                    for (idx, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(idx) {
                            // Try to parse as number
                            let value = if let Ok(num) = field.parse::<f64>() {
                                Value::Number(serde_json::Number::from_f64(num).unwrap())
                            } else if let Ok(bool_val) = field.parse::<bool>() {
                                Value::Bool(bool_val)
                            } else {
                                Value::String(field.to_string())
                            };
                            row_map.insert(header.to_string(), value);
                        }
                    }

                    let json_line =
                        serde_json::to_string(&row_map).map_err(|e| LLMSpellError::Internal {
                            message: format!("Failed to serialize to JSON: {}", e),
                            source: None,
                        })?;
                    output.push_str(&json_line);
                    output.push('\n');
                }

                Ok(output)
            }
            ExportFormat::Tsv => {
                let mut wtr = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .from_writer(vec![]);

                let headers = reader.headers().map_err(Self::csv_error)?.clone();

                // Write headers
                let header_vec: Vec<&str> = headers.iter().collect();
                wtr.write_record(&header_vec).map_err(Self::csv_error)?;

                // Write records
                for result in reader.records() {
                    let record = result.map_err(Self::csv_error)?;
                    wtr.write_record(&record).map_err(Self::csv_error)?;
                }

                let data = wtr.into_inner().map_err(|e| LLMSpellError::Internal {
                    message: format!("Failed to write TSV: {}", e),
                    source: None,
                })?;

                String::from_utf8(data).map_err(|e| LLMSpellError::Internal {
                    message: format!("Failed to convert TSV to string: {}", e),
                    source: None,
                })
            }
            _ => unreachable!("Other formats handled elsewhere"),
        }
    }

    /// Filter CSV rows with streaming
    async fn filter_csv_streaming(&self, content: &str, filter_expr: &str) -> Result<String> {
        debug!("Filtering CSV with expression: {}", filter_expr);

        // Basic filter syntax: "column_name == value" or "column_name > value"
        let parts: Vec<&str> = filter_expr.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(LLMSpellError::Validation {
                message: "Filter expression must be in format: 'column operator value'".to_string(),
                field: Some("filter".to_string()),
            });
        }

        let column_name = parts[0];
        let operator = parts[1];
        let filter_value = parts[2].trim_matches('"');

        let delimiter = self.detect_delimiter(content)?;
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

        let headers = reader.headers().map_err(Self::csv_error)?.clone();
        let column_index = headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| LLMSpellError::Validation {
                message: format!("Column '{}' not found", column_name),
                field: Some("column".to_string()),
            })?;

        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(&headers).map_err(Self::csv_error)?;

        for result in reader.records() {
            let record = result.map_err(Self::csv_error)?;
            if let Some(field) = record.get(column_index) {
                let matches = match operator {
                    "==" => field == filter_value,
                    "!=" => field != filter_value,
                    ">" => {
                        if let (Ok(field_num), Ok(filter_num)) =
                            (field.parse::<f64>(), filter_value.parse::<f64>())
                        {
                            field_num > filter_num
                        } else {
                            field > filter_value
                        }
                    }
                    "<" => {
                        if let (Ok(field_num), Ok(filter_num)) =
                            (field.parse::<f64>(), filter_value.parse::<f64>())
                        {
                            field_num < filter_num
                        } else {
                            field < filter_value
                        }
                    }
                    ">=" => {
                        if let (Ok(field_num), Ok(filter_num)) =
                            (field.parse::<f64>(), filter_value.parse::<f64>())
                        {
                            field_num >= filter_num
                        } else {
                            field >= filter_value
                        }
                    }
                    "<=" => {
                        if let (Ok(field_num), Ok(filter_num)) =
                            (field.parse::<f64>(), filter_value.parse::<f64>())
                        {
                            field_num <= filter_num
                        } else {
                            field <= filter_value
                        }
                    }
                    _ => {
                        return Err(LLMSpellError::Validation {
                            message: format!("Unknown operator: {}", operator),
                            field: Some("operator".to_string()),
                        })
                    }
                };

                if matches {
                    wtr.write_record(&record).map_err(Self::csv_error)?;
                }
            }
        }

        let data = wtr.into_inner().map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to write CSV: {}", e),
            source: None,
        })?;

        String::from_utf8(data).map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to convert CSV to string: {}", e),
            source: None,
        })
    }

    /// Sample CSV rows
    async fn sample_csv(&self, content: &str, sample_size: usize) -> Result<String> {
        debug!("Sampling {} rows from CSV", sample_size);

        let delimiter = self.detect_delimiter(content)?;
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

        let headers = reader.headers().map_err(Self::csv_error)?.clone();

        let mut wtr = Writer::from_writer(vec![]);
        wtr.write_record(&headers).map_err(Self::csv_error)?;

        for (idx, result) in reader.records().enumerate() {
            if idx >= sample_size {
                break;
            }
            let record = result.map_err(Self::csv_error)?;
            wtr.write_record(&record).map_err(Self::csv_error)?;
        }

        let data = wtr.into_inner().map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to write CSV: {}", e),
            source: None,
        })?;

        String::from_utf8(data).map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to convert CSV to string: {}", e),
            source: None,
        })
    }

    /// Transform CSV data based on transformation rules
    async fn transform_csv(&self, content: &str, options: &Value) -> Result<String> {
        info!("Transforming CSV data");

        // Parse transformation options
        let add_columns = options.get("add_columns").and_then(|v| v.as_object());
        let rename_columns = options.get("rename_columns").and_then(|v| v.as_object());

        // Parse CSV
        let encoding = self.detect_encoding(content.as_bytes());
        let (decoded, _, _) = encoding.decode(content.as_bytes());
        let mut reader = ReaderBuilder::new()
            .delimiter(self.config.default_delimiter)
            .from_reader(decoded.as_bytes());

        // Get headers
        let headers = reader.headers().map_err(Self::csv_error)?.clone();
        let mut new_headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

        // Apply column renames
        if let Some(renames) = rename_columns {
            for (old_name, new_name) in renames {
                if let Some(new_name_str) = new_name.as_str() {
                    for header in &mut new_headers {
                        if header == old_name {
                            *header = new_name_str.to_string();
                        }
                    }
                }
            }
        }

        // Add new columns
        let mut new_column_names = Vec::new();
        let mut new_column_expressions = Vec::new();
        if let Some(additions) = add_columns {
            for (name, expr) in additions {
                new_headers.push(name.clone());
                new_column_names.push(name.clone());
                if let Some(expr_str) = expr.as_str() {
                    new_column_expressions.push(expr_str.to_string());
                }
            }
        }

        // Write transformed CSV
        let mut output = Vec::new();
        {
            let mut writer = Writer::from_writer(&mut output);

            // Write new headers
            writer.write_record(&new_headers).map_err(Self::csv_error)?;

            // Process each row
            for result in reader.records() {
                let record = result.map_err(Self::csv_error)?;
                let mut new_record: Vec<String> = Vec::new();

                // Copy existing fields (with renames applied)
                for field in record.iter() {
                    new_record.push(field.to_string());
                }

                // Calculate new columns
                for expr in new_column_expressions.iter() {
                    // Simple expression evaluation (supports basic arithmetic)
                    let value = self.evaluate_expression(expr, &headers, &record)?;
                    new_record.push(value);
                }

                writer.write_record(&new_record).map_err(Self::csv_error)?;
            }

            writer.flush().map_err(|e| LLMSpellError::Internal {
                message: format!("Failed to flush CSV writer: {}", e),
                source: None,
            })?;
        }

        String::from_utf8(output).map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to convert transformed CSV to string: {}", e),
            source: None,
        })
    }

    /// Evaluate a simple arithmetic expression for CSV transformation
    fn evaluate_expression(
        &self,
        expr: &str,
        headers: &csv::StringRecord,
        record: &csv::StringRecord,
    ) -> Result<String> {
        // Simple expression evaluator for basic arithmetic
        // Supports: column_name * column_name, column_name + number, etc.

        // For now, implement a simple case for multiplication
        if expr.contains(" * ") {
            let parts: Vec<&str> = expr.split(" * ").collect();
            if parts.len() == 2 {
                let left_val = self.get_column_value(parts[0].trim(), headers, record)?;
                let right_val = self.get_column_value(parts[1].trim(), headers, record)?;

                if let (Ok(left), Ok(right)) = (left_val.parse::<f64>(), right_val.parse::<f64>()) {
                    return Ok(format!("{:.2}", left * right));
                }
            }
        }

        // Default: return the expression as-is
        Ok(expr.to_string())
    }

    /// Get column value by name or return literal value
    fn get_column_value(
        &self,
        name: &str,
        headers: &csv::StringRecord,
        record: &csv::StringRecord,
    ) -> Result<String> {
        // Try to find column by name
        for (i, header) in headers.iter().enumerate() {
            if header == name {
                return Ok(record.get(i).unwrap_or("").to_string());
            }
        }

        // If not a column name, return as literal
        Ok(name.to_string())
    }

    /// Validate CSV data against rules
    async fn validate_csv(&self, content: &str, options: &Value) -> Result<Value> {
        info!("Validating CSV data");

        let rules = options
            .get("rules")
            .and_then(|v| v.as_object())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Validate operation requires 'rules' in options".to_string(),
                field: Some("rules".to_string()),
            })?;

        // Parse CSV
        let encoding = self.detect_encoding(content.as_bytes());
        let (decoded, _, _) = encoding.decode(content.as_bytes());
        let mut reader = ReaderBuilder::new()
            .delimiter(self.config.default_delimiter)
            .from_reader(decoded.as_bytes());

        let headers = reader.headers().map_err(Self::csv_error)?.clone();
        let mut errors = Vec::new();
        let mut row_number = 1; // Header is row 0

        for result in reader.records() {
            let record = result.map_err(Self::csv_error)?;

            // Check each rule
            for (column_name, rule) in rules {
                if let Some(rule_str) = rule.as_str() {
                    // Find column index
                    if let Some(col_index) = headers.iter().position(|h| h == column_name) {
                        if let Some(value) = record.get(col_index) {
                            if let Some(error) =
                                self.validate_field(value, rule_str, column_name, row_number)
                            {
                                errors.push(error);
                            }
                        }
                    }
                }
            }

            row_number += 1;
        }

        Ok(serde_json::json!({
            "valid": errors.is_empty(),
            "errors": errors,
            "total_rows": row_number - 1,
            "error_count": errors.len()
        }))
    }

    /// Validate a single field against a rule
    fn validate_field(&self, value: &str, rule: &str, column: &str, row: usize) -> Option<Value> {
        // Email validation
        if rule == "email" && (!value.contains('@') || !value.contains('.')) {
            return Some(serde_json::json!({
                "row": row,
                "column": column,
                "value": value,
                "error": "Invalid email format"
            }));
        }

        // Range validation (e.g., "range:0-120")
        if rule.starts_with("range:") {
            if let Some(range_str) = rule.strip_prefix("range:") {
                if let Some((min_str, max_str)) = range_str.split_once('-') {
                    if let (Ok(min), Ok(max), Ok(val)) = (
                        min_str.parse::<f64>(),
                        max_str.parse::<f64>(),
                        value.parse::<f64>(),
                    ) {
                        if val < min || val > max {
                            return Some(serde_json::json!({
                                "row": row,
                                "column": column,
                                "value": value,
                                "error": format!("Value {} is outside range {}-{}", val, min, max)
                            }));
                        }
                    } else {
                        return Some(serde_json::json!({
                            "row": row,
                            "column": column,
                            "value": value,
                            "error": "Value is not a valid number"
                        }));
                    }
                }
            }
        }

        None
    }

    /// Convert CSV error to LLMSpellError
    fn csv_error(e: csv::Error) -> LLMSpellError {
        LLMSpellError::Validation {
            message: format!("CSV error: {}", e),
            field: None,
        }
    }
}

/// Helper function to write a chunk of data to Parquet
fn write_parquet_chunk(
    writer: &mut ArrowWriter<&mut Vec<u8>>,
    schema: &Arc<Schema>,
    column_types: &[ColumnType],
    chunk_data: &[Vec<String>],
) -> Result<()> {
    let mut columns: Vec<ArrayRef> = Vec::new();

    for (col_data, col_type) in chunk_data.iter().zip(column_types.iter()) {
        let array: ArrayRef = match col_type {
            ColumnType::Integer => {
                let values: Vec<Option<i64>> = col_data
                    .iter()
                    .map(|s| s.trim().parse::<i64>().ok())
                    .collect();
                Arc::new(Int64Array::from(values))
            }
            ColumnType::Float => {
                let values: Vec<Option<f64>> = col_data
                    .iter()
                    .map(|s| s.trim().parse::<f64>().ok())
                    .collect();
                Arc::new(Float64Array::from(values))
            }
            ColumnType::Boolean => {
                let values: Vec<Option<bool>> = col_data
                    .iter()
                    .map(|s| match s.trim().to_lowercase().as_str() {
                        "true" | "yes" | "y" | "1" => Some(true),
                        "false" | "no" | "n" | "0" => Some(false),
                        _ => None,
                    })
                    .collect();
                Arc::new(BooleanArray::from(values))
            }
            _ => {
                let values: Vec<Option<&str>> = col_data.iter().map(|s| Some(s.as_str())).collect();
                Arc::new(StringArray::from(values))
            }
        };
        columns.push(array);
    }

    let batch =
        RecordBatch::try_new(schema.clone(), columns).map_err(|e| LLMSpellError::Internal {
            message: format!("Failed to create record batch: {}", e),
            source: None,
        })?;

    writer.write(&batch).map_err(|e| LLMSpellError::Internal {
        message: format!("Failed to write Parquet batch: {}", e),
        source: None,
    })?;

    Ok(())
}

impl Default for CsvAnalyzerTool {
    fn default() -> Self {
        Self::new(CsvAnalyzerConfig::default())
    }
}

#[async_trait]
impl BaseAgent for CsvAnalyzerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        let (operation, content, options) = self.parse_parameters(params)?;

        info!("Executing CSV {} operation", operation);

        let result = match operation {
            CsvOperation::Analyze => {
                let analysis = self.analyze_csv_streaming(&content).await?;
                serde_json::to_value(analysis)?
            }
            CsvOperation::Convert => {
                let format_str = options
                    .as_ref()
                    .and_then(|o| o.get("format"))
                    .and_then(|f| f.as_str())
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Convert operation requires 'format' in options".to_string(),
                        field: Some("format".to_string()),
                    })?;
                let format: ExportFormat = format_str.parse()?;

                let result_bytes = self.convert_csv_streaming(&content, format).await?;

                // For binary formats, return base64 encoded
                match format {
                    ExportFormat::Parquet | ExportFormat::Excel => {
                        use base64::{engine::general_purpose, Engine as _};
                        let encoded = general_purpose::STANDARD.encode(&result_bytes);
                        serde_json::json!({
                            "format": format_str,
                            "encoding": "base64",
                            "data": encoded,
                            "size_bytes": result_bytes.len()
                        })
                    }
                    ExportFormat::Json => {
                        // For JSON format, return the parsed JSON directly for backward compatibility
                        let text = String::from_utf8(result_bytes).map_err(|e| {
                            LLMSpellError::Internal {
                                message: format!("Failed to convert result to string: {}", e),
                                source: None,
                            }
                        })?;
                        serde_json::from_str(&text)?
                    }
                    _ => {
                        // Other text formats (TSV, JsonLines)
                        let text = String::from_utf8(result_bytes).map_err(|e| {
                            LLMSpellError::Internal {
                                message: format!("Failed to convert result to string: {}", e),
                                source: None,
                            }
                        })?;
                        Value::String(text)
                    }
                }
            }
            CsvOperation::Filter => {
                let filter_expr = options
                    .as_ref()
                    .and_then(|o| o.get("filter"))
                    .and_then(|f| f.as_str())
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Filter operation requires 'filter' expression in options"
                            .to_string(),
                        field: Some("filter".to_string()),
                    })?;

                let filtered = self.filter_csv_streaming(&content, filter_expr).await?;
                Value::String(filtered)
            }
            CsvOperation::Sample => {
                let sample_size = options
                    .as_ref()
                    .and_then(|o| o.get("size"))
                    .and_then(|s| s.as_u64())
                    .unwrap_or(10) as usize;

                let sampled = self.sample_csv(&content, sample_size).await?;
                Value::String(sampled)
            }
            CsvOperation::Transform => {
                if options.is_none() {
                    return Err(LLMSpellError::Validation {
                        message: "Transform operation requires options".to_string(),
                        field: Some("options".to_string()),
                    });
                }
                let transformed = self
                    .transform_csv(&content, options.as_ref().unwrap())
                    .await?;
                Value::String(transformed)
            }
            CsvOperation::Validate => {
                if options.is_none() {
                    return Err(LLMSpellError::Validation {
                        message: "Validate operation requires options with rules".to_string(),
                        field: Some("options".to_string()),
                    });
                }
                self.validate_csv(&content, options.as_ref().unwrap())
                    .await?
            }
        };

        // Use ResponseBuilder for metadata
        let message = match operation {
            CsvOperation::Analyze => "CSV analysis completed",
            CsvOperation::Convert => "CSV conversion completed",
            CsvOperation::Filter => "CSV filtering completed",
            CsvOperation::Transform => "CSV transformation completed",
            CsvOperation::Validate => "CSV validation completed",
            CsvOperation::Sample => "CSV sampling completed",
        };

        let response = ResponseBuilder::success(operation.to_string())
            .with_message(message.to_string())
            .with_result(result.clone())
            .build();

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata.extra.insert(
            "operation".to_string(),
            Value::String(operation.to_string()),
        );
        metadata.extra.insert("response".to_string(), response);

        // Add the result to metadata for Analyze and Validate operations
        if matches!(operation, CsvOperation::Analyze) {
            metadata
                .extra
                .insert("analysis_result".to_string(), result.clone());
        } else if matches!(operation, CsvOperation::Validate) {
            metadata
                .extra
                .insert("validation_result".to_string(), result.clone());
        }

        // For data processing tools, return the actual result as text
        let output_text = match &result {
            Value::String(s) => s.clone(),
            _ => serde_json::to_string_pretty(&result)?,
        };

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }

        // Check size limit
        if let Some(params) = input.parameters.get("parameters") {
            if let Some(content) = params.get("input").and_then(|c| c.as_str()) {
                if content.len() > self.config.max_file_size {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "File size {} bytes exceeds maximum {} bytes",
                            content.len(),
                            self.config.max_file_size
                        ),
                        field: Some("input".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "CSV processing error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for CsvAnalyzerTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "csv_analyzer".to_string(),
            description: "Analyze and process CSV files with streaming support".to_string(),
            parameters: vec![
                ParameterDef {
                    name: "operation".to_string(),
                    description: "Operation to perform: analyze, convert, filter, sample"
                        .to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!("analyze")),
                },
                ParameterDef {
                    name: "input".to_string(),
                    description: "CSV content to process".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "options".to_string(),
                    description: "Operation-specific options".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(100 * 1024 * 1024) // 100MB limit
            .with_cpu_limit(30000) // 30 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_parsing() {
        assert_eq!(
            "analyze".parse::<CsvOperation>().unwrap(),
            CsvOperation::Analyze
        );
        assert_eq!(
            "convert".parse::<CsvOperation>().unwrap(),
            CsvOperation::Convert
        );
        assert_eq!(
            "filter".parse::<CsvOperation>().unwrap(),
            CsvOperation::Filter
        );
        assert!("invalid".parse::<CsvOperation>().is_err());
    }

    #[test]
    fn test_export_format_parsing() {
        assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!(
            "parquet".parse::<ExportFormat>().unwrap(),
            ExportFormat::Parquet
        );
        assert_eq!(
            "excel".parse::<ExportFormat>().unwrap(),
            ExportFormat::Excel
        );
        assert!("invalid".parse::<ExportFormat>().is_err());
    }

    #[tokio::test]
    async fn test_csv_analyzer_creation() {
        let config = CsvAnalyzerConfig::default();
        let tool = CsvAnalyzerTool::new(config);

        assert_eq!(tool.metadata().name, "csv-analyzer-tool");
    }

    #[tokio::test]
    async fn test_streaming_column_stats() {
        let mut stats = StreamingColumnStats::new("test".to_string());

        stats.update("10");
        stats.update("20");
        stats.update("30");
        stats.update("");

        assert_eq!(stats.count, 3);
        assert_eq!(stats.null_count, 1);
        assert_eq!(stats.sum, Some(60.0));

        let final_stats = stats.finalize();
        assert_eq!(final_stats.mean, Some(20.0));
    }
}
