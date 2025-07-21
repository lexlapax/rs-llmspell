-- ABOUTME: Workflow-Agent integration example demonstrating combined capabilities
-- ABOUTME: Shows how agents and workflows work together for complex automation

-- Workflow-Agent Integration Example
-- Demonstrates how to combine agents and workflows for intelligent automation

print("=== Workflow-Agent Integration Example ===\n")

-- Create specialized agents for the workflow
local agents = {}

-- Data Analysis Agent
agents.analyzer = Agent.createAsync({
    name = "data_analyzer",
    model = "gpt-4",
    system_prompt = [[
You are a data analysis expert. Analyze data and provide:
1. Key insights and patterns
2. Statistical summary
3. Anomalies or concerns
4. Recommendations
Be concise and data-driven.
]],
    temperature = 0.3
})

-- Decision Making Agent
agents.decision_maker = Agent.createAsync({
    name = "decision_agent",
    model = "gpt-4",
    system_prompt = [[
You are a decision-making specialist. Based on analysis:
1. Evaluate options
2. Assess risks
3. Make clear recommendations
4. Provide action steps
Be decisive and practical.
]],
    temperature = 0.2
})

-- Report Generator Agent
agents.reporter = Agent.createAsync({
    name = "report_generator",
    model = "gpt-3.5-turbo",
    system_prompt = [[
You generate clear, professional reports. You:
1. Summarize findings concisely
2. Use clear structure
3. Highlight key points
4. Make reports actionable
]],
    temperature = 0.5
})

-- Register agents
for name, agent in pairs(agents) do
    Agent.register(name, agent)
end

-- Example 1: Sequential Workflow with Agent Steps
print("Example 1: Sequential Workflow with Agent Steps")
print("-" .. string.rep("-", 47))

-- Create sample business data
local business_data = {
    revenue = {
        q1 = 250000,
        q2 = 280000,
        q3 = 265000,
        q4 = 310000
    },
    expenses = {
        q1 = 180000,
        q2 = 195000,
        q3 = 205000,
        q4 = 220000
    },
    customers = {
        q1 = 1200,
        q2 = 1350,
        q3 = 1280,
        q4 = 1450
    }
}

-- Save data
Tools.get("json_processor"):execute({
    operation = "stringify",
    input = business_data,
    pretty = true
}):chain(function(result)
    return Tools.get("file_operations"):execute({
        operation = "write",
        path = "/tmp/business_data.json",
        content = result.output
    })
end)

-- Create integrated workflow
local analysis_workflow = Workflow.sequential({
    name = "business_analysis_pipeline",
    description = "Analyze business data with AI assistance",
    
    steps = {
        -- Step 1: Load and prepare data
        {
            name = "load_data",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/business_data.json"
            }
        },
        
        -- Step 2: Calculate metrics
        {
            name = "calculate_metrics",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:load_data:output}}",
                operation = "parse"
            },
            on_complete = function(result)
                -- Calculate additional metrics
                local data = result.output
                local total_revenue = 0
                local total_expenses = 0
                
                for _, rev in pairs(data.revenue) do
                    total_revenue = total_revenue + rev
                end
                for _, exp in pairs(data.expenses) do
                    total_expenses = total_expenses + exp
                end
                
                -- Store calculated metrics (would use State in Phase 5)
                calculated_metrics = {
                    total_revenue = total_revenue,
                    total_expenses = total_expenses,
                    profit = total_revenue - total_expenses,
                    profit_margin = ((total_revenue - total_expenses) / total_revenue) * 100
                }
            end
        },
        
        -- Step 3: AI Analysis
        {
            name = "ai_analysis",
            type = "agent",
            agent = agents.analyzer,
            input = {
                prompt = string.format([[
Analyze this business data:

%s

Additional Metrics:
- Total Revenue: $%d
- Total Expenses: $%d
- Profit: $%d
- Profit Margin: %.1f%%

Provide insights on:
1. Revenue trends
2. Expense management
3. Customer growth
4. Overall health
]], 
                    "{{step:load_data:output}}",
                    calculated_metrics.total_revenue,
                    calculated_metrics.total_expenses,
                    calculated_metrics.profit,
                    calculated_metrics.profit_margin
                )
            }
        },
        
        -- Step 4: Decision Making
        {
            name = "make_decisions",
            type = "agent",
            agent = agents.decision_maker,
            input = {
                prompt = [[
Based on this analysis, make strategic recommendations:

{{step:ai_analysis:output}}

Consider:
1. Growth opportunities
2. Cost optimization
3. Risk mitigation
4. Next quarter priorities
]]
            }
        },
        
        -- Step 5: Generate Report
        {
            name = "generate_report",
            type = "agent",
            agent = agents.reporter,
            input = {
                prompt = [[
Create an executive summary report:

Analysis:
{{step:ai_analysis:output}}

Recommendations:
{{step:make_decisions:output}}

Format as a professional business report.
]]
            }
        },
        
        -- Step 6: Save Report
        {
            name = "save_report",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/business_analysis_report.md",
                content = "{{step:generate_report:output}}"
            }
        }
    }
})

print("Executing business analysis workflow...")
local analysis_result = analysis_workflow:execute()

if analysis_result.success then
    print("✓ Analysis completed successfully!")
    print("Report saved to: /tmp/business_analysis_report.md")
else
    print("✗ Analysis failed: " .. (analysis_result.error and analysis_result.error.message or "Unknown"))
end

-- Example 2: Conditional Workflow with Agent Decisions
print("\n\nExample 2: Conditional Workflow with Agent Decisions")
print("-" .. string.rep("-", 52))

-- Customer support ticket
local support_ticket = {
    id = "TICKET-1234",
    customer = "john.doe@example.com",
    subject = "Product not working after update",
    message = "After the latest update, the app crashes on startup. I've tried reinstalling but the problem persists.",
    priority = "high",
    product = "ProApp Enterprise"
}

-- Support ticket variable (would use State in Phase 5)
local current_ticket = support_ticket

-- Agent-driven conditional workflow
local support_workflow = Workflow.conditional({
    name = "intelligent_support_router",
    description = "Route support tickets based on AI analysis",
    
    -- First, analyze the ticket
    pre_steps = {
        {
            name = "analyze_ticket",
            type = "agent",
            agent = Agent.createAsync({
                name = "ticket_analyzer",
                model = "gpt-3.5-turbo",
                system_prompt = "Analyze support tickets and categorize them as: technical, billing, feature_request, or general"
            }),
            input = {
                prompt = string.format([[
Analyze this support ticket and categorize it:

Ticket ID: %s
Subject: %s
Message: %s
Priority: %s

Respond with just the category: technical, billing, feature_request, or general
]], support_ticket.id, support_ticket.subject, 
    support_ticket.message, support_ticket.priority)
            }
        }
    },
    
    branches = {
        -- Technical issues branch
        {
            name = "technical_support",
            condition = {
                type = "step_output_contains",
                step_name = "analyze_ticket",
                substring = "technical"
            },
            steps = {
                {
                    name = "diagnose_issue",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "tech_support_agent",
                        model = "gpt-4",
                        system_prompt = "You are a technical support specialist. Diagnose issues and provide solutions."
                    }),
                    input = {
                        prompt = "Diagnose and provide solution for: " .. support_ticket.message
                    }
                },
                {
                    name = "create_response",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = [[
Dear {{customer}},

Thank you for reporting this issue with {{product}}.

{{diagnosis}}

Please let us know if this resolves your issue.

Best regards,
Technical Support Team
]],
                        variables = {
                            customer = support_ticket.customer,
                            product = support_ticket.product,
                            diagnosis = "{{step:diagnose_issue:output}}"
                        }
                    }
                }
            }
        },
        
        -- Billing issues branch
        {
            name = "billing_support",
            condition = {
                type = "step_output_contains",
                step_name = "analyze_ticket",
                substring = "billing"
            },
            steps = {
                {
                    name = "billing_response",
                    type = "tool",
                    tool = "template_engine",
                    input = {
                        template = "Routing to billing department for: {{ticket}}",
                        variables = { ticket = support_ticket.id }
                    }
                }
            }
        },
        
        -- Default branch
        {
            name = "general_support",
            condition = { type = "always" },
            steps = {
                {
                    name = "general_response",
                    type = "agent",
                    agent = agents.reporter,
                    input = {
                        prompt = "Create a polite response acknowledging the ticket: " .. support_ticket.subject
                    }
                }
            }
        }
    }
})

print("Executing intelligent support workflow...")
local support_result = support_workflow:execute()
print("Support ticket routed and processed")

-- Example 3: Parallel Workflow with Multiple Agents
print("\n\nExample 3: Parallel Workflow with Multiple Agents")
print("-" .. string.rep("-", 49))

-- Market research scenario
local market_data = {
    product = "Smart Home Assistant",
    competitors = {"AlexaHome", "GoogleNest", "AppleHome"},
    target_market = "Tech-savvy homeowners aged 25-45",
    price_point = "$199"
}

-- Market research variable
local market_research = market_data

-- Parallel multi-agent analysis
local research_workflow = Workflow.parallel({
    name = "market_research_analysis",
    description = "Comprehensive market analysis using multiple AI agents",
    
    branches = {
        -- Competitive Analysis
        {
            name = "competitive_analysis",
            steps = {
                {
                    name = "analyze_competition",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "competitive_analyst",
                        model = "gpt-4",
                        system_prompt = "You are a competitive analysis expert."
                    }),
                    input = {
                        prompt = string.format([[
Analyze competitive landscape for %s
Competitors: %s
Price: %s

Provide competitive advantages and disadvantages.
]], market_data.product, table.concat(market_data.competitors, ", "), market_data.price_point)
                    }
                }
            }
        },
        
        -- Market Opportunity
        {
            name = "market_opportunity",
            steps = {
                {
                    name = "assess_opportunity",
                    type = "agent",
                    agent = agents.analyzer,
                    input = {
                        prompt = string.format([[
Assess market opportunity for: %s
Target: %s
Price: %s

Identify market size, growth potential, and risks.
]], market_data.product, market_data.target_market, market_data.price_point)
                    }
                }
            }
        },
        
        -- Marketing Strategy
        {
            name = "marketing_strategy",
            steps = {
                {
                    name = "develop_strategy",
                    type = "agent",
                    agent = agents.decision_maker,
                    input = {
                        prompt = string.format([[
Develop marketing strategy for: %s
Target: %s

Suggest channels, messaging, and campaign ideas.
]], market_data.product, market_data.target_market)
                    }
                }
            }
        }
    },
    
    -- Synthesize results
    post_steps = {
        {
            name = "synthesize_research",
            type = "agent",
            agent = agents.reporter,
            input = {
                prompt = [[
Synthesize these market research findings into a cohesive strategy:

Competitive Analysis:
{{branch:competitive_analysis:analyze_competition:output}}

Market Opportunity:
{{branch:market_opportunity:assess_opportunity:output}}

Marketing Strategy:
{{branch:marketing_strategy:develop_strategy:output}}

Create an executive summary with key recommendations.
]]
            }
        }
    }
})

print("Executing parallel market research...")
local research_result = research_workflow:execute()
print("Market research completed with " .. research_result.data.successful_branches .. " analyses")

-- Example 4: Loop Workflow with Agent Processing
print("\n\nExample 4: Loop Workflow with Agent Processing")
print("-" .. string.rep("-", 46))

-- Customer feedback to process
local feedback_items = {
    "The new feature is amazing but the UI could be more intuitive",
    "Great product! Shipping was fast and packaging was excellent",
    "App crashes frequently on Android. Very frustrating experience",
    "Love the design. Would pay extra for premium features",
    "Customer service was unhelpful. Waited 2 hours for response"
}

-- Feedback tracking variables
local sentiment_summary = { positive = 0, negative = 0, neutral = 0 }

-- Loop with agent processing
local feedback_workflow = Workflow.loop({
    name = "feedback_processor",
    description = "Process customer feedback with AI sentiment analysis",
    
    iterator = {
        collection = feedback_items
    },
    
    body = {
        {
            name = "analyze_sentiment",
            type = "agent",
            agent = Agent.createAsync({
                name = "sentiment_analyzer",
                model = "gpt-3.5-turbo",
                system_prompt = "Analyze sentiment as positive, negative, or neutral. Respond with just the sentiment.",
                temperature = 0.1
            }),
            input = {
                prompt = "Analyze sentiment of: {{loop:current_item}}"
            }
        },
        {
            name = "extract_insights",
            type = "agent",
            agent = agents.analyzer,
            input = {
                prompt = [[
Extract key insights from this feedback:
{{loop:current_item}}

Identify:
1. Main topic (product, service, shipping, etc.)
2. Specific issue or praise
3. Actionable suggestion
]]
            }
        },
        {
            name = "update_summary",
            type = "custom",
            execute = function(context)
                local sentiment = context.steps.analyze_sentiment.output:lower()
                local summary = sentiment_summary
                
                if sentiment:find("positive") then
                    summary.positive = summary.positive + 1
                elseif sentiment:find("negative") then
                    summary.negative = summary.negative + 1
                else
                    summary.neutral = summary.neutral + 1
                end
                
                sentiment_summary = summary
                
                return {
                    success = true,
                    output = "Processed feedback #" .. context.current_index
                }
            end
        }
    },
    
    -- Generate summary report
    on_complete = function()
        local summary = sentiment_summary
        local total = summary.positive + summary.negative + summary.neutral
        
        print("\nFeedback Analysis Summary:")
        print(string.format("- Positive: %d (%.1f%%)", 
              summary.positive, (summary.positive/total)*100))
        print(string.format("- Negative: %d (%.1f%%)", 
              summary.negative, (summary.negative/total)*100))
        print(string.format("- Neutral: %d (%.1f%%)", 
              summary.neutral, (summary.neutral/total)*100))
    end
})

print("Processing customer feedback...")
local feedback_result = feedback_workflow:execute()

-- Example 5: Complex Integration - Multi-Stage Pipeline
print("\n\nExample 5: Complex Multi-Stage Pipeline")
print("-" .. string.rep("-", 39))

-- Document processing pipeline
local document_pipeline = Workflow.sequential({
    name = "document_processing_pipeline",
    description = "Complex document processing with multiple AI stages",
    
    steps = {
        -- Stage 1: Document Analysis
        {
            name = "analyze_document",
            type = "parallel",
            workflow = Workflow.parallel({
                branches = {
                    {
                        name = "extract_topics",
                        steps = {{
                            name = "topic_extraction",
                            type = "agent",
                            agent = agents.analyzer,
                            input = {
                                prompt = "Extract main topics from this document: [Document content here]"
                            }
                        }}
                    },
                    {
                        name = "check_compliance",
                        steps = {{
                            name = "compliance_check",
                            type = "agent",
                            agent = Agent.createAsync({
                                name = "compliance_checker",
                                model = "gpt-4",
                                system_prompt = "Check documents for compliance issues."
                            }),
                            input = {
                                prompt = "Check for compliance issues: [Document content here]"
                            }
                        }}
                    }
                }
            })
        },
        
        -- Stage 2: Decision Point
        {
            name = "routing_decision",
            type = "conditional",
            workflow = Workflow.conditional({
                branches = {
                    {
                        name = "needs_revision",
                        condition = {
                            type = "step_output_contains",
                            step_name = "analyze_document",
                            substring = "compliance issue"
                        },
                        steps = {{
                            name = "revision_notes",
                            type = "agent",
                            agent = agents.reporter,
                            input = {
                                prompt = "Create revision notes for compliance issues"
                            }
                        }}
                    },
                    {
                        name = "ready_to_publish",
                        condition = { type = "always" },
                        steps = {{
                            name = "create_summary",
                            type = "agent",
                            agent = agents.reporter,
                            input = {
                                prompt = "Create publishable summary"
                            }
                        }}
                    }
                }
            })
        }
    }
})

print("Executing complex document pipeline...")
local pipeline_result = document_pipeline:execute()
print("Document pipeline completed")

-- Performance and Summary
print("\n\n=== Workflow-Agent Integration Summary ===")
print("Integration patterns demonstrated:")
print("1. Sequential workflow with agent analysis steps")
print("2. Conditional routing based on agent decisions")
print("3. Parallel execution with multiple specialized agents")
print("4. Loop processing with agent-based analysis")
print("5. Complex multi-stage pipelines")
print("\nKey capabilities shown:")
print("- Agents as workflow steps")
print("- Agent output driving workflow logic")
print("- Multiple agents working in parallel")
print("- State sharing between agents and workflows")
print("- Complex orchestration patterns")

print("\n=== Workflow-Agent Integration Example Complete ===")