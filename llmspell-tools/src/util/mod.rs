//! ABOUTME: Utility tools module for various helper functions
//! ABOUTME: Provides template rendering, validation, and other utilities

pub mod base64_encoder;
pub mod data_validation;
pub mod hash_calculator;
pub mod template_engine;
pub mod text_manipulator;
pub mod uuid_generator;

pub use base64_encoder::Base64EncoderTool;
pub use data_validation::{
    DataValidationConfig, DataValidationTool, ValidationError, ValidationResult, ValidationRule,
    ValidationRules,
};
pub use hash_calculator::{HashCalculatorConfig, HashCalculatorTool};
pub use template_engine::{TemplateEngineConfig, TemplateEngineTool};
pub use text_manipulator::{TextManipulatorConfig, TextManipulatorTool, TextOperation};
pub use uuid_generator::{UuidGeneratorConfig, UuidGeneratorTool};
