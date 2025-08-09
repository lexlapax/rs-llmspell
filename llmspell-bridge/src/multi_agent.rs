//! ABOUTME: Multi-agent coordination patterns via workflows
//! ABOUTME: Demonstrates how to coordinate multiple agents using workflow patterns

use llmspell_core::{ComponentId, Result};
use llmspell_workflows::{
    LoopWorkflowBuilder, ParallelBranch, ParallelWorkflowBuilder, SequentialWorkflowBuilder,
    StepType, WorkflowStep,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Multi-agent coordination pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationPattern {
    /// Agents work in sequence, passing results forward
    Pipeline,
    /// Agents work in parallel on different aspects
    ForkJoin,
    /// Agents vote or reach consensus
    Consensus,
    /// One agent delegates tasks to others
    Delegation,
    /// Agents collaborate with shared state
    Collaboration,
    /// Hierarchical agent organization
    Hierarchical,
}

/// Configuration for multi-agent coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    /// Coordination pattern to use
    pub pattern: CoordinationPattern,
    /// Participating agent IDs
    pub agents: Vec<String>,
    /// Pattern-specific configuration
    pub config: Value,
}

/// Create a pipeline coordination workflow
/// Agents process data sequentially, each building on previous results
///
/// # Errors
///
/// Returns an error if workflow creation fails
pub fn create_pipeline_workflow(
    name: &str,
    agents: &[String],
    initial_input: &Value,
) -> Result<llmspell_workflows::SequentialWorkflow> {
    let mut builder = SequentialWorkflowBuilder::new(name.to_string());

    // Add each agent as a sequential step
    for (i, agent_id) in agents.iter().enumerate() {
        let step_name = format!("pipeline_step_{}", i + 1);
        let step_input = if i == 0 {
            initial_input.clone()
        } else {
            // Use output from previous step
            serde_json::json!({
                "input": format!("$step_{}_output", i),
                "previous_agent": agents.get(i - 1).cloned(),
            })
        };

        let step = WorkflowStep::new(
            step_name,
            StepType::Agent {
                agent_id: ComponentId::from_name(agent_id),
                input: step_input.to_string(),
            },
        );

        builder = builder.add_step(step);
    }

    Ok(builder.build())
}

/// Create a fork-join coordination workflow
/// Agents work in parallel on different aspects of a problem
///
/// # Errors
///
/// Returns an error if workflow creation fails
pub fn create_fork_join_workflow(
    name: &str,
    agent_tasks: &[(String, String, Value)], // (agent_id, task_name, input)
    #[allow(unused_variables)] aggregation_agent: Option<&str>,
) -> Result<llmspell_workflows::ParallelWorkflow> {
    let mut builder = ParallelWorkflowBuilder::new(name.to_string());

    // Create parallel branches for each agent task
    for (agent_id, task_name, input) in agent_tasks.iter() {
        let branch = ParallelBranch::new(task_name.clone())
            .with_description(format!("Task handled by agent: {agent_id}"))
            .add_step(WorkflowStep::new(
                format!("{task_name}_execution"),
                StepType::Agent {
                    agent_id: ComponentId::from_name(agent_id),
                    input: input.to_string(),
                },
            ));

        builder = builder.add_branch(branch);
    }

    // If aggregation agent specified, could be added as post-processing
    // (would require a wrapper workflow combining parallel + sequential)

    builder.build()
}

/// Create a consensus coordination workflow
/// Multiple agents evaluate and vote on options
///
/// # Errors
///
/// Returns an error if workflow creation fails
pub fn create_consensus_workflow(
    name: &str,
    evaluator_agents: &[String],
    #[allow(unused_variables)] consensus_threshold: f64,
    options: &Value,
) -> Result<llmspell_workflows::ParallelWorkflow> {
    let mut builder = ParallelWorkflowBuilder::new(name.to_string());

    // Each agent evaluates options in parallel
    for agent_id in evaluator_agents {
        let branch = ParallelBranch::new(format!("{agent_id}_evaluation"))
            .with_description(format!("Evaluation by agent: {agent_id}"))
            .add_step(WorkflowStep::new(
                format!("{agent_id}_vote"),
                StepType::Agent {
                    agent_id: ComponentId::from_name(agent_id),
                    input: serde_json::json!({
                        "task": "evaluate_options",
                        "options": options,
                        "return_format": "score_and_reasoning"
                    })
                    .to_string(),
                },
            ));

        builder = builder.add_branch(branch);
    }

    // Note: Actual consensus calculation would happen in post-processing
    builder.build()
}

/// Create a delegation coordination workflow
/// One agent delegates subtasks to specialized agents
///
/// # Errors
///
/// Returns an error if workflow creation fails
pub fn create_delegation_workflow(
    name: &str,
    coordinator_agent: &str,
    worker_agents: &[(String, String)], // (agent_id, specialization)
    task: &Value,
) -> Result<llmspell_workflows::SequentialWorkflow> {
    let mut builder = SequentialWorkflowBuilder::new(name.to_string());

    // Step 1: Coordinator analyzes task and creates delegation plan
    builder = builder.add_step(WorkflowStep::new(
        "task_analysis".to_string(),
        StepType::Agent {
            agent_id: ComponentId::from_name(coordinator_agent),
            input: serde_json::json!({
                "task": task,
                "available_workers": worker_agents.iter().map(|(id, spec)| {
                    serde_json::json!({
                        "agent_id": id,
                        "specialization": spec
                    })
                }).collect::<Vec<_>>(),
                "action": "create_delegation_plan"
            })
            .to_string(),
        },
    ));

    // Step 2: Execute delegated tasks (would be dynamic based on plan)
    // For demo, add a parallel execution step
    builder = builder.add_step(WorkflowStep::new(
        "execute_delegations".to_string(),
        StepType::Custom {
            function_name: "execute_delegation_plan".to_string(),
            parameters: serde_json::json!({
                "plan": "$task_analysis_output.delegation_plan",
                "workers": worker_agents,
            }),
        },
    ));

    // Step 3: Coordinator aggregates results
    builder = builder.add_step(WorkflowStep::new(
        "aggregate_results".to_string(),
        StepType::Agent {
            agent_id: ComponentId::from_name(coordinator_agent),
            input: serde_json::json!({
                "task": "aggregate_delegated_results",
                "delegation_results": "$execute_delegations_output"
            })
            .to_string(),
        },
    ));

    Ok(builder.build())
}

/// Create a collaboration workflow
/// Agents work together with shared context and iterative refinement
///
/// # Errors
///
/// Returns an error if workflow creation fails
pub fn create_collaboration_workflow(
    name: &str,
    collaborating_agents: &[String],
    collaboration_rounds: usize,
    initial_context: &Value,
) -> Result<llmspell_workflows::LoopWorkflow> {
    let mut builder = LoopWorkflowBuilder::new(name.to_string()).with_range(
        0,
        i64::try_from(collaboration_rounds).unwrap_or(i64::MAX),
        1,
    );

    // Each round, agents collaborate
    for (i, agent_id) in collaborating_agents.iter().enumerate() {
        builder = builder.add_step(WorkflowStep::new(
            format!("collaborate_agent_{i}"),
            StepType::Agent {
                agent_id: ComponentId::from_name(agent_id),
                input: serde_json::json!({
                    "action": "collaborate",
                    "context": if i == 0 {
                        initial_context.clone()
                    } else {
                        serde_json::json!({
                            "previous_contributions": format!("$loop_state.contributions"),
                            "round": "$loop_state.current_iteration"
                        })
                    },
                    "other_agents": collaborating_agents.iter()
                        .filter(|&id| id != agent_id)
                        .cloned()
                        .collect::<Vec<_>>(),
                })
                .to_string(),
            },
        ));
    }

    builder.build()
}

/// Create a hierarchical coordination workflow
/// Agents organized in hierarchy with managers and workers
///
/// # Errors
///
/// Returns an error if workflow creation fails
pub fn create_hierarchical_workflow(
    name: &str,
    manager_agent: &str,
    team_leads: &[String],
    workers: &[Vec<String>], // Workers per team lead
    task: &Value,
) -> Result<llmspell_workflows::SequentialWorkflow> {
    let mut builder = SequentialWorkflowBuilder::new(name.to_string());

    // Step 1: Manager creates high-level plan
    builder = builder.add_step(WorkflowStep::new(
        "manager_planning".to_string(),
        StepType::Agent {
            agent_id: ComponentId::from_name(manager_agent),
            input: serde_json::json!({
                "task": task,
                "team_structure": {
                    "team_leads": team_leads,
                    "workers_per_team": workers
                },
                "action": "create_work_breakdown"
            })
            .to_string(),
        },
    ));

    // Step 2: Team leads receive assignments and delegate to workers
    // (This would be a parallel workflow in practice)
    builder = builder.add_step(WorkflowStep::new(
        "team_execution".to_string(),
        StepType::Custom {
            function_name: "execute_team_assignments".to_string(),
            parameters: serde_json::json!({
                "work_breakdown": "$manager_planning_output.work_breakdown",
                "team_leads": team_leads,
                "workers": workers,
            }),
        },
    ));

    // Step 3: Team leads report back to manager
    builder = builder.add_step(WorkflowStep::new(
        "consolidate_reports".to_string(),
        StepType::Custom {
            function_name: "consolidate_team_reports".to_string(),
            parameters: serde_json::json!({
                "team_results": "$team_execution_output",
                "team_leads": team_leads,
            }),
        },
    ));

    // Step 4: Manager creates final report
    builder = builder.add_step(WorkflowStep::new(
        "final_report".to_string(),
        StepType::Agent {
            agent_id: ComponentId::from_name(manager_agent),
            input: serde_json::json!({
                "action": "create_final_report",
                "team_reports": "$consolidate_reports_output",
                "original_task": task,
            })
            .to_string(),
        },
    ));

    Ok(builder.build())
}

/// Helper to create agent coordination examples
pub struct MultiAgentExamples;

impl MultiAgentExamples {
    /// Research paper analysis pipeline
    ///
    /// # Errors
    ///
    /// Returns an error if workflow creation fails
    pub fn research_pipeline_example() -> Result<llmspell_workflows::SequentialWorkflow> {
        create_pipeline_workflow(
            "research_paper_pipeline",
            &vec![
                "paper_reader_agent".to_string(),
                "concept_extractor_agent".to_string(),
                "critic_agent".to_string(),
                "summary_writer_agent".to_string(),
            ],
            &serde_json::json!({
                "paper_url": "https://example.com/paper.pdf",
                "analysis_depth": "detailed"
            }),
        )
    }

    /// Multi-perspective analysis fork-join
    ///
    /// # Errors
    ///
    /// Returns an error if workflow creation fails
    pub fn multi_perspective_analysis() -> Result<llmspell_workflows::ParallelWorkflow> {
        create_fork_join_workflow(
            "multi_perspective_analysis",
            &vec![
                (
                    "technical_analyst".to_string(),
                    "technical_review".to_string(),
                    serde_json::json!({"focus": "technical_feasibility"}),
                ),
                (
                    "business_analyst".to_string(),
                    "business_review".to_string(),
                    serde_json::json!({"focus": "business_value"}),
                ),
                (
                    "security_analyst".to_string(),
                    "security_review".to_string(),
                    serde_json::json!({"focus": "security_implications"}),
                ),
            ],
            Some("integration_agent"),
        )
    }

    /// Decision making consensus
    ///
    /// # Errors
    ///
    /// Returns an error if workflow creation fails
    pub fn investment_consensus() -> Result<llmspell_workflows::ParallelWorkflow> {
        create_consensus_workflow(
            "investment_decision",
            &vec![
                "fundamental_analyst".to_string(),
                "technical_analyst".to_string(),
                "risk_analyst".to_string(),
                "market_analyst".to_string(),
            ],
            0.75, // 75% consensus required
            &serde_json::json!({
                "investment_options": [
                    {"ticker": "AAPL", "action": "buy"},
                    {"ticker": "GOOGL", "action": "hold"},
                    {"ticker": "MSFT", "action": "sell"}
                ]
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pipeline_workflow_creation() {
        let workflow = create_pipeline_workflow(
            "test_pipeline",
            &vec!["agent1".to_string(), "agent2".to_string()],
            &serde_json::json!({"data": "test"}),
        )
        .unwrap();

        assert_eq!(workflow.step_count(), 2);
    }
    #[test]
    fn test_fork_join_workflow_creation() {
        let workflow = create_fork_join_workflow(
            "test_fork_join",
            &vec![
                (
                    "agent1".to_string(),
                    "task1".to_string(),
                    serde_json::json!({}),
                ),
                (
                    "agent2".to_string(),
                    "task2".to_string(),
                    serde_json::json!({}),
                ),
            ],
            None,
        )
        .unwrap();

        assert_eq!(workflow.branch_count(), 2);
    }
}
