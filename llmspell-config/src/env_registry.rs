//! ABOUTME: Standard environment variable registrations for LLMSpell
//! ABOUTME: Defines and registers all built-in environment variables using config paths

use crate::env::{EnvCategory, EnvRegistry, EnvVarDefBuilder};

/// Register all standard LLMSpell environment variables
pub fn register_standard_vars(registry: &EnvRegistry) -> Result<(), String> {
    // Core Runtime Variables
    register_runtime_vars(registry)?;

    // State Persistence Variables
    register_state_vars(registry)?;

    // Provider Configuration Variables
    register_provider_vars(registry)?;

    // Tool Configuration Variables
    register_tool_vars(registry)?;

    // Session/Hook Variables
    register_session_hook_vars(registry)?;

    // Memory System Variables
    register_memory_vars(registry)?;

    // Path Discovery Variables
    register_path_vars(registry)?;

    // Event System Variables
    register_event_vars(registry)?;

    Ok(())
}

/// Register core runtime environment variables
fn register_runtime_vars(registry: &EnvRegistry) -> Result<(), String> {
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_DEFAULT_ENGINE")
            .description("Default script engine (lua, javascript)")
            .category(EnvCategory::Runtime)
            .config_path("default_engine")
            .default("lua")
            .validator(|v| match v {
                "lua" | "javascript" => Ok(()),
                _ => Err(format!("Invalid engine: {}", v)),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MAX_CONCURRENT_SCRIPTS")
            .description("Maximum number of concurrent scripts")
            .category(EnvCategory::Runtime)
            .config_path("runtime.max_concurrent_scripts")
            .default("10")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid number: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_SCRIPT_TIMEOUT_SECONDS")
            .description("Script execution timeout in seconds")
            .category(EnvCategory::Runtime)
            .config_path("runtime.script_timeout_seconds")
            .default("300")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_ALLOW_FILE_ACCESS")
            .description("Allow file system access in scripts")
            .category(EnvCategory::Runtime)
            .config_path("runtime.security.allow_file_access")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_ALLOW_NETWORK_ACCESS")
            .description("Allow network access in scripts")
            .category(EnvCategory::Runtime)
            .config_path("runtime.security.allow_network_access")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_ALLOW_PROCESS_SPAWN")
            .description("Allow process spawning in scripts")
            .category(EnvCategory::Runtime)
            .config_path("runtime.security.allow_process_spawn")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MAX_MEMORY_BYTES")
            .description("Maximum memory usage in bytes")
            .category(EnvCategory::Runtime)
            .config_path("runtime.security.max_memory_bytes")
            .default("50000000")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid memory limit: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MAX_EXECUTION_TIME_MS")
            .description("Maximum execution time in milliseconds")
            .category(EnvCategory::Runtime)
            .config_path("runtime.security.max_execution_time_ms")
            .default("300000")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid execution time: {}", e))
            })
            .build(),
    )?;

    Ok(())
}

/// Register state persistence environment variables
fn register_state_vars(registry: &EnvRegistry) -> Result<(), String> {
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_ENABLED")
            .description("Enable state persistence")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.enabled")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_BACKEND")
            .description("State persistence backend (memory, sqlite, postgres)")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.backend_type")
            .default("memory")
            .validator(|v| match v {
                "memory" | "sqlite" | "postgres" => Ok(()),
                _ => Err(format!("Invalid backend: {}", v)),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_PATH")
            .description("Storage path for file-based state backends")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.schema_directory")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_MIGRATION_ENABLED")
            .description("Enable state migration functionality")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.migration_enabled")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_BACKUP_ENABLED")
            .description("Enable state backup functionality")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.backup_enabled")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_BACKUP_ON_MIGRATION")
            .description("Automatic backup on migration")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.backup_on_migration")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_BACKUP_DIR")
            .description("Directory for state backups")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.backup.backup_dir")
            .default("./backups")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_STATE_MAX_SIZE_BYTES")
            .description("Maximum state size per key in bytes")
            .category(EnvCategory::State)
            .config_path("runtime.state_persistence.max_state_size_bytes")
            .default("10000000")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid size: {}", e))
            })
            .build(),
    )?;

    Ok(())
}

/// Register provider configuration environment variables
fn register_provider_vars(registry: &EnvRegistry) -> Result<(), String> {
    // Standard OpenAI API key
    registry.register_var(
        EnvVarDefBuilder::new("OPENAI_API_KEY")
            .description("OpenAI API key")
            .category(EnvCategory::Provider)
            .config_path("providers.openai.api_key")
            .sensitive()
            .build(),
    )?;

    // Standard Anthropic API key
    registry.register_var(
        EnvVarDefBuilder::new("ANTHROPIC_API_KEY")
            .description("Anthropic API key")
            .category(EnvCategory::Provider)
            .config_path("providers.anthropic.api_key")
            .sensitive()
            .build(),
    )?;

    // Google Gemini API key
    registry.register_var(
        EnvVarDefBuilder::new("GEMINI_API_KEY")
            .description("Google Gemini API key")
            .category(EnvCategory::Provider)
            .config_path("providers.gemini.api_key")
            .sensitive()
            .build(),
    )?;

    // OpenRouter API key
    registry.register_var(
        EnvVarDefBuilder::new("OPENROUTER_API_KEY")
            .description("OpenRouter API key")
            .category(EnvCategory::Provider)
            .config_path("providers.openrouter.api_key")
            .sensitive()
            .build(),
    )?;

    // Groq API key
    registry.register_var(
        EnvVarDefBuilder::new("GROQ_API_KEY")
            .description("Groq API key")
            .category(EnvCategory::Provider)
            .config_path("providers.groq.api_key")
            .sensitive()
            .build(),
    )?;

    // xAI API key
    registry.register_var(
        EnvVarDefBuilder::new("XAI_API_KEY")
            .description("xAI API key")
            .category(EnvCategory::Provider)
            .config_path("providers.xai.api_key")
            .sensitive()
            .build(),
    )?;

    // HuggingFace API key
    registry.register_var(
        EnvVarDefBuilder::new("HFHUB_API_KEY")
            .description("HuggingFace Hub API token")
            .category(EnvCategory::Provider)
            .config_path("providers.huggingface.api_key")
            .sensitive()
            .build(),
    )?;

    // LLMSpell-specific provider format
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_OPENAI_API_KEY")
            .description("OpenAI API key (LLMSpell format)")
            .category(EnvCategory::Provider)
            .config_path("providers.openai.api_key")
            .sensitive()
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_ANTHROPIC_API_KEY")
            .description("Anthropic API key (LLMSpell format)")
            .category(EnvCategory::Provider)
            .config_path("providers.anthropic.api_key")
            .sensitive()
            .build(),
    )?;


    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_GEMINI_API_KEY")
            .description("Google Gemini API key (LLMSpell format)")
            .category(EnvCategory::Provider)
            .config_path("providers.gemini.api_key")
            .sensitive()
            .build(),
    )?;

    // Provider base URLs and endpoints
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_OPENAI_BASE_URL")
            .description("OpenAI API base URL")
            .category(EnvCategory::Provider)
            .config_path("providers.openai.base_url")
            .default("https://api.openai.com/v1")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_ANTHROPIC_BASE_URL")
            .description("Anthropic API base URL")
            .category(EnvCategory::Provider)
            .config_path("providers.anthropic.base_url")
            .default("https://api.anthropic.com")
            .build(),
    )?;

    // Additional provider configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_OPENAI_MODEL")
            .description("Default OpenAI model")
            .category(EnvCategory::Provider)
            .config_path("providers.openai.default_model")
            .default("gpt-3.5-turbo")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_ANTHROPIC_MODEL")
            .description("Default Anthropic model")
            .category(EnvCategory::Provider)
            .config_path("providers.anthropic.default_model")
            .default("claude-3-5-haiku-latest")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_OPENAI_TIMEOUT")
            .description("OpenAI request timeout (seconds)")
            .category(EnvCategory::Provider)
            .config_path("providers.openai.timeout_seconds")
            .default("30")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_ANTHROPIC_TIMEOUT")
            .description("Anthropic request timeout (seconds)")
            .category(EnvCategory::Provider)
            .config_path("providers.anthropic.timeout_seconds")
            .default("30")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_OPENAI_MAX_RETRIES")
            .description("OpenAI maximum retry count")
            .category(EnvCategory::Provider)
            .config_path("providers.openai.max_retries")
            .default("3")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid retry count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_PROVIDER_ANTHROPIC_MAX_RETRIES")
            .description("Anthropic maximum retry count")
            .category(EnvCategory::Provider)
            .config_path("providers.anthropic.max_retries")
            .default("3")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid retry count: {}", e))
            })
            .build(),
    )?;

    Ok(())
}

/// Register tool configuration environment variables
/// Maps to actual fields in ToolsConfig and its sub-configs
fn register_tool_vars(registry: &EnvRegistry) -> Result<(), String> {
    // Global tool settings (maps to ToolsConfig fields)
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_ENABLED")
            .description("Enable tool system globally")
            .category(EnvCategory::Tool)
            .config_path("tools.enabled") // Maps to ToolsConfig.enabled
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_RATE_LIMIT")
            .description("Global rate limiting for tools (requests per minute)")
            .category(EnvCategory::Tool)
            .config_path("tools.rate_limit_per_minute")
            .default("60")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid rate limit: {}", e))
            })
            .build(),
    )?;

    // File operations tools
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_FILE_OPS_ENABLED")
            .description("Enable file operations tools")
            .category(EnvCategory::Tool)
            .config_path("tools.file_operations.enabled")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_MAX_FILE_SIZE")
            .description("Maximum file size for operations (bytes)")
            .category(EnvCategory::Tool)
            .config_path("tools.file_operations.max_file_size")
            .default("104857600") // 100MB
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid size: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_ALLOWED_PATHS")
            .description("Comma-separated list of allowed paths for file operations")
            .category(EnvCategory::Tool)
            .config_path("tools.file_operations.allowed_paths")
            // No default - use config file value
            .build(),
    )?;

    // Network settings
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_NETWORK_TIMEOUT")
            .description("Default network timeout for tools (seconds)")
            .category(EnvCategory::Tool)
            .config_path("tools.network.timeout_seconds")
            .default("30")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_NETWORK_MAX_RETRIES")
            .description("Maximum network retries for tools")
            .category(EnvCategory::Tool)
            .config_path("tools.network.max_retries")
            .default("3")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid retry count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_NETWORK_VERIFY_SSL")
            .description("Verify SSL certificates for network operations")
            .category(EnvCategory::Tool)
            .config_path("tools.network.verify_ssl")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    // Web tools configuration (maps to WebToolsConfig fields)
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_WEB_USER_AGENT")
            .description("User agent for web tools")
            .category(EnvCategory::Tool)
            .config_path("tools.web_tools.user_agent") // Maps to WebToolsConfig.user_agent
            .default("llmspell-web/1.0")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_WEB_ALLOWED_DOMAINS")
            .description("Comma-separated list of allowed domains for web tools")
            .category(EnvCategory::Tool)
            .config_path("tools.web_tools.allowed_domains") // Maps to WebToolsConfig.allowed_domains
            .default("*")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_WEB_BLOCKED_DOMAINS")
            .description("Comma-separated list of blocked domains for web tools")
            .category(EnvCategory::Tool)
            .config_path("tools.web_tools.blocked_domains") // Maps to WebToolsConfig.blocked_domains
            .default("")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_WEB_MAX_REDIRECTS")
            .description("Maximum redirects to follow")
            .category(EnvCategory::Tool)
            .config_path("tools.web_tools.max_redirects") // Maps to WebToolsConfig.max_redirects
            .default("5")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid redirect count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_WEB_SCRAPING_DELAY")
            .description("Delay between scraping requests (milliseconds)")
            .category(EnvCategory::Tool)
            .config_path("tools.web_tools.scraping_delay_ms") // Maps to WebToolsConfig.scraping_delay_ms
            .default("1000")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid delay: {}", e))
            })
            .build(),
    )?;

    // Media tools configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_MEDIA_MAX_SIZE")
            .description("Maximum media file size (bytes)")
            .category(EnvCategory::Tool)
            .config_path("tools.media.max_file_size")
            .default("524288000") // 500MB
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid size: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_MEDIA_PROCESSING_TIMEOUT")
            .description("Media processing timeout (seconds)")
            .category(EnvCategory::Tool)
            .config_path("tools.media.processing_timeout_seconds")
            .default("300") // 5 minutes
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_MEDIA_IMAGE_MAX_DIMENSIONS")
            .description("Maximum image dimensions (width x height)")
            .category(EnvCategory::Tool)
            .config_path("tools.media.image_max_dimensions")
            .default("4096x4096")
            .build(),
    )?;

    // Database tools configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DB_CONNECTION_TIMEOUT")
            .description("Database connection timeout (seconds)")
            .category(EnvCategory::Tool)
            .config_path("tools.database.connection_timeout_seconds")
            .default("10")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DB_MAX_CONNECTIONS")
            .description("Maximum database connections")
            .category(EnvCategory::Tool)
            .config_path("tools.database.max_connections")
            .default("10")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid connection count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DB_ALLOWED_HOSTS")
            .description("Comma-separated list of allowed database hosts")
            .category(EnvCategory::Tool)
            .config_path("tools.database.allowed_hosts")
            .default("localhost,127.0.0.1")
            .build(),
    )?;

    // Email tools configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_EMAIL_SMTP_HOST")
            .description("SMTP host for email sending")
            .category(EnvCategory::Tool)
            .config_path("tools.email.smtp_host")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_EMAIL_SMTP_PORT")
            .description("SMTP port for email sending")
            .category(EnvCategory::Tool)
            .config_path("tools.email.smtp_port")
            .default("587")
            .validator(|v| {
                v.parse::<u16>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid port: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_EMAIL_FROM_ADDRESS")
            .description("Default from address for emails")
            .category(EnvCategory::Tool)
            .config_path("tools.email.from_address")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_EMAIL_RATE_LIMIT")
            .description("Email rate limit (emails per minute)")
            .category(EnvCategory::Tool)
            .config_path("tools.email.rate_limit_per_minute")
            .default("2") // 2 emails per minute = 120 per hour (reasonable for email)
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid rate limit: {}", e))
            })
            .build(),
    )?;

    // System tools configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC")
            .description("Allow process execution tools")
            .category(EnvCategory::Tool)
            .config_path("tools.system.allow_process_execution")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS")
            .description("Comma-separated list of allowed system commands")
            .category(EnvCategory::Tool)
            .config_path("tools.system.allowed_commands")
            .default("ls,cat,echo,pwd")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_SYSTEM_MAX_OUTPUT_SIZE")
            .description("Maximum output size from system commands (bytes)")
            .category(EnvCategory::Tool)
            .config_path("tools.system.max_output_size")
            .default("1048576") // 1MB
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid size: {}", e))
            })
            .build(),
    )?;

    // Data processing tools
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DATA_MAX_CSV_SIZE")
            .description("Maximum CSV file size for processing (bytes)")
            .category(EnvCategory::Tool)
            .config_path("tools.data.max_csv_size")
            .default("104857600") // 100MB
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid size: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DATA_MAX_JSON_DEPTH")
            .description("Maximum JSON nesting depth")
            .category(EnvCategory::Tool)
            .config_path("tools.data.max_json_depth")
            .default("100")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid depth: {}", e))
            })
            .build(),
    )?;

    // Academic tools
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_ACADEMIC_CITATION_API_KEY")
            .description("API key for citation services")
            .category(EnvCategory::Tool)
            .config_path("tools.academic.citation_api_key")
            .sensitive()
            .build(),
    )?;

    // Weather API key
    registry.register_var(
        EnvVarDefBuilder::new("WEATHER_API_KEY")
            .description("Weather API key")
            .category(EnvCategory::Tool)
            .config_path("tools.weather.api_key")
            .sensitive()
            .build(),
    )?;

    // Brave Search API key
    registry.register_var(
        EnvVarDefBuilder::new("BRAVE_API_KEY")
            .description("Brave Search API key")
            .category(EnvCategory::Tool)
            .config_path("tools.search.brave_api_key")
            .sensitive()
            .build(),
    )?;

    // Tavily API key
    registry.register_var(
        EnvVarDefBuilder::new("TAVILY_API_KEY")
            .description("Tavily Search API key")
            .category(EnvCategory::Tool)
            .config_path("tools.search.tavily_api_key")
            .sensitive()
            .build(),
    )?;

    // SerpApi key
    registry.register_var(
        EnvVarDefBuilder::new("SERPAPI_API_KEY")
            .description("SerpApi key")
            .category(EnvCategory::Tool)
            .config_path("tools.search.serpapi_api_key")
            .sensitive()
            .build(),
    )?;

    // SerperDev key
    registry.register_var(
        EnvVarDefBuilder::new("SERPERDEV_API_KEY")
            .description("Serper.dev API key")
            .category(EnvCategory::Tool)
            .config_path("tools.search.serper_api_key")
            .sensitive()
            .build(),
    )?;

    // GitHub API key
    registry.register_var(
        EnvVarDefBuilder::new("GITHUB_API_KEY")
            .description("GitHub API Token")
            .category(EnvCategory::Tool)
            .config_path("tools.github.api_key")
            .sensitive()
            .build(),
    )?;

    // PubMed API key
    registry.register_var(
        EnvVarDefBuilder::new("PUBMED_API_KEY")
            .description("PubMed API key")
            .category(EnvCategory::Tool)
            .config_path("tools.academic.pubmed_api_key")
            .sensitive()
            .build(),
    )?;

    // CORE API key
    registry.register_var(
        EnvVarDefBuilder::new("COREAC_API_KEY")
            .description("CORE Academic Search API key")
            .category(EnvCategory::Tool)
            .config_path("tools.academic.core_api_key")
            .sensitive()
            .build(),
    )?;

    // NewsAPI key
    registry.register_var(
        EnvVarDefBuilder::new("NEWSAPI_API_KEY")
            .description("NewsAPI key")
            .category(EnvCategory::Tool)
            .config_path("tools.news.api_key")
            .sensitive()
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_ACADEMIC_MAX_REFERENCES")
            .description("Maximum references to process")
            .category(EnvCategory::Tool)
            .config_path("tools.academic.max_references")
            .default("1000")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid count: {}", e))
            })
            .build(),
    )?;

    // Document tools
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DOC_MAX_PDF_SIZE")
            .description("Maximum PDF file size (bytes)")
            .category(EnvCategory::Tool)
            .config_path("tools.document.max_pdf_size")
            .default("52428800") // 50MB
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid size: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_DOC_EXTRACT_IMAGES")
            .description("Extract images from documents")
            .category(EnvCategory::Tool)
            .config_path("tools.document.extract_images")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    // Search tools
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_SEARCH_ENGINE")
            .description("Default search engine (google, bing, duckduckgo)")
            .category(EnvCategory::Tool)
            .config_path("tools.search.default_engine")
            .default("duckduckgo")
            .validator(|v| match v {
                "google" | "bing" | "duckduckgo" => Ok(()),
                _ => Err(format!("Invalid search engine: {}", v)),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_SEARCH_MAX_RESULTS")
            .description("Maximum search results to return")
            .category(EnvCategory::Tool)
            .config_path("tools.search.max_results")
            .default("10")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_TOOLS_SEARCH_API_KEY")
            .description("API key for search services")
            .category(EnvCategory::Tool)
            .config_path("tools.search.api_key")
            .sensitive()
            .build(),
    )?;

    Ok(())
}

/// Register session and hook environment variables
fn register_session_hook_vars(registry: &EnvRegistry) -> Result<(), String> {
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_SESSIONS_ENABLED")
            .description("Enable session management")
            .category(EnvCategory::Session)
            .config_path("runtime.sessions.enabled")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_SESSIONS_MAX")
            .description("Maximum number of concurrent sessions")
            .category(EnvCategory::Session)
            .config_path("runtime.sessions.max_sessions")
            .default("100")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_SESSIONS_TIMEOUT_SECONDS")
            .description("Session timeout in seconds")
            .category(EnvCategory::Session)
            .config_path("runtime.sessions.session_timeout_seconds")
            .default("3600")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid timeout: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_SESSIONS_BACKEND")
            .description("Session storage backend (memory, sqlite, postgres)")
            .category(EnvCategory::Session)
            .config_path("runtime.sessions.storage_backend")
            .default("memory")
            .validator(|v| match v {
                "memory" | "sqlite" | "postgres" => Ok(()),
                _ => Err(format!("Invalid backend: {}", v)),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_SESSIONS_MAX_ARTIFACTS")
            .description("Maximum artifacts per session")
            .category(EnvCategory::Session)
            .config_path("runtime.sessions.max_artifacts_per_session")
            .default("1000")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_HOOKS_ENABLED")
            .description("Enable hook system")
            .category(EnvCategory::Hook)
            .config_path("hooks.enabled")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_HOOKS_RATE_LIMIT")
            .description("Hook rate limiting (executions per minute)")
            .category(EnvCategory::Hook)
            .config_path("hooks.rate_limit_per_minute")
            .default("100")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid rate limit: {}", e))
            })
            .build(),
    )?;

    Ok(())
}

/// Register memory system environment variables
fn register_memory_vars(registry: &EnvRegistry) -> Result<(), String> {
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_ENABLED")
            .description("Enable memory system functionality")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.enabled")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME")
            .description("Provider name for memory consolidation LLM")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.consolidation.provider_name")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_CONSOLIDATION_BATCH_SIZE")
            .description("Number of episodes to consolidate in one batch")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.consolidation.batch_size")
            .default("10")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid batch size: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_CONSOLIDATION_MAX_CONCURRENT")
            .description("Maximum concurrent consolidation operations")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.consolidation.max_concurrent")
            .default("3")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid count: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_CONSOLIDATION_ACTIVE_SESSION_THRESHOLD_SECS")
            .description("Active session threshold in seconds")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.consolidation.active_session_threshold_secs")
            .default("300")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid threshold: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_DAEMON_ENABLED")
            .description("Enable background consolidation daemon")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.daemon.enabled")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_DAEMON_FAST_INTERVAL_SECS")
            .description("Fast consolidation interval in seconds")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.daemon.fast_interval_secs")
            .default("30")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid interval: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_DAEMON_NORMAL_INTERVAL_SECS")
            .description("Normal consolidation interval in seconds")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.daemon.normal_interval_secs")
            .default("300")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid interval: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_DAEMON_SLOW_INTERVAL_SECS")
            .description("Slow consolidation interval in seconds")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.daemon.slow_interval_secs")
            .default("600")
            .validator(|v| {
                v.parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid interval: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_DAEMON_QUEUE_THRESHOLD_FAST")
            .description("Queue size threshold for fast interval")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.daemon.queue_threshold_fast")
            .default("10")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid threshold: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_MEMORY_DAEMON_QUEUE_THRESHOLD_SLOW")
            .description("Queue size threshold for slow interval")
            .category(EnvCategory::Runtime)
            .config_path("runtime.memory.daemon.queue_threshold_slow")
            .default("3")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid threshold: {}", e))
            })
            .build(),
    )?;

    Ok(())
}

/// Register path discovery environment variables
fn register_path_vars(registry: &EnvRegistry) -> Result<(), String> {
    // Standard environment variables for path discovery
    // Note: These are read-only system variables used for config discovery
    // They don't map to config fields but are used by discover_config_file()
    registry.register_var(
        EnvVarDefBuilder::new("HOME")
            .description("User home directory (system)")
            .category(EnvCategory::Path)
            // No config_path - used directly by discovery logic
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("USERPROFILE")
            .description("Windows user profile directory (system)")
            .category(EnvCategory::Path)
            // No config_path - used directly by discovery logic
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("XDG_CONFIG_HOME")
            .description("XDG configuration directory (system)")
            .category(EnvCategory::Path)
            // No config_path - used directly by discovery logic
            .build(),
    )?;

    Ok(())
}

/// Register event system environment variables
fn register_event_vars(registry: &EnvRegistry) -> Result<(), String> {
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_ENABLED")
            .description("Enable event system globally")
            .category(EnvCategory::Runtime)
            .config_path("events.enabled")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_BUFFER_SIZE")
            .description("Event bus buffer size for queuing events")
            .category(EnvCategory::Runtime)
            .config_path("events.buffer_size")
            .default("10000")
            .validator(|v| {
                v.parse::<usize>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid buffer size: {}", e))
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EMIT_TIMING_EVENTS")
            .description("Enable timing/performance events")
            .category(EnvCategory::Runtime)
            .config_path("events.emit_timing_events")
            .default("true")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EMIT_STATE_EVENTS")
            .description("Enable state change events")
            .category(EnvCategory::Runtime)
            .config_path("events.emit_state_events")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EMIT_DEBUG_EVENTS")
            .description("Enable debug-level events")
            .category(EnvCategory::Runtime)
            .config_path("events.emit_debug_events")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_MAX_EVENTS_PER_SECOND")
            .description("Maximum events per second (rate limiting)")
            .category(EnvCategory::Runtime)
            .config_path("events.max_events_per_second")
            .validator(|v| {
                v.parse::<u32>()
                    .map(|_| ())
                    .map_err(|e| format!("Invalid rate limit: {}", e))
            })
            .build(),
    )?;

    // Filtering configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_FILTERING_INCLUDE_TYPES")
            .description("Event types to include (comma-separated glob patterns)")
            .category(EnvCategory::Runtime)
            .config_path("events.filtering.include_types")
            .default("*")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_FILTERING_EXCLUDE_TYPES")
            .description("Event types to exclude (comma-separated glob patterns)")
            .category(EnvCategory::Runtime)
            .config_path("events.filtering.exclude_types")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_FILTERING_INCLUDE_COMPONENTS")
            .description("Component IDs to include (comma-separated glob patterns)")
            .category(EnvCategory::Runtime)
            .config_path("events.filtering.include_components")
            .default("*")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_FILTERING_EXCLUDE_COMPONENTS")
            .description("Component IDs to exclude (comma-separated glob patterns)")
            .category(EnvCategory::Runtime)
            .config_path("events.filtering.exclude_components")
            .build(),
    )?;

    // Export configuration
    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EXPORT_STDOUT")
            .description("Export events to stdout (for debugging)")
            .category(EnvCategory::Runtime)
            .config_path("events.export.stdout")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EXPORT_FILE")
            .description("Export events to file (path)")
            .category(EnvCategory::Runtime)
            .config_path("events.export.file")
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EXPORT_WEBHOOK")
            .description("Export events to webhook (URL)")
            .category(EnvCategory::Runtime)
            .config_path("events.export.webhook")
            .validator(|v| {
                if v.starts_with("http://") || v.starts_with("https://") {
                    Ok(())
                } else {
                    Err("Webhook URL must start with http:// or https://".to_string())
                }
            })
            .build(),
    )?;

    registry.register_var(
        EnvVarDefBuilder::new("LLMSPELL_EVENTS_EXPORT_PRETTY_JSON")
            .description("Pretty-print JSON output")
            .category(EnvCategory::Runtime)
            .config_path("events.export.pretty_json")
            .default("false")
            .validator(|v| match v {
                "true" | "false" => Ok(()),
                _ => Err("Value must be 'true' or 'false'".to_string()),
            })
            .build(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_standard_vars() {
        let registry = EnvRegistry::isolated();

        // Register all standard variables
        register_standard_vars(&registry).unwrap();

        // Check core runtime variables
        assert!(registry.is_registered("LLMSPELL_DEFAULT_ENGINE"));
        assert!(registry.is_registered("LLMSPELL_MAX_CONCURRENT_SCRIPTS"));
        assert!(registry.is_registered("LLMSPELL_ALLOW_FILE_ACCESS"));

        // Check state variables
        assert!(registry.is_registered("LLMSPELL_STATE_ENABLED"));
        assert!(registry.is_registered("LLMSPELL_STATE_BACKEND"));
        assert!(registry.is_registered("LLMSPELL_STATE_MIGRATION_ENABLED"));
        assert!(registry.is_registered("LLMSPELL_STATE_BACKUP_ENABLED"));
        assert!(registry.is_registered("LLMSPELL_STATE_BACKUP_ON_MIGRATION"));

        // Check provider variables (standard and LLMSpell format)
        assert!(registry.is_registered("OPENAI_API_KEY"));
        assert!(registry.is_registered("ANTHROPIC_API_KEY"));
        assert!(registry.is_registered("LLMSPELL_PROVIDER_OPENAI_API_KEY"));
        assert!(registry.is_registered("LLMSPELL_PROVIDER_OPENAI_MODEL"));
        assert!(registry.is_registered("LLMSPELL_PROVIDER_OPENAI_TIMEOUT"));
        assert!(registry.is_registered("LLMSPELL_PROVIDER_OPENAI_MAX_RETRIES"));
        assert!(registry.is_registered("LLMSPELL_PROVIDER_ANTHROPIC_MODEL"));
        assert!(registry.is_registered("LLMSPELL_PROVIDER_ANTHROPIC_TIMEOUT"));

        // Check tool variables
        assert!(registry.is_registered("LLMSPELL_TOOLS_FILE_OPS_ENABLED"));
        assert!(registry.is_registered("LLMSPELL_TOOLS_MAX_FILE_SIZE"));
        assert!(registry.is_registered("LLMSPELL_TOOLS_RATE_LIMIT"));

        // Check session/hook variables
        assert!(registry.is_registered("LLMSPELL_SESSIONS_ENABLED"));
        assert!(registry.is_registered("LLMSPELL_SESSIONS_BACKEND"));
        assert!(registry.is_registered("LLMSPELL_SESSIONS_MAX_ARTIFACTS"));
        assert!(registry.is_registered("LLMSPELL_HOOKS_ENABLED"));
        assert!(registry.is_registered("LLMSPELL_HOOKS_RATE_LIMIT"));

        // Check path variables
        assert!(registry.is_registered("HOME"));
        assert!(registry.is_registered("XDG_CONFIG_HOME"));

        // Check categories
        let vars = registry.list_vars().unwrap();
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::Runtime)));
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::Provider)));
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::State)));
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::Tool)));
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::Session)));
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::Hook)));
        assert!(vars
            .iter()
            .any(|(_, _, cat, _)| matches!(cat, EnvCategory::Path)));

        // Count total variables (should be comprehensive)
        assert!(
            vars.len() >= 40,
            "Expected at least 40 variables, got {}",
            vars.len()
        );
    }

    #[test]
    fn test_build_config_from_registry() {
        let registry = EnvRegistry::isolated();

        // Register variables
        register_standard_vars(&registry).unwrap();

        // Set some overrides
        let mut overrides = std::collections::HashMap::new();
        overrides.insert(
            "LLMSPELL_DEFAULT_ENGINE".to_string(),
            "javascript".to_string(),
        );
        overrides.insert("LLMSPELL_STATE_ENABLED".to_string(), "true".to_string());
        overrides.insert("OPENAI_API_KEY".to_string(), "test-key".to_string());

        registry.with_overrides(overrides).unwrap();

        // Build config
        let config = registry.build_config().unwrap();

        // Check that values were applied correctly
        assert_eq!(config["default_engine"], "javascript");
        assert_eq!(config["runtime"]["state_persistence"]["enabled"], true);
        assert_eq!(config["providers"]["openai"]["api_key"], "test-key");
    }
}
