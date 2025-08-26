//! ABOUTME: Provider-specific rate limit configurations
//! ABOUTME: Defines rate limits for various external API providers

use crate::rate_limiting::retry_handler::BackoffStrategy;
use serde::{Deserialize, Serialize};

/// Rate limit configuration for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Requests per hour (optional)
    pub requests_per_hour: Option<u32>,
    /// Daily limit (optional)
    pub daily_limit: Option<u32>,
    /// Allow burst requests
    pub allow_burst: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Backoff strategy for retries
    pub backoff_strategy: BackoffStrategy,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: None,
            daily_limit: None,
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 100 },
        }
    }
}

/// Pre-configured rate limits for known providers
pub struct ProviderLimits;

impl ProviderLimits {
    /// `OpenAI` API rate limits (GPT-3.5-turbo tier)
    #[must_use]
    pub fn openai() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 3_500,
            requests_per_hour: None,
            daily_limit: Some(200_000),
            allow_burst: true,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 1000 },
        }
    }

    /// Anthropic Claude API rate limits
    #[must_use]
    pub fn anthropic() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 50,
            requests_per_hour: Some(1000),
            daily_limit: None,
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 500 },
        }
    }

    /// Google Search API rate limits
    #[must_use]
    pub fn google_search() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 100,
            requests_per_hour: None,
            daily_limit: Some(10_000),
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Linear { increment_ms: 1000 },
        }
    }

    /// `DuckDuckGo` API rate limits (conservative estimate)
    #[must_use]
    pub fn duckduckgo() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 20,
            requests_per_hour: Some(1000),
            daily_limit: None,
            allow_burst: false,
            max_retries: 5,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 2000 },
        }
    }

    /// Bing Search API rate limits
    #[must_use]
    pub fn bing_search() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 1000,
            requests_per_hour: None,
            daily_limit: None,
            allow_burst: true,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 100 },
        }
    }

    /// Brave Search API rate limits
    #[must_use]
    pub fn brave_search() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: Some(2000),
            daily_limit: None,
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Linear { increment_ms: 500 },
        }
    }

    /// `SerpAPI` rate limits
    #[must_use]
    pub fn serpapi() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: None,
            daily_limit: Some(5000),
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 1000 },
        }
    }

    /// GitHub API rate limits (authenticated)
    #[must_use]
    pub fn github() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 5000, // 5000 per hour = ~83 per minute
            requests_per_hour: Some(5000),
            daily_limit: None,
            allow_burst: true,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 500 },
        }
    }

    /// Slack API rate limits
    #[must_use]
    pub fn slack() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: None,
            daily_limit: None,
            allow_burst: true,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 1000 },
        }
    }

    /// `SendGrid` API rate limits
    #[must_use]
    pub fn sendgrid() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 600,
            requests_per_hour: None,
            daily_limit: Some(100_000),
            allow_burst: true,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Linear { increment_ms: 200 },
        }
    }

    /// AWS SES rate limits (default)
    #[must_use]
    pub fn aws_ses() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 14, // 14 emails per second = 840 per minute, being conservative
            requests_per_hour: None,
            daily_limit: Some(50_000),
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 1000 },
        }
    }

    /// Generic API rate limits (conservative defaults)
    #[must_use]
    pub fn generic() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: Some(1000),
            daily_limit: None,
            allow_burst: false,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 1000 },
        }
    }

    /// Get rate limit config for a provider by name
    #[must_use]
    pub fn for_provider(provider: &str) -> RateLimitConfig {
        match provider.to_lowercase().as_str() {
            "openai" => Self::openai(),
            "anthropic" => Self::anthropic(),
            "google" | "google_search" => Self::google_search(),
            "duckduckgo" | "ddg" => Self::duckduckgo(),
            "bing" | "bing_search" => Self::bing_search(),
            "brave" | "brave_search" => Self::brave_search(),
            "serpapi" => Self::serpapi(),
            "github" => Self::github(),
            "slack" => Self::slack(),
            "sendgrid" => Self::sendgrid(),
            "aws_ses" | "ses" => Self::aws_ses(),
            _ => Self::generic(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_provider_limits() {
        let openai = ProviderLimits::openai();
        assert_eq!(openai.requests_per_minute, 3_500);
        assert_eq!(openai.daily_limit, Some(200_000));
        assert!(openai.allow_burst);

        let duckduckgo = ProviderLimits::duckduckgo();
        assert_eq!(duckduckgo.requests_per_minute, 20);
        assert!(!duckduckgo.allow_burst);
    }
    #[test]
    fn test_for_provider() {
        let config = ProviderLimits::for_provider("openai");
        assert_eq!(config.requests_per_minute, 3_500);

        let config = ProviderLimits::for_provider("unknown");
        assert_eq!(config.requests_per_minute, 60); // generic default
    }
}
