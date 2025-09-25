//! Test REPL completion integration
//!
//! This test verifies that language-specific completions from the script executor
//! properly flow through to the REPL readline interface.

#[cfg(test)]
mod repl_completion_tests {
    use llmspell_core::error::LLMSpellError;
    use llmspell_core::traits::script_executor::{
        ScriptExecutionMetadata, ScriptExecutionOutput, ScriptExecutor,
    };
    use llmspell_kernel::repl::readline::{ScriptExecutorCompletionAdapter, ScriptCompletionProvider};
    use std::sync::Arc;
    use std::time::Duration;

    /// Mock script executor for testing
    struct MockScriptExecutor {
        completions: Vec<(String, String)>,
    }

    impl MockScriptExecutor {
        fn new() -> Self {
            Self {
                completions: vec![
                    ("print".to_string(), "function".to_string()),
                    ("pairs".to_string(), "function".to_string()),
                    ("table".to_string(), "module".to_string()),
                    ("string".to_string(), "module".to_string()),
                    ("myvar".to_string(), "variable".to_string()),
                ],
            }
        }
    }

    #[async_trait::async_trait]
    impl ScriptExecutor for MockScriptExecutor {
        async fn execute_script(
            &self,
            _script: &str,
        ) -> Result<ScriptExecutionOutput, LLMSpellError> {
            Ok(ScriptExecutionOutput {
                output: serde_json::json!("test"),
                console_output: vec![],
                metadata: ScriptExecutionMetadata {
                    duration: Duration::from_millis(10),
                    language: "test".to_string(),
                    exit_code: Some(0),
                    warnings: vec![],
                },
            })
        }

        fn language(&self) -> &'static str {
            "test"
        }

        async fn is_ready(&self) -> bool {
            true
        }

        fn get_completion_candidates(&self, line: &str, cursor_pos: usize) -> Vec<(String, String)> {
            // Get the word being completed
            let before_cursor = &line[..cursor_pos.min(line.len())];
            let word_start = before_cursor
                .rfind(|c: char| c.is_whitespace())
                .map_or(0, |i| i + 1);
            let word = &line[word_start..cursor_pos.min(line.len())];

            // Filter completions that match the prefix
            self.completions
                .iter()
                .filter(|(text, _)| text.starts_with(word))
                .cloned()
                .collect()
        }
    }

    #[test]
    fn test_completion_adapter() {
        // Create a mock script executor
        let executor = Arc::new(MockScriptExecutor::new());

        // Create the completion adapter
        let adapter = ScriptExecutorCompletionAdapter::new(executor.clone());

        // Test getting completions for "pr"
        let completions = adapter.get_completions("pr", 2);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "print");
        assert_eq!(completions[0].1, "function");

        // Test getting completions for "str"
        let completions = adapter.get_completions("str", 3);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "string");
        assert_eq!(completions[0].1, "module");

        // Test getting completions for empty prefix (should return all)
        let completions = adapter.get_completions("", 0);
        assert_eq!(completions.len(), 5);

        // Test getting completions in middle of line
        let completions = adapter.get_completions("local x = pr", 12);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "print");
    }

    #[test]
    fn test_completion_with_partial_word() {
        let executor = Arc::new(MockScriptExecutor::new());
        let adapter = ScriptExecutorCompletionAdapter::new(executor.clone());

        // Test partial completions
        let completions = adapter.get_completions("ta", 2);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "table");

        let completions = adapter.get_completions("pa", 2);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "pairs");
    }

    #[test]
    fn test_no_completions() {
        let executor = Arc::new(MockScriptExecutor::new());
        let adapter = ScriptExecutorCompletionAdapter::new(executor.clone());

        // Test with non-matching prefix
        let completions = adapter.get_completions("xyz", 3);
        assert_eq!(completions.len(), 0);
    }
}