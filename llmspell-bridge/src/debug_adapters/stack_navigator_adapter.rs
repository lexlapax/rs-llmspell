//! Protocol adapter for `StackNavigator`
//!
//! Wraps the existing `StackNavigator` to implement the `DebugCapability` trait,
//! enabling protocol-based access to stack navigation functionality.

use crate::execution_bridge::StackFrame;
use crate::stack_navigator::StackNavigator;
use async_trait::async_trait;
use llmspell_core::{
    debug::{DebugCapability, DebugRequest, DebugResponse, LocationInfo, StackFrameInfo},
    Result,
};
use std::sync::Arc;

/// Adapter that wraps `StackNavigator` to provide `DebugCapability`
pub struct StackNavigatorAdapter {
    stack_navigator: Arc<dyn StackNavigator>,
    /// Cached stack frames for navigation
    stack_cache: Arc<tokio::sync::RwLock<Vec<StackFrame>>>,
}

impl StackNavigatorAdapter {
    /// Create a new adapter wrapping a `StackNavigator`
    pub fn new(stack_navigator: Arc<dyn StackNavigator>) -> Self {
        Self {
            stack_navigator,
            stack_cache: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Convert `StackFrame` to `StackFrameInfo`
    fn convert_stack_frame(frame: &StackFrame, index: usize) -> StackFrameInfo {
        StackFrameInfo {
            index,
            name: frame.name.clone(),
            location: LocationInfo {
                source: frame.source.clone(),
                line: frame.line,
                column: frame.column,
                function: Some(frame.name.clone()),
            },
            locals: frame.locals.iter().map(|v| v.name.clone()).collect(),
            is_user_code: frame.is_user_code,
        }
    }

    /// Update the cached stack frames
    pub async fn update_stack(&self, frames: Vec<StackFrame>) {
        let mut cache = self.stack_cache.write().await;
        *cache = frames;
    }
}

#[async_trait]
impl DebugCapability for StackNavigatorAdapter {
    async fn process_debug_request(&self, request: DebugRequest) -> Result<DebugResponse> {
        match request {
            DebugRequest::NavigateStack { frame_index } => {
                let stack = self.stack_cache.read().await;

                // Use the StackNavigator trait to navigate
                match self.stack_navigator.navigate_to_frame(frame_index, &stack) {
                    Ok(frame) => {
                        let frame_info = Self::convert_stack_frame(&frame, frame_index);
                        Ok(DebugResponse::StackFrame(frame_info))
                    }
                    Err(e) => Ok(DebugResponse::Error {
                        message: format!("Failed to navigate to frame {frame_index}: {e}"),
                        details: None,
                    }),
                }
            }

            DebugRequest::GetStackTrace => {
                let stack = self.stack_cache.read().await;
                let frame_infos: Vec<StackFrameInfo> = stack
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Self::convert_stack_frame(f, i))
                    .collect();
                drop(stack);
                Ok(DebugResponse::StackTrace(frame_infos))
            }

            _ => Ok(DebugResponse::Error {
                message: "Request should be handled by different capability".to_string(),
                details: Some(format!("Request type: {request:?}")),
            }),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "stack_navigation".to_string(),
            "stack_trace".to_string(),
            "frame_switching".to_string(),
        ]
    }

    fn name(&self) -> &'static str {
        "stack_navigator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_bridge::Variable;
    use crate::execution_context::SharedExecutionContext;
    use serde_json::Value as JsonValue;
    use std::collections::HashMap;
    use std::error::Error;

    // Mock implementation for testing
    struct MockStackNavigator;

    impl StackNavigator for MockStackNavigator {
        fn navigate_to_frame(
            &self,
            frame_index: usize,
            stack: &[StackFrame],
        ) -> std::result::Result<StackFrame, Box<dyn Error>> {
            stack.get(frame_index).cloned().ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Frame index out of bounds",
                )) as Box<dyn Error>
            })
        }

        fn format_frame(&self, frame: &StackFrame) -> String {
            format!("{} at {}:{}", frame.name, frame.source, frame.line)
        }

        fn get_frame_variables(
            &self,
            _frame: &StackFrame,
            _context: &SharedExecutionContext,
        ) -> HashMap<String, JsonValue> {
            HashMap::new()
        }

        fn format_stack_trace(&self, stack: &[StackFrame], current_frame: usize) -> String {
            stack
                .iter()
                .enumerate()
                .map(|(i, frame)| {
                    let marker = if i == current_frame { ">" } else { " " };
                    format!(
                        "{} {} at {}:{}",
                        marker, frame.name, frame.source, frame.line
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    #[tokio::test]
    async fn test_stack_navigator_adapter() {
        let navigator = Arc::new(MockStackNavigator);
        let adapter = StackNavigatorAdapter::new(navigator);

        // Test capabilities
        assert!(adapter
            .capabilities()
            .contains(&"stack_navigation".to_string()));
        assert_eq!(adapter.name(), "stack_navigator");

        // Set up test stack
        let test_frame = StackFrame {
            id: "frame_0".to_string(),
            name: "test_function".to_string(),
            source: "test.lua".to_string(),
            line: 10,
            column: Some(5),
            locals: vec![Variable {
                name: "test_var".to_string(),
                value: "test_value".to_string(),
                var_type: "string".to_string(),
                has_children: false,
                reference: None,
            }],
            is_user_code: true,
        };
        adapter.update_stack(vec![test_frame]).await;

        // Test stack trace retrieval
        let response = adapter
            .process_debug_request(DebugRequest::GetStackTrace)
            .await
            .unwrap();

        if let DebugResponse::StackTrace(frames) = response {
            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0].name, "test_function");
        } else {
            panic!("Expected StackTrace response");
        }
    }
}
