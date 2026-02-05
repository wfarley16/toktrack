//! Model name normalization service
//!
//! Normalizes model names to a canonical form for consistent pricing lookup
//! and aggregation across different data sources.

/// Normalize a model name to canonical form.
///
/// Transformations:
/// - Dots to hyphens: "claude-opus-4.5" → "claude-opus-4-5"
/// - Remove date suffix: "claude-opus-4-5-20251101" → "claude-opus-4-5"
///
/// # Examples
/// ```
/// use toktrack::services::normalizer::normalize_model_name;
///
/// assert_eq!(normalize_model_name("claude-opus-4-5-20251101"), "claude-opus-4-5");
/// assert_eq!(normalize_model_name("claude-opus-4.5"), "claude-opus-4-5");
/// ```
pub fn normalize_model_name(model: &str) -> String {
    // Step 1: Replace dots with hyphens
    let normalized = model.replace('.', "-");

    // Step 2: Remove 8-digit date suffix at end (e.g., -20251101)
    // Pattern: ends with -YYYYMMDD where YYYYMMDD is 8 digits starting with 20
    if let Some(suffix_start) = normalized.rfind('-') {
        let suffix = &normalized[suffix_start + 1..];
        if suffix.len() == 8
            && suffix.starts_with("20")
            && suffix.chars().all(|c| c.is_ascii_digit())
        {
            return normalized[..suffix_start].to_string();
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Dot to hyphen conversion ==========

    #[test]
    fn test_dot_to_hyphen_single() {
        assert_eq!(normalize_model_name("claude-opus-4.5"), "claude-opus-4-5");
    }

    #[test]
    fn test_dot_to_hyphen_multiple() {
        assert_eq!(normalize_model_name("model-1.2.3"), "model-1-2-3");
    }

    // ========== Date suffix removal ==========

    #[test]
    fn test_remove_date_suffix_claude_opus() {
        assert_eq!(
            normalize_model_name("claude-opus-4-5-20251101"),
            "claude-opus-4-5"
        );
    }

    #[test]
    fn test_remove_date_suffix_claude_sonnet() {
        assert_eq!(
            normalize_model_name("claude-sonnet-4-20250514"),
            "claude-sonnet-4"
        );
    }

    #[test]
    fn test_remove_date_suffix_with_dot_and_date() {
        // Combined: dot + date
        assert_eq!(
            normalize_model_name("claude-opus-4.5-20251101"),
            "claude-opus-4-5"
        );
    }

    // ========== No-op cases ==========

    #[test]
    fn test_already_normalized() {
        assert_eq!(normalize_model_name("claude-opus-4-5"), "claude-opus-4-5");
    }

    #[test]
    fn test_no_date_suffix() {
        assert_eq!(normalize_model_name("gpt-4o"), "gpt-4o");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(normalize_model_name(""), "");
    }

    #[test]
    fn test_unknown_model() {
        assert_eq!(normalize_model_name("unknown-model"), "unknown-model");
    }

    // ========== Edge cases ==========

    #[test]
    fn test_short_date_not_removed() {
        // 8-digit number in middle shouldn't be removed
        assert_eq!(
            normalize_model_name("model-12345678-extra"),
            "model-12345678-extra"
        );
    }

    #[test]
    fn test_date_suffix_at_end_only() {
        // Date must be at end
        assert_eq!(normalize_model_name("20251101-claude"), "20251101-claude");
    }
}
