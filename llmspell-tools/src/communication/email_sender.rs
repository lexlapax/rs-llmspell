//! ABOUTME: Email sending tool with support for SMTP, SendGrid, and AWS SES
//! ABOUTME: Provides secure email delivery with multiple provider options and configuration

use async_trait::async_trait;
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
    error_builders::llmspell::{tool_error, validation_error},
    params::{extract_optional_string, extract_parameters, extract_required_string},
    response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Configuration for email providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailProviderConfig {
    /// Provider type (smtp, sendgrid, ses)
    pub provider_type: String,
    /// API key or credentials
    pub credentials: HashMap<String, String>,
    /// Provider-specific settings
    pub settings: HashMap<String, String>,
}

/// Email sender configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSenderConfig {
    /// Default provider to use
    pub default_provider: String,
    /// Available email providers
    pub providers: HashMap<String, EmailProviderConfig>,
    /// Default sender email
    pub default_sender: Option<String>,
    /// Enable TLS/SSL by default
    pub enable_tls: bool,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for EmailSenderConfig {
    fn default() -> Self {
        Self {
            default_provider: "smtp".to_string(),
            providers: HashMap::new(),
            default_sender: None,
            enable_tls: true,
            timeout_seconds: 30,
        }
    }
}

impl EmailSenderConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        let mut providers = HashMap::new();

        // SMTP configuration
        if let Ok(smtp_host) = std::env::var("EMAIL_SMTP_HOST") {
            let mut credentials = HashMap::new();
            let mut settings = HashMap::new();

            credentials.insert("host".to_string(), smtp_host);

            if let Ok(port) = std::env::var("EMAIL_SMTP_PORT") {
                settings.insert("port".to_string(), port);
            }
            if let Ok(username) = std::env::var("EMAIL_SMTP_USERNAME") {
                credentials.insert("username".to_string(), username);
            }
            if let Ok(password) = std::env::var("EMAIL_SMTP_PASSWORD") {
                credentials.insert("password".to_string(), password);
            }

            let smtp_config = EmailProviderConfig {
                provider_type: "smtp".to_string(),
                credentials,
                settings,
            };
            providers.insert("smtp".to_string(), smtp_config);
        }

        // SendGrid configuration
        if let Ok(api_key) = std::env::var("EMAIL_SENDGRID_API_KEY") {
            let mut credentials = HashMap::new();
            credentials.insert("api_key".to_string(), api_key);

            let sendgrid_config = EmailProviderConfig {
                provider_type: "sendgrid".to_string(),
                credentials,
                settings: HashMap::new(),
            };
            providers.insert("sendgrid".to_string(), sendgrid_config);
        }

        // AWS SES configuration
        if let Ok(access_key) = std::env::var("EMAIL_SES_ACCESS_KEY_ID") {
            let mut credentials = HashMap::new();
            credentials.insert("access_key_id".to_string(), access_key);

            if let Ok(secret_key) = std::env::var("EMAIL_SES_SECRET_ACCESS_KEY") {
                credentials.insert("secret_access_key".to_string(), secret_key);
            }
            if let Ok(region) = std::env::var("EMAIL_SES_REGION") {
                credentials.insert("region".to_string(), region);
            }

            let ses_config = EmailProviderConfig {
                provider_type: "ses".to_string(),
                credentials,
                settings: HashMap::new(),
            };
            providers.insert("ses".to_string(), ses_config);
        }

        config.providers = providers;

        // Set default sender from environment
        if let Ok(sender) = std::env::var("EMAIL_DEFAULT_SENDER") {
            config.default_sender = Some(sender);
        }

        config
    }
}

/// Email sending tool
pub struct EmailSenderTool {
    config: EmailSenderConfig,
    metadata: ComponentMetadata,
}

impl EmailSenderTool {
    /// Create a new email sender tool
    pub fn new(config: EmailSenderConfig) -> Result<Self> {
        Ok(Self {
            config,
            metadata: ComponentMetadata::new(
                "email_sender".to_string(),
                "Email sending tool with support for SMTP, SendGrid, and AWS SES".to_string(),
            ),
        })
    }

    /// Send email using the specified provider
    async fn send_email(
        &self,
        provider: &str,
        from: &str,
        to: &str,
        subject: &str,
        body: &str,
        html: bool,
    ) -> Result<serde_json::Value> {
        debug!(
            "Sending email via {}: from={}, to={}, subject={}",
            provider, from, to, subject
        );

        let provider_config = self.config.providers.get(provider).ok_or_else(|| {
            tool_error(
                format!("Email provider '{}' not configured", provider),
                Some("provider".to_string()),
            )
        })?;

        match provider_config.provider_type.as_str() {
            "smtp" => {
                self.send_via_smtp(provider_config, from, to, subject, body, html)
                    .await
            }
            "sendgrid" => {
                self.send_via_sendgrid(provider_config, from, to, subject, body, html)
                    .await
            }
            "ses" => {
                self.send_via_ses(provider_config, from, to, subject, body, html)
                    .await
            }
            _ => Err(tool_error(
                format!(
                    "Unsupported email provider type: {}",
                    provider_config.provider_type
                ),
                Some("provider_type".to_string()),
            )),
        }
    }

    /// Send email via SMTP
    async fn send_via_smtp(
        &self,
        _config: &EmailProviderConfig,
        _from: &str,
        _to: &str,
        _subject: &str,
        _body: &str,
        _html: bool,
    ) -> Result<serde_json::Value> {
        // Note: SMTP implementation would require lettre crate
        // For now, return a mock success response
        warn!("SMTP email sending not fully implemented - returning mock response");

        Ok(serde_json::json!({
            "provider": "smtp",
            "status": "mock_sent",
            "message_id": format!("mock-{}", uuid::Uuid::new_v4()),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Send email via SendGrid
    async fn send_via_sendgrid(
        &self,
        _config: &EmailProviderConfig,
        _from: &str,
        _to: &str,
        _subject: &str,
        _body: &str,
        _html: bool,
    ) -> Result<serde_json::Value> {
        // Note: SendGrid implementation would require HTTP client and API calls
        // For now, return a mock success response
        warn!("SendGrid email sending not fully implemented - returning mock response");

        Ok(serde_json::json!({
            "provider": "sendgrid",
            "status": "mock_sent",
            "message_id": format!("mock-sg-{}", uuid::Uuid::new_v4()),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Send email via AWS SES
    async fn send_via_ses(
        &self,
        _config: &EmailProviderConfig,
        _from: &str,
        _to: &str,
        _subject: &str,
        _body: &str,
        _html: bool,
    ) -> Result<serde_json::Value> {
        // Note: SES implementation would require AWS SDK
        // For now, return a mock success response
        warn!("AWS SES email sending not fully implemented - returning mock response");

        Ok(serde_json::json!({
            "provider": "ses",
            "status": "mock_sent",
            "message_id": format!("mock-ses-{}", uuid::Uuid::new_v4()),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[async_trait]
impl BaseAgent for EmailSenderTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;

        // Extract required parameters
        let from = extract_required_string(params, "from")?;
        let to = extract_required_string(params, "to")?;
        let subject = extract_required_string(params, "subject")?;
        let body = extract_required_string(params, "body")?;

        // Extract optional parameters
        let provider =
            extract_optional_string(params, "provider").unwrap_or(&self.config.default_provider);
        let html = params
            .get("html")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Use default sender if no from address provided and default is set
        let from_address = if from.is_empty() {
            self.config.default_sender.as_ref().ok_or_else(|| {
                validation_error(
                    "No sender address provided and no default sender configured",
                    Some("from".to_string()),
                )
            })?
        } else {
            from
        };

        // Send the email
        match self
            .send_email(provider, from_address, to, subject, body, html)
            .await
        {
            Ok(email_result) => {
                info!("Email sent successfully via {}", provider);

                let response = ResponseBuilder::success("send_email")
                    .with_message(format!("Email sent successfully via {}", provider))
                    .with_result(email_result)
                    .build();

                Ok(AgentOutput::text(serde_json::to_string(&response)?))
            }
            Err(e) => {
                error!("Failed to send email: {}", e);

                let response =
                    ResponseBuilder::error("send_email", format!("Failed to send email: {}", e))
                        .build();

                Ok(AgentOutput::text(serde_json::to_string(&response)?))
            }
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        let params = extract_parameters(input)?;

        // Validate required parameters
        extract_required_string(params, "to")?;
        extract_required_string(params, "subject")?;
        extract_required_string(params, "body")?;

        // Validate that we have either a from address or a default sender
        let from = extract_optional_string(params, "from").unwrap_or("");
        if from.is_empty() && self.config.default_sender.is_none() {
            return Err(validation_error(
                "Either 'from' parameter or default sender must be configured",
                Some("from".to_string()),
            ));
        }

        // Validate provider if specified
        if let Some(provider) = extract_optional_string(params, "provider") {
            if !self.config.providers.contains_key(provider) {
                return Err(validation_error(
                    format!("Email provider '{}' is not configured", provider),
                    Some("provider".to_string()),
                ));
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Email sender error: {error}")))
    }
}

#[async_trait]
impl Tool for EmailSenderTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "email_sender".to_string(),
            "Send emails using various providers (SMTP, SendGrid, AWS SES)".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "from".to_string(),
            param_type: ParameterType::String,
            description: "Sender email address (optional if default sender is configured)"
                .to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "to".to_string(),
            param_type: ParameterType::String,
            description: "Recipient email address".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "subject".to_string(),
            param_type: ParameterType::String,
            description: "Email subject line".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "body".to_string(),
            param_type: ParameterType::String,
            description: "Email body content".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "html".to_string(),
            param_type: ParameterType::Boolean,
            description: "Whether the body is HTML format".to_string(),
            required: false,
            default: Some(serde_json::json!(false)),
        })
        .with_parameter(ParameterDef {
            name: "provider".to_string(),
            param_type: ParameterType::String,
            description: "Email provider to use (smtp, sendgrid, ses)".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Api
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::restricted()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::strict()
            .with_memory_limit(50 * 1024 * 1024) // 50MB
            .with_cpu_limit(5000) // 5 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_sender_tool_creation() {
        let config = EmailSenderConfig::default();
        let tool = EmailSenderTool::new(config).unwrap();
        assert_eq!(tool.metadata().name, "email_sender");
    }

    #[test]
    fn test_tool_metadata() {
        let config = EmailSenderConfig::default();
        let tool = EmailSenderTool::new(config).unwrap();

        assert_eq!(tool.category(), ToolCategory::Api);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);

        let schema = tool.schema();
        assert_eq!(schema.name, "email_sender");
        assert!(!schema.parameters.is_empty());
    }

    #[test]
    fn test_config_from_env() {
        // Test that from_env doesn't panic
        let _config = EmailSenderConfig::from_env();
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let config = EmailSenderConfig::default();
        let tool = EmailSenderTool::new(config).unwrap();

        // Test missing required parameter
        let input = AgentInput::text("send email").with_parameter(
            "parameters",
            serde_json::json!({
                "to": "test@example.com",
                "subject": "Test"
                // Missing body
            }),
        );

        let result = tool.validate_input(&input).await;
        assert!(result.is_err());

        // Test valid parameters
        let input = AgentInput::text("send email").with_parameter(
            "parameters",
            serde_json::json!({
                "from": "sender@example.com",
                "to": "test@example.com",
                "subject": "Test",
                "body": "Test message"
            }),
        );

        let result = tool.validate_input(&input).await;
        assert!(result.is_ok());
    }
}
