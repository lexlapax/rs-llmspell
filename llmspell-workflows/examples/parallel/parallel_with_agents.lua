-- ABOUTME: Parallel workflows with agent integration for concurrent AI processing
-- ABOUTME: Demonstrates multi-agent collaboration, parallel analysis, and decision synthesis

-- Parallel multi-agent analysis workflow
-- Multiple AI agents analyze the same data from different perspectives

-- Create specialized analysis agents
local agents = {
    financial = Agent.createAsync({
        name = "financial_analyst",
        model = "gpt-4",
        system_prompt = "You are a financial analyst. Analyze data from a financial perspective, focusing on costs, ROI, and profitability.",
        temperature = 0.3
    }),
    
    technical = Agent.createAsync({
        name = "technical_analyst",
        model = "gpt-4",
        system_prompt = "You are a technical architect. Analyze feasibility, technical requirements, and implementation complexity.",
        temperature = 0.4
    }),
    
    market = Agent.createAsync({
        name = "market_analyst",
        model = "gpt-4",
        system_prompt = "You are a market research expert. Analyze market fit, competition, and growth potential.",
        temperature = 0.5
    }),
    
    risk = Agent.createAsync({
        name = "risk_analyst",
        model = "gpt-4",
        system_prompt = "You are a risk assessment specialist. Identify potential risks, vulnerabilities, and mitigation strategies.",
        temperature = 0.3
    })
}

-- Project proposal for analysis
local project_proposal = [[
Project: AI-Powered Customer Service Platform
Budget: $500,000
Timeline: 6 months
Features: 
- Natural language chat interface
- Sentiment analysis
- Automated ticket routing
- Integration with existing CRM
- 24/7 availability
- Multi-language support
Expected Benefits:
- 50% reduction in response time
- 30% cost savings on support staff
- Improved customer satisfaction
]]

State.set("project_proposal", project_proposal)

-- Multi-perspective analysis workflow
local analysis_workflow = Workflow.parallel({
    name = "multi_perspective_analysis",
    description = "Analyze project from multiple expert perspectives simultaneously",
    
    branches = {
        -- Financial analysis branch
        {
            name = "financial_analysis",
            required = true,
            steps = {
                {
                    name = "analyze_finances",
                    type = "agent",
                    agent = agents.financial,
                    input = {
                        prompt = [[
Analyze this project proposal from a financial perspective:

{{proposal}}

Provide:
1. ROI analysis
2. Cost breakdown estimate
3. Break-even timeline
4. Financial risks
5. Recommendation (approve/reject/modify)
]],
                        variables = {
                            proposal = State.get("project_proposal")
                        }
                    }
                },
                {
                    name = "extract_roi",
                    type = "tool",
                    tool = "text_manipulator",
                    input = {
                        input = "{{step:analyze_finances:output}}",
                        operation = "extract",
                        pattern = "ROI[:\\s]*(\\d+)%"
                    }
                }
            }
        },
        
        -- Technical analysis branch
        {
            name = "technical_analysis",
            required = true,
            steps = {
                {
                    name = "analyze_technical",
                    type = "agent",
                    agent = agents.technical,
                    input = {
                        prompt = [[
Analyze this project proposal from a technical perspective:

{{proposal}}

Provide:
1. Technical complexity (1-10)
2. Required technologies
3. Implementation challenges
4. Resource requirements
5. Feasibility assessment
]],
                        variables = {
                            proposal = State.get("project_proposal")
                        }
                    }
                },
                {
                    name = "assess_complexity",
                    type = "custom",
                    execute = function(context)
                        local analysis = context.steps.analyze_technical.output
                        local complexity = analysis:match("complexity[:\\s]*(\\d+)") or "5"
                        State.set("technical_complexity", tonumber(complexity))
                        return {
                            success = true,
                            output = "Complexity: " .. complexity .. "/10"
                        }
                    end
                }
            }
        },
        
        -- Market analysis branch
        {
            name = "market_analysis",
            required = false,
            steps = {
                {
                    name = "analyze_market",
                    type = "agent",
                    agent = agents.market,
                    input = {
                        prompt = [[
Analyze market potential for this project:

{{proposal}}

Provide:
1. Market size estimate
2. Competition analysis
3. Growth potential (high/medium/low)
4. Target audience
5. Market entry strategy
]],
                        variables = {
                            proposal = State.get("project_proposal")
                        }
                    }
                }
            }
        },
        
        -- Risk analysis branch
        {
            name = "risk_analysis",
            required = true,
            steps = {
                {
                    name = "analyze_risks",
                    type = "agent",
                    agent = agents.risk,
                    input = {
                        prompt = [[
Identify and assess risks for this project:

{{proposal}}

List top 5 risks with:
- Risk description
- Probability (high/medium/low)
- Impact (high/medium/low)
- Mitigation strategy
]],
                        variables = {
                            proposal = State.get("project_proposal")
                        }
                    }
                },
                {
                    name = "count_high_risks",
                    type = "custom",
                    execute = function(context)
                        local risks = context.steps.analyze_risks.output
                        local high_risk_count = 0
                        for _ in risks:gmatch("Probability: high") do
                            high_risk_count = high_risk_count + 1
                        end
                        State.set("high_risk_count", high_risk_count)
                        return {
                            success = true,
                            output = "High risks identified: " .. high_risk_count
                        }
                    end
                }
            }
        }
    },
    
    -- Synthesize results after parallel analysis
    post_steps = {
        {
            name = "synthesize_results",
            type = "agent",
            agent = Agent.createAsync({
                name = "synthesizer",
                model = "gpt-4",
                system_prompt = "You synthesize multiple expert analyses into actionable recommendations.",
                temperature = 0.4
            }),
            input = {
                prompt = [[
Synthesize these expert analyses into a final recommendation:

Financial Analysis:
{{financial}}

Technical Analysis:
{{technical}}

Market Analysis:
{{market}}

Risk Analysis:
{{risk}}

Provide:
1. Overall recommendation (Approve/Reject/Modify)
2. Key findings summary
3. Action items
4. Success probability (percentage)
]],
                variables = {
                    financial = "{{branch:financial_analysis:analyze_finances:output}}",
                    technical = "{{branch:technical_analysis:analyze_technical:output}}",
                    market = "{{branch:market_analysis:analyze_market:output}}",
                    risk = "{{branch:risk_analysis:analyze_risks:output}}"
                }
            }
        },
        
        {
            name = "generate_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
# Project Analysis Report
Date: {{date}}

## Executive Summary
{{synthesis}}

## Analysis Details
- Financial ROI: {{roi}}%
- Technical Complexity: {{complexity}}/10
- High-Risk Items: {{risks}}

## Decision
{{decision}}
]],
                variables = {
                    date = os.date("%Y-%m-%d"),
                    synthesis = "{{step:synthesize_results:output}}",
                    roi = State.get("extracted_roi") or "N/A",
                    complexity = State.get("technical_complexity") or "N/A",
                    risks = State.get("high_risk_count") or 0,
                    decision = "Pending committee review"
                }
            }
        }
    },
    
    max_concurrency = 4,
    timeout_ms = 120000  -- 2 minutes for AI operations
})

print("Starting multi-perspective project analysis...")
local analysis_result = analysis_workflow:execute()

if analysis_result.success then
    print("✓ Analysis completed successfully!")
    print("All perspectives analyzed in: " .. analysis_result.duration_ms .. "ms")
else
    print("✗ Analysis failed: " .. (analysis_result.error and analysis_result.error.message or "Unknown"))
end

-- Parallel content generation workflow
-- Multiple agents create different content pieces simultaneously
local content_workflow = Workflow.parallel({
    name = "content_creator",
    description = "Generate multiple content pieces in parallel",
    
    branches = {
        -- Blog post creation
        {
            name = "blog_post",
            steps = {
                {
                    name = "write_blog",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "blog_writer",
                        model = "gpt-4",
                        system_prompt = "You are a technical blog writer. Create engaging, informative content.",
                        temperature = 0.8
                    }),
                    input = {
                        prompt = "Write a 300-word blog post about the benefits of workflow automation"
                    }
                },
                {
                    name = "optimize_seo",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "seo_optimizer",
                        model = "gpt-3.5-turbo",
                        system_prompt = "You optimize content for SEO. Add keywords and meta descriptions.",
                        temperature = 0.5
                    }),
                    input = {
                        prompt = "Add SEO keywords and meta description to: {{step:write_blog:output}}"
                    }
                }
            }
        },
        
        -- Social media posts
        {
            name = "social_media",
            steps = {
                {
                    name = "create_tweets",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "social_media_writer",
                        model = "gpt-3.5-turbo",
                        system_prompt = "Create engaging social media content. Be concise and use relevant hashtags.",
                        temperature = 0.9
                    }),
                    input = {
                        prompt = "Create 3 tweets about workflow automation benefits. Include hashtags."
                    }
                },
                {
                    name = "create_linkedin",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "linkedin_writer",
                        model = "gpt-3.5-turbo",
                        system_prompt = "Write professional LinkedIn posts for B2B audience.",
                        temperature = 0.6
                    }),
                    input = {
                        prompt = "Write a LinkedIn post about digital transformation through workflow automation"
                    }
                }
            }
        },
        
        -- Email campaign
        {
            name = "email_campaign",
            steps = {
                {
                    name = "write_email",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "email_writer",
                        model = "gpt-4",
                        system_prompt = "Write compelling marketing emails with clear CTAs.",
                        temperature = 0.7
                    }),
                    input = {
                        prompt = "Write a marketing email promoting our workflow automation platform"
                    }
                },
                {
                    name = "create_subject_lines",
                    type = "agent",
                    agent = Agent.createAsync({
                        name = "subject_writer",
                        model = "gpt-3.5-turbo",
                        system_prompt = "Create attention-grabbing email subject lines.",
                        temperature = 0.9
                    }),
                    input = {
                        prompt = "Create 5 subject line variations for the workflow automation email"
                    }
                }
            }
        }
    },
    
    -- Store all content pieces
    on_complete = function(results)
        local content = {
            blog = results.branches.blog_post,
            social = results.branches.social_media,
            email = results.branches.email_campaign,
            generated_at = os.date("%Y-%m-%d %H:%M:%S")
        }
        State.set("generated_content", content)
    end
})

print("\n\nStarting parallel content generation...")
local content_result = content_workflow:execute()
print("Content generation completed: " .. (content_result.success and "Success" or "Failed"))
print("Generated " .. content_result.data.successful_branches .. " content pieces")