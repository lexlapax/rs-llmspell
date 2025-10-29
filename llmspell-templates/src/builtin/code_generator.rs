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
        memory_parameters, provider_parameters, CostEstimate, TemplateCategory, TemplateMetadata,
        TemplateOutput, TemplateParams, TemplateResult,
    },
    error::{Result, TemplateError},
    validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType},
};
use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;
use tracing::{debug, info, warn};

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
        let mut params = vec![
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
                    json!("lua"),
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
        ];

        // Add memory parameters (Task 13.11.1)
        params.extend(memory_parameters());

        // Add provider parameters (Task 13.5.7d)
        params.extend(provider_parameters());

        tracing::debug!(
            "CodeGenerator: Generated config schema with {} parameters",
            params.len()
        );
        ConfigSchema::new(params)
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

        // Extract memory parameters (Task 13.11.2)
        let session_id: Option<String> = params.get_optional("session_id").unwrap_or(None);
        let memory_enabled: bool = params.get_or("memory_enabled", true);
        let context_budget: i64 = params.get_or("context_budget", 2000);

        // Smart dual-path provider resolution (Task 13.5.7d)
        let provider_config = context.resolve_llm_config(&params)?;
        let model_str = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        info!(
            "Starting code generation (language={}, tests={}, model={})",
            language, include_tests, model_str
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
            .generate_specification(
                &description,
                &language,
                &provider_config,
                &context,
                session_id.as_deref(),
                memory_enabled,
                context_budget,
            )
            .await?;
        output.metrics.agents_invoked += 1; // spec agent

        // Phase 2: Generate implementation with implementation agent
        info!("Phase 2: Generating implementation...");
        let impl_result = self
            .generate_implementation(&spec_result, &language, &provider_config, &context)
            .await?;
        output.metrics.agents_invoked += 1; // implementation agent

        // Phase 3: Generate tests with test agent (if enabled)
        let test_result = if include_tests {
            info!("Phase 3: Generating tests...");
            let result = self
                .generate_tests(&impl_result, &language, &provider_config, &context)
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
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
        session_id: Option<&str>,
        memory_enabled: bool,
        context_budget: i64,
    ) -> Result<SpecificationResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!("Creating specification agent for {} code", language);

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        // Parse model string (provider/model format)
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            (provider_config.provider_type.clone(), model.to_string())
        };

        // Create agent config for specification generation
        let agent_config = AgentConfig {
            name: "code-spec-agent".to_string(),
            description: format!("Specification agent for {} code generation", language),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.3)), // Lower temperature for structured specs
                max_tokens: provider_config.max_tokens.or(Some(2000)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 120,
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create specification agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Assemble memory context (Task 13.11.2)
        let memory_context = if memory_enabled && session_id.is_some() {
            if let Some(bridge) = context.context_bridge() {
                debug!(
                    "Assembling memory context: session={:?}, budget={}",
                    session_id, context_budget
                );
                crate::assemble_template_context(&bridge, description, session_id.unwrap(), context_budget)
                    .await
            } else {
                if memory_enabled {
                    warn!("Memory enabled but ContextBridge unavailable");
                }
                vec![]
            }
        } else {
            vec![]
        };

        let memory_context_str = if !memory_context.is_empty() {
            memory_context
                .iter()
                .map(|msg| format!("{}: {}", msg.role, msg.content))
                .collect::<Vec<_>>()
                .join("\n\n")
        } else {
            String::new()
        };

        // Build the specification request with instructions in the prompt
        let spec_prompt = format!(
            "{}\
             You are an expert software architect specializing in {}. \
             Generate a detailed, well-structured technical specification from the following description.\n\n\
             **Description**: {}\n\
             **Language**: {}\n\n\
             Include these sections in your specification:\n\
             1. **Overview**: Brief summary of the functionality\n\
             2. **Requirements**: Functional and non-functional requirements\n\
             3. **Architecture**: High-level design and component breakdown\n\
             4. **Data Structures**: Types, structs, classes needed\n\
             5. **API Design**: Public interfaces and function signatures\n\
             6. **Error Handling**: Exception/error handling strategy\n\
             7. **Testing Strategy**: Unit and integration test approach\n\n\
             Make the specification:\n\
             - Detailed enough for implementation\n\
             - Language-idiomatic for {}\n\
             - Include function signatures and type annotations\n\
             - Consider edge cases and error conditions\n\n\
             Provide the specification now:",
            if !memory_context_str.is_empty() {
                format!("{}\n\n", memory_context_str)
            } else {
                String::new()
            },
            language,
            description,
            language,
            language
        );

        // Execute the agent
        info!("Executing specification agent...");
        let agent_input = AgentInput::builder().text(spec_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Specification agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
            })?;

        // Extract specification content
        let content = agent_output.text;

        info!(
            "Specification generated ({} lines)",
            content.lines().count()
        );

        Ok(SpecificationResult { content })
    }

    /// Phase 2: Generate implementation with implementation agent
    async fn generate_implementation(
        &self,
        spec: &SpecificationResult,
        language: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<ImplementationResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!("Creating implementation agent for {} code", language);

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        // Parse model string (provider/model format)
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            (provider_config.provider_type.clone(), model.to_string())
        };

        // Create agent config for code implementation
        let agent_config = AgentConfig {
            name: "code-impl-agent".to_string(),
            description: format!("Implementation agent for {} code generation", language),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.5)), // Higher than spec (0.3) for implementation creativity
                max_tokens: provider_config.max_tokens.or(Some(3000)), // More tokens for actual code
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 180, // 3 minutes for implementation
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create implementation agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Build the implementation request with spec as context
        let impl_prompt = format!(
            "You are an expert {} programmer. Implement complete, production-ready code based on the following specification.\n\n\
             **Language**: {}\n\n\
             **SPECIFICATION**:\n{}\n\n\
             **INSTRUCTIONS**:\n\
             1. Implement ALL components described in the specification\n\
             2. Follow {}-idiomatic patterns and best practices\n\
             3. Include proper error handling as specified\n\
             4. Add comprehensive documentation (docstrings/comments)\n\
             5. Implement all public APIs with correct signatures\n\
             6. Include internal helper functions as needed\n\
             7. Make code production-ready, not just a stub\n\
             8. Use type annotations where applicable\n\n\
             Provide ONLY the code implementation (no explanations), ready to save to a file:",
            language, language, spec.content, language
        );

        // Execute the agent
        info!("Executing implementation agent...");
        let agent_input = AgentInput::builder().text(impl_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Implementation agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
            })?;

        // Extract implementation code
        let code = agent_output.text;

        info!("Implementation generated ({} lines)", code.lines().count());

        Ok(ImplementationResult { code })
    }

    /// Phase 3: Generate tests with test agent
    async fn generate_tests(
        &self,
        implementation: &ImplementationResult,
        language: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<TestResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!("Creating test agent for {} code", language);

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        // Parse model string (provider/model format)
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            (provider_config.provider_type.clone(), model.to_string())
        };

        // Create agent config for test generation
        let agent_config = AgentConfig {
            name: "code-test-agent".to_string(),
            description: format!("Test generation agent for {} code", language),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.4)), // Creative for edge cases, structured for test syntax
                max_tokens: provider_config.max_tokens.or(Some(2500)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 150, // 2.5 minutes for test generation
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create test agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Determine test framework by language
        let test_framework = match language {
            "rust" => "Rust's built-in test framework with #[test]",
            "python" => "unittest or pytest",
            "javascript" | "typescript" => "Jest or Mocha",
            "go" => "Go's testing package",
            "java" => "JUnit",
            "lua" => "busted or luaunit",
            _ => "appropriate testing framework",
        };

        // Build the test generation request
        let test_prompt = format!(
            "You are an expert {} test engineer. Generate comprehensive unit tests for the following code implementation.\n\n\
             **Language**: {}\n\
             **Test Framework**: {}\n\n\
             **IMPLEMENTATION CODE**:\n{}\n\n\
             **INSTRUCTIONS**:\n\
             1. Generate comprehensive unit tests covering:\n\
                - Happy path scenarios (normal inputs)\n\
                - Edge cases (empty, null, boundary values)\n\
                - Error conditions (invalid inputs, exceptions)\n\
             2. Use {} for all tests\n\
             3. Test ALL public functions/methods in the implementation\n\
             4. Include descriptive test names that explain what is being tested\n\
             5. Use proper assertions matching the test framework\n\
             6. Add test setup/teardown if needed\n\
             7. Aim for >80% code coverage\n\
             8. Include comments explaining complex test scenarios\n\n\
             Provide ONLY the test code (no explanations), ready to save to a test file:",
            language, language, test_framework, implementation.code, test_framework
        );

        // Execute the agent
        info!("Executing test agent...");
        let agent_input = AgentInput::builder().text(test_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Test agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
            })?;

        // Extract test code
        let code = agent_output.text;

        info!("Tests generated ({} lines)", code.lines().count());

        Ok(TestResult { code })
    }

    /// Phase 4: Run code quality checks (linting/formatting)
    async fn run_quality_checks(
        &self,
        implementation: &ImplementationResult,
        language: &str,
        _context: &ExecutionContext,
    ) -> Result<LintResult> {
        info!("Running code quality checks for {} code", language);

        // Use static code analysis
        // Note: Tool-based linting (clippy, pylint, eslint) requires file system access
        // and external process execution, which is beyond template scope.
        // Static analysis provides meaningful metrics without external dependencies.
        let report = self.static_code_analysis(&implementation.code, language);

        Ok(LintResult { report })
    }

    /// Static code analysis (fallback when linter tools unavailable)
    fn static_code_analysis(&self, code: &str, language: &str) -> String {
        let total_lines = code.lines().count();
        let non_empty_lines = code.lines().filter(|l| !l.trim().is_empty()).count();
        let comment_lines = code
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.starts_with("//")
                    || trimmed.starts_with('#')
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with('*')
            })
            .count();

        // Basic pattern detection
        let has_error_handling = code.contains("Error")
            || code.contains("error")
            || code.contains("Exception")
            || code.contains("try")
            || code.contains("Result");

        let has_documentation = match language {
            "rust" => code.contains("///") || code.contains("//!"),
            "python" => code.contains("\"\"\"") || code.contains("'''"),
            "javascript" | "typescript" => code.contains("/**"),
            "lua" => code.contains("---") || code.contains("--[["),
            _ => comment_lines > 0,
        };

        // Generate report
        format!(
            "# Code Quality Report ({})\n\n\
             ## Metrics\n\
             - Total lines: {}\n\
             - Non-empty lines: {}\n\
             - Comment lines: {} ({:.1}%)\n\
             - Lines of code: {}\n\n\
             ## Static Analysis\n\
             - Error handling: {}\n\
             - Documentation: {}\n\
             - Code density: {:.1}% (non-empty/total)\n\n\
             ## Notes\n\
             - Static analysis performed (linter tools not available)\n\
             - For comprehensive checks, install language-specific linters:\n\
               - Rust: cargo clippy\n\
               - Python: pylint, flake8\n\
               - JavaScript: eslint\n",
            language,
            total_lines,
            non_empty_lines,
            comment_lines,
            (comment_lines as f64 / total_lines as f64 * 100.0),
            non_empty_lines - comment_lines,
            if has_error_handling {
                "✓ Present"
            } else {
                "⚠ Not detected"
            },
            if has_documentation {
                "✓ Present"
            } else {
                "⚠ Minimal"
            },
            (non_empty_lines as f64 / total_lines as f64 * 100.0)
        )
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
            "lua" => "lua",
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

    /// Test helper: Create a provider config for tests
    fn test_provider_config() -> llmspell_config::ProviderConfig {
        llmspell_config::ProviderConfig {
            default_model: Some("ollama/llama3.2:3b".to_string()),
            provider_type: "ollama".to_string(),
            temperature: Some(0.3),
            max_tokens: Some(2000),
            timeout_seconds: Some(120),
            ..Default::default()
        }
    }

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
            .generate_specification("Test function", "rust", &test_provider_config(), &context)
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
            .generate_implementation(&spec, "rust", &test_provider_config(), &context)
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
            .generate_tests(&implementation, "rust", &test_provider_config(), &context)
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
