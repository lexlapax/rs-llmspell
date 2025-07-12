//! ABOUTME: Search tools for web, semantic, and code searching
//! ABOUTME: Provides various search capabilities with rate limiting and result formatting

pub mod providers;
pub mod web_search;

pub use web_search::{WebSearchConfig, WebSearchTool};
