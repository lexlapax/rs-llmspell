use utoipa::OpenApi;
use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::providers::list_providers,
        handlers::static_config::restart_server,
    ),
    tags(
        (name = "providers", description = "LLM Provider management"),
        (name = "config", description = "System configuration"),
    )
)]
pub struct ApiDoc;
