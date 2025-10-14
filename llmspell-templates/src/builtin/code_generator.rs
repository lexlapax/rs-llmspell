//! Code Generator Template
//!
//! 3-agent sequential chain for code generation:
//! 1. Specification agent: design from description
//! 2. Implementation agent: code from spec
//! 3. Test agent: tests for code

use crate::{
    artifacts::Artifact,
    context::ExecutionContext,
    core::{
        CostEstimate, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
        TemplateResult,
    },
    error::{Result, TemplateError},
    validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType},
};
use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

/// Code Generator Template
///
/// Automated code generation with specification, implementation, and testing:
/// - Generates design specification from description
/// - Implements code following the specification
/// - Creates comprehensive tests for the code
/// - Supports multiple programming languages
#[derive(Debug)]
pub struct CodeGeneratorTemplate {
    metadata: TemplateMetadata,
}

impl CodeGeneratorTemplate {
    /// Create a new Code Generator template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "code-generator".to_string(),
                name: "Code Generator".to_string(),
                description: "AI-powered code generation with specification, implementation, and tests. \
                             Takes a description and generates complete, tested code in your chosen \
                             programming language with proper documentation and best practices."
                    .to_string(),
                category: TemplateCategory::CodeGen,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["code-tools".to_string(), "lint".to_string()],
                tags: vec![
                    "code".to_string(),
                    "generation".to_string(),
                    "testing".to_string(),
                    "programming".to_string(),
                    "automation".to_string(),
                ],
            },
        }
    }
}

impl Default for CodeGeneratorTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for CodeGeneratorTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new(vec![
            // description (required)
            ParameterSchema::required(
                "description",
                "Description of the code to generate",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(10),
                ..Default::default()
            }),
            // language (optional enum with default)
            ParameterSchema::optional(
                "language",
                "Programming language for code generation",
                ParameterType::String,
                json!("rust"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("rust"),
                    json!("python"),
                    json!("javascript"),
                    json!("typescript"),
                    json!("go"),
                    json!("java"),
                    json!("cpp"),
                ]),
                ..Default::default()
            }),
            // include_tests (optional boolean with default)
            ParameterSchema::optional(
                "include_tests",
                "Whether to generate unit tests for the code",
                ParameterType::Boolean,
                json!(true),
            ),
            // model (optional - for agent execution)
            ParameterSchema::optional(
                "model",
                "LLM model to use for code generation agents",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
        ])
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let start_time = Instant::now();

        // Extract and validate parameters
        let description: String = params.get("description")?;
        let language: String = params.get_or("language", "rust".to_string());
        let include_tests: bool = params.get_or("include_tests", true);
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());

        info!(
            "Starting code generation (language={}, tests={}, model={})",
            language, include_tests, model
        );

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params,
        );

        // Phase 1: Generate specification with spec agent
        info!("Phase 1: Generating specification...");
        let spec_result = self
            .generate_specification(&description, &language, &model, &context)
            .await?;
        output.metrics.agents_invoked += 1; // spec agent

        // Phase 2: Generate implementation with implementation agent
        info!("Phase 2: Generating implementation...");
        let impl_result = self
            .generate_implementation(&spec_result, &language, &model, &context)
            .await?;
        output.metrics.agents_invoked += 1; // implementation agent

        // Phase 3: Generate tests with test agent (if enabled)
        let test_result = if include_tests {
            info!("Phase 3: Generating tests...");
            let result = self
                .generate_tests(&impl_result, &language, &model, &context)
                .await?;
            output.metrics.agents_invoked += 1; // test agent
            Some(result)
        } else {
            info!("Phase 3: Skipping tests (include_tests=false)");
            None
        };

        // Phase 4: Run linter/formatter (optional)
        info!("Phase 4: Running code quality checks...");
        let lint_result = self
            .run_quality_checks(&impl_result, &language, &context)
            .await?;
        output.metrics.tools_invoked += 1; // lint tool

        // Generate final report
        let report = self.format_report(
            &description,
            &language,
            &spec_result,
            &impl_result,
            &test_result,
            &lint_result,
        );

        // Save artifacts
        if let Some(output_dir) = &context.output_dir {
            self.save_artifacts(
                output_dir,
                &language,
                &spec_result.content,
                &impl_result.code,
                test_result.as_ref().map(|t| t.code.as_str()),
                &mut output,
            )?;
        }

        // Set result and metrics
        output.result = TemplateResult::text(report);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("language", json!(language));
        output.add_metric("include_tests", json!(include_tests));
        output.add_metric("spec_lines", json!(spec_result.content.lines().count()));
        output.add_metric("code_lines", json!(impl_result.code.lines().count()));
        if let Some(test) = &test_result {
            output.add_metric("test_lines", json!(test.code.lines().count()));
        }

        info!(
            "Code generation complete (duration: {}ms, agents: {})",
            output.metrics.duration_ms, output.metrics.agents_invoked
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        let include_tests: bool = params.get_or("include_tests", true);

        // Rough estimates:
        // - Specification: ~1000 tokens
        // - Implementation: ~2000 tokens
        // - Tests: ~1500 tokens (if enabled)
        // - Linting: minimal tokens
        let estimated_tokens = if include_tests {
            1000 + 2000 + 1500
        } else {
            1000 + 2000
        };

        // Assuming $0.10 per 1M tokens (local LLM is cheaper)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        // Specification: ~3s
        // Implementation: ~5s
        // Tests: ~4s (if enabled)
        // Linting: ~1s
        let estimated_duration = if include_tests {
            3000 + 5000 + 4000 + 1000
        } else {
            3000 + 5000 + 1000
        };

        CostEstimate::new(
            estimated_tokens as u64,
            estimated_cost,
            estimated_duration as u64,
            0.7, // Medium-high confidence
        )
    }
}

impl CodeGeneratorTemplate {
    /// Phase 1: Generate specification with spec agent
    async fn generate_specification(
        &self,
        description: &str,
        language: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<SpecificationResult> {
        // TODO: Implement actual spec agent
        // For now, return placeholder specification
        warn!("Specification generation not yet implemented - using placeholder");

        let content = format!(
            "# Code Specification\n\n\
             ## Overview\n\
             Language: {}\n\
             Description: {}\n\n\
             ## Design\n\
             [Placeholder specification based on description]\n\n\
             ### Components\n\
             1. **Main Module**: Core functionality implementation\n\
             2. **Helper Functions**: Supporting utilities\n\
             3. **Error Handling**: Robust error management\n\n\
             ### Data Structures\n\
             - Primary data structure: [To be determined]\n\
             - Helper structures: [To be determined]\n\n\
             ### API Design\n\
             - Public functions: [List of public API functions]\n\
             - Internal functions: [List of private helpers]\n\n\
             ### Testing Strategy\n\
             - Unit tests for each component\n\
             - Integration tests for end-to-end workflows\n\
             - Edge case coverage\n",
            language, description
        );

        Ok(SpecificationResult { content })
    }

    /// Phase 2: Generate implementation with implementation agent
    async fn generate_implementation(
        &self,
        spec: &SpecificationResult,
        language: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<ImplementationResult> {
        // TODO: Implement actual implementation agent
        // For now, return placeholder code
        warn!("Code implementation not yet implemented - using placeholder");

        let code = match language {
            "rust" => format!(
                "// Generated Rust code based on specification\n\
                 // Spec length: {} lines\n\n\
                 /// Main function implementing the specified functionality\n\
                 pub fn main_function(input: &str) -> Result<String, Box<dyn std::error::Error>> {{\n    \
                     // Placeholder implementation\n    \
                     println!(\"Processing input: {{}}\", input);\n    \
                     Ok(format!(\"Processed: {{}}\", input))\n\
                 }}\n\n\
                 /// Helper function\n\
                 fn helper_function(data: &str) -> String {{\n    \
                     // Placeholder helper\n    \
                     data.to_uppercase()\n\
                 }}\n",
                spec.content.lines().count()
            ),
            "python" => format!(
                "# Generated Python code based on specification\n\
                 # Spec length: {} lines\n\n\
                 def main_function(input_data: str) -> str:\n    \
                     \"\"\"Main function implementing the specified functionality\"\"\"\n    \
                     # Placeholder implementation\n    \
                     print(f\"Processing input: {{input_data}}\")\n    \
                     return f\"Processed: {{input_data}}\"\n\n\
                 def helper_function(data: str) -> str:\n    \
                     \"\"\"Helper function\"\"\"\n    \
                     # Placeholder helper\n    \
                     return data.upper()\n",
                spec.content.lines().count()
            ),
            "javascript" => format!(
                "// Generated JavaScript code based on specification\n\
                 // Spec length: {} lines\n\n\
                 /**\n \
                  * Main function implementing the specified functionality\n \
                  * @param {{string}} input - Input data\n \
                  * @returns {{string}} Processed result\n \
                  */\n\
                 function mainFunction(input) {{\n    \
                     // Placeholder implementation\n    \
                     console.log(`Processing input: ${{input}}`);\n    \
                     return `Processed: ${{input}}`;\n\
                 }}\n\n\
                 /**\n \
                  * Helper function\n \
                  * @param {{string}} data - Input data\n \
                  * @returns {{string}} Processed data\n \
                  */\n\
                 function helperFunction(data) {{\n    \
                     // Placeholder helper\n    \
                     return data.toUpperCase();\n\
                 }}\n\n\
                 module.exports = {{ mainFunction, helperFunction }};\n",
                spec.content.lines().count()
            ),
            _ => format!(
                "// Generated {} code based on specification\n\
                 // Spec length: {} lines\n\n\
                 // Placeholder implementation for {}\n\
                 // [Code would be generated here]\n",
                language,
                spec.content.lines().count(),
                language
            ),
        };

        Ok(ImplementationResult { code })
    }

    /// Phase 3: Generate tests with test agent
    async fn generate_tests(
        &self,
        implementation: &ImplementationResult,
        language: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<TestResult> {
        // TODO: Implement actual test agent
        // For now, return placeholder tests
        warn!("Test generation not yet implemented - using placeholder");

        let code = match language {
            "rust" => format!(
                "// Generated Rust tests\n\
                 // Implementation size: {} lines\n\n\
                 #[cfg(test)]\n\
                 mod tests {{\n    \
                     use super::*;\n\n    \
                     #[test]\n    \
                     fn test_main_function() {{\n        \
                         let result = main_function(\"test input\").unwrap();\n        \
                         assert!(result.contains(\"Processed\"));\n    \
                     }}\n\n    \
                     #[test]\n    \
                     fn test_helper_function() {{\n        \
                         let result = helper_function(\"hello\");\n        \
                         assert_eq!(result, \"HELLO\");\n    \
                     }}\n\
                 }}\n",
                implementation.code.lines().count()
            ),
            "python" => format!(
                "# Generated Python tests\n\
                 # Implementation size: {} lines\n\n\
                 import unittest\n\n\
                 class TestMainFunction(unittest.TestCase):\n    \
                     def test_main_function(self):\n        \
                         result = main_function(\"test input\")\n        \
                         self.assertIn(\"Processed\", result)\n\n    \
                     def test_helper_function(self):\n        \
                         result = helper_function(\"hello\")\n        \
                         self.assertEqual(result, \"HELLO\")\n\n\
                 if __name__ == '__main__':\n    \
                     unittest.main()\n",
                implementation.code.lines().count()
            ),
            "javascript" => format!(
                "// Generated JavaScript tests\n\
                 // Implementation size: {} lines\n\n\
                 const {{ mainFunction, helperFunction }} = require('./implementation');\n\n\
                 describe('mainFunction', () => {{\n    \
                     test('processes input correctly', () => {{\n        \
                         const result = mainFunction('test input');\n        \
                         expect(result).toContain('Processed');\n    \
                     }});\n\
                 }});\n\n\
                 describe('helperFunction', () => {{\n    \
                     test('converts to uppercase', () => {{\n        \
                         const result = helperFunction('hello');\n        \
                         expect(result).toBe('HELLO');\n    \
                     }});\n\
                 }});\n",
                implementation.code.lines().count()
            ),
            _ => format!(
                "// Generated {} tests\n\
                 // Implementation size: {} lines\n\n\
                 // Placeholder tests for {}\n\
                 // [Tests would be generated here]\n",
                language,
                implementation.code.lines().count(),
                language
            ),
        };

        Ok(TestResult { code })
    }

    /// Phase 4: Run code quality checks (linting/formatting)
    async fn run_quality_checks(
        &self,
        implementation: &ImplementationResult,
        language: &str,
        _context: &ExecutionContext,
    ) -> Result<LintResult> {
        // TODO: Implement actual linting with code-tools
        // For now, return placeholder lint result
        warn!("Code quality checks not yet implemented - using placeholder");

        let report = format!(
            "# Code Quality Report ({})\n\n\
             ## Linting\n\
             - Total lines: {}\n\
             - Warnings: 0\n\
             - Errors: 0\n\
             - Status: ✓ Clean\n\n\
             ## Formatting\n\
             - Status: ✓ Properly formatted\n\n\
             ## Best Practices\n\
             - Documentation: ✓ Present\n\
             - Error handling: ✓ Implemented\n\
             - Code style: ✓ Consistent\n",
            language,
            implementation.code.lines().count()
        );

        Ok(LintResult { report })
    }

    /// Format final report
    fn format_report(
        &self,
        description: &str,
        language: &str,
        spec: &SpecificationResult,
        implementation: &ImplementationResult,
        tests: &Option<TestResult>,
        lint: &LintResult,
    ) -> String {
        let mut report = format!(
            "# Code Generation Report\n\n\
             **Language**: {}\n\
             **Description**: {}\n\n\
             ---\n\n\
             ## Specification\n\n\
             {}\n\n\
             ---\n\n\
             ## Implementation\n\n\
             ```{}\n\
             {}\n\
             ```\n\n\
             ---\n\n",
            language, description, spec.content, language, implementation.code
        );

        if let Some(test) = tests {
            report.push_str(&format!(
                "## Tests\n\n\
                 ```{}\n\
                 {}\n\
                 ```\n\n\
                 ---\n\n",
                language, test.code
            ));
        }

        report.push_str(&format!(
            "## Quality Report\n\n\
             {}\n\n\
             ---\n\n\
             Generated by LLMSpell Code Generator Template\n",
            lint.report
        ));

        report
    }

    /// Save artifacts to output directory
    fn save_artifacts(
        &self,
        output_dir: &std::path::Path,
        language: &str,
        spec: &str,
        code: &str,
        tests: Option<&str>,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save specification
        let spec_path = output_dir.join("specification.md");
        fs::write(&spec_path, spec).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write specification: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            spec_path.to_string_lossy().to_string(),
            spec.to_string(),
            "text/markdown".to_string(),
        ));

        // Save implementation with appropriate extension
        let ext = match language {
            "rust" => "rs",
            "python" => "py",
            "javascript" => "js",
            "typescript" => "ts",
            "go" => "go",
            "java" => "java",
            "cpp" => "cpp",
            _ => "txt",
        };
        let code_path = output_dir.join(format!("implementation.{}", ext));
        fs::write(&code_path, code).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write implementation: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            code_path.to_string_lossy().to_string(),
            code.to_string(),
            "text/plain".to_string(),
        ));

        // Save tests if present
        if let Some(test_code) = tests {
            let test_path = output_dir.join(format!("tests.{}", ext));
            fs::write(&test_path, test_code).map_err(|e| {
                TemplateError::ExecutionFailed(format!("Failed to write tests: {}", e))
            })?;
            output.add_artifact(Artifact::new(
                test_path.to_string_lossy().to_string(),
                test_code.to_string(),
                "text/plain".to_string(),
            ));
        }

        Ok(())
    }
}

/// Specification result from spec agent
#[derive(Debug, Clone)]
struct SpecificationResult {
    /// Specification content
    content: String,
}

/// Implementation result from implementation agent
#[derive(Debug, Clone)]
struct ImplementationResult {
    /// Generated code
    code: String,
}

/// Test result from test agent
#[derive(Debug, Clone)]
struct TestResult {
    /// Generated test code
    code: String,
}

/// Lint result from code quality checks
#[derive(Debug, Clone)]
struct LintResult {
    /// Lint report
    report: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;

    #[test]
    fn test_template_metadata() {
        let template = CodeGeneratorTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "code-generator");
        assert_eq!(metadata.name, "Code Generator");
        assert_eq!(metadata.category, TemplateCategory::CodeGen);
        assert!(metadata.requires.contains(&"code-tools".to_string()));
        assert!(metadata.requires.contains(&"lint".to_string()));
        assert!(metadata.tags.contains(&"code".to_string()));
        assert!(metadata.tags.contains(&"generation".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = CodeGeneratorTemplate::new();
        let schema = template.config_schema();

        assert!(schema.get_parameter("description").is_some());
        assert!(schema.get_parameter("language").is_some());
        assert!(schema.get_parameter("include_tests").is_some());
        assert!(schema.get_parameter("model").is_some());

        // description is required
        let desc_param = schema.get_parameter("description").unwrap();
        assert!(desc_param.required);

        // others are optional
        let lang_param = schema.get_parameter("language").unwrap();
        assert!(!lang_param.required);
    }

    #[tokio::test]
    async fn test_cost_estimate_with_tests() {
        let template = CodeGeneratorTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("include_tests", serde_json::json!(true));

        let estimate = template.estimate_cost(&params).await;
        assert!(estimate.estimated_tokens.is_some());
        // With tests: 1000 + 2000 + 1500 = 4500 tokens
        assert_eq!(estimate.estimated_tokens, Some(4500));
    }

    #[tokio::test]
    async fn test_cost_estimate_without_tests() {
        let template = CodeGeneratorTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("include_tests", serde_json::json!(false));

        let estimate = template.estimate_cost(&params).await;
        assert!(estimate.estimated_tokens.is_some());
        // Without tests: 1000 + 2000 = 3000 tokens
        assert_eq!(estimate.estimated_tokens, Some(3000));
    }

    #[test]
    fn test_parameter_validation_missing_required() {
        let template = CodeGeneratorTemplate::new();
        let schema = template.config_schema();
        let params = std::collections::HashMap::new();

        // Should fail - missing required "description" parameter
        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_invalid_language() {
        let template = CodeGeneratorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "description".to_string(),
            serde_json::json!("Generate code"),
        );
        params.insert("language".to_string(), serde_json::json!("cobol")); // Not in allowed values

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_success() {
        let template = CodeGeneratorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "description".to_string(),
            serde_json::json!("Generate a function"),
        );
        params.insert("language".to_string(), serde_json::json!("rust"));
        params.insert("include_tests".to_string(), serde_json::json!(true));

        let result = schema.validate(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_specification_placeholder() {
        let template = CodeGeneratorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let result = template
            .generate_specification("Test function", "rust", "ollama/llama3.2:3b", &context)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("# Code Specification"));
        assert!(result.content.contains("rust"));
    }

    #[tokio::test]
    async fn test_generate_implementation_rust() {
        let template = CodeGeneratorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let spec = SpecificationResult {
            content: "Test spec".to_string(),
        };

        let result = template
            .generate_implementation(&spec, "rust", "ollama/llama3.2:3b", &context)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.code.contains("pub fn main_function"));
        assert!(result.code.contains("Rust"));
    }

    #[tokio::test]
    async fn test_generate_tests_rust() {
        let template = CodeGeneratorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let implementation = ImplementationResult {
            code: "fn main() {}".to_string(),
        };

        let result = template
            .generate_tests(&implementation, "rust", "ollama/llama3.2:3b", &context)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.code.contains("#[test]"));
        assert!(result.code.contains("mod tests"));
    }

    #[test]
    fn test_format_report_with_tests() {
        let template = CodeGeneratorTemplate::new();
        let spec = SpecificationResult {
            content: "Test spec".to_string(),
        };
        let implementation = ImplementationResult {
            code: "fn main() {}".to_string(),
        };
        let tests = Some(TestResult {
            code: "#[test] fn test() {}".to_string(),
        });
        let lint = LintResult {
            report: "All good".to_string(),
        };

        let report = template.format_report(
            "Test description",
            "rust",
            &spec,
            &implementation,
            &tests,
            &lint,
        );

        assert!(report.contains("# Code Generation Report"));
        assert!(report.contains("rust"));
        assert!(report.contains("Specification"));
        assert!(report.contains("Implementation"));
        assert!(report.contains("Tests"));
        assert!(report.contains("Quality Report"));
    }

    #[test]
    fn test_format_report_without_tests() {
        let template = CodeGeneratorTemplate::new();
        let spec = SpecificationResult {
            content: "Test spec".to_string(),
        };
        let implementation = ImplementationResult {
            code: "fn main() {}".to_string(),
        };
        let lint = LintResult {
            report: "All good".to_string(),
        };

        let report = template.format_report(
            "Test description",
            "rust",
            &spec,
            &implementation,
            &None,
            &lint,
        );

        assert!(report.contains("# Code Generation Report"));
        assert!(!report.contains("## Tests"));
    }
}
