//! ABOUTME: JavaScript script engine implementation of ScriptEngineBridge
//! ABOUTME: Provides ES2020 JavaScript with async/await and generator-based streaming

pub mod engine;
pub mod globals;

pub use engine::JSEngine;
