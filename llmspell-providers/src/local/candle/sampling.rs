//! Token sampling strategies for text generation
//!
//! Implements various sampling methods: temperature, top-p, top-k, repeat penalty.

use anyhow::{anyhow, Result};
use candle_core::Tensor;
use tracing::{debug, trace};

/// Sampling configuration
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Temperature for logit scaling (0.0 = greedy, >1.0 = more random)
    pub temperature: f64,
    /// Top-p (nucleus) sampling threshold (0.0-1.0)
    pub top_p: Option<f64>,
    /// Top-k sampling limit (number of tokens to consider)
    pub top_k: Option<usize>,
    /// Repeat penalty factor (1.0 = no penalty, >1.0 = penalize repeats)
    pub repeat_penalty: f64,
    /// Context window for repeat penalty
    pub repeat_last_n: usize,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: Some(0.9),
            top_k: Some(40),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

impl SamplingConfig {
    /// Greedy sampling configuration (always pick highest probability)
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_p: None,
            top_k: None,
            repeat_penalty: 1.0,
            repeat_last_n: 0,
        }
    }

    /// Conservative sampling (low temperature, focused)
    pub fn conservative() -> Self {
        Self {
            temperature: 0.5,
            top_p: Some(0.85),
            top_k: Some(20),
            repeat_penalty: 1.15,
            repeat_last_n: 64,
        }
    }

    /// Creative sampling (high temperature, diverse)
    pub fn creative() -> Self {
        Self {
            temperature: 0.9,
            top_p: Some(0.95),
            top_k: Some(50),
            repeat_penalty: 1.05,
            repeat_last_n: 32,
        }
    }
}

/// Sample next token from logits
///
/// # Arguments
/// * `logits` - Raw logits from model (shape: `[vocab_size]`)
/// * `config` - Sampling configuration
/// * `context_tokens` - Recent tokens for repeat penalty
///
/// # Returns
/// * `Ok(u32)` - Sampled token ID
/// * `Err(anyhow::Error)` - Sampling error
pub fn sample_token(
    logits: &Tensor,
    config: &SamplingConfig,
    context_tokens: &[u32],
) -> Result<u32> {
    trace!("Sampling token with config: {:?}", config);

    // Extract logits as Vec<f32>
    let logits_vec = logits.to_vec1::<f32>()?;
    let vocab_size = logits_vec.len();

    debug!("Sampling from vocab_size={}", vocab_size);

    // Apply repeat penalty
    let mut logits_vec = if config.repeat_penalty != 1.0 && config.repeat_last_n > 0 {
        apply_repeat_penalty(
            &logits_vec,
            context_tokens,
            config.repeat_penalty,
            config.repeat_last_n,
        )
    } else {
        logits_vec
    };

    // Apply temperature
    if config.temperature > 0.0 {
        for logit in &mut logits_vec {
            *logit /= config.temperature as f32;
        }
    } else {
        // Greedy: pick argmax
        let token_id = argmax(&logits_vec)?;
        debug!("Greedy sampling selected token: {}", token_id);
        return Ok(token_id as u32);
    }

    // Convert logits to probabilities (softmax)
    let probs = softmax(&logits_vec);

    // Apply top-k filtering
    let probs = if let Some(top_k) = config.top_k {
        apply_top_k(&probs, top_k)
    } else {
        probs
    };

    // Apply top-p (nucleus) filtering
    let probs = if let Some(top_p) = config.top_p {
        apply_top_p(&probs, top_p)
    } else {
        probs
    };

    // Sample from filtered distribution
    let token_id = sample_from_probs(&probs)?;

    trace!("Sampled token: {}", token_id);
    Ok(token_id as u32)
}

/// Apply softmax to logits
fn softmax(logits: &[f32]) -> Vec<f32> {
    let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let exp_logits: Vec<f32> = logits.iter().map(|&x| (x - max_logit).exp()).collect();
    let sum_exp: f32 = exp_logits.iter().sum();

    exp_logits.iter().map(|&x| x / sum_exp).collect()
}

/// Apply repeat penalty to logits
fn apply_repeat_penalty(
    logits: &[f32],
    context_tokens: &[u32],
    penalty: f64,
    last_n: usize,
) -> Vec<f32> {
    let mut logits = logits.to_vec();

    // Get last N tokens
    let start = context_tokens.len().saturating_sub(last_n);
    let recent_tokens = &context_tokens[start..];

    // Apply penalty to each token in context
    for &token in recent_tokens {
        let idx = token as usize;
        if idx < logits.len() {
            if logits[idx] >= 0.0 {
                logits[idx] /= penalty as f32;
            } else {
                logits[idx] *= penalty as f32;
            }
        }
    }

    logits
}

/// Apply top-k filtering (keep only top-k highest probability tokens)
fn apply_top_k(probs: &[f32], k: usize) -> Vec<f32> {
    let mut indexed_probs: Vec<(usize, f32)> = probs.iter().copied().enumerate().collect();

    // Sort by probability descending
    indexed_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Zero out probabilities outside top-k
    let mut filtered = vec![0.0; probs.len()];
    for (idx, prob) in indexed_probs.iter().take(k) {
        filtered[*idx] = *prob;
    }

    // Renormalize
    let sum: f32 = filtered.iter().sum();
    if sum > 0.0 {
        filtered.iter().map(|&p| p / sum).collect()
    } else {
        filtered
    }
}

/// Apply top-p (nucleus) filtering (keep tokens until cumulative probability >= p)
fn apply_top_p(probs: &[f32], p: f64) -> Vec<f32> {
    let mut indexed_probs: Vec<(usize, f32)> = probs.iter().copied().enumerate().collect();

    // Sort by probability descending
    indexed_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Find nucleus (tokens whose cumulative probability <= p)
    let mut cumsum = 0.0f32;
    let mut cutoff = indexed_probs.len();

    for (i, (_, prob)) in indexed_probs.iter().enumerate() {
        cumsum += prob;
        if cumsum >= p as f32 {
            cutoff = i + 1;
            break;
        }
    }

    // Zero out probabilities outside nucleus
    let mut filtered = vec![0.0; probs.len()];
    for (idx, prob) in indexed_probs.iter().take(cutoff) {
        filtered[*idx] = *prob;
    }

    // Renormalize
    let sum: f32 = filtered.iter().sum();
    if sum > 0.0 {
        filtered.iter().map(|&p| p / sum).collect()
    } else {
        filtered
    }
}

/// Sample token ID from probability distribution
fn sample_from_probs(probs: &[f32]) -> Result<usize> {
    // Simple multinomial sampling using random number
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let r: f32 = rng.gen();

    let mut cumsum = 0.0f32;
    for (idx, &prob) in probs.iter().enumerate() {
        cumsum += prob;
        if r < cumsum {
            return Ok(idx);
        }
    }

    // Fallback: return highest probability token
    argmax(probs)
}

/// Find index of maximum value
fn argmax(values: &[f32]) -> Result<usize> {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .ok_or_else(|| anyhow!("Empty logits vector"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_softmax() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax(&logits);

        // Check sum to 1.0
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);

        // Check monotonic (higher logit = higher prob)
        assert!(probs[2] > probs[1]);
        assert!(probs[1] > probs[0]);
    }

    #[test]
    fn test_argmax() {
        let values = vec![0.1, 0.5, 0.3, 0.8, 0.2];
        let idx = argmax(&values).unwrap();
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_apply_top_k() {
        let probs = vec![0.1, 0.2, 0.3, 0.25, 0.15];
        let filtered = apply_top_k(&probs, 2);

        // Only top 2 should be non-zero
        let non_zero_count = filtered.iter().filter(|&&p| p > 0.0).count();
        assert_eq!(non_zero_count, 2);
    }

    #[test]
    fn test_apply_top_p() {
        let probs = vec![0.4, 0.3, 0.2, 0.1];
        let filtered = apply_top_p(&probs, 0.75);

        // Should keep tokens until cumsum >= 0.75
        // In this case: 0.4 + 0.3 = 0.7, next would be 0.9, so keep first 3
        let non_zero_count = filtered.iter().filter(|&&p| p > 0.0).count();
        assert!(non_zero_count <= 3);
    }

    #[test]
    fn test_sampling_config_presets() {
        let greedy = SamplingConfig::greedy();
        assert_eq!(greedy.temperature, 0.0);

        let conservative = SamplingConfig::conservative();
        assert!(conservative.temperature < 0.7);

        let creative = SamplingConfig::creative();
        assert!(creative.temperature > 0.7);
    }
}
