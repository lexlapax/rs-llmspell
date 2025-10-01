//! ABOUTME: Document processing tools for various formats
//! ABOUTME: PDF text extraction, metadata parsing, and document analysis

#[cfg(feature = "pdf")]
pub mod pdf_processor;

#[cfg(feature = "pdf")]
pub use pdf_processor::PdfProcessorTool;
