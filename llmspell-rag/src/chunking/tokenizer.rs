//! Tokenizer utilities for accurate token counting

use anyhow::Result;
use tiktoken_rs::{cl100k_base, p50k_base, p50k_edit, r50k_base, CoreBPE};

/// Trait for token counting
pub trait TokenCounter: Send + Sync {
    /// Count tokens in text
    fn count_tokens(&self, text: &str) -> usize;

    /// Tokenize text and return token IDs
    fn tokenize(&self, text: &str) -> Vec<usize>;

    /// Get tokenizer name
    fn name(&self) -> &str;
}

/// Tiktoken-based token counter for OpenAI models
#[derive(Debug)]
pub struct TiktokenCounter {
    /// The underlying BPE tokenizer
    tokenizer: CoreBPE,

    /// Model name for reference
    model_name: String,
}

impl TiktokenCounter {
    /// Create tokenizer for specific model
    pub fn for_model(model: &str) -> Result<Self> {
        let (tokenizer, model_name) = match model {
            // GPT-4 and GPT-3.5-turbo use cl100k_base
            "gpt-4"
            | "gpt-4-32k"
            | "gpt-4-0125-preview"
            | "gpt-4-turbo-preview"
            | "gpt-3.5-turbo"
            | "gpt-3.5-turbo-16k"
            | "gpt-3.5-turbo-0125"
            | "text-embedding-3-small"
            | "text-embedding-3-large" => (cl100k_base()?, "cl100k_base"),

            // Older GPT-3 models
            "text-davinci-003" | "text-davinci-002" | "code-davinci-002" => {
                (p50k_base()?, "p50k_base")
            }

            // Edit models
            "text-davinci-edit-001" | "code-davinci-edit-001" => (p50k_edit()?, "p50k_edit"),

            // Legacy models
            "davinci" | "curie" | "babbage" | "ada" | "text-embedding-ada-002" => {
                (r50k_base()?, "r50k_base")
            }

            // Default to cl100k_base for unknown models
            _ => (cl100k_base()?, "cl100k_base"),
        };

        Ok(Self {
            tokenizer,
            model_name: model_name.to_string(),
        })
    }

    /// Create default tokenizer (cl100k_base)
    pub fn default() -> Result<Self> {
        Ok(Self {
            tokenizer: cl100k_base()?,
            model_name: "cl100k_base".to_string(),
        })
    }
}

impl TokenCounter for TiktokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer.encode_ordinary(text).len()
    }

    fn tokenize(&self, text: &str) -> Vec<usize> {
        self.tokenizer.encode_ordinary(text)
    }

    fn name(&self) -> &str {
        &self.model_name
    }
}

/// Simple character-based token counter (fallback)
#[derive(Debug, Default)]
pub struct CharacterTokenCounter {
    /// Characters per token estimate
    chars_per_token: usize,
}

impl CharacterTokenCounter {
    /// Create with custom characters per token
    #[must_use]
    pub fn new(chars_per_token: usize) -> Self {
        Self { chars_per_token }
    }

    /// Create with default estimate (4 chars per token)
    #[must_use]
    pub fn default_estimate() -> Self {
        Self { chars_per_token: 4 }
    }
}

impl TokenCounter for CharacterTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        text.len() / self.chars_per_token.max(1)
    }

    fn tokenize(&self, text: &str) -> Vec<usize> {
        // Simple mock tokenization - not real tokens
        text.chars()
            .enumerate()
            .step_by(self.chars_per_token.max(1))
            .map(|(i, _)| i)
            .collect()
    }

    fn name(&self) -> &str {
        "character_estimate"
    }
}

/// Token counter factory
#[derive(Debug)]
pub struct TokenCounterFactory;

impl TokenCounterFactory {
    /// Create token counter for a specific model
    pub fn for_model(model: &str) -> Box<dyn TokenCounter> {
        match TiktokenCounter::for_model(model) {
            Ok(counter) => Box::new(counter),
            Err(_) => Box::new(CharacterTokenCounter::default_estimate()),
        }
    }

    /// Create default token counter
    pub fn default() -> Box<dyn TokenCounter> {
        match TiktokenCounter::default() {
            Ok(counter) => Box::new(counter),
            Err(_) => Box::new(CharacterTokenCounter::default_estimate()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiktoken_counter() {
        let counter = TiktokenCounter::for_model("gpt-3.5-turbo").unwrap();

        let text = "Hello, world! This is a test.";
        let token_count = counter.count_tokens(text);

        // Should be reasonable token count
        assert!(token_count > 0);
        assert!(token_count < text.len()); // Tokens should be fewer than characters

        let tokens = counter.tokenize(text);
        assert_eq!(tokens.len(), token_count);
    }

    #[test]
    fn test_character_counter() {
        let counter = CharacterTokenCounter::new(4);

        let text = "Hello, world!";
        let token_count = counter.count_tokens(text);

        assert_eq!(token_count, text.len() / 4);
    }

    #[test]
    fn test_model_specific_tokenizers() {
        // Test that different models get appropriate tokenizers
        let models = vec![
            "gpt-4",
            "gpt-3.5-turbo",
            "text-embedding-3-small",
            "text-davinci-003",
            "text-embedding-ada-002",
        ];

        for model in models {
            let counter = TiktokenCounter::for_model(model);
            assert!(counter.is_ok(), "Failed to create tokenizer for {}", model);
        }
    }

    #[test]
    fn test_factory() {
        let counter = TokenCounterFactory::for_model("gpt-4");
        assert_eq!(counter.name(), "cl100k_base");

        let default = TokenCounterFactory::default();
        assert!(!default.name().is_empty());
    }

    #[test]
    fn test_unknown_model_fallback() {
        let counter = TokenCounterFactory::for_model("unknown-model-xyz");
        // Should fall back gracefully
        let count = counter.count_tokens("test text");
        assert!(count > 0);
    }
}
