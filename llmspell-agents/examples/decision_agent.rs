//! ABOUTME: Decision-making agent example demonstrating complex decision logic
//! ABOUTME: Shows how agents can evaluate options, apply criteria, and make informed choices

use llmspell_agents::templates::{AgentTemplate, TemplateInstantiationParams, ToolAgentTemplate};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::Instant;
use tracing::{info, Level};

/// Example demonstrating a decision-making agent that evaluates multiple criteria,
/// weighs options, and makes informed decisions with confidence scoring.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Decision-Making Agent Example");

    // Create a decision-making agent with analytical tools
    let decision_template = ToolAgentTemplate::new();
    let decision_params = TemplateInstantiationParams::new("decision-001".to_string())
        .with_parameter("agent_name", "Strategic Decision Maker".into())
        .with_parameter(
            "allowed_tools",
            vec![
                "calculator",
                "data_validation",
                "json_processor",
                "csv_analyzer",
                "text_manipulator",
                "web_search",
                "file_operations",
            ]
            .into(),
        )
        .with_parameter("tool_selection_strategy", "analytical".into())
        .with_parameter("decision_framework", "multi_criteria".into())
        .with_parameter("confidence_threshold", 0.7.into())
        .with_parameter("enable_explanation", true.into());

    let decision_result = decision_template.instantiate(decision_params).await?;
    let decision_maker = decision_result.agent;

    // Example 1: Technology Stack Selection
    println!("\n=== Example 1: Technology Stack Selection ===");

    let tech_decision = AgentInput::text(
        "Select the best technology stack for a new web application:\n\
         Options:\n\
         1. React + Node.js + PostgreSQL\n\
         2. Vue.js + Python/Django + MySQL\n\
         3. Angular + Java/Spring + MongoDB\n\
         \n\
         Criteria:\n\
         - Team expertise (current: JavaScript, some Python)\n\
         - Scalability requirements (expected: 100k users)\n\
         - Time to market (target: 3 months)\n\
         - Maintenance cost\n\
         - Community support\n\
         \n\
         Provide recommendation with confidence score and rationale.",
    );

    let start = Instant::now();
    let tech_output = decision_maker
        .execute(tech_decision, ExecutionContext::default())
        .await?;
    println!("Technology Decision:\n{}", tech_output.text);
    println!("Decision Time: {:?}", start.elapsed());

    // Example 2: Investment Portfolio Allocation
    println!("\n=== Example 2: Investment Portfolio Allocation ===");

    let investment_decision = AgentInput::text(
        "Allocate $100,000 investment portfolio:\n\
         Options:\n\
         - Stocks (Tech, Healthcare, Finance sectors)\n\
         - Bonds (Government, Corporate)\n\
         - Real Estate (REITs)\n\
         - Commodities (Gold, Silver)\n\
         - Cryptocurrency\n\
         \n\
         Constraints:\n\
         - Risk tolerance: Moderate\n\
         - Investment horizon: 10 years\n\
         - Need 5% annual liquidity\n\
         - ESG preferences: High\n\
         \n\
         Provide allocation percentages with justification.",
    );

    let investment_output = decision_maker
        .execute(investment_decision, ExecutionContext::default())
        .await?;
    println!("Investment Allocation:\n{}", investment_output.text);

    // Example 3: Hiring Decision
    println!("\n=== Example 3: Hiring Decision ===");

    let hiring_decision = AgentInput::text(
        "Evaluate candidates for Senior Engineer position:\n\
         \n\
         Candidate A:\n\
         - 8 years experience\n\
         - Strong technical skills (9/10)\n\
         - Leadership experience\n\
         - Salary expectation: $150k\n\
         - Available immediately\n\
         \n\
         Candidate B:\n\
         - 5 years experience\n\
         - Excellent problem-solving (10/10)\n\
         - No leadership experience\n\
         - Salary expectation: $120k\n\
         - Available in 2 months\n\
         \n\
         Candidate C:\n\
         - 10 years experience\n\
         - Good technical skills (7/10)\n\
         - Management background\n\
         - Salary expectation: $180k\n\
         - Available in 1 month\n\
         \n\
         Team needs: Technical excellence, mentoring capability, budget: $160k",
    );

    let hiring_output = decision_maker
        .execute(hiring_decision, ExecutionContext::default())
        .await?;
    println!("Hiring Recommendation:\n{}", hiring_output.text);

    // Example 4: Strategic Business Decision
    println!("\n=== Example 4: Strategic Business Decision ===");

    let business_decision = AgentInput::text(
        "Decide on market expansion strategy:\n\
         \n\
         Option A: Expand to European market\n\
         - Investment: $5M\n\
         - Time to profit: 2 years\n\
         - Risk: Medium (regulatory challenges)\n\
         - Potential ROI: 150%\n\
         \n\
         Option B: Expand product line domestically\n\
         - Investment: $3M\n\
         - Time to profit: 1 year\n\
         - Risk: Low\n\
         - Potential ROI: 80%\n\
         \n\
         Option C: Acquire competitor\n\
         - Investment: $8M\n\
         - Time to profit: 6 months\n\
         - Risk: High (integration challenges)\n\
         - Potential ROI: 200%\n\
         \n\
         Company status: $10M available, stable domestic market, growth target: 50%",
    );

    let business_output = decision_maker
        .execute(business_decision, ExecutionContext::default())
        .await?;
    println!("Strategic Decision:\n{}", business_output.text);

    // Example 5: Multi-Stage Decision Process
    println!("\n=== Example 5: Multi-Stage Decision Process ===");

    let multistage_decision = AgentInput::text(
        "Make sequential decisions for product launch:\n\
         \n\
         Stage 1: Launch timing\n\
         - Q1: Competition low, market not ready\n\
         - Q2: Competition increasing, market warming\n\
         - Q3: High competition, market ready\n\
         \n\
         Stage 2: Pricing strategy (depends on timing)\n\
         - Premium: High margin, low volume\n\
         - Competitive: Medium margin, medium volume\n\
         - Penetration: Low margin, high volume\n\
         \n\
         Stage 3: Marketing channels (depends on pricing)\n\
         - Digital only\n\
         - Mixed (digital + traditional)\n\
         - Full scale\n\
         \n\
         Optimize for: Market share and profitability balance",
    );

    let multistage_output = decision_maker
        .execute(multistage_decision, ExecutionContext::default())
        .await?;
    println!("Multi-Stage Decision Plan:\n{}", multistage_output.text);

    // Example 6: Risk Assessment Decision
    println!("\n=== Example 6: Risk Assessment Decision ===");

    let risk_decision = AgentInput::text(
        "Assess and decide on cybersecurity investment:\n\
         \n\
         Current vulnerabilities:\n\
         - Outdated firewall (Risk: High)\n\
         - No employee training (Risk: Medium)\n\
         - Basic encryption only (Risk: Medium)\n\
         - No incident response plan (Risk: High)\n\
         \n\
         Solution options:\n\
         1. Basic package: $50k (addresses 40% of risks)\n\
         2. Standard package: $150k (addresses 70% of risks)\n\
         3. Premium package: $300k (addresses 95% of risks)\n\
         4. Custom solution: $200k (addresses specific high risks)\n\
         \n\
         Context: Recent industry breaches, compliance requirements, budget constraints",
    );

    let risk_output = decision_maker
        .execute(risk_decision, ExecutionContext::default())
        .await?;
    println!("Risk Mitigation Decision:\n{}", risk_output.text);

    // Example 7: A/B Test Decision
    println!("\n=== Example 7: A/B Test Decision ===");

    let ab_decision = AgentInput::text(
        "Analyze A/B test results and decide on implementation:\n\
         \n\
         Test: New checkout flow\n\
         Duration: 30 days\n\
         Sample size: 50,000 users per variant\n\
         \n\
         Variant A (Control):\n\
         - Conversion rate: 3.2%\n\
         - Average order value: $85\n\
         - Cart abandonment: 68%\n\
         - User satisfaction: 7.5/10\n\
         \n\
         Variant B (New flow):\n\
         - Conversion rate: 3.8%\n\
         - Average order value: $82\n\
         - Cart abandonment: 65%\n\
         - User satisfaction: 8.2/10\n\
         \n\
         Statistical significance: 95%\n\
         Implementation cost: $25k\n\
         \n\
         Make decision with confidence interval and expected impact.",
    );

    let ab_output = decision_maker
        .execute(ab_decision, ExecutionContext::default())
        .await?;
    println!("A/B Test Decision:\n{}", ab_output.text);

    // Decision Frameworks
    println!("\n=== Decision Frameworks ===");
    println!("1. **Multi-Criteria Analysis**: Weight multiple factors");
    println!("2. **Cost-Benefit Analysis**: Quantify trade-offs");
    println!("3. **Risk-Reward Matrix**: Balance risk vs opportunity");
    println!("4. **Decision Tree**: Map out scenarios and probabilities");
    println!("5. **SWOT Analysis**: Strengths, Weaknesses, Opportunities, Threats");

    // Best Practices
    println!("\n=== Decision-Making Best Practices ===");
    println!("1. **Define Clear Criteria**: Establish decision factors upfront");
    println!("2. **Quantify When Possible**: Use data to support decisions");
    println!("3. **Consider Constraints**: Account for limitations and requirements");
    println!("4. **Document Rationale**: Explain the reasoning process");
    println!("5. **Monitor Outcomes**: Track decision results for learning");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::environment_helpers::create_test_context;

    #[tokio::test]
    async fn test_decision_agent_creation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-decision".to_string())
            .with_parameter("agent_name", "Test Decision Maker".into())
            .with_parameter(
                "allowed_tools",
                vec!["calculator", "data_validation"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_simple_decision() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("simple-decision".to_string())
            .with_parameter("agent_name", "Simple Decision Maker".into())
            .with_parameter("allowed_tools", vec!["calculator"].into())
            .with_parameter("confidence_threshold", 0.6.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Choose between option A (score: 8) and option B (score: 6)");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_multi_criteria_decision() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("multi-criteria".to_string())
            .with_parameter("agent_name", "Multi-Criteria Analyzer".into())
            .with_parameter("allowed_tools", vec!["calculator", "json_processor"].into())
            .with_parameter("decision_framework", "multi_criteria".into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Evaluate options based on cost, quality, and time");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_risk_based_decision() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("risk-decision".to_string())
            .with_parameter("agent_name", "Risk Analyzer".into())
            .with_parameter(
                "allowed_tools",
                vec!["calculator", "data_validation"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Assess risk levels and make recommendation");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_decision_explanation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("explained-decision".to_string())
            .with_parameter("agent_name", "Explainable Decision Maker".into())
            .with_parameter("allowed_tools", vec!["text_manipulator"].into())
            .with_parameter("enable_explanation", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Make a decision and explain the reasoning");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }
}
