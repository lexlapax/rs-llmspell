-- ABOUTME: Conditional workflow with agent-based decision making
-- ABOUTME: Shows how agents can evaluate conditions and make intelligent routing decisions

-- Conditional workflow with AI-powered decision making
-- Agents analyze data and determine which branch to execute

-- Create specialized agents for different scenarios
local risk_analyzer = Agent.createAsync({
    name = "risk_analyzer",
    model = "gpt-4",
    system_prompt = "You are a risk assessment expert. Analyze data and identify risk levels (low, medium, high).",
    temperature = 0.3  -- Lower temperature for consistent analysis
})

local opportunity_finder = Agent.createAsync({
    name = "opportunity_finder",
    model = "gpt-4",
    system_prompt = "You are a business opportunity analyst. Identify growth opportunities in data.",
    temperature = 0.7
})

-- Intelligent routing workflow
local intelligent_router = Workflow.conditional({
    name = "ai_powered_router",
    description = "Route processing based on AI analysis",
    
    -- Initial analysis step
    pre_steps = {
        {
            name = "initial_analysis",
            type = "agent",
            agent = risk_analyzer,
            input = {
                prompt = [[
Analyze this business data and determine the risk level:
- Revenue: ${{revenue}}
- Costs: ${{costs}}
- Customer Count: {{customers}}
- Market Trend: {{trend}}

Respond with only: "low risk", "medium risk", or "high risk"
]],
                variables = {
                    revenue = State.get("revenue") or 50000,
                    costs = State.get("costs") or 45000,
                    customers = State.get("customers") or 100,
                    trend = State.get("trend") or "stable"
                }
            }
        }
    },
    
    branches = {
        -- Low risk - aggressive growth strategy
        {
            name = "growth_strategy",
            condition = {
                type = "step_output_contains",
                step_name = "initial_analysis",
                substring = "low risk"
            },
            steps = {
                {
                    name = "find_opportunities",
                    type = "agent",
                    agent = opportunity_finder,
                    input = {
                        prompt = "Based on low risk profile, suggest 3 growth opportunities"
                    }
                },
                {
                    name = "calculate_investment",
                    type = "tool",
                    tool = "calculator",
                    input = { 
                        input = "{{revenue}} * 0.20"  -- Invest 20% for growth
                    }
                },
                {
                    name = "generate_growth_plan",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = [[
# Growth Strategy Plan
Risk Level: Low
Opportunities:
{{opportunities}}

Recommended Investment: ${{investment}}
]],
                        variables = {
                            opportunities = "{{step:find_opportunities:output}}",
                            investment = "{{step:calculate_investment:output}}"
                        }
                    }
                }
            }
        },
        
        -- Medium risk - balanced approach
        {
            name = "balanced_strategy",
            condition = {
                type = "step_output_contains",
                step_name = "initial_analysis",
                substring = "medium risk"
            },
            steps = {
                {
                    name = "analyze_balance",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "balance_advisor",
                        model = "gpt-3.5-turbo",
                        system_prompt = "Provide balanced business advice for medium-risk scenarios"
                    }),
                    input = {
                        prompt = "Suggest a balanced approach for medium risk business"
                    }
                },
                {
                    name = "calculate_reserve",
                    type = "tool",
                    tool = "calculator",
                    input = { 
                        input = "{{revenue}} * 0.10"  -- Keep 10% as reserve
                    }
                }
            }
        },
        
        -- High risk - defensive strategy
        {
            name = "defensive_strategy",
            condition = {
                type = "step_output_contains",
                step_name = "initial_analysis",
                substring = "high risk"
            },
            steps = {
                {
                    name = "identify_risks",
                    type = "agent",
                    agent = risk_analyzer,
                    input = {
                        prompt = "List the top 5 risks that need immediate attention"
                    }
                },
                {
                    name = "cost_reduction",
                    type = "tool",
                    tool = "calculator",
                    input = {
                        input = "{{costs}} * 0.85"  -- Target 15% cost reduction
                    }
                },
                {
                    name = "create_action_plan",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = [[
# Risk Mitigation Plan
Risk Level: HIGH
Identified Risks:
{{risks}}

Target Cost Reduction: ${{reduction}}
Immediate Actions Required!
]],
                        variables = {
                            risks = "{{step:identify_risks:output}}",
                            reduction = "{{step:cost_reduction:output}}"
                        }
                    }
                }
            }
        }
    },
    
    error_strategy = "fail_fast"
})

-- Customer sentiment analysis workflow
local sentiment_workflow = Workflow.conditional({
    name = "sentiment_based_response",
    description = "Respond to customers based on sentiment analysis",
    
    branches = {
        -- Positive sentiment
        {
            name = "positive_response",
            condition = {
                type = "agent_evaluates_true",
                agent = Agent.createAsync({
                    name = "sentiment_analyzer",
                    model = "gpt-3.5-turbo",
                    system_prompt = "Analyze sentiment. Respond only with: positive, negative, or neutral"
                }),
                prompt = "Is this customer feedback positive? '{{feedback}}'"
            },
            steps = {
                {
                    name = "thank_customer",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "responder",
                        model = "gpt-3.5-turbo",
                        system_prompt = "Write friendly, appreciative responses"
                    }),
                    input = {
                        prompt = "Write a thank you response for positive feedback"
                    }
                },
                {
                    name = "offer_upgrade",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Special offer for valued customer: 20% off upgrade!",
                        variables = {}
                    }
                }
            }
        },
        
        -- Negative sentiment
        {
            name = "recovery_response",
            condition = {
                type = "agent_evaluates_true",
                agent = Agent.createAsync({
                    name = "sentiment_analyzer",
                    model = "gpt-3.5-turbo",
                    system_prompt = "Analyze sentiment. Respond only with: positive, negative, or neutral"
                }),
                prompt = "Is this customer feedback negative? '{{feedback}}'"
            },
            steps = {
                {
                    name = "apologize",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "support_agent",
                        model = "gpt-4",
                        system_prompt = "Write empathetic customer service responses"
                    }),
                    input = {
                        prompt = "Write an apology and solution for unhappy customer"
                    }
                },
                {
                    name = "create_ticket",
                    type = "tool",
                    tool = "uuid_generator",
                    input = { version = "v4" }
                },
                {
                    name = "escalate",
                    type = "custom",
                    execute = function()
                        State.set("escalation_required", true)
                        return {
                            success = true,
                            output = "Escalated to management"
                        }
                    end
                }
            }
        },
        
        -- Neutral/unclear sentiment (default)
        {
            name = "clarification_response",
            condition = { type = "always" },
            steps = {
                {
                    name = "request_clarification",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "clarifier",
                        model = "gpt-3.5-turbo",
                        system_prompt = "Ask clarifying questions politely"
                    }),
                    input = {
                        prompt = "Ask for more details about customer needs"
                    }
                }
            }
        }
    }
})

-- Test the workflows
print("Testing AI-powered routing workflow...")

-- Set business data
State.set("revenue", 75000)
State.set("costs", 68000)
State.set("customers", 150)
State.set("trend", "declining")

local route_result = intelligent_router:execute()
print("Routing decision made: " .. (route_result.success and "Success" or "Failed"))

-- Test sentiment workflow
print("\nTesting sentiment-based response workflow...")
State.set("feedback", "Your product is terrible and the support is even worse!")

local sentiment_result = sentiment_workflow:execute()
print("Response generated: " .. (sentiment_result.success and "Success" or "Failed"))