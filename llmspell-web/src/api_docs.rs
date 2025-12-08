use crate::handlers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::providers::list_providers,
        handlers::static_config::restart_server,
        handlers::static_config::get_config_source,
        handlers::static_config::update_config_source,
        handlers::static_config::get_config_schema,
        handlers::static_config::get_profiles,
        handlers::config::get_config,
        handlers::config::update_config,
        handlers::scripts::execute_script,
        handlers::sessions::list_sessions,
        handlers::sessions::get_session,
        handlers::memory::search_memory,
        handlers::agents::list_agents,
        handlers::agents::execute_agent,
        handlers::tools::list_tools,
        handlers::tools::execute_tool,
        handlers::templates::list_templates,
        handlers::templates::get_template,
        handlers::templates::launch_template,
    ),
    tags(
        (name = "providers", description = "LLM Provider management"),
        (name = "config", description = "System configuration"),
        (name = "scripts", description = "Script execution execution"),
        (name = "sessions", description = "Session management"),
        (name = "memory", description = "Memory management"),
        (name = "agents", description = "Agent management"),
        (name = "tools", description = "Tool management"),
        (name = "templates", description = "Template management"),
    )
)]
pub struct ApiDoc;
