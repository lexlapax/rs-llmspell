//! Comprehensive tab completion tests for the REPL
//!
//! Tests command completion, file paths, and script-specific completions.

#[cfg(test)]
mod tab_completion_tests {
    use llmspell_kernel::repl::readline::{ReplHelper, ScriptCompletionProvider};
    use rustyline::completion::Completer;
    use rustyline::Context;
    use std::sync::Arc;

    use std::fs;
    use tempfile::TempDir;

    /// Mock script completion provider for testing
    struct MockCompletionProvider {
        completions: Vec<(String, String)>,
    }

    impl ScriptCompletionProvider for MockCompletionProvider {
        fn get_completions(&self, _line: &str, _cursor_pos: usize) -> Vec<(String, String)> {
            self.completions.clone()
        }
    }

    /// Test command completion matching
    #[test]
    fn test_command_completion() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Test .help completion
        let (start, candidates) = helper.complete(".he", 3, &ctx).unwrap();
        assert_eq!(start, 0);
        assert!(candidates.iter().any(|p| p.display == ".help"));

        // Test multiple matches
        let (_start, candidates) = helper.complete(".h", 2, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display == ".help"));
        assert!(candidates.iter().any(|p| p.display == ".history"));
        assert!(candidates.iter().any(|p| p.display == ".hist"));

        // Test exact match
        let (_start, candidates) = helper.complete(".exit", 5, &ctx).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].display, ".exit");

        // Test debug commands
        let (start, candidates) = helper.complete("db:", 3, &ctx).unwrap();
        assert_eq!(start, 0);
        assert!(candidates.iter().any(|p| p.display == "db:"));

        // Test debug command shortcuts
        let (_start, candidates) = helper.complete("b", 1, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display == "break"));
        assert!(candidates.iter().any(|p| p.display == "b"));
        assert!(candidates.iter().any(|p| p.display == "backtrace"));
    }

    /// Test file path completion
    #[test]
    fn test_file_path_completion() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create test files and directories
        fs::create_dir(test_dir.join("scripts")).unwrap();
        fs::write(test_dir.join("scripts/test.lua"), "").unwrap();
        fs::write(test_dir.join("scripts/example.lua"), "").unwrap();
        fs::write(test_dir.join("config.txt"), "").unwrap();
        fs::create_dir(test_dir.join("data")).unwrap();

        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Change to test directory for testing
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(test_dir).unwrap();

        // Test .load command path completion
        let (_start, candidates) = helper.complete(".load ", 6, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display.contains("scripts/")));
        assert!(candidates.iter().any(|p| p.display.contains("config.txt")));
        assert!(candidates.iter().any(|p| p.display.contains("data/")));

        // Test partial path completion
        let (_start, candidates) = helper.complete(".load scripts/", 14, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display.contains("test.lua")));
        assert!(candidates.iter().any(|p| p.display.contains("example.lua")));

        // Test .save command path completion
        let (_start, candidates) = helper.complete(".save ", 6, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display.contains("scripts/")));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    /// Test partial matches and ambiguous completions
    #[test]
    fn test_partial_and_ambiguous_matches() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Ambiguous prefix ".c"
        let (_start, candidates) = helper.complete(".c", 2, &ctx).unwrap();
        assert!(candidates.len() > 1);
        assert!(candidates.iter().any(|p| p.display == ".clear"));
        assert!(candidates.iter().any(|p| p.display == ".cls"));
        assert!(candidates.iter().any(|p| p.display == ".cd"));
        assert!(candidates.iter().any(|p| p.display == ".clear-history"));

        // Longer prefix reduces ambiguity
        let (_start, candidates) = helper.complete(".cle", 4, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display == ".clear"));
        assert!(candidates.iter().any(|p| p.display == ".clear-history"));
        assert!(!candidates.iter().any(|p| p.display == ".cd"));
    }

    /// Test case sensitivity
    #[test]
    fn test_case_sensitivity() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Commands are case-sensitive
        let (_start, candidates) = helper.complete(".HELP", 5, &ctx).unwrap();
        assert!(candidates.is_empty()); // No match for uppercase

        let (_start, candidates) = helper.complete(".help", 5, &ctx).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].display, ".help");
    }

    /// Test script-specific completions integration
    #[test]
    fn test_script_completions() {
        let helper = ReplHelper::new();

        // Set up mock completion provider
        let provider = MockCompletionProvider {
            completions: vec![
                ("print".to_string(), "function".to_string()),
                ("pairs".to_string(), "function".to_string()),
                ("myVar".to_string(), "variable".to_string()),
            ],
        };
        helper.set_script_completion_provider(Arc::new(provider));

        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Test script completion (not a command)
        let (_start, candidates) = helper.complete("pr", 2, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display.contains("print")));

        // Should not get script completions for commands
        let (_start, candidates) = helper.complete(".pr", 3, &ctx).unwrap();
        assert!(candidates.is_empty());
    }

    /// Test completion at different cursor positions
    #[test]
    fn test_cursor_position_completion() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Completion in middle of line
        let line = ".help some args";
        let (start, candidates) = helper.complete(line, 5, &ctx).unwrap(); // After ".help"
        assert_eq!(start, 0);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].display, ".help");

        // Completion at start of word
        let (_start, candidates) = helper.complete(".clear", 0, &ctx).unwrap();
        assert!(!candidates.is_empty()); // Should show all commands

        // Completion at end of line
        let (_start, candidates) = helper.complete(".hist", 5, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display == ".history"));
        assert!(candidates.iter().any(|p| p.display == ".hist"));
    }

    /// Performance test with many candidates
    #[test]
    fn test_performance_with_many_candidates() {
        use std::time::Instant;

        let helper = ReplHelper::new();

        // Create provider with many completions
        let mut completions = Vec::new();
        for i in 0..1000 {
            completions.push((format!("var_{}", i), "variable".to_string()));
        }

        let provider = MockCompletionProvider { completions };
        helper.set_script_completion_provider(Arc::new(provider));

        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Measure completion time
        let start = Instant::now();
        let (_, candidates) = helper.complete("var_", 4, &ctx).unwrap();
        let duration = start.elapsed();

        // Should complete quickly even with many candidates
        assert!(duration.as_millis() < 100, "Completion took {:?}", duration);
        assert_eq!(candidates.len(), 1000);
    }

    /// Test empty input completion
    #[test]
    fn test_empty_input_completion() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Empty input should show all commands
        let (start, candidates) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(start, 0);
        assert!(candidates.len() > 10); // Should have many commands
    }

    /// Test special characters in completions
    #[test]
    fn test_special_characters() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Test hyphenated commands
        let (_start, candidates) = helper.complete(".clear-", 7, &ctx).unwrap();
        assert!(candidates.iter().any(|p| p.display == ".clear-history"));
    }

    /// Test no completions available
    #[test]
    fn test_no_completions() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Non-existent command
        let (_start, candidates) = helper.complete(".nonexistent", 12, &ctx).unwrap();
        assert!(candidates.is_empty());

        // Non-matching prefix
        let (_start, candidates) = helper.complete(".xyz", 4, &ctx).unwrap();
        assert!(candidates.is_empty());
    }

    /// Test completion hint generation
    #[test]
    fn test_completion_hints() {
        use rustyline::hint::Hinter;

        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = Context::new(&history);

        // Should provide hints for partial commands
        let hint = helper.hint(".hel", 4, &ctx);
        assert_eq!(hint, Some("p".to_string())); // Complete to .help

        let hint = helper.hint(".hist", 5, &ctx);
        assert_eq!(hint, Some("ory".to_string())); // Complete to .history

        // No hint for complete command
        let hint = helper.hint(".help", 5, &ctx);
        assert_eq!(hint, None);

        // No hint for non-matching
        let hint = helper.hint(".xyz", 4, &ctx);
        assert_eq!(hint, None);
    }
}
