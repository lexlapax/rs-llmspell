//! Context assembly and structuring
//!
//! Assembles ranked chunks into coherent context with temporal ordering,
//! token budget management, and metadata preservation.

pub mod assembler;

pub use assembler::ContextAssembler;
