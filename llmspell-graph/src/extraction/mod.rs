//! Entity and relationship extraction from text
//!
//! Provides extractors for converting unstructured text into structured knowledge graph elements.
//!
//! # Extractors
//!
//! - `RegexExtractor`: Pattern-based extraction using regular expressions (Phase 13.2.4)
//! - Future: LLM-based extraction, coreference resolution, entity linking
//!
//! # Usage
//!
//! ```rust,ignore
//! use llmspell_graph::extraction::RegexExtractor;
//!
//! let extractor = RegexExtractor::new();
//! let text = "Rust is a systems programming language. Rust has memory safety.";
//! let entities = extractor.extract_entities(text);
//! let relationships = extractor.extract_relationships(text);
//! ```

pub mod regex;

pub use regex::RegexExtractor;
