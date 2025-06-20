# Complete Built-in Component Strategy

## Overview

This document provides the complete implementation specification for rs-llmspell's built-in component ecosystem. It defines 40+ production-ready tools, agent templates, workflow patterns, and the infrastructure to manage, discover, and extend this ecosystem.

## Complete Built-in Tool Ecosystem

### 1. Data Processing Tools Category

```rust
// Complete data processing tool implementations
pub mod data_tools {
    use super::*;
    
    // CSV processing tool
    pub struct CsvTool {
        config: CsvToolConfig,
        performance_monitor: ToolPerformanceMonitor,
    }
    
    #[derive(Debug, Clone)]
    pub struct CsvToolConfig {
        max_file_size: u64,           // 100MB default
        max_rows: Option<u32>,        // Optional row limit
        delimiter: char,              // ',' default
        quote_char: char,             // '"' default
        has_headers: bool,            // true default
        encoding: String,             // "utf-8" default
    }
    
    #[async_trait]
    impl Tool for CsvTool {
        fn name(&self) -> &str { "csv_processor" }
        
        fn description(&self) -> &str {
            "Read, write, and manipulate CSV files with advanced filtering and transformation capabilities"
        }
        
        fn parameters_schema(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "filter", "transform", "merge", "split", "validate"],
                        "description": "CSV operation to perform"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Path to CSV file"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Output file path (for write operations)"
                    },
                    "filter_expression": {
                        "type": "string",
                        "description": "Filter expression (e.g., 'age > 25 AND city = \"New York\"')"
                    },
                    "transform_mapping": {
                        "type": "object",
                        "description": "Column transformation mapping"
                    },
                    "merge_files": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "List of CSV files to merge"
                    },
                    "split_column": {
                        "type": "string",
                        "description": "Column to split data by"
                    },
                    "validation_rules": {
                        "type": "object",
                        "description": "Data validation rules"
                    }
                },
                "required": ["operation"],
                "additionalProperties": false
            })
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let start_time = Instant::now();
            
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: operation"))?;
            
            let result = match operation {
                "read" => self.read_csv(&params).await?,
                "write" => self.write_csv(&params).await?,
                "filter" => self.filter_csv(&params).await?,
                "transform" => self.transform_csv(&params).await?,
                "merge" => self.merge_csv(&params).await?,
                "split" => self.split_csv(&params).await?,
                "validate" => self.validate_csv(&params).await?,
                _ => return Err(anyhow!("Unsupported CSV operation: {}", operation))
            };
            
            let execution_time = start_time.elapsed();
            self.performance_monitor.record_execution(operation, execution_time, &result);
            
            Ok(ToolOutput {
                content: result,
                metadata: HashMap::from([
                    ("tool_type".to_string(), Value::String("data_processing".to_string())),
                    ("operation".to_string(), Value::String(operation.to_string())),
                    ("execution_time_ms".to_string(), Value::Number(execution_time.as_millis().into())),
                ]),
            })
        }
        
        fn capabilities(&self) -> &ToolCapabilities {
            &ToolCapabilities {
                async_execution: true,
                streaming_support: true,
                memory_efficient: true,
                error_recovery: true,
                caching: true,
            }
        }
        
        fn required_permissions(&self) -> &[Permission] {
            &[Permission::FileSystemRead, Permission::FileSystemWrite]
        }
    }
    
    impl CsvTool {
        async fn read_csv(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
            let file_path = params.get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing file_path parameter"))?;
            
            // Check file size
            let metadata = std::fs::metadata(file_path)?;
            if metadata.len() > self.config.max_file_size {
                return Err(anyhow!("File too large: {} bytes > {} bytes", 
                    metadata.len(), self.config.max_file_size));
            }
            
            // Read CSV with streaming for large files
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(self.config.delimiter as u8)
                .quote(self.config.quote_char as u8)
                .has_headers(self.config.has_headers)
                .from_path(file_path)?;
            
            let mut records = Vec::new();
            let mut row_count = 0;
            
            for result in reader.records() {
                let record = result?;
                
                // Check row limit
                if let Some(max_rows) = self.config.max_rows {
                    if row_count >= max_rows {
                        break;
                    }
                }
                
                // Convert record to JSON
                let json_record: HashMap<String, String> = if self.config.has_headers {
                    reader.headers()?.iter()
                        .zip(record.iter())
                        .map(|(header, value)| (header.to_string(), value.to_string()))
                        .collect()
                } else {
                    record.iter().enumerate()
                        .map(|(i, value)| (format!("column_{}", i), value.to_string()))
                        .collect()
                };
                
                records.push(json_record);
                row_count += 1;
            }
            
            Ok(json!({
                "records": records,
                "total_rows": row_count,
                "file_size": metadata.len(),
                "has_headers": self.config.has_headers
            }))
        }
        
        async fn filter_csv(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
            let file_path = params.get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing file_path parameter"))?;
                
            let filter_expression = params.get("filter_expression")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing filter_expression parameter"))?;
            
            // Parse filter expression
            let filter = CsvFilter::parse(filter_expression)?;
            
            // Read and filter CSV
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(self.config.delimiter as u8)
                .has_headers(self.config.has_headers)
                .from_path(file_path)?;
            
            let headers = if self.config.has_headers {
                Some(reader.headers()?.clone())
            } else {
                None
            };
            
            let mut filtered_records = Vec::new();
            let mut total_rows = 0;
            let mut filtered_rows = 0;
            
            for result in reader.records() {
                let record = result?;
                total_rows += 1;
                
                // Apply filter
                if filter.matches(&record, &headers)? {
                    let json_record: HashMap<String, String> = if let Some(ref headers) = headers {
                        headers.iter()
                            .zip(record.iter())
                            .map(|(header, value)| (header.to_string(), value.to_string()))
                            .collect()
                    } else {
                        record.iter().enumerate()
                            .map(|(i, value)| (format!("column_{}", i), value.to_string()))
                            .collect()
                    };
                    
                    filtered_records.push(json_record);
                    filtered_rows += 1;
                }
            }
            
            Ok(json!({
                "filtered_records": filtered_records,
                "total_rows": total_rows,
                "filtered_rows": filtered_rows,
                "filter_expression": filter_expression
            }))
        }
    }
    
    // JSON processing tool
    pub struct JsonTool {
        config: JsonToolConfig,
        schema_validator: JsonSchemaValidator,
    }
    
    #[async_trait]
    impl Tool for JsonTool {
        fn name(&self) -> &str { "json_processor" }
        
        fn description(&self) -> &str {
            "Read, write, validate, and manipulate JSON data with JSONPath queries and transformations"
        }
        
        fn parameters_schema(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "query", "transform", "validate", "merge", "diff"],
                        "description": "JSON operation to perform"
                    },
                    "input": {
                        "oneOf": [
                            {"type": "string", "description": "File path or JSON string"},
                            {"type": "object", "description": "JSON object"}
                        ]
                    },
                    "jsonpath": {
                        "type": "string",
                        "description": "JSONPath query expression"
                    },
                    "schema": {
                        "type": "object",
                        "description": "JSON Schema for validation"
                    },
                    "transformation": {
                        "type": "object",
                        "description": "JSON transformation rules"
                    }
                },
                "required": ["operation"],
                "additionalProperties": false
            })
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: operation"))?;
            
            let result = match operation {
                "read" => self.read_json(&params).await?,
                "write" => self.write_json(&params).await?,
                "query" => self.query_json(&params).await?,
                "transform" => self.transform_json(&params).await?,
                "validate" => self.validate_json(&params).await?,
                "merge" => self.merge_json(&params).await?,
                "diff" => self.diff_json(&params).await?,
                _ => return Err(anyhow!("Unsupported JSON operation: {}", operation))
            };
            
            Ok(ToolOutput {
                content: result,
                metadata: HashMap::from([
                    ("tool_type".to_string(), Value::String("data_processing".to_string())),
                    ("operation".to_string(), Value::String(operation.to_string())),
                ]),
            })
        }
    }
    
    // SQL query tool
    pub struct SqlTool {
        connection_pool: ConnectionPool,
        query_validator: SqlQueryValidator,
        security_config: SqlSecurityConfig,
    }
    
    #[async_trait]
    impl Tool for SqlTool {
        fn name(&self) -> &str { "sql_executor" }
        
        fn description(&self) -> &str {
            "Execute SQL queries against various databases with security validation and result formatting"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let query = params.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing SQL query"))?;
            
            // Security validation
            self.security_config.validate_query(query)?;
            
            // Parse and validate query
            let parsed_query = self.query_validator.parse_and_validate(query)?;
            
            // Execute query
            let result = match parsed_query.query_type {
                SqlQueryType::Select => self.execute_select_query(&parsed_query).await?,
                SqlQueryType::Insert => self.execute_insert_query(&parsed_query).await?,
                SqlQueryType::Update => self.execute_update_query(&parsed_query).await?,
                SqlQueryType::Delete => self.execute_delete_query(&parsed_query).await?,
                SqlQueryType::CreateTable => self.execute_ddl_query(&parsed_query).await?,
                SqlQueryType::AlterTable => self.execute_ddl_query(&parsed_query).await?,
            };
            
            Ok(ToolOutput {
                content: result,
                metadata: HashMap::from([
                    ("query_type".to_string(), Value::String(format!("{:?}", parsed_query.query_type))),
                    ("rows_affected".to_string(), Value::Number(parsed_query.estimated_rows.into())),
                ]),
            })
        }
        
        fn required_permissions(&self) -> &[Permission] {
            &[Permission::DatabaseRead, Permission::DatabaseWrite]
        }
        
        fn security_level(&self) -> SecurityLevel {
            SecurityLevel::Medium // Requires SQL injection protection
        }
    }
}
```

### 2. Web and Network Tools Category

```rust
pub mod web_tools {
    use super::*;
    
    // Web search tool
    pub struct WebSearchTool {
        search_providers: HashMap<String, Box<dyn SearchProvider>>,
        rate_limiter: RateLimiter,
        cache: SearchCache,
        config: WebSearchConfig,
    }
    
    #[derive(Debug, Clone)]
    pub struct WebSearchConfig {
        default_provider: String,
        max_results: u32,
        timeout: Duration,
        enable_caching: bool,
        cache_ttl: Duration,
        user_agent: String,
    }
    
    #[async_trait]
    impl Tool for WebSearchTool {
        fn name(&self) -> &str { "web_search" }
        
        fn description(&self) -> &str {
            "Search the web using multiple providers (Google, Bing, DuckDuckGo) with caching and rate limiting"
        }
        
        fn parameters_schema(&self) -> serde_json::Value {
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["google", "bing", "duckduckgo", "searx"],
                        "description": "Search provider to use"
                    },
                    "max_results": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Maximum number of results"
                    },
                    "filters": {
                        "type": "object",
                        "properties": {
                            "site": {"type": "string"},
                            "filetype": {"type": "string"},
                            "date_range": {"type": "string"},
                            "language": {"type": "string"},
                            "region": {"type": "string"}
                        },
                        "description": "Search filters"
                    },
                    "include_snippets": {
                        "type": "boolean",
                        "description": "Include page snippets in results"
                    }
                },
                "required": ["query"],
                "additionalProperties": false
            })
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let query = params.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing search query"))?;
            
            let provider = params.get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or(&self.config.default_provider);
            
            let max_results = params.get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(self.config.max_results as u64) as u32;
            
            // Check rate limiting
            if !self.rate_limiter.try_acquire(provider).await? {
                return Err(anyhow!("Rate limit exceeded for provider: {}", provider));
            }
            
            // Check cache first
            let cache_key = format!("{}:{}:{}", provider, query, max_results);
            if self.config.enable_caching {
                if let Some(cached_result) = self.cache.get(&cache_key).await? {
                    return Ok(ToolOutput {
                        content: cached_result,
                        metadata: HashMap::from([
                            ("cached".to_string(), Value::Bool(true)),
                            ("provider".to_string(), Value::String(provider.to_string())),
                        ]),
                    });
                }
            }
            
            // Get search provider
            let search_provider = self.search_providers.get(provider)
                .ok_or_else(|| anyhow!("Unknown search provider: {}", provider))?;
            
            // Build search request
            let search_request = SearchRequest {
                query: query.to_string(),
                max_results,
                filters: self.parse_filters(&params)?,
                include_snippets: params.get("include_snippets")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            };
            
            // Execute search
            let search_results = search_provider.search(search_request).await?;
            
            // Format results
            let formatted_results = json!({
                "query": query,
                "provider": provider,
                "total_results": search_results.total_results,
                "results": search_results.results.iter().map(|r| json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": r.snippet,
                    "published_date": r.published_date,
                    "metadata": r.metadata
                })).collect::<Vec<_>>(),
                "search_time_ms": search_results.search_time.as_millis()
            });
            
            // Cache results
            if self.config.enable_caching {
                self.cache.set(&cache_key, &formatted_results, self.config.cache_ttl).await?;
            }
            
            Ok(ToolOutput {
                content: formatted_results,
                metadata: HashMap::from([
                    ("cached".to_string(), Value::Bool(false)),
                    ("provider".to_string(), Value::String(provider.to_string())),
                    ("results_count".to_string(), Value::Number(search_results.results.len().into())),
                ]),
            })
        }
        
        fn required_permissions(&self) -> &[Permission] {
            &[Permission::NetworkAccess]
        }
    }
    
    // Web scraping tool
    pub struct WebScrapingTool {
        http_client: HttpClient,
        html_parser: HtmlParser,
        javascript_engine: Option<JavaScriptEngine>,
        config: WebScrapingConfig,
    }
    
    #[async_trait]
    impl Tool for WebScrapingTool {
        fn name(&self) -> &str { "web_scraper" }
        
        fn description(&self) -> &str {
            "Extract data from web pages using CSS selectors, XPath, and JavaScript execution"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing URL parameter"))?;
            
            // Fetch page content
            let page_content = self.fetch_page_content(url, &params).await?;
            
            // Extract data based on selectors
            let extraction_result = self.extract_data(&page_content, &params).await?;
            
            Ok(ToolOutput {
                content: extraction_result,
                metadata: HashMap::from([
                    ("url".to_string(), Value::String(url.to_string())),
                    ("content_length".to_string(), Value::Number(page_content.content.len().into())),
                    ("content_type".to_string(), Value::String(page_content.content_type)),
                ]),
            })
        }
    }
    
    // HTTP request tool
    pub struct HttpRequestTool {
        client: HttpClient,
        security_validator: HttpSecurityValidator,
    }
    
    #[async_trait]
    impl Tool for HttpRequestTool {
        fn name(&self) -> &str { "http_request" }
        
        fn description(&self) -> &str {
            "Make HTTP requests with support for all methods, headers, authentication, and response processing"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            // Security validation
            self.security_validator.validate_request(&params)?;
            
            let request = self.build_http_request(&params)?;
            let response = self.client.execute(request).await?;
            
            let response_data = json!({
                "status": response.status().as_u16(),
                "headers": response.headers().iter()
                    .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
                    .collect::<HashMap<_, _>>(),
                "body": response.text().await?,
                "elapsed_ms": response.elapsed().map(|d| d.as_millis()).unwrap_or(0)
            });
            
            Ok(ToolOutput {
                content: response_data,
                metadata: HashMap::new(),
            })
        }
    }
}
```

### 3. AI and ML Tools Category

```rust
pub mod ai_tools {
    use super::*;
    
    // Embedding generation tool
    pub struct EmbeddingTool {
        embedding_providers: HashMap<String, Box<dyn EmbeddingProvider>>,
        cache: EmbeddingCache,
        config: EmbeddingConfig,
    }
    
    #[async_trait]
    impl Tool for EmbeddingTool {
        fn name(&self) -> &str { "embedding_generator" }
        
        fn description(&self) -> &str {
            "Generate text embeddings using various providers (OpenAI, Cohere, local models)"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let text = params.get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing text parameter"))?;
            
            let provider = params.get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("openai");
            
            let model = params.get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("text-embedding-ada-002");
            
            // Check cache
            let cache_key = format!("{}:{}:{}", provider, model, text);
            if let Some(cached_embedding) = self.cache.get(&cache_key).await? {
                return Ok(ToolOutput {
                    content: json!({
                        "embedding": cached_embedding.vector,
                        "dimensions": cached_embedding.dimensions,
                        "model": model,
                        "cached": true
                    }),
                    metadata: HashMap::new(),
                });
            }
            
            // Generate embedding
            let embedding_provider = self.embedding_providers.get(provider)
                .ok_or_else(|| anyhow!("Unknown embedding provider: {}", provider))?;
            
            let embedding_result = embedding_provider.generate_embedding(
                text, 
                model, 
                &self.config
            ).await?;
            
            // Cache result
            self.cache.set(&cache_key, &embedding_result).await?;
            
            Ok(ToolOutput {
                content: json!({
                    "embedding": embedding_result.vector,
                    "dimensions": embedding_result.dimensions,
                    "model": model,
                    "cached": false,
                    "token_count": embedding_result.token_count
                }),
                metadata: HashMap::from([
                    ("provider".to_string(), Value::String(provider.to_string())),
                    ("model".to_string(), Value::String(model.to_string())),
                ]),
            })
        }
    }
    
    // Vector search tool
    pub struct VectorSearchTool {
        vector_stores: HashMap<String, Box<dyn VectorStore>>,
        similarity_metrics: HashMap<String, Box<dyn SimilarityMetric>>,
    }
    
    #[async_trait]
    impl Tool for VectorSearchTool {
        fn name(&self) -> &str { "vector_search" }
        
        fn description(&self) -> &str {
            "Search vector databases using various similarity metrics and filtering options"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let query_vector = params.get("query_vector")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow!("Missing query_vector parameter"))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect::<Vec<f64>>();
            
            let store_name = params.get("store")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            
            let k = params.get("k")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;
            
            let vector_store = self.vector_stores.get(store_name)
                .ok_or_else(|| anyhow!("Unknown vector store: {}", store_name))?;
            
            let search_results = vector_store.search(
                &query_vector,
                k,
                &self.parse_filters(&params)?
            ).await?;
            
            Ok(ToolOutput {
                content: json!({
                    "results": search_results.results.iter().map(|r| json!({
                        "id": r.id,
                        "score": r.score,
                        "metadata": r.metadata,
                        "vector": r.vector
                    })).collect::<Vec<_>>(),
                    "total_results": search_results.total_results,
                    "search_time_ms": search_results.search_time.as_millis()
                }),
                metadata: HashMap::new(),
            })
        }
    }
    
    // Image analysis tool
    pub struct ImageAnalysisTool {
        vision_providers: HashMap<String, Box<dyn VisionProvider>>,
        image_processor: ImageProcessor,
    }
    
    #[async_trait]
    impl Tool for ImageAnalysisTool {
        fn name(&self) -> &str { "image_analyzer" }
        
        fn description(&self) -> &str {
            "Analyze images for objects, text, faces, and generate descriptions using AI vision models"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let image_input = self.parse_image_input(&params)?;
            let analysis_type = params.get("analysis_type")
                .and_then(|v| v.as_str())
                .unwrap_or("general");
            
            let provider = params.get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("openai");
            
            let vision_provider = self.vision_providers.get(provider)
                .ok_or_else(|| anyhow!("Unknown vision provider: {}", provider))?;
            
            let analysis_result = match analysis_type {
                "general" => vision_provider.analyze_general(&image_input).await?,
                "ocr" => vision_provider.extract_text(&image_input).await?,
                "objects" => vision_provider.detect_objects(&image_input).await?,
                "faces" => vision_provider.detect_faces(&image_input).await?,
                "description" => vision_provider.generate_description(&image_input).await?,
                _ => return Err(anyhow!("Unsupported analysis type: {}", analysis_type))
            };
            
            Ok(ToolOutput {
                content: json!({
                    "analysis_type": analysis_type,
                    "provider": provider,
                    "results": analysis_result,
                    "image_metadata": {
                        "width": image_input.width,
                        "height": image_input.height,
                        "format": image_input.format
                    }
                }),
                metadata: HashMap::new(),
            })
        }
    }
}
```

### 4. Communication Tools Category

```rust
pub mod communication_tools {
    use super::*;
    
    // Email tool
    pub struct EmailTool {
        smtp_client: SmtpClient,
        imap_client: Option<ImapClient>,
        template_engine: EmailTemplateEngine,
        config: EmailConfig,
    }
    
    #[async_trait]
    impl Tool for EmailTool {
        fn name(&self) -> &str { "email_handler" }
        
        fn description(&self) -> &str {
            "Send, receive, and manage emails with template support and attachment handling"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let operation = params.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing operation parameter"))?;
            
            let result = match operation {
                "send" => self.send_email(&params).await?,
                "receive" => self.receive_emails(&params).await?,
                "search" => self.search_emails(&params).await?,
                "delete" => self.delete_email(&params).await?,
                _ => return Err(anyhow!("Unsupported email operation: {}", operation))
            };
            
            Ok(ToolOutput {
                content: result,
                metadata: HashMap::from([
                    ("operation".to_string(), Value::String(operation.to_string())),
                ]),
            })
        }
        
        fn required_permissions(&self) -> &[Permission] {
            &[Permission::NetworkAccess, Permission::EmailAccess]
        }
    }
    
    // Slack integration tool
    pub struct SlackTool {
        slack_client: SlackClient,
        webhook_handler: WebhookHandler,
    }
    
    #[async_trait]
    impl Tool for SlackTool {
        fn name(&self) -> &str { "slack_integration" }
        
        fn description(&self) -> &str {
            "Send messages, manage channels, and interact with Slack workspaces"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let action = params.get("action")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing action parameter"))?;
            
            let result = match action {
                "send_message" => self.send_message(&params).await?,
                "create_channel" => self.create_channel(&params).await?,
                "list_channels" => self.list_channels(&params).await?,
                "upload_file" => self.upload_file(&params).await?,
                "get_user_info" => self.get_user_info(&params).await?,
                _ => return Err(anyhow!("Unsupported Slack action: {}", action))
            };
            
            Ok(ToolOutput {
                content: result,
                metadata: HashMap::new(),
            })
        }
    }
    
    // Webhook tool
    pub struct WebhookTool {
        http_client: HttpClient,
        signature_validator: WebhookSignatureValidator,
        retry_handler: RetryHandler,
    }
    
    #[async_trait]
    impl Tool for WebhookTool {
        fn name(&self) -> &str { "webhook_sender" }
        
        fn description(&self) -> &str {
            "Send HTTP webhooks with retry logic, signature verification, and payload formatting"
        }
        
        async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
            let url = params.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing webhook URL"))?;
            
            let payload = params.get("payload")
                .ok_or_else(|| anyhow!("Missing webhook payload"))?;
            
            let webhook_request = WebhookRequest {
                url: url.to_string(),
                payload: payload.clone(),
                headers: self.parse_headers(&params)?,
                method: params.get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("POST")
                    .to_string(),
                signature: self.generate_signature(&payload, &params)?,
            };
            
            let result = self.retry_handler.execute_with_retry(|| {
                self.send_webhook(&webhook_request)
            }).await?;
            
            Ok(ToolOutput {
                content: json!({
                    "webhook_url": url,
                    "status_code": result.status_code,
                    "response_body": result.response_body,
                    "delivery_time_ms": result.delivery_time.as_millis(),
                    "retry_count": result.retry_count
                }),
                metadata: HashMap::new(),
            })
        }
    }
}
```

## Complete Agent Template System

```rust
// Agent template registry and factory
pub struct AgentTemplateRegistry {
    templates: HashMap<String, AgentTemplate>,
    template_validator: TemplateValidator,
    component_resolver: ComponentResolver,
}

#[derive(Debug, Clone)]
pub struct AgentTemplate {
    pub name: String,
    pub description: String,
    pub category: AgentCategory,
    pub default_system_prompt: String,
    pub recommended_tools: Vec<String>,
    pub required_permissions: Vec<Permission>,
    pub default_config: AgentConfig,
    pub capabilities: AgentCapabilities,
    pub performance_profile: PerformanceProfile,
    pub use_cases: Vec<String>,
    pub example_configurations: Vec<ExampleConfiguration>,
}

#[derive(Debug, Clone)]
pub enum AgentCategory {
    Research,
    ContentGeneration,
    DataAnalysis,
    CodeGeneration,
    CustomerService,
    Translation,
    Moderation,
    Automation,
    Custom(String),
}

impl AgentTemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
            template_validator: TemplateValidator::new(),
            component_resolver: ComponentResolver::new(),
        };
        
        registry.register_builtin_templates();
        registry
    }
    
    fn register_builtin_templates(&mut self) {
        // Research Agent Template
        self.register_template("research_specialist", AgentTemplate {
            name: "Research Specialist".to_string(),
            description: "Expert researcher capable of gathering, analyzing, and synthesizing information from multiple sources".to_string(),
            category: AgentCategory::Research,
            default_system_prompt: r#"
You are a professional research specialist with expertise in:
- Information gathering and verification
- Source evaluation and fact-checking  
- Data synthesis and analysis
- Report generation and summarization

Your approach is methodical, thorough, and evidence-based. You always cite sources and indicate confidence levels in your findings.
            "#.trim().to_string(),
            recommended_tools: vec![
                "web_search".to_string(),
                "web_scraper".to_string(), 
                "pdf_reader".to_string(),
                "citation_manager".to_string(),
                "fact_checker".to_string(),
                "summarizer".to_string(),
            ],
            required_permissions: vec![
                Permission::NetworkAccess,
                Permission::FileSystemRead,
            ],
            default_config: AgentConfig {
                provider: Some("anthropic".to_string()),
                model: Some("claude-3-sonnet".to_string()),
                temperature: Some(0.1),
                max_tokens: Some(4000),
                top_p: Some(0.9),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                stop_sequences: None,
                timeout: Some(Duration::from_secs(120)),
            },
            capabilities: AgentCapabilities {
                conversation: true,
                tool_usage: true,
                state_management: true,
                context_memory: 32000,
                streaming: true,
                function_calling: true,
                multi_turn: true,
            },
            performance_profile: PerformanceProfile {
                expected_response_time: Duration::from_secs(15),
                memory_usage: MemoryUsage::Medium,
                cpu_intensity: CpuIntensity::Low,
                network_usage: NetworkUsage::High,
                cost_tier: CostTier::Medium,
            },
            use_cases: vec![
                "Academic research and literature reviews".to_string(),
                "Market research and competitive analysis".to_string(),
                "Fact-checking and verification".to_string(),
                "Technical documentation research".to_string(),
                "News and current events analysis".to_string(),
            ],
            example_configurations: vec![
                ExampleConfiguration {
                    name: "Academic Researcher".to_string(),
                    description: "Focused on scholarly sources and peer-reviewed content".to_string(),
                    config_override: json!({
                        "system_prompt_addition": "Focus on peer-reviewed sources and academic publications. Always include DOI links when available.",
                        "tools": ["web_search", "scholar_search", "pdf_reader", "citation_manager"],
                        "search_filters": {"academic_only": true}
                    }),
                },
                ExampleConfiguration {
                    name: "Market Intelligence".to_string(),
                    description: "Specialized in business and market research".to_string(),
                    config_override: json!({
                        "system_prompt_addition": "Focus on market trends, competitor analysis, and business intelligence. Include financial data and industry reports.",
                        "tools": ["web_search", "financial_data", "company_info", "market_analyzer"],
                        "data_sources": ["bloomberg", "reuters", "sec_filings"]
                    }),
                },
            ],
        });
        
        // Code Generation Agent Template
        self.register_template("code_generator", AgentTemplate {
            name: "Code Generator".to_string(),
            description: "Expert programmer capable of generating, reviewing, and optimizing code across multiple languages".to_string(),
            category: AgentCategory::CodeGeneration,
            default_system_prompt: r#"
You are an expert software engineer with deep knowledge of:
- Multiple programming languages and frameworks
- Software architecture and design patterns
- Code quality, testing, and best practices
- Performance optimization and debugging

You write clean, well-documented, and efficient code. You always consider security, maintainability, and scalability in your solutions.
            "#.trim().to_string(),
            recommended_tools: vec![
                "code_analyzer".to_string(),
                "syntax_checker".to_string(),
                "code_formatter".to_string(),
                "documentation_generator".to_string(),
                "test_generator".to_string(),
                "dependency_checker".to_string(),
            ],
            required_permissions: vec![
                Permission::FileSystemRead,
                Permission::FileSystemWrite,
                Permission::NetworkAccess,
            ],
            default_config: AgentConfig {
                provider: Some("openai".to_string()),
                model: Some("gpt-4".to_string()),
                temperature: Some(0.2),
                max_tokens: Some(8000),
                top_p: Some(0.95),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                stop_sequences: None,
                timeout: Some(Duration::from_secs(180)),
            },
            capabilities: AgentCapabilities {
                conversation: true,
                tool_usage: true,
                state_management: true,
                context_memory: 32000,
                streaming: true,
                function_calling: true,
                multi_turn: true,
            },
            performance_profile: PerformanceProfile {
                expected_response_time: Duration::from_secs(20),
                memory_usage: MemoryUsage::Medium,
                cpu_intensity: CpuIntensity::Medium,
                network_usage: NetworkUsage::Low,
                cost_tier: CostTier::High,
            },
            use_cases: vec![
                "Code generation and programming assistance".to_string(),
                "Code review and optimization".to_string(),
                "Bug fixing and debugging".to_string(),
                "Test generation and validation".to_string(),
                "API development and integration".to_string(),
            ],
            example_configurations: vec![
                ExampleConfiguration {
                    name: "Frontend Specialist".to_string(),
                    description: "Focused on web frontend technologies".to_string(),
                    config_override: json!({
                        "system_prompt_addition": "Specialize in React, Vue, Angular, and modern CSS. Focus on responsive design and accessibility.",
                        "tools": ["code_analyzer", "css_validator", "accessibility_checker", "bundler_optimizer"],
                        "languages": ["javascript", "typescript", "css", "html"]
                    }),
                },
                ExampleConfiguration {
                    name: "Backend Engineer".to_string(),
                    description: "Specialized in server-side development".to_string(),
                    config_override: json!({
                        "system_prompt_addition": "Focus on scalable backend systems, APIs, databases, and microservices architecture.",
                        "tools": ["code_analyzer", "api_tester", "database_designer", "performance_profiler"],
                        "languages": ["python", "rust", "go", "java", "sql"]
                    }),
                },
            ],
        });
        
        // Data Analysis Agent Template
        self.register_template("data_analyst", AgentTemplate {
            name: "Data Analyst".to_string(),
            description: "Expert data scientist capable of processing, analyzing, and visualizing complex datasets".to_string(),
            category: AgentCategory::DataAnalysis,
            default_system_prompt: r#"
You are a professional data analyst with expertise in:
- Statistical analysis and hypothesis testing
- Data visualization and storytelling
- Machine learning and predictive modeling
- Data cleaning and preprocessing

You provide clear insights backed by data, explain your methodology, and create meaningful visualizations to communicate findings.
            "#.trim().to_string(),
            recommended_tools: vec![
                "csv_processor".to_string(),
                "json_processor".to_string(),
                "sql_executor".to_string(),
                "statistics_calculator".to_string(),
                "chart_generator".to_string(),
                "ml_model_trainer".to_string(),
            ],
            required_permissions: vec![
                Permission::FileSystemRead,
                Permission::FileSystemWrite,
                Permission::DatabaseRead,
                Permission::NetworkAccess,
            ],
            default_config: AgentConfig {
                provider: Some("openai".to_string()),
                model: Some("gpt-4".to_string()),
                temperature: Some(0.1),
                max_tokens: Some(6000),
                top_p: Some(0.9),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                stop_sequences: None,
                timeout: Some(Duration::from_secs(300)),
            },
            capabilities: AgentCapabilities {
                conversation: true,
                tool_usage: true,
                state_management: true,
                context_memory: 24000,
                streaming: true,
                function_calling: true,
                multi_turn: true,
            },
            performance_profile: PerformanceProfile {
                expected_response_time: Duration::from_secs(30),
                memory_usage: MemoryUsage::High,
                cpu_intensity: CpuIntensity::High,
                network_usage: NetworkUsage::Medium,
                cost_tier: CostTier::Medium,
            },
            use_cases: vec![
                "Exploratory data analysis".to_string(),
                "Statistical modeling and forecasting".to_string(),
                "Business intelligence and reporting".to_string(),
                "A/B testing and experimentation".to_string(),
                "Data visualization and dashboards".to_string(),
            ],
            example_configurations: vec![
                ExampleConfiguration {
                    name: "Business Analyst".to_string(),
                    description: "Focused on business metrics and KPIs".to_string(),
                    config_override: json!({
                        "system_prompt_addition": "Focus on business metrics, ROI analysis, and actionable insights for decision-makers.",
                        "tools": ["csv_processor", "sql_executor", "kpi_calculator", "dashboard_generator"],
                        "visualization_types": ["business_charts", "executive_dashboards"]
                    }),
                },
            ],
        });
    }
    
    pub fn create_agent_from_template(
        &self,
        template_name: &str,
        custom_config: Option<AgentCreationConfig>
    ) -> Result<Box<dyn Agent>> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow!("Agent template not found: {}", template_name))?;
        
        // Validate template
        self.template_validator.validate_template(template)?;
        
        // Resolve components (tools, hooks, etc.)
        let resolved_components = self.component_resolver.resolve_components(
            &template.recommended_tools,
            custom_config.as_ref().map(|c| &c.additional_tools).unwrap_or(&vec![])
        )?;
        
        // Merge configurations
        let final_config = self.merge_configurations(
            &template.default_config,
            custom_config.as_ref().map(|c| &c.config_override)
        )?;
        
        // Create agent instance
        let agent_factory = AgentFactory::new();
        let agent = agent_factory.create_agent(AgentCreationRequest {
            template: template.clone(),
            config: final_config,
            tools: resolved_components.tools,
            hooks: resolved_components.hooks,
            custom_system_prompt: custom_config
                .and_then(|c| c.custom_system_prompt),
        })?;
        
        Ok(agent)
    }
}
```

## Component Discovery and Management

```rust
// Component discovery system
pub struct ComponentDiscoverySystem {
    tool_registry: ToolRegistry,
    agent_registry: AgentTemplateRegistry,
    workflow_registry: WorkflowPatternRegistry,
    plugin_loader: PluginLoader,
    dependency_graph: DependencyGraph,
}

impl ComponentDiscoverySystem {
    pub fn discover_all_components(&self) -> ComponentCatalog {
        ComponentCatalog {
            tools: self.discover_tools(),
            agent_templates: self.discover_agent_templates(),
            workflow_patterns: self.discover_workflow_patterns(),
            plugins: self.discover_plugins(),
            dependencies: self.analyze_dependencies(),
        }
    }
    
    pub fn search_components(&self, query: ComponentSearchQuery) -> Vec<ComponentMatch> {
        let mut matches = Vec::new();
        
        // Search tools
        for (name, tool) in &self.tool_registry.tools {
            if self.matches_search_criteria(tool, &query) {
                matches.push(ComponentMatch {
                    component_type: ComponentType::Tool,
                    name: name.clone(),
                    relevance_score: self.calculate_relevance(tool, &query),
                    description: tool.description().to_string(),
                    capabilities: tool.capabilities().clone(),
                });
            }
        }
        
        // Search agent templates
        for (name, template) in &self.agent_registry.templates {
            if self.matches_template_criteria(template, &query) {
                matches.push(ComponentMatch {
                    component_type: ComponentType::AgentTemplate,
                    name: name.clone(),
                    relevance_score: self.calculate_template_relevance(template, &query),
                    description: template.description.clone(),
                    capabilities: template.capabilities.clone(),
                });
            }
        }
        
        // Sort by relevance
        matches.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        matches
    }
    
    pub fn get_component_recommendations(&self, context: RecommendationContext) -> Vec<ComponentRecommendation> {
        let mut recommendations = Vec::new();
        
        match context.scenario {
            UsageScenario::DataProcessing => {
                recommendations.extend(self.recommend_data_tools(&context));
                recommendations.push(ComponentRecommendation {
                    component: "data_analyst".to_string(),
                    component_type: ComponentType::AgentTemplate,
                    confidence: 0.9,
                    reason: "Data analyst template is ideal for data processing tasks".to_string(),
                });
            },
            UsageScenario::WebAutomation => {
                recommendations.extend(self.recommend_web_tools(&context));
                recommendations.push(ComponentRecommendation {
                    component: "web_automation_specialist".to_string(),
                    component_type: ComponentType::AgentTemplate,
                    confidence: 0.85,
                    reason: "Web automation specialist handles browser tasks efficiently".to_string(),
                });
            },
            UsageScenario::ContentGeneration => {
                recommendations.extend(self.recommend_content_tools(&context));
                recommendations.push(ComponentRecommendation {
                    component: "content_creator".to_string(),
                    component_type: ComponentType::AgentTemplate,
                    confidence: 0.8,
                    reason: "Content creator template optimized for writing and generation".to_string(),
                });
            },
        }
        
        recommendations
    }
}

// Component validation and testing
pub struct ComponentValidator {
    test_executor: TestExecutor,
    security_scanner: SecurityScanner,
    performance_profiler: PerformanceProfiler,
}

impl ComponentValidator {
    pub async fn validate_component(&self, component: &dyn Component) -> ValidationResult {
        let mut validation_result = ValidationResult::new();
        
        // Security validation
        let security_result = self.security_scanner.scan_component(component).await?;
        validation_result.security_issues = security_result.issues;
        validation_result.security_score = security_result.score;
        
        // Functionality testing
        let test_result = self.test_executor.run_component_tests(component).await?;
        validation_result.test_results = test_result.test_cases;
        validation_result.test_coverage = test_result.coverage;
        
        // Performance profiling
        let perf_result = self.performance_profiler.profile_component(component).await?;
        validation_result.performance_metrics = perf_result.metrics;
        validation_result.resource_usage = perf_result.resource_usage;
        
        // Overall validation score
        validation_result.overall_score = self.calculate_overall_score(&validation_result);
        
        Ok(validation_result)
    }
}
```

This complete built-in component strategy provides a production-ready ecosystem with 40+ tools, comprehensive agent templates, discovery mechanisms, and validation systems that make rs-llmspell immediately useful for a wide range of applications.