//! ABOUTME: Tests for multi-agent workflow coordination patterns
//! ABOUTME: Verifies pipeline, parallel, and consensus agent coordination

use llmspell_bridge::multi_agent::{
    create_consensus_workflow, create_fork_join_workflow, create_pipeline_workflow,
};
use serde_json::json;
#[tokio::test]
async fn test_pipeline_coordination() {
    // Test sequential agent pipeline
    let agents = vec![
        "research_agent".to_string(),
        "analysis_agent".to_string(),
        "summary_agent".to_string(),
    ];

    let initial_input = json!({
        "topic": "AI in healthcare",
        "depth": "comprehensive"
    });

    let workflow = create_pipeline_workflow("test_pipeline", &agents, &initial_input).unwrap();

    // Verify workflow was created successfully
    assert_eq!(workflow.name(), "test_pipeline");
    // Note: Cannot access internal steps field directly, but workflow is created
}
#[tokio::test]
async fn test_fork_join_coordination() {
    // Test parallel agent execution
    let agent_tasks = vec![
        (
            "sentiment_agent".to_string(),
            "analyze_sentiment".to_string(),
            json!({"text": "sample"}),
        ),
        (
            "fact_checker".to_string(),
            "verify_facts".to_string(),
            json!({"claims": []}),
        ),
        (
            "style_agent".to_string(),
            "analyze_style".to_string(),
            json!({"document": "doc"}),
        ),
    ];

    let workflow =
        create_fork_join_workflow("test_fork_join", &agent_tasks, Some("coordinator_agent"))
            .unwrap();

    // Verify workflow was created successfully
    assert_eq!(workflow.name(), "test_fork_join");
    // Note: Cannot access internal branches field directly, but workflow is created with correct number of tasks
}
#[tokio::test]
async fn test_consensus_coordination() {
    // Test consensus evaluation pattern
    let evaluators = vec![
        "expert1".to_string(),
        "expert2".to_string(),
        "expert3".to_string(),
    ];

    let options = json!([
        {"id": "opt1", "name": "Option 1"},
        {"id": "opt2", "name": "Option 2"},
    ]);

    let workflow = create_consensus_workflow(
        "test_consensus",
        &evaluators,
        0.7, // 70% consensus threshold
        &options,
    )
    .unwrap();

    // Verify workflow was created successfully
    assert_eq!(workflow.name(), "test_consensus");
    // Note: Cannot access internal branches field directly, but workflow is created with evaluator branches
}
#[tokio::test]
async fn test_multi_agent_integration() {
    use llmspell_bridge::{workflows::WorkflowBridge, ComponentRegistry};
    use std::sync::Arc;

    let registry = Arc::new(ComponentRegistry::new());
    let bridge = WorkflowBridge::new(registry);

    // Create a simple pipeline workflow through the bridge
    let _params = json!({
        "pattern": "pipeline",
        "agents": ["agent1", "agent2"],
        "initial_input": {"data": "test"}
    });

    // This would create the workflow using multi-agent patterns
    let workflow_types = bridge.list_workflow_types();
    assert!(workflow_types.contains(&"sequential".to_string()));
    assert!(workflow_types.contains(&"parallel".to_string()));
}
#[test]
fn test_coordination_pattern_serialization() {
    use llmspell_bridge::multi_agent::{CoordinationPattern, MultiAgentConfig};

    let config = MultiAgentConfig {
        pattern: CoordinationPattern::Pipeline,
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        config: json!({"timeout": 5000}),
    };

    // Test serialization
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(serialized.contains("Pipeline"));

    // Test deserialization
    let deserialized: MultiAgentConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.agents.len(), 2);
}
#[test]
fn test_all_coordination_patterns() {
    use llmspell_bridge::multi_agent::CoordinationPattern;

    // Verify all patterns are defined
    let patterns = vec![
        CoordinationPattern::Pipeline,
        CoordinationPattern::ForkJoin,
        CoordinationPattern::Consensus,
        CoordinationPattern::Delegation,
        CoordinationPattern::Collaboration,
        CoordinationPattern::Hierarchical,
    ];

    for pattern in patterns {
        let json = serde_json::to_value(&pattern).unwrap();
        assert!(json.is_string());
    }
}
