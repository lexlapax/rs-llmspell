# LLMSpell Agent Examples Guide

This comprehensive guide covers all agent examples in the LLMSpell framework, providing detailed explanations, usage patterns, and best practices.

## Table of Contents

1. [Tool Orchestrator Agent](#tool-orchestrator-agent)
2. [Multi-Agent Coordinator](#multi-agent-coordinator)
3. [Monitoring Agent](#monitoring-agent)
4. [Data Pipeline Agent](#data-pipeline-agent)
5. [Research Agent](#research-agent)
6. [Code Generation Agent](#code-generation-agent)
7. [Decision-Making Agent](#decision-making-agent)
8. [Agent Library Catalog](#agent-library-catalog)
9. [Common Patterns](#common-patterns)
10. [Best Practices](#best-practices)

## Tool Orchestrator Agent

**File**: `tool_orchestrator.rs`  
**Purpose**: Demonstrates how agents can discover, select, and chain tools to solve complex tasks.

### Key Features
- Automatic tool discovery from registry
- Intelligent tool selection based on task requirements
- Tool chaining for complex workflows
- Parallel tool execution for performance
- Error handling and recovery

### Usage Example
```rust
// Create orchestrator with specific tool access
let template = OrchestratorAgentTemplate::new();
let params = TemplateInstantiationParams::new("orchestrator-001".to_string())
    .with_parameter("agent_name", "Tool Orchestrator".into())
    .with_parameter("orchestration_strategy", "sequential".into());

let result = template.instantiate(params).await?;
let agent = result.agent;

// Execute complex task requiring multiple tools
let input = AgentInput::text(
    "Read data.csv, analyze trends, and generate report.json"
);
let output = agent.execute(input, ExecutionContext::default()).await?;
```

### Common Use Cases
1. **Data Processing Pipelines**: Chain file reading, transformation, and writing tools
2. **Analysis Workflows**: Combine data analysis, validation, and reporting tools
3. **Automation Tasks**: Orchestrate multiple operations with conditional logic
4. **Integration Scenarios**: Connect different systems through tool coordination

## Multi-Agent Coordinator

**File**: `multi_agent_coordinator.rs`  
**Purpose**: Shows hierarchical agent coordination where specialized agents work together.

### Key Features
- Hierarchical agent management
- Task delegation to specialized agents
- Parallel agent execution
- Result aggregation and consensus building
- Fault tolerance with agent redundancy

### Usage Example
```rust
// Create coordinator and specialized agents
let orchestrator = OrchestratorAgentTemplate::new();
let data_agent = ToolAgentTemplate::new();
let analysis_agent = ToolAgentTemplate::new();
let monitor_agent = MonitorAgentTemplate::new();

// Coordinator manages the specialized agents
let task = AgentInput::text(
    "Research sustainable energy: collect data, analyze trends, monitor progress"
);
```

### Coordination Patterns
1. **Sequential Pipeline**: Agents process in order with dependencies
2. **Parallel Execution**: Multiple agents work simultaneously
3. **Dynamic Allocation**: Assign agents based on task requirements
4. **Consensus Building**: Multiple agents analyze and reach agreement
5. **Fault Recovery**: Backup agents handle failures

## Monitoring Agent

**File**: `monitoring_agent.rs`  
**Purpose**: Tracks system health, agent performance, and generates alerts.

### Key Features
- Real-time health monitoring
- Resource usage tracking
- Performance metrics collection
- Alert generation with thresholds
- Historical trend analysis
- Comprehensive reporting

### Configuration Options
```rust
let params = TemplateInstantiationParams::new("monitor-001".to_string())
    .with_parameter("monitoring_interval", 5.into())      // Check every 5 seconds
    .with_parameter("alert_threshold", 0.75.into())       // Alert at 75% usage
    .with_parameter("enable_performance_tracking", true.into())
    .with_parameter("history_retention", 3600.into());    // Keep 1 hour history
```

### Monitoring Capabilities
- **System Health**: CPU, memory, disk, network monitoring
- **Agent Performance**: Response times, success rates, throughput
- **Resource Tracking**: Usage patterns and limits
- **Alert Management**: Configurable thresholds and notifications
- **Trend Analysis**: Historical patterns and predictions

## Data Pipeline Agent

**File**: `data_pipeline_agent.rs`  
**Purpose**: Implements ETL operations with intelligent routing and validation.

### Key Features
- Extract-Transform-Load (ETL) operations
- Multi-format support (CSV, JSON, XML)
- Conditional processing logic
- Data validation and quality checks
- Parallel processing capabilities
- Error handling with dead letter queues

### Pipeline Patterns
```rust
// Configure pipeline with specific tools
let params = TemplateInstantiationParams::new("pipeline-001".to_string())
    .with_parameter("allowed_tools", vec![
        "csv_analyzer",
        "json_processor",
        "data_validation",
        "file_operations"
    ].into())
    .with_parameter("enable_parallel_processing", true.into())
    .with_parameter("batch_size", 1000.into());
```

### Common Pipelines
1. **CSV to JSON**: Read, clean, transform, validate, write
2. **Multi-Source Integration**: Combine data from various sources
3. **Data Quality**: Profile, validate, clean, report
4. **Migration**: Transform legacy formats to modern schemas
5. **Real-time Processing**: Stream processing with micro-batches

## Research Agent

**File**: `research_agent.rs`  
**Purpose**: Gathers information from multiple sources and synthesizes findings.

### Key Features
- Multi-source information gathering
- Web search and scraping capabilities
- Data analysis and synthesis
- Citation management
- Caching for efficiency
- Structured report generation

### Research Workflows
```rust
// Configure research agent with search tools
let params = TemplateInstantiationParams::new("research-001".to_string())
    .with_parameter("allowed_tools", vec![
        "web_search",
        "web_scraper",
        "file_operations",
        "text_manipulator"
    ].into())
    .with_parameter("search_depth", 3.into())
    .with_parameter("max_sources", 10.into());
```

### Research Types
1. **Topic Research**: Comprehensive analysis of specific subjects
2. **Competitive Analysis**: Compare products, features, market position
3. **Technical Documentation**: Gather and summarize technical resources
4. **Data-Driven Research**: Statistical analysis and insights
5. **Real-time Tracking**: Monitor current events and trends

## Code Generation Agent

**File**: `code_gen_agent.rs`  
**Purpose**: Generates, validates, and refines code based on specifications.

### Key Features
- Code generation from specifications
- Multiple language support
- Test generation (TDD approach)
- Code validation and linting
- Iterative refinement
- Documentation generation

### Generation Patterns
```rust
// Configure code generation with development tools
let params = TemplateInstantiationParams::new("codegen-001".to_string())
    .with_parameter("allowed_tools", vec![
        "file_operations",
        "process_executor",
        "template_engine",
        "diff_calculator"
    ].into())
    .with_parameter("enable_iterative_refinement", true.into())
    .with_parameter("test_driven_development", true.into());
```

### Code Generation Types
1. **Function Generation**: Create functions with tests and docs
2. **API Endpoints**: Generate REST/GraphQL endpoints
3. **Data Structures**: Create models with validation
4. **Test Suites**: Generate comprehensive test coverage
5. **CLI Tools**: Build command-line applications
6. **Documentation**: Generate API docs and guides

## Decision-Making Agent

**File**: `decision_agent.rs`  
**Purpose**: Evaluates options and makes informed decisions with confidence scoring.

### Key Features
- Multi-criteria decision analysis
- Confidence scoring
- Risk assessment
- Constraint handling
- Decision explanation
- A/B test analysis

### Decision Frameworks
```rust
// Configure decision-making parameters
let params = TemplateInstantiationParams::new("decision-001".to_string())
    .with_parameter("decision_framework", "multi_criteria".into())
    .with_parameter("confidence_threshold", 0.7.into())
    .with_parameter("enable_explanation", true.into());
```

### Decision Types
1. **Technology Selection**: Choose best tech stack
2. **Investment Allocation**: Portfolio optimization
3. **Hiring Decisions**: Candidate evaluation
4. **Strategic Planning**: Business expansion choices
5. **Risk Mitigation**: Security investment decisions
6. **A/B Testing**: Statistical decision making

## Agent Library Catalog

**File**: `agent_library.rs`  
**Purpose**: Demonstrates reusable agent templates and composition patterns.

### Key Features
- Template catalog organization
- Parameterized agent templates
- Template composition
- Configuration inheritance
- Version management
- Export/import capabilities

### Template Categories
```rust
// Catalog organization
let mut agent_catalog = HashMap::new();

// Customer Service Templates
agent_catalog.insert("customer_service_faq", faq_params);
agent_catalog.insert("customer_service_complaints", complaint_params);

// Development Templates
agent_catalog.insert("dev_code_reviewer", reviewer_params);
agent_catalog.insert("dev_doc_generator", doc_params);

// Data Analysis Templates
agent_catalog.insert("data_etl_pipeline", etl_params);
agent_catalog.insert("data_report_generator", report_params);
```

### Template Patterns
1. **Base Templates**: Standard configurations for common use cases
2. **Specialized Templates**: Domain-specific agent configurations
3. **Composite Templates**: Combine multiple templates
4. **Customizable Templates**: Override parameters for specific needs
5. **Template Versioning**: Track template evolution

## Common Patterns

### 1. Agent Composition
```rust
// Agents working together
let orchestrator = create_orchestrator();
let tool_agent = create_tool_agent();
let monitor = create_monitor();

// Orchestrator manages tool agent while monitor tracks health
```

### 2. Error Handling
```rust
// Graceful error recovery
let result = agent.execute(input, context).await;
match result {
    Ok(output) => process_success(output),
    Err(e) => {
        // Try recovery strategy
        let recovery_output = recovery_agent.execute(recovery_input, context).await?;
    }
}
```

### 3. Resource Management
```rust
// Set resource limits
let params = params
    .with_parameter("max_execution_time_secs", 300.into())
    .with_parameter("max_memory_mb", 512.into())
    .with_parameter("max_tool_calls", 100.into());
```

### 4. Caching Strategy
```rust
// Enable caching for efficiency
let params = params
    .with_parameter("enable_caching", true.into())
    .with_parameter("cache_ttl", 3600.into())
    .with_parameter("max_cache_size", 100.into());
```

## Best Practices

### 1. Agent Design
- **Single Responsibility**: Each agent should have a clear, focused purpose
- **Tool Selection**: Only include tools the agent actually needs
- **Resource Limits**: Always set appropriate resource constraints
- **Error Handling**: Implement robust error recovery strategies

### 2. Performance Optimization
- **Parallel Execution**: Use parallel processing where possible
- **Caching**: Enable caching for expensive operations
- **Batch Processing**: Process data in batches for efficiency
- **Tool Reuse**: Share tool instances across agents when safe

### 3. Monitoring and Debugging
- **Comprehensive Logging**: Use structured logging for traceability
- **Metrics Collection**: Track key performance indicators
- **Health Checks**: Regular health monitoring for long-running agents
- **Audit Trails**: Maintain records of agent decisions and actions

### 4. Security Considerations
- **Input Validation**: Always validate and sanitize inputs
- **Resource Limits**: Prevent resource exhaustion attacks
- **Access Control**: Limit tool access based on requirements
- **Audit Logging**: Track all security-relevant operations

### 5. Testing Strategies
- **Unit Tests**: Test individual agent components
- **Integration Tests**: Test agent-tool interactions
- **End-to-End Tests**: Test complete workflows
- **Performance Tests**: Benchmark agent performance
- **Chaos Testing**: Test failure scenarios

## Running Examples

All examples can be run using:
```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example tool_orchestrator
cargo run --example multi_agent_coordinator
cargo run --example monitoring_agent
# ... etc
```

## Advanced Usage

### Custom Agent Creation
```rust
// Create custom agent by extending templates
let custom_template = ToolAgentTemplate::new();
let custom_params = base_params
    .with_parameter("custom_behavior", true.into())
    .with_config_override("special_mode", "advanced".into());
```

### Dynamic Tool Loading
```rust
// Dynamically add tools based on requirements
let mut allowed_tools = vec!["base_tool"];
if needs_web_access {
    allowed_tools.push("web_search");
    allowed_tools.push("web_scraper");
}
```

### Multi-Environment Support
```rust
// Configure for different environments
let params = match env {
    "production" => params.with_parameter("safety_mode", true.into()),
    "development" => params.with_parameter("debug_mode", true.into()),
    _ => params,
};
```

## Troubleshooting

### Common Issues

1. **Agent Creation Fails**
   - Check required parameters are provided
   - Verify tool dependencies are available
   - Ensure resource limits are reasonable

2. **Poor Performance**
   - Enable parallel processing
   - Implement caching
   - Reduce tool call overhead
   - Optimize batch sizes

3. **Unexpected Behavior**
   - Check agent configuration
   - Verify tool permissions
   - Review execution context
   - Enable debug logging

### Debug Tips
```rust
// Enable detailed logging
tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .init();

// Add execution tracing
let context = ExecutionContext::default()
    .with_tracing(true)
    .with_metrics(true);
```

## Future Enhancements

The agent examples will continue to evolve with:
- More sophisticated coordination patterns
- Enhanced learning capabilities
- Improved performance optimizations
- Additional tool integrations
- Advanced monitoring features

For the latest updates and additional examples, check the repository's examples directory.