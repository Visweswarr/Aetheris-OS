//! Deterministic browser/page assistance tools.
//!
//! Source: `reference-os/services/ai/src/browser_assist.rs` in this
//! repository. The source is repo-owned and MIT-compatible with Polymera.
//! The implementation is intentionally local and heuristic-only: no network
//! fetch, no model call, no remote telemetry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryResponse {
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifyResponse {
    pub suspicious: bool,
    pub reasons: Vec<String>,
}

/// Deterministic summarization: pick the first three sentences within a
/// character cap. This is deliberately simple so replay output is byte-stable.
pub fn summarize_text(text: &str, max_chars: usize) -> SummaryResponse {
    let cap = max_chars.max(1);
    let sentences: Vec<&str> = text
        .split(|c| c == '.' || c == '!' || c == '?')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let mut out = String::new();
    for sentence in sentences.into_iter().take(3) {
        if !out.is_empty() {
            out.push_str(". ");
        }
        out.push_str(sentence);
        if out.len() >= cap {
            break;
        }
    }

    if out.is_empty() {
        out = text.trim().chars().take(cap).collect();
    }
    if !out.is_empty() && !out.ends_with('.') {
        out.push('.');
    }

    SummaryResponse {
        summary: out.chars().take(cap).collect(),
    }
}

/// Deterministic classification: suspicious URL/domain patterns or local text
/// heuristics imply a suspicious page. This does not contact the URL.
pub fn classify_page(url: &str, text: &str) -> ClassifyResponse {
    let mut reasons = Vec::new();
    let url_lower = url.to_lowercase();
    let text_lower = text.to_lowercase();

    for domain in ["evil.example.com", "phish.local"] {
        if url_lower.contains(domain) {
            reasons.push(format!("blacklisted-domain:{domain}"));
        }
    }

    for marker in [
        "seed phrase",
        "private key",
        "urgent login",
        "verify account",
        "wallet recovery",
        "password reset required",
    ] {
        if text_lower.contains(marker) {
            reasons.push(format!("suspicious-text:{marker}"));
        }
    }

    ClassifyResponse {
        suspicious: !reasons.is_empty(),
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_text_is_deterministic_and_local() {
        let input = "First sentence. Second sentence! Third sentence? Fourth sentence.";
        let a = summarize_text(input, 80);
        let b = summarize_text(input, 80);
        assert_eq!(a, b);
        assert_eq!(a.summary, "First sentence. Second sentence. Third sentence.");
    }

    #[test]
    fn classify_page_flags_known_suspicious_patterns() {
        let result = classify_page(
            "https://evil.example.com/login",
            "Urgent login required to verify account",
        );
        assert!(result.suspicious);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.starts_with("blacklisted-domain:")));
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.starts_with("suspicious-text:")));
    }
}
