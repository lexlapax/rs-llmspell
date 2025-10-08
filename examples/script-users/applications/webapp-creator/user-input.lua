-- WebApp Creator User Input File
-- Modify this file to specify your web application requirements
-- The main.lua file will load these specifications and generate your app

return {
    -- Project Basic Information
    project = {
        name = "TaskFlow",
        description = "Modern task management with real-time collaboration",
        version = "1.0.0"
    },
    
    -- Main Requirements (this is what the AI agents will analyze)
    requirements = [[
Create a modern task management web application with real-time collaboration features.
Users should be able to create projects, add tasks, assign them to team members,
and track progress. Need both web and mobile responsive design.
    ]],
    
    -- UX Requirements and Preferences
    ux = {
        -- Target users
        target_users = "Small to medium teams, remote workers, project managers",
        
        -- Design preferences
        design_style = "Modern, clean, minimalist",
        color_scheme = "Professional blues and grays",
        
        -- Specific UX questions to explore
        ux_questions = {
            "Should we prioritize mobile or desktop experience?",
            "What level of real-time collaboration is needed?",
            "How important are offline capabilities?",
            "Should we include gamification elements?",
            "What accessibility standards must we meet?"
        },
        
        -- Required features
        must_have_features = {
            "User authentication",
            "Project creation and management",
            "Task assignment and tracking",
            "Real-time updates",
            "Mobile responsive design",
            "Team collaboration"
        },
        
        -- Nice to have features
        nice_to_have_features = {
            "Dark mode",
            "Kanban board view",
            "Gantt charts",
            "Time tracking",
            "Integrations with other tools",
            "Advanced reporting"
        }
    },
    
    -- Technical Preferences
    technical = {
        -- Frontend preferences
        frontend = {
            framework = "React",  -- React, Vue, Angular, Svelte, or "any"
            styling = "Tailwind", -- Tailwind, Bootstrap, Material-UI, or "any"
            typescript = true,
            testing = true
        },
        
        -- Backend preferences
        backend = {
            runtime = "Node.js",  -- Node.js, Python, Go, or "any"
            api_style = "GraphQL", -- REST, GraphQL, or "both"
            database = "PostgreSQL", -- PostgreSQL, MySQL, MongoDB, or "any"
            authentication = "JWT",
            realtime = "WebSockets"
        },
        
        -- Infrastructure preferences
        infrastructure = {
            containerization = "Docker",
            orchestration = "docker-compose", -- docker-compose, Kubernetes, or "none"
            ci_cd = "GitHub Actions",
            monitoring = true,
            cloud_provider = "any" -- AWS, GCP, Azure, or "any"
        }
    },
    
    -- Constraints
    constraints = {
        budget = "medium",  -- low, medium, high
        timeline = "2 weeks", -- Expected development timeline
        team_size = "3-5 developers",
        performance = {
            load_time = "< 2 seconds",
            concurrent_users = "1000",
            response_time = "< 100ms"
        },
        compliance = {
            "GDPR",
            "WCAG 2.1 AA"
        }
    },
    
    -- Output Preferences
    output = {
        -- What to generate
        generate = {
            frontend_code = true,
            backend_code = true,
            database_schema = true,
            api_documentation = true,
            deployment_config = true,
            testing_suite = true,
            documentation = true
        },
        
        -- Output format preferences
        include_comments = true,
        include_tests = true,
        include_docker = true,
        include_ci_cd = true
    },
    
    -- Advanced Options
    advanced = {
        -- Number of iterations for refinement
        max_iterations = 3,
        
        -- Cost limits
        max_cost = 10.00,  -- Maximum API cost in USD
        
        -- Agent preferences (optional overrides)
        preferred_models = {
            -- Uncomment to override default model selection
            -- requirements_analyst = "openai/gpt-4",
            -- frontend_developer = "anthropic/claude-3-opus-20240229"
        },
        
        -- Research depth
        research_depth = "comprehensive", -- minimal, standard, comprehensive
        
        -- Number of web searches to perform
        max_web_searches = 15
    }
}

-- Example of alternative project configurations:
-- Just uncomment and modify the return statement above

-- E-commerce Platform Example
-- return {
--     project = {
--         name = "ShopEasy",
--         description = "Modern e-commerce platform with AI recommendations"
--     },
--     requirements = "Build an e-commerce platform with product catalog, shopping cart, " ..
--         "secure checkout, order tracking, and AI-powered product recommendations. " ..
--         "Include admin dashboard for inventory management.",
--     -- ... rest of configuration
-- }

-- Social Media Dashboard Example
-- return {
--     project = {
--         name = "SocialHub",
--         description = "Unified social media management dashboard"
--     },
--     requirements = "Create a dashboard to manage multiple social media accounts. " ..
--         "Features: post scheduling, analytics, engagement tracking, " ..
--         "content calendar, and team collaboration.",
--     -- ... rest of configuration
-- }

-- Educational Platform Example
-- return {
--     project = {
--         name = "LearnFlow",
--         description = "Interactive online learning platform"
--     },
--     requirements = "Develop an online learning platform with video courses, " ..
--         "interactive quizzes, progress tracking, certificates, " ..
--         "and live tutoring sessions.",
--     -- ... rest of configuration
-- }