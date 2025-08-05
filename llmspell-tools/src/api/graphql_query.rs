//! ABOUTME: GraphQL query tool with schema introspection, caching, and safe variable handling
//! ABOUTME: Provides comprehensive GraphQL client capabilities for queries, mutations, and subscriptions

use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
    extract_optional_object, extract_optional_string, extract_parameters, extract_required_string,
    response::ResponseBuilder,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// GraphQL operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphQLOperation {
    Query,
    Mutation,
    Subscription,
    Introspection,
}

impl std::fmt::Display for GraphQLOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::Mutation => write!(f, "mutation"),
            Self::Subscription => write!(f, "subscription"),
            Self::Introspection => write!(f, "introspection"),
        }
    }
}

impl std::str::FromStr for GraphQLOperation {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "query" => Ok(Self::Query),
            "mutation" => Ok(Self::Mutation),
            "subscription" => Ok(Self::Subscription),
            "introspection" => Ok(Self::Introspection),
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown GraphQL operation: {s}"),
                field: Some("operation".to_string()),
            }),
        }
    }
}

/// GraphQL request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}

/// GraphQL response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GraphQLError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Value>,
}

/// GraphQL error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ErrorLocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}

/// Schema cache entry
#[derive(Debug, Clone)]
pub struct SchemaCacheEntry {
    pub schema: Value,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl SchemaCacheEntry {
    fn is_expired(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        u64::try_from(age.num_seconds().max(0)).unwrap_or(u64::MAX) > self.ttl_seconds
    }
}

/// GraphQL tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLConfig {
    /// Default timeout in seconds
    pub timeout_seconds: u64,
    /// Enable schema caching
    pub enable_schema_cache: bool,
    /// Schema cache TTL in seconds
    pub schema_cache_ttl: u64,
    /// Maximum query depth
    pub max_query_depth: Option<u32>,
    /// User agent string
    pub user_agent: String,
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            enable_schema_cache: true,
            schema_cache_ttl: 3600, // 1 hour
            max_query_depth: Some(10),
            user_agent: "LLMSpell-GraphQLTool/1.0".to_string(),
        }
    }
}

/// GraphQL query tool
pub struct GraphQLQueryTool {
    metadata: ComponentMetadata,
    config: GraphQLConfig,
    client: Client,
    schema_cache: Arc<RwLock<HashMap<String, SchemaCacheEntry>>>,
}

impl GraphQLQueryTool {
    /// Creates a new GraphQL query tool with the given configuration.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be created or rate limiter setup fails.
    pub fn new(config: GraphQLConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| LLMSpellError::Internal {
                message: format!("Failed to create HTTP client: {e}"),
                source: None,
            })?;

        Ok(Self {
            metadata: ComponentMetadata::new(
                "graphql-query-tool".to_string(),
                "GraphQL client with schema introspection and caching".to_string(),
            ),
            config,
            client,
            schema_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Execute GraphQL request
    async fn execute_graphql(
        &self,
        endpoint: &str,
        request: GraphQLRequest,
        headers: Option<HashMap<String, String>>,
    ) -> Result<GraphQLResponse> {
        debug!("Executing GraphQL request to {}", endpoint);

        let mut req = self.client.post(endpoint).json(&request);

        // Add custom headers
        if let Some(headers) = headers {
            for (name, value) in headers {
                req = req.header(name, value);
            }
        }

        let response = req.send().await.map_err(|e| LLMSpellError::Network {
            message: format!("GraphQL request failed: {e}"),
            source: None,
        })?;

        if !response.status().is_success() {
            return Err(LLMSpellError::Tool {
                message: format!("GraphQL endpoint returned status: {}", response.status()),
                tool_name: Some("graphql_query".to_string()),
                source: None,
            });
        }

        let graphql_response: GraphQLResponse =
            response.json().await.map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to parse GraphQL response: {e}"),
                tool_name: Some("graphql_query".to_string()),
                source: None,
            })?;

        // Check for GraphQL errors
        if let Some(errors) = &graphql_response.errors {
            if !errors.is_empty() && graphql_response.data.is_none() {
                return Err(LLMSpellError::Tool {
                    message: format!("GraphQL errors: {errors:?}"),
                    tool_name: Some("graphql_query".to_string()),
                    source: None,
                });
            }
        }

        Ok(graphql_response)
    }

    /// Get or fetch schema with caching
    async fn get_schema(
        &self,
        endpoint: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        if !self.config.enable_schema_cache {
            return self.fetch_schema(endpoint, headers).await;
        }

        // Check cache
        {
            let cache = self.schema_cache.read().await;
            if let Some(entry) = cache.get(endpoint) {
                if !entry.is_expired() {
                    debug!("Using cached schema for {}", endpoint);
                    return Ok(entry.schema.clone());
                }
            }
        }

        // Fetch new schema
        let schema = self.fetch_schema(endpoint, headers).await?;

        // Update cache
        {
            let mut cache = self.schema_cache.write().await;
            cache.insert(
                endpoint.to_string(),
                SchemaCacheEntry {
                    schema: schema.clone(),
                    cached_at: Utc::now(),
                    ttl_seconds: self.config.schema_cache_ttl,
                },
            );
        }

        Ok(schema)
    }

    /// Fetch schema from endpoint
    async fn fetch_schema(
        &self,
        endpoint: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        info!("Fetching GraphQL schema from {}", endpoint);

        let introspection_query = r"
            query IntrospectionQuery {
                __schema {
                    queryType { name }
                    mutationType { name }
                    subscriptionType { name }
                    types {
                        ...FullType
                    }
                }
            }

            fragment FullType on __Type {
                kind
                name
                description
                fields(includeDeprecated: true) {
                    name
                    description
                    args {
                        ...InputValue
                    }
                    type {
                        ...TypeRef
                    }
                    isDeprecated
                    deprecationReason
                }
                inputFields {
                    ...InputValue
                }
                interfaces {
                    ...TypeRef
                }
                enumValues(includeDeprecated: true) {
                    name
                    description
                    isDeprecated
                    deprecationReason
                }
                possibleTypes {
                    ...TypeRef
                }
            }

            fragment InputValue on __InputValue {
                name
                description
                type { ...TypeRef }
                defaultValue
            }

            fragment TypeRef on __Type {
                kind
                name
                ofType {
                    kind
                    name
                    ofType {
                        kind
                        name
                        ofType {
                            kind
                            name
                        }
                    }
                }
            }
        ";

        let request = GraphQLRequest {
            query: introspection_query.to_string(),
            variables: None,
            operation_name: None,
        };

        let response = self.execute_graphql(endpoint, request, headers).await?;

        response.data.ok_or_else(|| LLMSpellError::Tool {
            message: "Schema introspection returned no data".to_string(),
            tool_name: Some("graphql_query".to_string()),
            source: None,
        })
    }

    /// Validate variables against expected types
    fn validate_variables(variables: &Value) -> Result<()> {
        // Basic validation - ensure it's an object
        if !variables.is_object() {
            return Err(LLMSpellError::Validation {
                message: "Variables must be a JSON object".to_string(),
                field: Some("variables".to_string()),
            });
        }

        // Additional validation could be added here based on schema
        Ok(())
    }

    /// Estimate query depth
    fn estimate_query_depth(query: &str) -> u32 {
        // Simple depth estimation based on brace nesting
        let mut depth: u32 = 0;
        let mut max_depth = 0;

        for char in query.chars() {
            match char {
                '{' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                '}' => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        max_depth
    }

    /// Parse parameters from input
    fn parse_parameters(params: &Value) -> Result<GraphQLParameters> {
        let operation_str = extract_optional_string(params, "operation").unwrap_or("query");
        let operation: GraphQLOperation = operation_str.parse()?;

        let endpoint = extract_required_string(params, "endpoint")?.to_string();

        let query = if operation == GraphQLOperation::Introspection {
            // For introspection, query is optional (we'll use built-in)
            extract_optional_string(params, "query")
                .map(String::from)
                .unwrap_or_default()
        } else {
            extract_required_string(params, "input")?.to_string()
        };

        let variables = params.get("variables").cloned();
        let operation_name = extract_optional_string(params, "operation_name").map(String::from);

        let headers = extract_optional_object(params, "headers").map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        });

        Ok(GraphQLParameters {
            operation,
            endpoint,
            query,
            variables,
            operation_name,
            headers,
        })
    }
}

#[derive(Debug)]
struct GraphQLParameters {
    operation: GraphQLOperation,
    endpoint: String,
    query: String,
    variables: Option<Value>,
    operation_name: Option<String>,
    headers: Option<HashMap<String, String>>,
}

impl Default for GraphQLQueryTool {
    fn default() -> Self {
        Self::new(GraphQLConfig::default()).expect("Default config should be valid")
    }
}

#[async_trait]
impl BaseAgent for GraphQLQueryTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        let parameters = Self::parse_parameters(params)?;

        info!(
            "Executing GraphQL {} to {}",
            parameters.operation, parameters.endpoint
        );

        // Validate query depth if configured
        if let Some(max_depth) = self.config.max_query_depth {
            let depth = Self::estimate_query_depth(&parameters.query);
            if depth > max_depth {
                return Err(LLMSpellError::Validation {
                    message: format!("Query depth {depth} exceeds maximum allowed {max_depth}"),
                    field: Some("query".to_string()),
                });
            }
        }

        // Validate variables if provided
        if let Some(ref vars) = parameters.variables {
            Self::validate_variables(vars)?;
        }

        let result = match parameters.operation {
            GraphQLOperation::Introspection => {
                let schema = self
                    .get_schema(&parameters.endpoint, parameters.headers)
                    .await?;
                json!({
                    "schema": schema,
                    "cached": self.config.enable_schema_cache
                })
            }
            GraphQLOperation::Query | GraphQLOperation::Mutation => {
                let request = GraphQLRequest {
                    query: parameters.query,
                    variables: parameters.variables,
                    operation_name: parameters.operation_name,
                };

                let response = self
                    .execute_graphql(&parameters.endpoint, request, parameters.headers)
                    .await?;

                serde_json::to_value(&response)?
            }
            GraphQLOperation::Subscription => {
                return Err(LLMSpellError::Tool {
                    message: "GraphQL subscriptions not yet supported".to_string(),
                    tool_name: Some("graphql_query".to_string()),
                    source: None,
                });
            }
        };

        // Create success message
        let message = match &parameters.operation {
            GraphQLOperation::Introspection => format!(
                "GraphQL introspection query completed for {}",
                parameters.endpoint
            ),
            GraphQLOperation::Query => format!(
                "GraphQL query executed successfully on {}",
                parameters.endpoint
            ),
            GraphQLOperation::Mutation => format!(
                "GraphQL mutation executed successfully on {}",
                parameters.endpoint
            ),
            GraphQLOperation::Subscription => unreachable!(),
        };

        // Build response
        let response = ResponseBuilder::success(parameters.operation.to_string())
            .with_message(message)
            .with_result(json!({
                "operation": parameters.operation.to_string(),
                "endpoint": parameters.endpoint,
                "result": result
            }))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
        }

        // Check required parameters
        if let Some(params) = input.parameters.get("parameters") {
            if params.get("endpoint").is_none() {
                return Err(LLMSpellError::Validation {
                    message: "Missing required parameter 'endpoint'".to_string(),
                    field: Some("endpoint".to_string()),
                });
            }
            // Query is only required for non-introspection operations
            let operation = params
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("query");

            if params.get("query").is_none() && operation != "introspection" {
                return Err(LLMSpellError::Validation {
                    message: "Missing required parameter 'query'".to_string(),
                    field: Some("query".to_string()),
                });
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("GraphQL query error: {error}")))
    }
}

#[async_trait]
impl Tool for GraphQLQueryTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Api
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Privileged
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "graphql_query".to_string(),
            description: "Execute GraphQL queries, mutations, and introspection".to_string(),
            parameters: vec![
                ParameterDef {
                    name: "operation".to_string(),
                    description: "Operation type: query, mutation, subscription, introspection"
                        .to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(json!("query")),
                },
                ParameterDef {
                    name: "endpoint".to_string(),
                    description: "GraphQL endpoint URL".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "input".to_string(),
                    description: "GraphQL query or mutation string".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "variables".to_string(),
                    description: "Query variables as JSON object".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "operation_name".to_string(),
                    description: "Operation name for queries with multiple operations".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ParameterDef {
                    name: "headers".to_string(),
                    description: "HTTP headers for the request".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
            ],
            returns: Some(ParameterType::Object),
        }
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: SecurityLevel::Privileged,
            file_permissions: vec![],
            network_permissions: vec!["*".to_string()],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        }
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(50 * 1024 * 1024) // 50MB
            .with_cpu_limit(self.config.timeout_seconds * 1000) // Convert to ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_operation_parsing() {
        assert_eq!(
            "query".parse::<GraphQLOperation>().unwrap(),
            GraphQLOperation::Query
        );
        assert_eq!(
            "mutation".parse::<GraphQLOperation>().unwrap(),
            GraphQLOperation::Mutation
        );
        assert_eq!(
            "introspection".parse::<GraphQLOperation>().unwrap(),
            GraphQLOperation::Introspection
        );
        assert!("invalid".parse::<GraphQLOperation>().is_err());
    }
    #[test]
    fn test_query_depth_estimation() {
        let tool = GraphQLQueryTool::default();

        let simple_query = "{ user { name } }";
        assert_eq!(GraphQLQueryTool::estimate_query_depth(simple_query), 2);

        let nested_query = "{ user { posts { comments { author { name } } } } }";
        assert_eq!(GraphQLQueryTool::estimate_query_depth(nested_query), 5);

        let flat_query = "{ user posts comments }";
        assert_eq!(GraphQLQueryTool::estimate_query_depth(flat_query), 1);
    }
    #[test]
    fn test_schema_cache_expiry() {
        let entry = SchemaCacheEntry {
            schema: json!({}),
            cached_at: Utc::now() - chrono::Duration::seconds(3700),
            ttl_seconds: 3600,
        };
        assert!(entry.is_expired());

        let fresh_entry = SchemaCacheEntry {
            schema: json!({}),
            cached_at: Utc::now(),
            ttl_seconds: 3600,
        };
        assert!(!fresh_entry.is_expired());
    }
    #[tokio::test]
    async fn test_graphql_tool_creation() {
        let config = GraphQLConfig::default();
        let tool = GraphQLQueryTool::new(config).unwrap();

        assert_eq!(tool.metadata().name, "graphql-query-tool");
    }
}
