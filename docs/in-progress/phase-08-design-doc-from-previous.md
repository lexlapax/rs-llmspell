# Phase 8: Workflow Orchestration Design Document

**Version**: 1.0  
**Date**: January 2025  
**Status**: Design Document (Extracted from Phase 3.3)  
**Timeline**: Weeks 25-26  

> **📋 Note**: This document was extracted from the original Phase 3.3 design in phase-03-design-doc.md and updated to reflect Phase 8 implementation requirements.

---

## Overview

Phase 8 implements comprehensive workflow orchestration patterns that leverage all 41+ standardized and secured tools from previous phases. This phase provides the foundation for complex multi-step operations, conditional logic, iterative processing, and parallel execution patterns.

## Goal

Implement workflow patterns leveraging full agent and tool infrastructure with persistent state management and integration with vector storage for workflow context.

## Dependencies

- **Phase 7**: Vector Storage for workflow context
- **Phase 6**: Session Management for workflow persistence
- **Phase 5**: Persistent State Management
- **Phase 4**: Hook System for workflow events
- **Phase 3**: Complete tool standardization and agent infrastructure
- **Phase 2**: Self-contained tools library
- **Phase 1**: Core execution runtime
- **Phase 0**: Foundation infrastructure

---

## 1. Workflow Trait System

```rust
// llmspell-workflows/src/traits.rs
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Unique identifier for the workflow
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Workflow metadata
    fn metadata(&self) -> &WorkflowMetadata;
    
    /// Execute the workflow
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput>;
    
    /// Validate workflow configuration
    fn validate(&self) -> Result<()>;
    
    /// Get workflow schema for validation
    fn schema(&self) -> WorkflowSchema;
}

#[derive(Debug, Clone)]
pub struct WorkflowMetadata {
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub required_tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowInput {
    pub initial_data: Value,
    pub parameters: HashMap<String, Value>,
    pub starting_step: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowOutput {
    pub final_result: Value,
    pub step_results: HashMap<String, StepResult>,
    pub execution_path: Vec<String>,
    pub metrics: WorkflowMetrics,
}
```

---

## 2. Workflow Patterns

### 2.1 Sequential Workflow

```rust
pub struct SequentialWorkflow {
    id: String,
    name: String,
    steps: Vec<WorkflowStep>,
    error_handling: ErrorStrategy,
}

impl SequentialWorkflow {
    pub fn builder(name: &str) -> SequentialWorkflowBuilder {
        SequentialWorkflowBuilder::new(name)
    }
}

#[async_trait]
impl Workflow for SequentialWorkflow {
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        let mut state = WorkflowState::new(input.initial_data);
        let mut step_results = HashMap::new();
        let mut execution_path = Vec::new();
        
        for step in &self.steps {
            execution_path.push(step.id.clone());
            
            // Execute step
            let step_input = self.prepare_step_input(&step, &state)?;
            let result = match self.execute_step(&step, step_input, &context).await {
                Ok(r) => r,
                Err(e) => {
                    match self.error_handling {
                        ErrorStrategy::Fail => return Err(e),
                        ErrorStrategy::Continue => {
                            step_results.insert(step.id.clone(), StepResult::Failed(e.to_string()));
                            continue;
                        }
                        ErrorStrategy::Retry(attempts) => {
                            self.retry_step(&step, step_input, &context, attempts).await?
                        }
                    }
                }
            };
            
            // Update state
            state.update(&step.id, &result)?;
            step_results.insert(step.id.clone(), StepResult::Success(result));
        }
        
        Ok(WorkflowOutput {
            final_result: state.get_final_result()?,
            step_results,
            execution_path,
            metrics: state.get_metrics(),
        })
    }
}
```

### 2.2 Conditional Workflow

```rust
pub struct ConditionalWorkflow {
    id: String,
    name: String,
    initial_step: String,
    steps: HashMap<String, ConditionalStep>,
    conditions: HashMap<String, Condition>,
}

#[derive(Debug, Clone)]
pub struct ConditionalStep {
    pub id: String,
    pub tool: String,
    pub parameters: Value,
    pub branches: Vec<Branch>,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub condition: String,
    pub next_step: Option<String>,
}

impl ConditionalWorkflow {
    async fn execute_step(&self, step_id: &str, state: &WorkflowState, context: &ExecutionContext) -> Result<(StepResult, Option<String>)> {
        let step = self.steps.get(step_id)
            .ok_or_else(|| WorkflowError::StepNotFound(step_id.to_string()))?;
        
        // Execute the tool
        let tool_result = self.execute_tool(&step.tool, &step.parameters, state, context).await?;
        
        // Evaluate branches
        for branch in &step.branches {
            let condition = self.conditions.get(&branch.condition)
                .ok_or_else(|| WorkflowError::ConditionNotFound(branch.condition.clone()))?;
            
            if condition.evaluate(&tool_result, state)? {
                return Ok((StepResult::Success(tool_result), branch.next_step.clone()));
            }
        }
        
        // No matching branch
        Ok((StepResult::Success(tool_result), None))
    }
}
```

### 2.3 Loop Workflow

```rust
pub struct LoopWorkflow {
    id: String,
    name: String,
    iterator: Iterator,
    body: Box<dyn Workflow>,
    max_iterations: Option<usize>,
    break_condition: Option<Condition>,
}

#[derive(Debug, Clone)]
pub enum Iterator {
    Collection(String),      // Iterate over collection in state
    Range(usize, usize),    // Iterate over numeric range
    WhileCondition(Condition), // While condition is true
}

impl LoopWorkflow {
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        let mut state = WorkflowState::new(input.initial_data);
        let mut iterations = 0;
        let mut all_results = Vec::new();
        
        loop {
            // Check max iterations
            if let Some(max) = self.max_iterations {
                if iterations >= max {
                    break;
                }
            }
            
            // Check break condition
            if let Some(condition) = &self.break_condition {
                if condition.evaluate(&Value::Null, &state)? {
                    break;
                }
            }
            
            // Get next item
            let item = match &self.iterator {
                Iterator::Collection(path) => {
                    state.get_collection_item(path, iterations)?
                }
                Iterator::Range(start, end) => {
                    if start + iterations >= *end {
                        break;
                    }
                    Value::Number((start + iterations).into())
                }
                Iterator::WhileCondition(condition) => {
                    if !condition.evaluate(&Value::Null, &state)? {
                        break;
                    }
                    Value::Null
                }
            };
            
            // Execute body with item
            let body_input = WorkflowInput {
                initial_data: item,
                parameters: state.get_loop_parameters(),
                starting_step: None,
            };
            
            let result = self.body.execute(body_input, context.clone()).await?;
            all_results.push(result.final_result);
            
            iterations += 1;
        }
        
        Ok(WorkflowOutput {
            final_result: Value::Array(all_results),
            step_results: HashMap::new(),
            execution_path: vec![format!("loop_{}_iterations", iterations)],
            metrics: WorkflowMetrics::default(),
        })
    }
}
```

### 2.4 Streaming Workflow

```rust
pub struct StreamingWorkflow {
    id: String,
    name: String,
    source: StreamSource,
    pipeline: Vec<StreamProcessor>,
    sink: StreamSink,
    backpressure_strategy: BackpressureStrategy,
}

#[derive(Debug, Clone)]
pub enum StreamSource {
    Tool(String, Value),           // Tool that produces stream
    File(PathBuf),                // File to stream
    Network(String),              // Network endpoint
    Collection(Vec<Value>),       // In-memory collection
}

impl StreamingWorkflow {
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        // Create stream from source
        let stream = self.create_stream(&self.source, &context).await?;
        
        // Apply processors
        let processed = self.pipeline.iter().fold(stream, |s, processor| {
            processor.apply(s, &context)
        });
        
        // Handle backpressure
        let controlled = match self.backpressure_strategy {
            BackpressureStrategy::Buffer(size) => processed.buffer_unordered(size),
            BackpressureStrategy::Drop => processed.drop_on_overflow(),
            BackpressureStrategy::Pause => processed.pause_on_pressure(),
        };
        
        // Collect to sink
        let results = self.sink.collect(controlled).await?;
        
        Ok(WorkflowOutput {
            final_result: results,
            step_results: HashMap::new(),
            execution_path: vec!["streaming".to_string()],
            metrics: WorkflowMetrics::default(),
        })
    }
}
```

### 2.5 ParallelWorkflow

Execute multiple workflow steps concurrently with configurable parallelism and result aggregation.

```rust
pub struct ParallelWorkflow {
    id: String,
    name: String,
    branches: Vec<ParallelBranch>,
    max_concurrency: usize,
    aggregation_strategy: AggregationStrategy,
    error_mode: ParallelErrorMode,
    timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct ParallelBranch {
    pub id: String,
    pub name: String,
    pub workflow: Box<dyn Workflow>,
    pub weight: f32,  // For weighted aggregation
    pub required: bool,  // Must succeed for overall success
}

#[derive(Debug, Clone)]
pub enum AggregationStrategy {
    FirstSuccess,           // Return first successful result
    AllSuccess,            // Require all to succeed
    PartialSuccess(usize), // Require N branches to succeed
    Weighted,              // Weight results by branch weight
    Custom(Box<dyn Fn(Vec<BranchResult>) -> Result<Value>>),
}

#[derive(Debug, Clone)]
pub enum ParallelErrorMode {
    FailFast,      // Cancel all on first error
    FailSlow,      // Let all complete, then report
    BestEffort,    // Continue with successful branches
}

#[derive(Debug, Clone)]
pub struct BranchResult {
    pub branch_id: String,
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub duration: Duration,
}

impl ParallelWorkflow {
    pub fn builder(name: &str) -> ParallelWorkflowBuilder {
        ParallelWorkflowBuilder::new(name)
    }
    
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        let start_time = Instant::now();
        let mut branch_handles = Vec::new();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        
        // Launch all branches with concurrency control
        for branch in &self.branches {
            let sem_clone = semaphore.clone();
            let branch_clone = branch.clone();
            let input_clone = input.clone();
            let context_clone = context.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = sem_clone.acquire().await?;
                let branch_start = Instant::now();
                
                let result = match branch_clone.workflow.execute(input_clone, context_clone).await {
                    Ok(output) => BranchResult {
                        branch_id: branch_clone.id,
                        success: true,
                        result: Some(output.final_result),
                        error: None,
                        duration: branch_start.elapsed(),
                    },
                    Err(e) => BranchResult {
                        branch_id: branch_clone.id,
                        success: false,
                        result: None,
                        error: Some(e.to_string()),
                        duration: branch_start.elapsed(),
                    },
                };
                
                Ok::<_, WorkflowError>(result)
            });
            
            branch_handles.push(handle);
        }
        
        // Handle results based on error mode
        let results = match self.error_mode {
            ParallelErrorMode::FailFast => {
                self.collect_fail_fast(branch_handles, self.timeout).await?
            }
            ParallelErrorMode::FailSlow => {
                self.collect_fail_slow(branch_handles, self.timeout).await?
            }
            ParallelErrorMode::BestEffort => {
                self.collect_best_effort(branch_handles, self.timeout).await?
            }
        };
        
        // Check required branches
        for branch in &self.branches {
            if branch.required {
                let branch_result = results.iter()
                    .find(|r| r.branch_id == branch.id)
                    .ok_or_else(|| WorkflowError::RequiredBranchFailed(branch.id.clone()))?;
                    
                if !branch_result.success {
                    return Err(WorkflowError::RequiredBranchFailed(branch.id.clone()));
                }
            }
        }
        
        // Aggregate results
        let final_result = self.aggregate_results(&results)?;
        
        // Build step results
        let mut step_results = HashMap::new();
        for result in &results {
            step_results.insert(
                result.branch_id.clone(),
                if result.success {
                    StepResult::Success(result.result.clone().unwrap_or(Value::Null))
                } else {
                    StepResult::Failed(result.error.clone().unwrap_or_default())
                }
            );
        }
        
        Ok(WorkflowOutput {
            final_result,
            step_results,
            execution_path: results.iter().map(|r| r.branch_id.clone()).collect(),
            metrics: WorkflowMetrics {
                total_duration: start_time.elapsed(),
                step_durations: results.iter()
                    .map(|r| (r.branch_id.clone(), r.duration))
                    .collect(),
                resource_usage: ResourceUsage::default(),
            },
        })
    }
    
    async fn collect_fail_fast(
        &self,
        handles: Vec<JoinHandle<Result<BranchResult>>>,
        timeout: Option<Duration>,
    ) -> Result<Vec<BranchResult>> {
        let mut results = Vec::new();
        let mut remaining_handles = handles;
        
        while !remaining_handles.is_empty() {
            let (result, _, rest) = futures::future::select_all(remaining_handles).await;
            remaining_handles = rest;
            
            match result {
                Ok(Ok(branch_result)) => {
                    if !branch_result.success && self.is_critical_failure(&branch_result) {
                        // Cancel remaining tasks
                        for handle in remaining_handles {
                            handle.abort();
                        }
                        return Err(WorkflowError::ParallelExecutionFailed(
                            format!("Branch {} failed: {:?}", branch_result.branch_id, branch_result.error)
                        ));
                    }
                    results.push(branch_result);
                }
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(WorkflowError::TaskJoinError(e.to_string())),
            }
        }
        
        Ok(results)
    }
    
    fn aggregate_results(&self, results: &[BranchResult]) -> Result<Value> {
        match &self.aggregation_strategy {
            AggregationStrategy::FirstSuccess => {
                results.iter()
                    .find(|r| r.success)
                    .and_then(|r| r.result.clone())
                    .ok_or_else(|| WorkflowError::NoSuccessfulBranches)
            }
            AggregationStrategy::AllSuccess => {
                if results.iter().all(|r| r.success) {
                    Ok(json!(results.iter()
                        .filter_map(|r| r.result.clone())
                        .collect::<Vec<_>>()))
                } else {
                    Err(WorkflowError::NotAllBranchesSucceeded)
                }
            }
            AggregationStrategy::PartialSuccess(min_success) => {
                let successful_count = results.iter().filter(|r| r.success).count();
                if successful_count >= *min_success {
                    Ok(json!(results.iter()
                        .filter(|r| r.success)
                        .filter_map(|r| r.result.clone())
                        .collect::<Vec<_>>()))
                } else {
                    Err(WorkflowError::InsufficientSuccessfulBranches(
                        *min_success,
                        successful_count
                    ))
                }
            }
            AggregationStrategy::Weighted => {
                let total_weight: f32 = self.branches.iter()
                    .map(|b| b.weight)
                    .sum();
                    
                let mut aggregated = json!({});
                for (branch, result) in self.branches.iter().zip(results.iter()) {
                    if result.success {
                        let weight_factor = branch.weight / total_weight;
                        // Merge weighted results (implementation depends on data structure)
                        self.merge_weighted(&mut aggregated, &result.result, weight_factor)?;
                    }
                }
                Ok(aggregated)
            }
            AggregationStrategy::Custom(func) => {
                func(results.to_vec())
            }
        }
    }
}

// Builder pattern for ParallelWorkflow
pub struct ParallelWorkflowBuilder {
    name: String,
    branches: Vec<ParallelBranch>,
    max_concurrency: usize,
    aggregation_strategy: AggregationStrategy,
    error_mode: ParallelErrorMode,
    timeout: Option<Duration>,
}

impl ParallelWorkflowBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            branches: Vec::new(),
            max_concurrency: 10,
            aggregation_strategy: AggregationStrategy::AllSuccess,
            error_mode: ParallelErrorMode::FailFast,
            timeout: None,
        }
    }
    
    pub fn add_branch(mut self, id: &str, workflow: Box<dyn Workflow>) -> Self {
        self.branches.push(ParallelBranch {
            id: id.to_string(),
            name: id.to_string(),
            workflow,
            weight: 1.0,
            required: false,
        });
        self
    }
    
    pub fn add_required_branch(mut self, id: &str, workflow: Box<dyn Workflow>) -> Self {
        self.branches.push(ParallelBranch {
            id: id.to_string(),
            name: id.to_string(),
            workflow,
            weight: 1.0,
            required: true,
        });
        self
    }
    
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }
    
    pub fn with_aggregation(mut self, strategy: AggregationStrategy) -> Self {
        self.aggregation_strategy = strategy;
        self
    }
    
    pub fn with_error_mode(mut self, mode: ParallelErrorMode) -> Self {
        self.error_mode = mode;
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn build(self) -> ParallelWorkflow {
        ParallelWorkflow {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            branches: self.branches,
            max_concurrency: self.max_concurrency,
            aggregation_strategy: self.aggregation_strategy,
            error_mode: self.error_mode,
            timeout: self.timeout,
        }
    }
}
```

---

## 3. Workflow State Management

```rust
pub struct WorkflowState {
    data: HashMap<String, Value>,
    history: Vec<StateChange>,
    variables: HashMap<String, Variable>,
}

impl WorkflowState {
    pub fn new(initial_data: Value) -> Self {
        let mut data = HashMap::new();
        data.insert("$input".to_string(), initial_data);
        
        Self {
            data,
            history: Vec::new(),
            variables: HashMap::new(),
        }
    }
    
    pub fn get(&self, path: &str) -> Result<&Value> {
        // JSONPath-like access
        jsonpath::select(&self.data, path)?
            .first()
            .ok_or_else(|| StateError::PathNotFound(path.to_string()))
    }
    
    pub fn set(&mut self, path: &str, value: Value) -> Result<()> {
        // Record change
        self.history.push(StateChange {
            timestamp: Instant::now(),
            path: path.to_string(),
            old_value: self.get(path).ok().cloned(),
            new_value: value.clone(),
        });
        
        // Update data
        jsonpath::set(&mut self.data, path, value)?;
        Ok(())
    }
    
    pub fn merge(&mut self, other: &WorkflowState) -> Result<()> {
        for (key, value) in &other.data {
            if key != "$input" {
                self.set(key, value.clone())?;
            }
        }
        Ok(())
    }
}
```

---

## 4. Workflow Examples

### 4.1 Research Workflow

```rust
let research_workflow = SequentialWorkflow::builder("research_assistant")
    .add_step("search", "web_search", json!({
        "input": "{{query}}",
        "max_results": 10
    }))
    .add_step("extract", "web_scraper", json!({
        "input": "{{search.results[0].url}}",
        "selectors": {
            "title": "h1",
            "content": ".article-body"
        }
    }))
    .add_step("summarize", "text_summarizer", json!({
        "input": "{{extract.result.content}}",
        "max_length": 500
    }))
    .add_step("analyze", "sentiment_analyzer", json!({
        "input": "{{summarize.result}}",
        "operations": ["sentiment", "entities", "keywords"]
    }))
    .with_error_handling(ErrorStrategy::Retry(3))
    .build()?;
```

### 4.2 Data Processing Pipeline

```rust
let etl_workflow = StreamingWorkflow::builder("etl_pipeline")
    .source(StreamSource::Tool("database_connector", json!({
        "query": "SELECT * FROM users WHERE created_at > ?",
        "parameters": ["2025-01-01"]
    })))
    .add_processor(StreamProcessor::Transform("data_transformer", json!({
        "operation": "map",
        "mapping": {
            "id": "user_id",
            "email": "contact_email"
        }
    })))
    .add_processor(StreamProcessor::Filter("data_validator", json!({
        "rules": {
            "contact_email": {"type": "email"}
        }
    })))
    .add_processor(StreamProcessor::Batch(100))
    .sink(StreamSink::Tool("file_writer", json!({
        "path": "/output/users.jsonl",
        "format": "jsonl"
    })))
    .with_backpressure(BackpressureStrategy::Buffer(1000))
    .build()?;
```

### 4.3 Parallel Data Enrichment

```rust
// Example: Enrich user data from multiple sources in parallel
let enrichment_workflow = ParallelWorkflow::builder("user_enrichment")
    .add_branch("social", Box::new(
        SequentialWorkflow::builder("social_lookup")
            .add_step("twitter", "api_tester", json!({
                "input": "https://api.twitter.com/users/{{user.handle}}",
                "method": "GET"
            }))
            .add_step("linkedin", "web_scraper", json!({
                "input": "https://linkedin.com/in/{{user.linkedin_id}}",
                "selectors": {
                    "title": ".pv-top-card__title",
                    "company": ".pv-top-card__subtitle"
                }
            }))
            .build()
    ))
    .add_branch("financial", Box::new(
        SequentialWorkflow::builder("financial_check")
            .add_step("credit", "api_tester", json!({
                "input": "https://api.creditbureau.com/score",
                "method": "POST",
                "body": {"ssn": "{{user.ssn_encrypted}}"}
            }))
            .build()
    ))
    .add_branch("public_records", Box::new(
        SequentialWorkflow::builder("public_search")
            .add_step("property", "database_connector", json!({
                "query": "SELECT * FROM property_records WHERE owner_name = ?",
                "parameters": ["{{user.full_name}}"]
            }))
            .add_step("court", "web_search", json!({
                "input": "{{user.full_name}} court records {{user.state}}",
                "max_results": 5
            }))
            .build()
    ))
    .with_max_concurrency(3)
    .with_aggregation(AggregationStrategy::AllSuccess)
    .with_error_mode(ParallelErrorMode::BestEffort)
    .with_timeout(Duration::from_secs(30))
    .build()?;
```

### 4.4 Multi-Source Search Aggregation

```rust
// Example: Search multiple sources in parallel and aggregate results
let search_workflow = ParallelWorkflow::builder("multi_search")
    .add_required_branch("google", Box::new(
        SequentialWorkflow::builder("google_search")
            .add_step("search", "web_search", json!({
                "input": "{{query}}",
                "provider": "google",
                "max_results": 10
            }))
            .build()
    ))
    .add_branch("bing", Box::new(
        SequentialWorkflow::builder("bing_search")
            .add_step("search", "web_search", json!({
                "input": "{{query}}",
                "provider": "bing",
                "max_results": 10
            }))
            .build()
    ))
    .add_branch("duckduckgo", Box::new(
        SequentialWorkflow::builder("ddg_search")
            .add_step("search", "web_search", json!({
                "input": "{{query}}",
                "provider": "duckduckgo",
                "max_results": 10
            }))
            .build()
    ))
    .add_branch("internal", Box::new(
        SequentialWorkflow::builder("internal_search")
            .add_step("vector", "semantic_search", json!({
                "input": "{{query}}",
                "collection": "knowledge_base",
                "limit": 10
            }))
            .add_step("database", "database_connector", json!({
                "query": "SELECT * FROM documents WHERE content LIKE ? LIMIT 10",
                "parameters": ["%{{query}}%"]
            }))
            .build()
    ))
    .with_max_concurrency(4)
    .with_aggregation(AggregationStrategy::Custom(Box::new(|results| {
        // Custom aggregation: merge and deduplicate results
        let mut all_results = Vec::new();
        let mut seen_urls = HashSet::new();
        
        for branch_result in results {
            if let Some(data) = branch_result.result {
                if let Some(items) = data.as_array() {
                    for item in items {
                        if let Some(url) = item.get("url").and_then(|u| u.as_str()) {
                            if seen_urls.insert(url.to_string()) {
                                all_results.push(item.clone());
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by relevance score
        all_results.sort_by(|a, b| {
            let score_a = a.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
            let score_b = b.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
        });
        
        Ok(json!(all_results))
    })))
    .with_error_mode(ParallelErrorMode::BestEffort)
    .build()?;
```

### 4.5 Parallel Validation Workflow

```rust
// Example: Validate data against multiple rules in parallel
let validation_workflow = ParallelWorkflow::builder("data_validation")
    .add_required_branch("schema", Box::new(
        SequentialWorkflow::builder("schema_validation")
            .add_step("validate", "data_validator", json!({
                "input": "{{data}}",
                "schema": "{{validation_schema}}"
            }))
            .build()
    ))
    .add_required_branch("business_rules", Box::new(
        SequentialWorkflow::builder("business_validation")
            .add_step("rules", "rule_engine", json!({
                "input": "{{data}}",
                "rules": ["uniqueness", "referential_integrity", "value_ranges"]
            }))
            .build()
    ))
    .add_branch("external", Box::new(
        ParallelWorkflow::builder("external_validation")
            .add_branch("email", Box::new(
                SequentialWorkflow::builder("email_check")
                    .add_step("verify", "email_validator", json!({
                        "input": "{{data.email}}",
                        "check_mx": true
                    }))
                    .build()
            ))
            .add_branch("phone", Box::new(
                SequentialWorkflow::builder("phone_check")
                    .add_step("verify", "phone_validator", json!({
                        "input": "{{data.phone}}",
                        "country": "{{data.country}}"
                    }))
                    .build()
            ))
            .with_aggregation(AggregationStrategy::AllSuccess)
            .build()
    ))
    .with_aggregation(AggregationStrategy::PartialSuccess(2)) // At least 2 must pass
    .with_error_mode(ParallelErrorMode::FailFast)
    .build()?;
```

---

## 5. Implementation Checklist

### Week 25 Tasks:
- [ ] Implement Workflow trait system
- [ ] Create SequentialWorkflow
- [ ] Create ConditionalWorkflow
- [ ] Create ParallelWorkflow with concurrency control
- [ ] Implement workflow state management
- [ ] Create workflow builder patterns

### Week 26 Tasks:
- [ ] Implement LoopWorkflow
- [ ] Create StreamingWorkflow
- [ ] Implement aggregation strategies for ParallelWorkflow
- [ ] Build workflow examples including parallel patterns
- [ ] Integration testing with all 41+ tools
- [ ] Performance benchmarking
- [ ] Concurrency and deadlock testing

---

## Testing Strategy

### 1. Unit Tests
- Each workflow pattern implementation
- State management operations
- Aggregation strategies
- Error handling mechanisms

### 2. Integration Tests
- Tool chain workflows
- Cross-tool data flow
- Workflow persistence and recovery
- Performance regression tests

### 3. Concurrency Tests
- Parallel workflow execution
- Deadlock detection
- Resource contention handling
- Timeout enforcement

### 4. Performance Tests
- Workflow initialization benchmarks
- Execution timing for complex workflows
- Memory usage profiling
- Concurrent execution stress tests

---

## Success Criteria

- [ ] All workflow patterns functional
- [ ] State passing between steps working
- [ ] Error recovery mechanisms in place
- [ ] Integration with full tool library verified (41+ tools)
- [ ] Performance benchmarks met (<10ms initialization)
- [ ] Workflow persistence and recovery working
- [ ] Parallel execution with proper concurrency control
- [ ] Custom aggregation strategies functional
- [ ] Integration with vector storage for context
- [ ] Documentation covers all patterns with examples

---

## Risk Mitigation

### 1. Complexity Management
- **Risk**: Workflow patterns become too complex
- **Mitigation**: Builder patterns and clear abstractions
- **Validation**: Comprehensive examples and tests

### 2. Performance Issues
- **Risk**: Workflow overhead impacts tool performance
- **Mitigation**: Lazy initialization, streaming where possible
- **Validation**: Continuous benchmarking

### 3. State Management
- **Risk**: State corruption in complex workflows
- **Mitigation**: Immutable state snapshots, transaction logs
- **Validation**: State integrity tests

### 4. Concurrency Issues
- **Risk**: Deadlocks in parallel workflows
- **Mitigation**: Proper semaphore usage, timeout enforcement
- **Validation**: Stress testing with high concurrency

---

This comprehensive design document provides the detailed specifications needed to implement Phase 8's workflow orchestration system, building on the complete tool and agent infrastructure from previous phases.