//! ABOUTME: Utility tools module for various helper functions
//! ABOUTME: Provides template rendering, validation, and other utilities

pub mod base64_encoder;
pub mod calculator;
pub mod data_validation;
pub mod date_time_handler;
pub mod diff_calculator;
pub mod hash_calculator;
#[cfg(feature = "templates")]
pub mod template_engine;
pub mod text_manipulator;
pub mod uuid_generator;

pub use base64_encoder::Base64EncoderTool;
pub use calculator::CalculatorTool;
pub use data_validation::{
    DataValidationConfig, DataValidationTool, ValidationError, ValidationResult, ValidationRule,
    ValidationRules,
};
pub use date_time_handler::DateTimeHandlerTool;
pub use diff_calculator::DiffCalculatorTool;
pub use hash_calculator::{HashCalculatorConfig, HashCalculatorTool};
#[cfg(feature = "templates")]
pub use template_engine::{TemplateEngineConfig, TemplateEngineTool};
pub use text_manipulator::{TextManipulatorConfig, TextManipulatorTool, TextOperation};
pub use uuid_generator::{UuidGeneratorConfig, UuidGeneratorTool};
