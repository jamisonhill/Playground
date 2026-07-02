// Model price table and context-window limits (SPEC §8: "keep in one config file").
//
// Used when Tier B (OTLP) is absent: cost is *estimated* from token counts ×
// these prices. Prices are USD per million tokens, sourced from the Claude
// platform docs (cached 2026-06). Cache reads bill at ~0.1× input; 5-minute
// cache writes at 1.25× input. Update here when Anthropic changes pricing.

/// USD per 1,000,000 tokens for one model family.
#[derive(Clone, Copy)]
pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

const fn pricing(input: f64, output: f64) -> ModelPricing {
    ModelPricing {
        input,
        output,
        cache_read: input * 0.1,
        cache_write: input * 1.25,
    }
}

const FABLE: ModelPricing = pricing(10.0, 50.0);
const OPUS: ModelPricing = pricing(5.0, 25.0);
const SONNET: ModelPricing = pricing(3.0, 15.0);
const HAIKU: ModelPricing = pricing(1.0, 5.0);

/// Match on the model-id substring so date-suffixed ids ("claude-haiku-4-5-20251001")
/// and future minor versions resolve without a table update. Unknown models fall
/// back to Opus pricing — over-estimating beats silently under-reporting cost.
pub fn pricing_for(model_id: &str) -> ModelPricing {
    if model_id.contains("fable") || model_id.contains("mythos") {
        FABLE
    } else if model_id.contains("sonnet") {
        SONNET
    } else if model_id.contains("haiku") {
        HAIKU
    } else {
        OPUS
    }
}

/// Context-window size in tokens, for the fuel gauge fallback
/// (statusline `used_percentage` is unavailable without hooks — SPEC §2 A4).
pub fn context_limit_for(model_id: &str) -> u64 {
    if model_id.contains("haiku") {
        200_000
    } else {
        // Fable 5, Opus 4.6+, Sonnet 4.6+ are all 1M-context models.
        1_000_000
    }
}

/// Estimated USD cost of one usage delta on the given model.
pub fn estimate_cost_usd(
    model_id: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
) -> f64 {
    let p = pricing_for(model_id);
    (input_tokens as f64 * p.input
        + output_tokens as f64 * p.output
        + cache_read_tokens as f64 * p.cache_read
        + cache_creation_tokens as f64 * p.cache_write)
        / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_model_families() {
        assert_eq!(pricing_for("claude-fable-5").input, 10.0);
        assert_eq!(pricing_for("claude-opus-4-8").output, 25.0);
        assert_eq!(pricing_for("claude-sonnet-5").input, 3.0);
        assert_eq!(pricing_for("claude-haiku-4-5-20251001").input, 1.0);
        assert_eq!(pricing_for("totally-unknown").input, 5.0); // Opus fallback
    }

    #[test]
    fn context_limits() {
        assert_eq!(context_limit_for("claude-haiku-4-5"), 200_000);
        assert_eq!(context_limit_for("claude-fable-5"), 1_000_000);
    }

    #[test]
    fn cost_estimate_matches_hand_calculation() {
        // 1000 output tokens on Opus = 1000/1M × $25 = $0.025
        let cost = estimate_cost_usd("claude-opus-4-8", 0, 1000, 0, 0);
        assert!((cost - 0.025).abs() < 1e-9);
    }
}
