use axum::{
    extract::{Path, State},
    Json,
};
use llmspell_templates::{
    core::{TemplateMetadata, TemplateParams},
    registry::global_registry,
    validation::ConfigSchema,
};
use serde::{Deserialize, Serialize};

use crate::error::WebError;
use crate::state::AppState;
use crate::workflow_builder::{NodePosition, NodeStatus, WorkflowGraphBuilder, WorkflowNode};
use llmspell_kernel::api::KernelHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// List all available templates
#[utoipa::path(
    get,
    path = "/api/templates",
    tag = "templates",
    responses(
        (status = 200, description = "List available templates", body = Vec<serde_json::Value>)
    )
)]
pub async fn list_templates() -> Result<Json<Vec<TemplateMetadata>>, WebError> {
    let registry = global_registry();
    let templates = registry.list_metadata();

    // Sort by name for better UX
    let mut templates = templates;
    templates.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Json(templates))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TemplateDetails {
    #[schema(value_type = Object)]
    pub metadata: TemplateMetadata,
    #[schema(value_type = Object)]
    pub schema: ConfigSchema,
}

/// Get details for a specific template
#[utoipa::path(
    get,
    path = "/api/templates/{id}",
    tag = "templates",
    params(
        ("id" = String, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Get template details", body = TemplateDetails),
        (status = 404, description = "Template not found")
    )
)]
pub async fn get_template(Path(id): Path<String>) -> Result<Json<TemplateDetails>, WebError> {
    let registry = global_registry();

    match registry.get(&id) {
        Ok(template) => Ok(Json(TemplateDetails {
            metadata: template.metadata().clone(),
            schema: template.config_schema(),
        })),
        Err(_) => Err(WebError::NotFound(format!("Template '{}' not found", id))),
    }
}

/// Request body for launching a template
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LaunchTemplateRequest {
    /// Parameters for the template
    #[schema(value_type = Option<Object>)]
    pub params: Option<TemplateParams>,
    /// Optional session ID to attach (if not creating new)
    pub session_id: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct LaunchResponse {
    pub session_id: String,
    pub template_id: String,
    pub status: String,
}

/// Launch a template (Create Session)
#[utoipa::path(
    post,
    path = "/api/templates/{id}/launch",
    tag = "templates",
    params(
        ("id" = String, Path, description = "Template ID")
    ),
    request_body = LaunchTemplateRequest,
    responses(
        (status = 200, description = "Template launched successfully", body = LaunchResponse),
        (status = 404, description = "Template not found"),
        (status = 400, description = "Invalid parameters")
    )
)]
pub async fn launch_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<LaunchTemplateRequest>,
) -> Result<Json<LaunchResponse>, WebError> {
    let registry = global_registry();

    let template = match registry.get(&id) {
        Ok(t) => t,
        Err(_) => return Err(WebError::NotFound(format!("Template '{}' not found", id))),
    };

    let params = payload.params.unwrap_or_default();

    // Validate parameters
    if let Err(e) = template.validate(&params) {
        return Err(WebError::BadRequest(format!("Invalid parameters: {}", e)));
    }

    // Create a real session using the Kernel's SessionManager
    let kernel = state.kernel.lock().await;

    // Build session options based on template
    let session_name = payload
        .session_id
        .clone()
        .unwrap_or_else(|| format!("template-{}", id));

    let options_builder = llmspell_kernel::sessions::CreateSessionOptionsBuilder::default()
        .name(session_name)
        .created_by("web-user".to_string())
        .add_tag("template".to_string())
        .add_tag(id.clone());

    let options = options_builder
        .metadata(params.values.clone().into_iter().collect())
        .build();

    let session_manager = kernel.session_manager();
    let session_id = session_manager
        .create_session(options)
        .await
        .map_err(|e| WebError::Internal(format!("Failed to create session: {}", e)))?;

    info!("Launched template '{}' as session {}", id, session_id);

    // NEW: Execute template asynchronously
    let session_id_clone = session_id;
    let kernel_clone = state.kernel.clone();
    let template_clone = template.clone();
    let params_clone = serde_json::to_value(params.values).unwrap_or_default();

    // Verify infrastructure availability and clone for background execution
    let tool_registry = state
        .tool_registry
        .as_ref()
        .ok_or_else(|| WebError::Internal("Infrastructure unavailable: tool_registry".to_string()))?
        .clone();
    let agent_registry = state
        .agent_registry
        .as_ref()
        .ok_or_else(|| {
            WebError::Internal("Infrastructure unavailable: agent_registry".to_string())
        })?
        .clone();
    let workflow_factory = state
        .workflow_factory
        .as_ref()
        .ok_or_else(|| {
            WebError::Internal("Infrastructure unavailable: workflow_factory".to_string())
        })?
        .clone();
    let provider_manager = state
        .provider_manager
        .as_ref()
        .ok_or_else(|| {
            WebError::Internal("Infrastructure unavailable: provider_manager".to_string())
        })?
        .clone();
    let provider_config = state
        .provider_config
        .as_ref()
        .ok_or_else(|| {
            WebError::Internal("Infrastructure unavailable: provider_config".to_string())
        })?
        .clone();

    let exec_params = TemplateExecutionParams {
        kernel: kernel_clone,
        session_id: session_id_clone.to_string(),
        template: template_clone,
        parameters: params_clone,
        tool_registry,
        agent_registry,
        workflow_factory,
        provider_manager,
        provider_config,
    };

    tokio::spawn(async move {
        if let Err(e) = execute_template_and_update_session(exec_params).await {
            tracing::error!("Template execution failed: {}", e);
        }
    });

    Ok(Json(LaunchResponse {
        session_id: session_id.to_string(),
        template_id: id,
        status: "started".to_string(),
    }))
}

struct TemplateExecutionParams {
    kernel: Arc<Mutex<KernelHandle>>,
    session_id: String,
    template: Arc<dyn llmspell_templates::Template>,
    parameters: serde_json::Value,
    tool_registry: Arc<llmspell_tools::ToolRegistry>,
    agent_registry: Arc<llmspell_agents::FactoryRegistry>,
    workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,
    provider_manager: Arc<llmspell_providers::ProviderManager>,
    provider_config: Arc<llmspell_config::providers::ProviderManagerConfig>,
}

async fn execute_template_and_update_session(
    params: TemplateExecutionParams,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kernel = params.kernel;
    let session_id = params.session_id;
    let template = params.template;
    let parameters = params.parameters;
    // 1. Create workflow graph builder
    let graph_builder = WorkflowGraphBuilder::new();

    // 2. Build execution context
    // No need to lock kernel for components anymore

    let mut builder = llmspell_templates::ExecutionContext::builder()
        .with_tool_registry(params.tool_registry)
        .with_agent_registry(params.agent_registry)
        .with_workflow_factory(params.workflow_factory)
        .with_providers(params.provider_manager)
        .with_provider_config(params.provider_config)
        .with_session_id(session_id.clone());

    // 2a. Inject Step Callback for real-time visualization
    let graph_builder_clone = graph_builder.clone();
    let session_id_clone = session_id.clone();
    let kernel_clone = kernel.clone();

    let callback = move |event: llmspell_templates::core::StepEvent| {
        let gb = graph_builder_clone.clone();
        let sid = session_id_clone.clone();
        let k = kernel_clone.clone();

        tokio::spawn(async move {
            // Update graph in-memory
            gb.upsert_node_from_event(event).await;

            // Build current snapshot
            let workflow = gb.build().await;

            // Persist to session state
            let kernel_lock = k.lock().await;
            let session_manager = kernel_lock.session_manager();

            // Just update state check
            if let Ok(session_id_typed) = std::str::FromStr::from_str(&sid) {
                if let Ok(session) = session_manager.get_session(&session_id_typed).await {
                    let _ = session
                        .set_state(
                            "workflow_execution".to_string(),
                            serde_json::to_value(&workflow).unwrap_or_default(),
                        )
                        .await;
                }
            }
        });
    };

    builder = builder.with_step_callback(callback);

    // If available, add memory etc.
    // For now we use the basic components.

    let context = builder.build()?;

    // 3. Execute template
    let params_inner: llmspell_templates::TemplateParams = parameters.into();
    let result = template.execute(params_inner, context).await;

    let (output, status, error) = match result {
        Ok(output) => (
            Some(serde_json::to_value(&output.result)?),
            NodeStatus::Completed,
            None,
        ),
        Err(e) => (None, NodeStatus::Failed, Some(e.to_string())),
    };

    // 4. Update final status
    let kernel_lock = kernel.lock().await;
    let session_manager = kernel_lock.session_manager();

    // Ensure we have at least one node (root)
    if graph_builder.build().await.nodes.is_empty() {
        graph_builder
            .add_node(WorkflowNode {
                id: "root".to_string(),
                label: template.metadata().name.clone(),
                type_: "template".to_string(),
                status: status.clone(),
                started_at: Some(chrono::Utc::now()),
                completed_at: Some(chrono::Utc::now()),
                duration_ms: None,
                output: output.clone(),
                error: error.clone(),
                position: NodePosition { x: 400.0, y: 100.0 }, // Top
            })
            .await;
    }

    // Final save of workflow
    let workflow = graph_builder.build().await;

    // Parse SessionId, handling error if invalid (though it should be valid as we created it)
    if let Ok(session_id_typed) = std::str::FromStr::from_str(&session_id) {
        if let Ok(session) = session_manager.get_session(&session_id_typed).await {
            session
                .set_state(
                    "workflow_execution".to_string(),
                    serde_json::to_value(&workflow)?,
                )
                .await?;

            // Update overall session status
            match status {
                NodeStatus::Completed => {
                    if let Err(e) = session_manager.complete_session(&session_id_typed).await {
                        tracing::warn!("Failed to complete session: {}", e);
                    }
                }
                NodeStatus::Failed => {
                    // SessionManager doesn't have fail_session, call directly on session and save
                    if let Err(e) = session.fail().await {
                        tracing::warn!("Failed to mark session as failed: {}", e);
                    } else if let Err(e) = session_manager.save_session(&session).await {
                        tracing::warn!("Failed to save failed session: {}", e);
                    }
                }
                _ => {} // Running/Pending, do nothing
            }
        }
    }

    Ok(())
}
