//! Text processing utilities for NLP tasks
//!
//! Provides shared text processing utilities used across the LLMSpell framework
//! for natural language processing tasks like entity extraction, keyword analysis,
//! and text cleanup.
//!
//! # Modules
//!
//! - [`stopwords`](crate::text::stopwords): Common stopword lists for entity extraction filtering
//!
//! # Usage
//!
//! ```rust
//! use llmspell_utils::text::stopwords::is_stopword;
//!
//! assert!(is_stopword("The"));
//! assert!(is_stopword("However"));
//! assert!(!is_stopword("Rust"));
//! ```

pub mod stopwords;

// Re-export commonly used functions
pub use stopwords::is_stopword;
