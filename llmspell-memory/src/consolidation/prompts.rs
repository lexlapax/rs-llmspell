//! Prompt templates for LLM-driven consolidation
//!
//! Provides system and user prompts for extracting entities, relationships,
//! and making consolidation decisions (ADD/UPDATE/DELETE/NOOP).
//!
//! # Prompt Versioning
//!
//! Prompts are versioned for A/B testing and iterative improvement:
//! - **V1**: Initial implementation (JSON schema, few-shot examples)
//! - **V2+**: Future enhancements (longer examples, refined instructions, etc.)
//!
//! Use `PromptVersion` to select prompt version, tracked in metrics (Phase 13.5.4).

use crate::error::Result;
use crate::types::EpisodicEntry;
use serde::{Deserialize, Serialize};

use super::prompt_schema::{ConsolidationResponse, OutputFormat};

/// Prompt version for A/B testing and iterative improvement
///
/// Each version represents a distinct prompt template iteration.
/// Tracked in consolidation metrics for performance comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PromptVersion {
    /// Initial implementation (JSON schema, 4 few-shot examples)
    V1,
    // Future versions: V2, V3, etc.
}

impl Default for PromptVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl std::fmt::Display for PromptVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "V1"),
        }
    }
}

/// Token budget allocation for prompt components
#[derive(Debug, Clone, Copy)]
pub struct TokenBudget {
    /// Maximum tokens for episodic content (40% default)
    pub episodic_tokens: usize,
    /// Maximum tokens for semantic context (40% default)
    pub semantic_tokens: usize,
    /// Maximum tokens for instructions (20% default)
    pub instruction_tokens: usize,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            episodic_tokens: 1600,   // 40% of 4000 token context
            semantic_tokens: 1600,   // 40% of 4000 token context
            instruction_tokens: 800, // 20% of 4000 token context
        }
    }
}

/// Configuration for consolidation prompts
#[derive(Debug, Clone)]
pub struct ConsolidationPromptConfig {
    /// Output format (JSON or natural language)
    pub output_format: OutputFormat,
    /// LLM model name
    pub model: String,
    /// Temperature for sampling (0.0 = deterministic)
    pub temperature: f32,
    /// Token budget allocation
    pub token_budget: TokenBudget,
    /// Prompt version for A/B testing
    pub version: PromptVersion,
}

impl Default for ConsolidationPromptConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Json,
            model: "ollama/llama3.2:3b".to_string(),
            temperature: 0.0,
            token_budget: TokenBudget::default(),
            version: PromptVersion::default(),
        }
    }
}

/// Builder for consolidation prompts
#[derive(Debug, Clone)]
pub struct ConsolidationPromptBuilder {
    config: ConsolidationPromptConfig,
}

impl ConsolidationPromptBuilder {
    /// Create new prompt builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ConsolidationPromptConfig::default(),
        }
    }

    /// Set output format
    #[must_use]
    pub const fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.config.output_format = format;
        self
    }

    /// Set model name
    #[must_use]
    pub fn with_model(mut self, model: String) -> Self {
        self.config.model = model;
        self
    }

    /// Set temperature
    #[must_use]
    pub const fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature;
        self
    }

    /// Set token budget
    #[must_use]
    pub const fn with_token_budget(mut self, budget: TokenBudget) -> Self {
        self.config.token_budget = budget;
        self
    }

    /// Set prompt version
    #[must_use]
    pub const fn with_version(mut self, version: PromptVersion) -> Self {
        self.config.version = version;
        self
    }

    /// Get current prompt version
    #[must_use]
    pub const fn version(&self) -> PromptVersion {
        self.config.version
    }

    /// Build system prompt
    ///
    /// Selects prompt template based on output format and version.
    /// Currently only V1 is implemented; future versions will have
    /// version-specific logic here.
    ///
    /// # Errors
    ///
    /// Returns error if prompt generation fails.
    pub fn build_system_prompt(&self) -> Result<String> {
        // Version selection (currently only V1)
        match self.config.version {
            PromptVersion::V1 => Ok(match self.config.output_format {
                OutputFormat::Json => Self::system_prompt_json(),
                OutputFormat::NaturalLanguage => Self::system_prompt_natural_language(),
            }),
            // Future versions: add version-specific prompt selection
        }
    }

    /// Build user prompt from episodic entry and semantic context
    ///
    /// # Errors
    ///
    /// Returns error if prompt generation fails.
    pub fn build_user_prompt(
        &self,
        episodic: &EpisodicEntry,
        semantic_context: &str,
    ) -> Result<String> {
        let episodic_text = Self::truncate_to_tokens(&episodic.content, self.config.token_budget.episodic_tokens);
        let semantic_text = Self::truncate_to_tokens(semantic_context, self.config.token_budget.semantic_tokens);

        Ok(format!(
            "# Episodic Memory Entry\n\n\
            **Session**: {}\n\
            **Role**: {}\n\
            **Timestamp**: {}\n\
            **Content**:\n{}\n\n\
            # Semantic Context (Existing Knowledge Graph)\n\n\
            {}\n\n\
            # Task\n\n\
            Analyze the episodic memory entry and decide what consolidation operations to perform.",
            episodic.session_id,
            episodic.role,
            episodic.timestamp,
            episodic_text,
            if semantic_text.is_empty() {
                "(No existing entities in knowledge graph)"
            } else {
                semantic_text
            }
        ))
    }

    /// System prompt for JSON output format
    fn system_prompt_json() -> String {
        format!(
            "# Role\n\n\
            You are a knowledge consolidation engine that extracts structured information \
            from episodic memories (interaction history) and integrates it into a semantic \
            knowledge graph.\n\n\
            # Output Format\n\n\
            Respond with **valid JSON only** following this schema:\n\n\
            ```json\n\
            {{\n  \
              \"entities\": [\n    \
                {{\n      \
                  \"id\": \"unique-id\",\n      \
                  \"name\": \"Entity Name\",\n      \
                  \"entity_type\": \"category\",\n      \
                  \"properties\": {{}}\n    \
                }}\n  \
              ],\n  \
              \"relationships\": [\n    \
                {{\n      \
                  \"from_entity\": \"entity-id-1\",\n      \
                  \"to_entity\": \"entity-id-2\",\n      \
                  \"relationship_type\": \"has_feature\",\n      \
                  \"properties\": {{}}\n    \
                }}\n  \
              ],\n  \
              \"decisions\": [\n    \
                {{\"type\": \"ADD\", \"entity_id\": \"id\"}},\n    \
                {{\"type\": \"UPDATE\", \"entity_id\": \"id\", \"changes\": {{}}}},\n    \
                {{\"type\": \"DELETE\", \"entity_id\": \"id\"}},\n    \
                {{\"type\": \"NOOP\"}}\n  \
              ],\n  \
              \"reasoning\": \"Brief explanation of decisions\"\n\
            }}\n\
            ```\n\n\
            # Decision Criteria\n\n\
            - **ADD**: Create new entity when episodic content mentions a new concept/person/thing\n\
            - **UPDATE**: Merge new information into existing entity (check semantic context)\n\
            - **DELETE**: Remove entity if explicitly stated as obsolete/incorrect/deprecated\n\
            - **NOOP**: Skip if episodic content has no extractable knowledge (small talk, weather, etc.)\n\n\
            # Few-Shot Examples\n\n\
            {}",
            Self::few_shot_examples_json()
        )
    }

    /// System prompt for natural language output format
    fn system_prompt_natural_language() -> String {
        "# Role\n\n\
        You are a knowledge consolidation engine that extracts information \
        from episodic memories and describes what operations should be performed \
        on the knowledge graph.\n\n\
        # Output Format\n\n\
        Respond in natural language describing:\n\
        1. What entities to ADD (if any)\n\
        2. What entities to UPDATE (if any)\n\
        3. What entities to DELETE (if any)\n\
        4. If NOOP (no action needed)\n\n\
        # Decision Criteria\n\n\
        - **ADD**: Create new entity when episodic content mentions a new concept\n\
        - **UPDATE**: Merge new information into existing entity\n\
        - **DELETE**: Remove entity if obsolete/incorrect/deprecated\n\
        - **NOOP**: Skip if no extractable knowledge\n"
            .to_string()
    }

    /// Few-shot examples for JSON output format
    fn few_shot_examples_json() -> String {
        r#"## Example 1: ADD Decision

**Episodic**: "Rust is a systems programming language with memory safety."

**Output**:
```json
{
  "entities": [
    {
      "id": "rust-lang",
      "name": "Rust",
      "entity_type": "programming_language",
      "properties": {"paradigm": "multi-paradigm", "feature": "memory safety"}
    }
  ],
  "relationships": [],
  "decisions": [
    {"type": "ADD", "entity_id": "rust-lang"}
  ],
  "reasoning": "New programming language entity extracted"
}
```

## Example 2: UPDATE Decision

**Episodic**: "Rust also has zero-cost abstractions."
**Existing**: Entity(rust-lang, features: ["memory safety"])

**Output**:
```json
{
  "entities": [],
  "relationships": [],
  "decisions": [
    {
      "type": "UPDATE",
      "entity_id": "rust-lang",
      "changes": {"features": "zero-cost abstractions"}
    }
  ],
  "reasoning": "Adding new feature to existing Rust entity"
}
```

## Example 3: DELETE Decision

**Episodic**: "Python 2.7 is deprecated and no longer supported."
**Existing**: Entity(python27, status: "active")

**Output**:
```json
{
  "entities": [],
  "relationships": [],
  "decisions": [
    {"type": "DELETE", "entity_id": "python27"}
  ],
  "reasoning": "Python 2.7 explicitly deprecated"
}
```

## Example 4: NOOP Decision

**Episodic**: "The weather is nice today."

**Output**:
```json
{
  "entities": [],
  "relationships": [],
  "decisions": [
    {"type": "NOOP"}
  ],
  "reasoning": "No extractable knowledge (small talk)"
}
```
"#
        .to_string()
    }

    /// Truncate text to approximate token count
    ///
    /// Uses rough heuristic: 1 token ≈ 4 characters
    fn truncate_to_tokens(text: &str, max_tokens: usize) -> &str {
        let max_chars = max_tokens * 4;
        if text.len() <= max_chars {
            text
        } else {
            &text[..max_chars]
        }
    }
}

impl Default for ConsolidationPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse LLM response into consolidation response
///
/// # Errors
///
/// Returns error if response cannot be parsed as JSON or natural language.
pub fn parse_llm_response(response: &str, format: OutputFormat) -> Result<ConsolidationResponse> {
    match format {
        OutputFormat::Json => ConsolidationResponse::from_json(response),
        OutputFormat::NaturalLanguage => {
            // For natural language, return empty response (parsing in Task 13.5.2b)
            Ok(ConsolidationResponse::empty())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ConsolidationPromptConfig::default();
        assert_eq!(config.output_format, OutputFormat::Json);
        assert_eq!(config.model, "ollama/llama3.2:3b");
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.token_budget.episodic_tokens, 1600);
    }

    #[test]
    fn test_builder_with_json_format() {
        let builder = ConsolidationPromptBuilder::new()
            .with_output_format(OutputFormat::Json)
            .with_temperature(0.3);

        assert_eq!(builder.config.output_format, OutputFormat::Json);
        assert_eq!(builder.config.temperature, 0.3);
    }

    #[test]
    fn test_system_prompt_json() {
        let builder = ConsolidationPromptBuilder::new();
        let prompt = builder.build_system_prompt().unwrap();

        assert!(prompt.contains("knowledge consolidation engine"));
        assert!(prompt.contains("valid JSON"));
        assert!(prompt.contains("ADD"));
        assert!(prompt.contains("UPDATE"));
        assert!(prompt.contains("DELETE"));
        assert!(prompt.contains("NOOP"));
        assert!(prompt.contains("Example 1"));
    }

    #[test]
    fn test_system_prompt_natural_language() {
        let builder = ConsolidationPromptBuilder::new()
            .with_output_format(OutputFormat::NaturalLanguage);
        let prompt = builder.build_system_prompt().unwrap();

        assert!(prompt.contains("natural language"));
        assert!(prompt.contains("ADD"));
        assert!(!prompt.contains("JSON"));
    }

    #[test]
    fn test_user_prompt() {
        let builder = ConsolidationPromptBuilder::new();
        let entry = EpisodicEntry::new(
            "session-123".to_string(),
            "user".to_string(),
            "Rust is a systems programming language.".to_string(),
        );

        let prompt = builder
            .build_user_prompt(&entry, "Existing: Entity(Rust, type=language)")
            .unwrap();

        assert!(prompt.contains("session-123"));
        assert!(prompt.contains("user"));
        assert!(prompt.contains("Rust is a systems programming language"));
        assert!(prompt.contains("Existing: Entity(Rust"));
    }

    #[test]
    fn test_user_prompt_empty_semantic_context() {
        let builder = ConsolidationPromptBuilder::new();
        let entry = EpisodicEntry::new(
            "session-123".to_string(),
            "user".to_string(),
            "New content".to_string(),
        );

        let prompt = builder.build_user_prompt(&entry, "").unwrap();

        assert!(prompt.contains("(No existing entities"));
    }

    #[test]
    fn test_truncate_to_tokens() {
        let text = "a".repeat(10000);
        let truncated = ConsolidationPromptBuilder::truncate_to_tokens(&text, 100);

        // 100 tokens ≈ 400 chars
        assert_eq!(truncated.len(), 400);
    }

    #[test]
    fn test_parse_json_response() {
        let json = r#"{
            "entities": [],
            "relationships": [],
            "decisions": [{"type": "NOOP"}],
            "reasoning": "Test"
        }"#;

        let response = parse_llm_response(json, OutputFormat::Json).unwrap();
        assert_eq!(response.decisions.len(), 1);
    }

    #[test]
    fn test_parse_natural_language_response() {
        let text = "Add entity Rust as programming_language";
        let response = parse_llm_response(text, OutputFormat::NaturalLanguage).unwrap();

        // Natural language parsing returns empty (implemented in Task 13.5.2b)
        assert!(response.is_empty());
    }

    #[test]
    fn test_token_budget_default() {
        let budget = TokenBudget::default();
        assert_eq!(budget.episodic_tokens, 1600);
        assert_eq!(budget.semantic_tokens, 1600);
        assert_eq!(budget.instruction_tokens, 800);

        // Total = 4000 tokens
        assert_eq!(
            budget.episodic_tokens + budget.semantic_tokens + budget.instruction_tokens,
            4000
        );
    }

    #[test]
    fn test_few_shot_examples() {
        let examples = ConsolidationPromptBuilder::few_shot_examples_json();

        assert!(examples.contains("Example 1: ADD"));
        assert!(examples.contains("Example 2: UPDATE"));
        assert!(examples.contains("Example 3: DELETE"));
        assert!(examples.contains("Example 4: NOOP"));
        assert!(examples.contains("Rust"));
        assert!(examples.contains("Python 2.7"));
    }

    #[test]
    fn test_prompt_version_default() {
        assert_eq!(PromptVersion::default(), PromptVersion::V1);
    }

    #[test]
    fn test_prompt_version_display() {
        assert_eq!(PromptVersion::V1.to_string(), "V1");
    }

    #[test]
    fn test_builder_with_version() {
        let builder = ConsolidationPromptBuilder::new().with_version(PromptVersion::V1);
        assert_eq!(builder.version(), PromptVersion::V1);
    }

    #[test]
    fn test_config_includes_version() {
        let config = ConsolidationPromptConfig::default();
        assert_eq!(config.version, PromptVersion::V1);
    }

    #[test]
    fn test_system_prompt_with_version() {
        let builder = ConsolidationPromptBuilder::new().with_version(PromptVersion::V1);
        let prompt = builder.build_system_prompt().unwrap();

        // V1 prompt should contain JSON schema
        assert!(prompt.contains("valid JSON"));
        assert!(prompt.contains("Example 1"));
    }

    #[test]
    fn test_version_serialization() {
        use serde_json;
        let version = PromptVersion::V1;
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, "\"V1\"");

        let deserialized: PromptVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PromptVersion::V1);
    }
}
