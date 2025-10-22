# Template Creation Guide

**Comprehensive guide for creating custom templates in rs-llmspell**

**Version**: 0.12.0 | **Last Updated**: Phase 12.10-12.13 Complete | **October 2025**

---

## Overview

Templates are production-ready AI workflows combining agents, tools, RAG, and LLM providers into reusable, configurable patterns. This guide covers creating custom templates from scratch.

**Prerequisites**:
- Rust knowledge (async/await, traits)
- Understanding of LLMSpell components (agents, tools, workflows)
- Familiarity with existing templates (see `llmspell-templates/src/builtin/`)

---

## Quick Start

### 1. Minimum Viable Template

```rust
use llmspell_templates::prelude::*;
use async_trait::async_trait;

pub struct MyTemplate {
    metadata: TemplateMetadata,
}

impl MyTemplate {
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                name: "my-template".to_string(),
                description: "Custom workflow".to_string(),
                version: "0.1.0".to_string(),
                category: TemplateCategory::Custom,
                tags: vec!["custom".to_string()],
                author: Some("Your Name".to_string()),
            },
        }
    }
}

#[async_trait]
impl Template for MyTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new()
            .add_parameter(
                "input",
                ParameterSchema::new(ParameterType::String)
                    .required(true)
                    .description("Input data"),
            )
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let input = params.get_string("input")?;

        // Your workflow logic here

        Ok(TemplateOutput::text(format!("Processed: {}", input)))
    }

    fn validate(&self, params: &TemplateParams) -> Result<()> {
        // Optional: custom validation beyond schema
        Ok(())
    }

    async fn estimate_cost(&self, _params: &TemplateParams) -> CostEstimate {
        CostEstimate {
            estimated_tokens: 1000,
            estimated_cost_usd: 0.001,
            estimated_duration_ms: 5000,
        }
    }
}
```

### 2. Register Template

```rust
// In llmspell-templates/src/builtin/mod.rs
mod my_template;
pub use my_template::MyTemplate;

pub fn register_builtin_templates(registry: &mut TemplateRegistry) -> Result<()> {
    // ... existing templates ...
    registry.register(Arc::new(MyTemplate::new()))?;
    Ok(())
}
```

### 3. Test Template

```bash
llmspell template list
llmspell template exec my-template --param input="test"
```

---

## Template Patterns (Phase 12.10-12.13)

### Code Review Pattern (Multi-Aspect Analysis)

**Use Case**: Configurable analysis with selective aspects

```rust
pub struct CodeReviewParams {
    pub code_path: String,
    pub aspects: Vec<String>,  // security, quality, performance, etc.
    pub model: String,
}

async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput> {
    let code = std::fs::read_to_string(params.get_string("code_path")?)?;
    let aspects = params.get_array("aspects")?;

    let mut findings = Vec::new();
    for aspect in aspects {
        let agent_config = AgentConfig {
            name: format!("{}-reviewer", aspect),
            system_prompt: format!("Review code for {} issues", aspect),
            model: Some(parse_model_spec(&params.get_string("model")?)),
            // ...
        };

        let agent = context.agent_registry().create_agent(agent_config).await?;
        let result = agent.execute(AgentInput::text(code.clone()), context.clone()).await?;
        findings.push((aspect, result));
    }

    Ok(TemplateOutput::json(serde_json::json!({ "findings": findings })))
}
```

**Key Feature**: Selective aspect execution reduces cost/time.

### Content Generation Pattern (Quality-Driven Iteration)

**Use Case**: Iterative refinement until quality threshold met

```rust
pub struct ContentGenerationParams {
    pub topic: String,
    pub content_type: String,
    pub quality_threshold: i32,
    pub max_iterations: usize,
}

async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput> {
    let topic = params.get_string("topic")?;
    let threshold = params.get_int("quality_threshold")?;
    let max_iter = params.get_int("max_iterations")? as usize;

    // Stage 1: Draft
    let draft_agent = context.agent_registry().create_agent(draft_config).await?;
    let mut content = draft_agent.execute(AgentInput::text(topic), context.clone()).await?;

    // Stage 2-3: Evaluate & Edit loop
    for iteration in 0..max_iter {
        let eval_agent = context.agent_registry().create_agent(eval_config).await?;
        let quality_score = eval_agent.execute(AgentInput::text(&content), context.clone()).await?;

        if quality_score >= threshold {
            break;  // Quality met
        }

        let edit_agent = context.agent_registry().create_agent(edit_config).await?;
        content = edit_agent.execute(AgentInput::text(&content), context.clone()).await?;
    }

    Ok(TemplateOutput::text(content))
}
```

**Key Feature**: Conditional iteration based on quality metric.

### File Classification Pattern (Scan-Classify-Act)

**Use Case**: Bulk operations with dry-run mode

```rust
pub struct FileClassificationParams {
    pub files_paths: Vec<String>,
    pub categories: HashMap<String, String>,
    pub dry_run: bool,
}

async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput> {
    let files = params.get_array("files_paths")?;
    let categories = params.get_object("categories")?;
    let dry_run = params.get_bool("dry_run")?;

    let agent = context.agent_registry().create_agent(classifier_config).await?;

    let mut results = Vec::new();
    for file_path in files {
        let content = std::fs::read_to_string(&file_path)?;
        let classification = agent.execute(AgentInput::text(&content), context.clone()).await?;

        if !dry_run {
            let target_dir = categories.get(&classification).unwrap();
            std::fs::rename(&file_path, format!("{}/{}", target_dir, file_path))?;
        }

        results.push((file_path, classification));
    }

    Ok(TemplateOutput::json(serde_json::json!({ "results": results, "dry_run": dry_run })))
}
```

**Key Feature**: Dry-run preview before executing actions.

### Knowledge Management Pattern (CRUD Operations)

**Use Case**: Multi-operation template with state persistence

```rust
pub struct KnowledgeManagementParams {
    pub operation: String,  // ingest, query, update, delete, list
    pub collection: String,
    pub content: Option<String>,
    pub query: Option<String>,
}

async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput> {
    let operation = params.get_string("operation")?;
    let collection = params.get_string("collection")?;

    match operation.as_str() {
        "ingest" => {
            let content = params.get_string("content")?;
            let chunks = chunk_document(&content, 500, 50);

            for chunk in chunks {
                context.state().set_json(
                    &format!("{}/doc_{}", collection, uuid::Uuid::new_v4()),
                    &chunk
                )?;
            }

            Ok(TemplateOutput::text("Documents ingested"))
        },
        "query" => {
            let query = params.get_string("query")?;
            let docs = context.state().get_all_with_prefix(&format!("{}/", collection))?;

            // Simple word-overlap search
            let results = search_documents(&query, docs);
            Ok(TemplateOutput::json(serde_json::json!({ "results": results })))
        },
        // ... other operations
        _ => Err(TemplateError::InvalidParameter(format!("Unknown operation: {}", operation)))
    }
}
```

**Key Feature**: Single template, multiple operations via parameter switching.

---

## Best Practices

### 1. Parameter Validation

**Always validate parameters** beyond schema checks:

```rust
fn validate(&self, params: &TemplateParams) -> Result<()> {
    let quality_threshold = params.get_int("quality_threshold")?;
    if quality_threshold < 1 || quality_threshold > 10 {
        return Err(TemplateError::InvalidParameter(
            "quality_threshold must be 1-10".to_string()
        ));
    }
    Ok(())
}
```

### 2. Agent Creation Pattern

**Use consistent agent creation** via context:

```rust
let (provider, model_id) = parse_model_spec(&params.get_string("model")?);

let agent_config = AgentConfig {
    name: "agent-name".to_string(),
    model: Some(ModelConfig {
        provider,
        model_id,
        temperature: Some(0.7),
        max_tokens: Some(2048),
        ..Default::default()
    }),
    resource_limits: ResourceLimits {
        max_execution_time: Duration::from_secs(60),
        max_memory: 512 * 1024 * 1024,
        ..Default::default()
    },
    ..Default::default()
};

let agent = context.agent_registry().create_agent(agent_config).await?;
```

### 3. Error Handling

**Provide actionable error messages**:

```rust
Err(TemplateError::ExecutionFailed(format!(
    "Agent '{}' failed: {}. Check model configuration and ensure LLM provider is available.",
    agent_name, err
)))
```

### 4. Cost Estimation

**Estimate based on workflow steps**:

```rust
async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
    let num_aspects = params.get_array("aspects")?.len();
    let estimated_tokens_per_aspect = 1500;

    CostEstimate {
        estimated_tokens: num_aspects * estimated_tokens_per_aspect,
        estimated_cost_usd: (num_aspects as f64) * 0.0015,
        estimated_duration_ms: (num_aspects as u64) * 5000,
    }
}
```

### 5. Testing

**Write integration tests** for all operations:

```rust
#[tokio::test]
async fn test_my_template_basic() {
    let template = MyTemplate::new();
    let mut params = TemplateParams::new();
    params.insert("input".to_string(), "test".into());

    let context = ExecutionContext::builder()
        .with_agent_registry(Arc::new(AgentRegistry::new()))
        .build();

    let result = template.execute(params, context).await;
    assert!(result.is_ok());
}
```

---

## Template Categories

Choose appropriate category for discoverability:

- **Research**: Information gathering, analysis, synthesis
- **Chat**: Conversational AI, Q&A, interactive sessions
- **Content**: Writing, editing, generation
- **Development**: Code generation, review, refactoring
- **Productivity**: File management, organization, automation
- **Document**: Extraction, transformation, processing
- **Workflow**: Multi-step orchestration, composition
- **Analysis**: Data analysis, visualization, statistics
- **Custom**: Domain-specific workflows

---

## Documentation Requirements

Every template MUST have:

1. **User Guide** (`docs/user-guide/templates/<name>.md`):
   - Quick Start example
   - Parameter reference
   - CLI + Lua examples
   - Troubleshooting

2. **Integration Tests** (`llmspell-templates/tests/integration_test.rs`):
   - At least 7 test scenarios
   - Happy path + error cases
   - <0.01s execution (no real LLM calls in tests)

3. **README Entry** (`docs/user-guide/templates/README.md`):
   - Template summary
   - Quick example
   - Link to full guide

---

## References

- **Template System Architecture**: `docs/technical/template-system-architecture.md`
- **Existing Templates**: `llmspell-templates/src/builtin/`
- **Bridge Pattern Guide**: `docs/developer-guide/bridge-pattern-guide.md`
- **Template User Guides**: `docs/user-guide/templates/`
- **Template Trait**: `llmspell-templates/src/core.rs`

---

## Appendix: Template Checklist

- [ ] Implement `Template` trait
- [ ] Add comprehensive parameter schema
- [ ] Implement `validate()` with business logic checks
- [ ] Implement `execute()` with error handling
- [ ] Implement `estimate_cost()` accurately
- [ ] Register in `llmspell-templates/src/builtin/mod.rs`
- [ ] Write 7+ integration tests
- [ ] Create user guide (docs/user-guide/templates/<name>.md)
- [ ] Add to template README
- [ ] Test via CLI: `llmspell template exec <name>`
- [ ] Test via Lua: `Template.execute("<name>", params)`
- [ ] Zero clippy warnings
- [ ] Zero rustdoc warnings

---

**For detailed architecture and advanced patterns, see**: [Template System Architecture](../technical/template-system-architecture.md)
