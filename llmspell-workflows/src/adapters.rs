//! ABOUTME: Input/Output adapters for workflow-agent integration
//! ABOUTME: Provides conversions between agent and workflow types for seamless interoperability

use llmspell_core::execution_context::ExecutionContext;
use llmspell_core::types::{AgentInput, AgentOutput, OutputMetadata};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

use crate::types::{WorkflowInput, WorkflowOutput};

/// Adapter for converting between agent and workflow inputs
pub struct WorkflowInputAdapter;

impl WorkflowInputAdapter {
    /// Convert `AgentInput` to `WorkflowInput`
    ///
    /// Extracts workflow-specific parameters from `AgentInput`:
    /// - Main text becomes the primary input
    /// - Parameters are mapped to context
    /// - Special handling for timeout parameter
    pub fn from_agent_input(input: AgentInput) -> WorkflowInput {
        // Extract timeout if provided in parameters
        let timeout = input
            .parameters
            .get("timeout_ms")
            .and_then(serde_json::Value::as_u64)
            .map(Duration::from_millis)
            .or_else(|| {
                input
                    .parameters
                    .get("timeout_secs")
                    .and_then(serde_json::Value::as_u64)
                    .map(Duration::from_secs)
            });

        // Convert agent parameters to workflow context
        let mut context = HashMap::new();
        for (key, value) in input.parameters {
            // Skip special parameters that we handle separately
            if key != "timeout_ms" && key != "timeout_secs" {
                context.insert(key, value);
            }
        }

        // Add execution context to workflow context if present
        if let Some(exec_context) = &input.context {
            context.insert(
                "conversation_id".to_string(),
                json!(exec_context.conversation_id),
            );

            // Store any additional context info
            if let Some(session_id) = &exec_context.session_id {
                context.insert("session_id".to_string(), json!(session_id));
            }
        }

        // Create workflow input with text as primary input
        WorkflowInput {
            input: json!({
                "text": input.text,
                "original_type": "agent_input"
            }),
            context,
            timeout,
        }
    }

    /// Convert `WorkflowInput` back to `AgentInput`
    ///
    /// Useful for workflow composition where workflows call other agents
    pub fn to_agent_input(workflow_input: WorkflowInput) -> AgentInput {
        // Extract text from workflow input
        let text = workflow_input
            .input
            .get("text")
            .and_then(|v| v.as_str())
            .map_or_else(
                || {
                    // Fallback: serialize entire input as text
                    workflow_input.input.to_string()
                },
                std::string::ToString::to_string,
            );

        // Convert context back to parameters
        let mut parameters = HashMap::new();
        for (key, value) in workflow_input.context {
            parameters.insert(key, value);
        }

        // Add timeout back as parameter if present
        if let Some(timeout) = workflow_input.timeout {
            parameters.insert("timeout_ms".to_string(), json!(timeout.as_millis()));
        }

        // Extract conversation_id if it was stored and create execution context
        let context = parameters
            .remove("conversation_id")
            .and_then(|v| v.as_str().map(String::from))
            .map(ExecutionContext::with_conversation);

        AgentInput {
            text,
            media: Vec::new(),
            context,
            parameters,
            output_modalities: Vec::new(),
        }
    }
}

/// Adapter for converting between workflow and agent outputs
pub struct WorkflowOutputAdapter;

impl WorkflowOutputAdapter {
    /// Convert `WorkflowOutput` to `AgentOutput`
    ///
    /// Preserves workflow execution metadata and results
    #[must_use]
    pub fn to_agent_output(workflow_output: WorkflowOutput) -> AgentOutput {
        // Create output text based on success/failure
        let text = if workflow_output.success {
            match &workflow_output.output {
                Value::String(s) => s.clone(),
                Value::Object(map) if map.contains_key("result") => map
                    .get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Workflow completed successfully")
                    .to_string(),
                _ => format!("Workflow completed: {}", workflow_output.output),
            }
        } else {
            format!(
                "Workflow failed: {}",
                workflow_output.error.as_deref().unwrap_or("Unknown error")
            )
        };

        // Build metadata from workflow execution details
        let mut metadata = OutputMetadata {
            execution_time_ms: Some(
                workflow_output.duration.as_millis().min(u64::MAX as u128) as u64
            ),
            ..Default::default()
        };

        // Add workflow-specific metadata
        metadata.extra.insert(
            "workflow_success".to_string(),
            json!(workflow_output.success),
        );
        metadata.extra.insert(
            "steps_executed".to_string(),
            json!(workflow_output.steps_executed),
        );
        metadata.extra.insert(
            "steps_failed".to_string(),
            json!(workflow_output.steps_failed),
        );

        // Add final context as metadata
        for (key, value) in workflow_output.final_context {
            metadata.extra.insert(format!("context_{}", key), value);
        }

        // Include raw workflow output for complete data preservation
        metadata.extra.insert(
            "workflow_output".to_string(),
            workflow_output.output.clone(),
        );

        // Create agent output with text and metadata
        let mut agent_output = AgentOutput::text(text);
        agent_output.metadata = metadata;

        // If workflow output contains structured tool calls, we could map them
        // For now, we'll store the workflow output in metadata
        if workflow_output.output.is_object() || workflow_output.output.is_array() {
            agent_output.metadata.extra.insert(
                "structured_output".to_string(),
                workflow_output.output.clone(),
            );
        }

        agent_output
    }

    /// Convert `AgentOutput` back to `WorkflowOutput`
    ///
    /// Useful when agents are used as workflow steps
    pub fn from_agent_output(agent_output: AgentOutput, duration: Duration) -> WorkflowOutput {
        // Extract success status from metadata if available
        let success = agent_output
            .metadata
            .extra
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Default to success if not specified

        // Extract error message if present
        let error = if !success {
            Some(agent_output.text.clone())
        } else {
            None
        };

        // Convert agent output to workflow output
        // Check if there's structured output in metadata
        let output = agent_output
            .metadata
            .extra
            .get("structured_output")
            .cloned()
            .unwrap_or_else(|| json!(agent_output.text));

        // Extract context from metadata
        let mut final_context = HashMap::new();
        for (key, value) in &agent_output.metadata.extra {
            if key.starts_with("context_") {
                let context_key = key.strip_prefix("context_").unwrap().to_string();
                final_context.insert(context_key, value.clone());
            }
        }

        WorkflowOutput {
            output,
            success,
            duration,
            steps_executed: 1, // Single agent execution
            steps_failed: if success { 0 } else { 1 },
            final_context,
            error,
        }
    }
}

/// Convenience functions for adapter usage
pub mod prelude {
    use super::*;

    /// Convert `AgentInput` to `WorkflowInput`
    pub fn agent_to_workflow_input(input: AgentInput) -> WorkflowInput {
        WorkflowInputAdapter::from_agent_input(input)
    }

    /// Convert `WorkflowInput` to `AgentInput`
    pub fn workflow_to_agent_input(input: WorkflowInput) -> AgentInput {
        WorkflowInputAdapter::to_agent_input(input)
    }

    /// Convert `WorkflowOutput` to `AgentOutput`
    pub fn workflow_to_agent_output(output: WorkflowOutput) -> AgentOutput {
        WorkflowOutputAdapter::to_agent_output(output)
    }

    /// Convert `AgentOutput` to `WorkflowOutput`
    pub fn agent_to_workflow_output(output: AgentOutput, duration: Duration) -> WorkflowOutput {
        WorkflowOutputAdapter::from_agent_output(output, duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_to_workflow_input_conversion() {
        let mut params = HashMap::new();
        params.insert("timeout_ms".to_string(), json!(5000));
        params.insert("max_retries".to_string(), json!(3));

        let mut agent_input = AgentInput::text("Process this data");
        agent_input.parameters = params;
        agent_input.context = Some(ExecutionContext::with_conversation("conv-123".to_string()));

        let workflow_input = WorkflowInputAdapter::from_agent_input(agent_input);

        assert_eq!(
            workflow_input.input.get("text").unwrap().as_str().unwrap(),
            "Process this data"
        );
        assert_eq!(workflow_input.timeout, Some(Duration::from_millis(5000)));
        assert_eq!(
            workflow_input.context.get("max_retries").unwrap(),
            &json!(3)
        );
        assert_eq!(
            workflow_input.context.get("conversation_id").unwrap(),
            &json!("conv-123")
        );
    }

    #[test]
    fn test_workflow_to_agent_output_conversion() {
        let mut context = HashMap::new();
        context.insert("processed_items".to_string(), json!(42));

        let workflow_output = WorkflowOutput {
            output: json!({"result": "Data processed successfully"}),
            success: true,
            duration: Duration::from_secs(2),
            steps_executed: 5,
            steps_failed: 0,
            final_context: context,
            error: None,
        };

        let agent_output = WorkflowOutputAdapter::to_agent_output(workflow_output);

        assert_eq!(agent_output.text, "Data processed successfully");

        let metadata = &agent_output.metadata;
        assert_eq!(metadata.execution_time_ms, Some(2000));
        assert_eq!(
            metadata.extra.get("workflow_success").unwrap(),
            &json!(true)
        );
        assert_eq!(metadata.extra.get("steps_executed").unwrap(), &json!(5));
        assert_eq!(
            metadata.extra.get("context_processed_items").unwrap(),
            &json!(42)
        );
    }

    #[test]
    fn test_bidirectional_conversion() {
        let original_input = AgentInput::text("Test input");
        let workflow_input = WorkflowInputAdapter::from_agent_input(original_input.clone());
        let converted_back = WorkflowInputAdapter::to_agent_input(workflow_input);

        assert_eq!(converted_back.text, original_input.text);
    }
}
