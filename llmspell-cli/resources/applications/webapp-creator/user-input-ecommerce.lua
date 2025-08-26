-- WebApp Creator User Input: E-commerce Platform
-- Alternative configuration for generating an e-commerce application

return {
    -- Project Basic Information
    project = {
        name = "ShopEasy",
        description = "Modern e-commerce platform with AI-powered recommendations",
        version = "1.0.0"
    },
    
    -- Main Requirements
    requirements = [[
Build a comprehensive e-commerce platform with product catalog, shopping cart,
secure checkout process, order tracking, and AI-powered product recommendations.
Include an admin dashboard for inventory management, order processing, and analytics.
Support multiple payment methods and international shipping.
    ]],
    
    -- UX Requirements and Preferences
    ux = {
        -- Target users
        target_users = "Online shoppers, small businesses, retailers",
        
        -- Design preferences
        design_style = "Clean, modern, trustworthy",
        color_scheme = "Professional with accent colors for CTAs",
        
        -- Specific UX questions to explore
        ux_questions = {
            "Should we optimize for conversion or browsing experience?",
            "How prominent should AI recommendations be?",
            "What's the ideal checkout flow (single page vs multi-step)?",
            "Should we include social shopping features?",
            "How important is mobile commerce vs desktop?"
        },
        
        -- Required features
        must_have_features = {
            "Product catalog with search and filters",
            "Shopping cart and wishlist",
            "Secure checkout with multiple payment options",
            "User accounts and order history",
            "Inventory management",
            "Order tracking and notifications",
            "Admin dashboard",
            "Product reviews and ratings"
        },
        
        -- Nice to have features
        nice_to_have_features = {
            "AI-powered recommendations",
            "Live chat support",
            "Loyalty program",
            "Gift cards and coupons",
            "Social media integration",
            "Abandoned cart recovery",
            "Multi-vendor marketplace",
            "AR product preview"
        }
    },
    
    -- Technical Preferences
    technical = {
        -- Frontend preferences
        frontend = {
            framework = "Next.js",  -- For SSR and SEO
            styling = "Tailwind",
            typescript = true,
            testing = true
        },
        
        -- Backend preferences
        backend = {
            runtime = "Node.js",
            api_style = "REST",  -- REST for product APIs
            database = "PostgreSQL",
            authentication = "JWT",
            realtime = "WebSockets",  -- For live inventory updates
            cache = "Redis"
        },
        
        -- Infrastructure preferences
        infrastructure = {
            containerization = "Docker",
            orchestration = "Kubernetes",
            ci_cd = "GitHub Actions",
            monitoring = true,
            cloud_provider = "AWS",  -- For S3, CloudFront CDN
            cdn = true
        }
    },
    
    -- Constraints
    constraints = {
        budget = "high",  -- E-commerce needs robust infrastructure
        timeline = "3 months",
        team_size = "5-8 developers",
        performance = {
            load_time = "< 1.5 seconds",
            concurrent_users = "10000",
            response_time = "< 50ms",
            uptime = "99.9%"
        },
        compliance = {
            "PCI DSS",  -- Payment card compliance
            "GDPR",
            "CCPA",
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
            documentation = true,
            security_audit = true,  -- Important for e-commerce
            performance_tests = true
        },
        
        -- Output format preferences
        include_comments = true,
        include_tests = true,
        include_docker = true,
        include_ci_cd = true,
        include_monitoring = true
    },
    
    -- Advanced Options
    advanced = {
        -- Number of iterations for refinement
        max_iterations = 5,  -- More iterations for complex e-commerce
        
        -- Cost limits
        max_cost = 25.00,  -- Higher budget for comprehensive platform
        
        -- Agent preferences (optimize for e-commerce expertise)
        preferred_models = {
            requirements_analyst = "openai/gpt-4",  -- Better for complex requirements
            database_architect = "openai/gpt-4",  -- Complex schema design
            security_auditor = "anthropic/claude-3-opus-20240229"  -- Critical for payments
        },
        
        -- Research depth
        research_depth = "comprehensive",
        
        -- Number of web searches to perform
        max_web_searches = 20  -- More research for e-commerce best practices
    }
}