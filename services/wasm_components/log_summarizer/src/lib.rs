//! WIT-compliant wasm component for log summarization.
//!
//! Implements the `polymera:ai/summarizer` world.
//! When compiled with `cargo-component build`, this produces a `.wasm` component
//! that the `SummarizerHost` in `wasm_driver` can load and execute.
//!
//! ## WIT interface
//! ```wit
//! package polymera:ai;
//!
//! world summarizer {
//!     export summarize: func(text: string) -> string;
//! }
//! ```

wit_bindgen::generate!({
    inline: r#"
        package polymera:ai;

        world summarizer {
            export summarize: func(text: string) -> string;
        }
    "#,
});

struct GuestImpl;

impl Guest for GuestImpl {
    fn summarize(text: String) -> String {
        summarize(&text)
    }
}

export!(GuestImpl);

use std::collections::HashMap;

/// Summarize the given text using a deterministic extractive approach.
/// This matches the existing `SummarizerHost::summarize()` logic in Rust,
/// ensuring that the wasm component produces the same output as the fallback.
pub fn summarize(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let line_count = lines.len();

    // Count severity levels
    let mut error_count = 0u64;
    let mut warn_count = 0u64;
    let mut info_count = 0u64;

    for line in &lines {
        let lower = line.to_lowercase();
        if lower.contains("error") || lower.contains("err ") || lower.contains("fatal") {
            error_count += 1;
        } else if lower.contains("warn") {
            warn_count += 1;
        } else if lower.contains("info") || lower.contains("debug") || lower.contains("trace") {
            info_count += 1;
        }
    }

    // Extract top terms (simple word frequency)
    let mut freq: HashMap<String, u64> = HashMap::new();
    let stop_words = [
        "the", "a", "an", "is", "was", "are", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "shall",
        "should", "may", "might", "can", "could", "at", "in", "on", "to",
        "for", "of", "with", "by", "from", "as", "into", "through", "during",
        "and", "but", "or", "nor", "not", "so", "yet", "both", "either",
        "neither", "each", "every", "all", "any", "few", "more", "most",
        "other", "some", "such", "no", "only", "own", "same", "than", "too",
        "very", "just", "about", "above", "after", "again", "also", "that",
        "this", "these", "those", "it", "its", "he", "she", "they", "we",
        "you", "i", "me", "my", "your", "his", "her", "our", "their",
    ];
    for line in &lines {
        for word in line.split_whitespace() {
            let w = word.to_lowercase();
            let w = w.trim_matches(|c: char| !c.is_alphanumeric());
            if w.len() >= 3 && !stop_words.contains(&w) {
                *freq.entry(w.to_string()).or_default() += 1;
            }
        }
    }
    let mut top: Vec<_> = freq.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));
    let top_terms: Vec<String> = top.iter().take(5).map(|(w, _)| w.clone()).collect();

    // Build summary
    let summary = format!(
        "{} lines: {} errors, {} warnings, {} info. Top terms: {}",
        line_count,
        error_count,
        warn_count,
        info_count,
        top_terms.join(", ")
    );

    summary
}

// When compiled as a wasm component, this would be the export.
// For now, we provide a native test harness.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_log() {
        let log = "\
INFO  2024-01-01 service started
WARN  2024-01-01 high latency detected
ERROR 2024-01-01 connection refused
INFO  2024-01-01 retry succeeded
ERROR 2024-01-02 timeout expired";

        let result = summarize(log);
        assert!(result.contains("5 lines"));
        assert!(result.contains("2 errors"));
        assert!(result.contains("1 warnings"));
        assert!(result.contains("2 info"));
    }
}
