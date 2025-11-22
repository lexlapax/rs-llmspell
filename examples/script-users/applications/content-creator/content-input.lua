-- Recommended profile: minimal
-- Run with: llmspell -p minimal run content-input.lua
-- Input file (no execution profile needed)

-- Content Input for Content Generation Platform
-- This file provides content requests to be processed
-- Similar pattern to code-review-assistant's code-input.lua

return {
    -- Blog content request
    {
        type = "blog",
        topic = "The Future of AI in Healthcare: How Machine Learning is Revolutionizing Patient Care",
        target_audience = "Healthcare professionals and tech enthusiasts",
        tone = "informative and optimistic",
        length = "1500-2000 words",
        seo_keywords = {"AI healthcare", "machine learning medicine", "medical AI", "healthcare technology"},
        requirements = {
            "Include recent statistics and research",
            "Cover both benefits and challenges",
            "Add real-world case studies",
            "Include expert quotes if possible"
        }
    },
    
    -- Social media content request
    {
        type = "social",
        topic = "Announcing our new AI-powered analytics dashboard - Transform your data into insights!",
        platforms = {"twitter", "linkedin", "instagram"},
        campaign = "product_launch",
        tone = "exciting and professional",
        requirements = {
            "Create platform-specific versions",
            "Include relevant hashtags",
            "Add call-to-action",
            "Keep within platform character limits"
        }
    },
    
    -- Email newsletter request
    {
        type = "email",
        topic = "Monthly Tech Digest: December 2024 Innovations",
        segments = {"subscribers", "premium_users"},
        tone = "friendly and informative",
        sections = {
            "Product updates and new features",
            "Customer success stories",
            "Industry news and trends",
            "Upcoming webinars and events"
        },
        personalization_fields = {"first_name", "company", "subscription_tier"}
    },
    
    -- Mixed content request (should trigger conditional routing)
    {
        type = "auto",  -- Let the system decide based on content
        topic = "Cybersecurity Best Practices for Remote Teams",
        context = "Educational content for our audience",
        tone = "authoritative yet accessible",
        notes = "This could work as a blog post, social campaign, or email series"
    }
}