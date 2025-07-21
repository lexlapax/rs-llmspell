--[[
ABOUTME: Multi-agent consensus coordination example
ABOUTME: Demonstrates how multiple agents can evaluate options and reach consensus

This example shows a decision-making scenario where multiple expert agents:
- Evaluate investment options independently
- Provide scores and reasoning
- Reach consensus through voting and aggregation
]]

-- Create expert evaluator agents with different specializations
local financial_expert = Agent.createAsync({
    name = "financial_analyst",
    model = "mock/finance-v1",
    capabilities = {"financial_analysis", "risk_assessment"},
    system_prompt = "Evaluate investments from a financial perspective, focusing on ROI and risk."
})

local market_expert = Agent.createAsync({
    name = "market_analyst",
    model = "mock/market-v1",
    capabilities = {"market_analysis", "trend_prediction"},
    system_prompt = "Evaluate investments based on market trends and competitive positioning."
})

local technical_expert = Agent.createAsync({
    name = "technical_analyst",
    model = "mock/tech-v1",
    capabilities = {"technical_evaluation", "innovation_assessment"},
    system_prompt = "Evaluate investments based on technical merit and innovation potential."
})

local sustainability_expert = Agent.createAsync({
    name = "sustainability_analyst",
    model = "mock/sustainability-v1",
    capabilities = {"esg_analysis", "sustainability_scoring"},
    system_prompt = "Evaluate investments based on environmental and social sustainability."
})

-- Investment options to evaluate
local investment_options = {
    {
        id = "opt_1",
        name = "GreenTech Solar Initiative",
        description = "Large-scale solar panel manufacturing with new efficiency technology",
        capital_required = 5000000,
        projected_roi = "22% over 5 years",
        risk_level = "medium"
    },
    {
        id = "opt_2", 
        name = "AI Healthcare Platform",
        description = "AI-driven diagnostic platform for remote healthcare",
        capital_required = 3000000,
        projected_roi = "35% over 5 years",
        risk_level = "high"
    },
    {
        id = "opt_3",
        name = "Urban Farming Network",
        description = "Vertical farming facilities in urban centers",
        capital_required = 4000000,
        projected_roi = "18% over 5 years",
        risk_level = "low"
    }
}

-- Create consensus evaluation workflow
local consensus_workflow = Workflow.parallel({
    name = "investment_consensus",
    steps = {
        {
            name = "financial_evaluation",
            agent = "financial_analyst",
            parameters = {
                options = investment_options,
                evaluation_criteria = {
                    "roi_analysis",
                    "risk_assessment", 
                    "payback_period",
                    "cash_flow_projection"
                },
                output_format = {
                    scores = "0-100 scale",
                    reasoning = "detailed",
                    recommendation = "ranked"
                }
            }
        },
        {
            name = "market_evaluation",
            agent = "market_analyst",
            parameters = {
                options = investment_options,
                evaluation_criteria = {
                    "market_size",
                    "growth_potential",
                    "competitive_advantage",
                    "market_timing"
                },
                output_format = {
                    scores = "0-100 scale",
                    reasoning = "detailed",
                    recommendation = "ranked"
                }
            }
        },
        {
            name = "technical_evaluation",
            agent = "technical_analyst",
            parameters = {
                options = investment_options,
                evaluation_criteria = {
                    "technical_feasibility",
                    "innovation_level",
                    "scalability",
                    "implementation_risk"
                },
                output_format = {
                    scores = "0-100 scale",
                    reasoning = "detailed",
                    recommendation = "ranked"
                }
            }
        },
        {
            name = "sustainability_evaluation",
            agent = "sustainability_analyst",
            parameters = {
                options = investment_options,
                evaluation_criteria = {
                    "environmental_impact",
                    "social_benefit",
                    "governance_quality",
                    "long_term_viability"
                },
                output_format = {
                    scores = "0-100 scale",
                    reasoning = "detailed",
                    recommendation = "ranked"
                }
            }
        }
    },
    max_concurrency = 4 -- All experts evaluate simultaneously
})

-- Execute consensus evaluation
print("Starting multi-agent consensus evaluation...")
local evaluation_result = Workflow.execute(consensus_workflow)

-- Process consensus results
if evaluation_result.success then
    print("\n=== Individual Expert Evaluations ===")
    
    -- Collect all scores for consensus calculation
    local consensus_scores = {}
    for _, option in ipairs(investment_options) do
        consensus_scores[option.id] = {
            option_name = option.name,
            scores = {},
            total = 0,
            count = 0
        }
    end
    
    -- Display individual evaluations and aggregate scores
    for expert_name, evaluation in pairs(evaluation_result.outputs) do
        print("\n" .. expert_name .. ":")
        print(JSON.stringify(evaluation, 2))
        
        -- Aggregate scores (assuming evaluation contains scores for each option)
        if evaluation.scores then
            for option_id, score in pairs(evaluation.scores) do
                if consensus_scores[option_id] then
                    table.insert(consensus_scores[option_id].scores, {
                        expert = expert_name,
                        score = score
                    })
                    consensus_scores[option_id].total = consensus_scores[option_id].total + score
                    consensus_scores[option_id].count = consensus_scores[option_id].count + 1
                end
            end
        end
    end
    
    -- Calculate consensus
    print("\n=== Consensus Results ===")
    local consensus_threshold = 0.7 -- 70% agreement threshold
    
    for option_id, data in pairs(consensus_scores) do
        local average_score = data.total / data.count
        local variance = 0
        
        -- Calculate score variance to measure agreement
        for _, score_data in ipairs(data.scores) do
            variance = variance + math.pow(score_data.score - average_score, 2)
        end
        variance = variance / data.count
        local agreement_level = 1 - (variance / 10000) -- Normalize variance
        
        print("\nOption: " .. data.option_name)
        print("Average Score: " .. string.format("%.2f", average_score))
        print("Agreement Level: " .. string.format("%.2f%%", agreement_level * 100))
        print("Consensus Reached: " .. (agreement_level >= consensus_threshold and "Yes" or "No"))
        
        -- Show individual expert scores
        print("Expert Scores:")
        for _, score_data in ipairs(data.scores) do
            print("  - " .. score_data.expert .. ": " .. score_data.score)
        end
    end
    
    -- Advanced: Weighted consensus based on expert track records
    local expert_weights = {
        financial_evaluation = 0.3,
        market_evaluation = 0.25,
        technical_evaluation = 0.25,
        sustainability_evaluation = 0.2
    }
    
    print("\n=== Weighted Consensus ===")
    for option_id, data in pairs(consensus_scores) do
        local weighted_score = 0
        for _, score_data in ipairs(data.scores) do
            local weight = expert_weights[score_data.expert] or 0.25
            weighted_score = weighted_score + (score_data.score * weight)
        end
        print(data.option_name .. ": " .. string.format("%.2f", weighted_score))
    end
    
else
    print("Consensus evaluation failed: " .. evaluation_result.error)
end

-- Create a decision-making workflow that acts on consensus
local decision_workflow = Workflow.sequential({
    name = "investment_decision",
    steps = {
        {
            name = "evaluate_options",
            type = "workflow",
            workflow = consensus_workflow
        },
        {
            name = "make_decision",
            agent = "decision_maker_agent",
            parameters = {
                consensus_data = "$evaluate_options.outputs",
                decision_criteria = {
                    min_consensus_score = 70,
                    min_agreement_level = 0.7,
                    require_majority = true
                }
            }
        },
        {
            name = "prepare_recommendation",
            agent = "report_generator_agent",
            parameters = {
                decision = "$make_decision.output",
                supporting_data = "$evaluate_options.outputs",
                report_type = "executive_summary"
            }
        }
    }
})

return {
    consensus_workflow = consensus_workflow,
    decision_workflow = decision_workflow,
    consensus_scores = consensus_scores,
    experts = {
        financial_expert.id,
        market_expert.id,
        technical_expert.id,
        sustainability_expert.id
    }
}