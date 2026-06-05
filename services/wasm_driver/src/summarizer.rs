//! Deterministic log summarizer host adapter.
//!
//! The primary path can be extended to call a Component Model summarizer export
//! once a signed component is available. Until then, this module keeps the
//! wasm-driver boundary explicit and uses a deterministic local fallback, so
//! AI Core can exercise the same host-facing path without fake telemetry.

use std::path::PathBuf;

use crate::host::Host;

#[derive(Debug, Clone)]
pub struct LogSummaryRequest {
    pub source_path: PathBuf,
    pub text: String,
    pub component_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct LogSummary {
    pub source_path: PathBuf,
    pub mode: String,
    pub line_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub top_terms: Vec<String>,
    pub summary: String,
}

wasmtime::component::bindgen!({
    inline: r#"
        package polymera:ai;

        world summarizer {
            export summarize: func(text: string) -> string;
        }
    "#,
    async: true,
});

#[derive(Debug, Default)]
pub struct SummarizerHost;

impl SummarizerHost {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to load the compiled wasm component from the well-known path.
    /// Returns `Some(path)` if a component binary was found, `None` otherwise.
    fn try_load_component() -> Option<PathBuf> {
        // Well-known paths where the compiled component might live.
        let candidates = [
            PathBuf::from("services/wasm_components/log_summarizer/target/wasm32-wasi/release/log_summarizer_component.wasm"),
            PathBuf::from("services/wasm_components/log_summarizer/target/wasm32-wasip1/release/log_summarizer_component.wasm"),
            PathBuf::from("services/wasm_components/log_summarizer/target/wasm32-unknown-unknown/release/log_summarizer_component.wasm"),
            PathBuf::from("dist/drivers/log_summarizer.wasm"),
        ];
        for candidate in &candidates {
            if candidate.exists() {
                eprintln!("[summarizer] found wasm component: {}", candidate.display());
                return Some(candidate.clone());
            }
        }
        None
    }

    async fn execute_wasm_summarizer(&self, path: &std::path::Path, text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let host = Host::new()?;
        let component = host.load_component(path)?;
        let (mut store, instance) = host.instantiate(&component).await?;
        let bindings = Summarizer::new(&mut store, &instance)?;
        let res = bindings.call_summarize(&mut store, text).await?;
        Ok(res)
    }

    pub async fn summarize(&self, request: LogSummaryRequest) -> LogSummary {
        // Phase 1A.2: Try the real wasm component first.
        let component_path = request
            .component_path
            .clone()
            .or_else(Self::try_load_component);

        if let Some(path) = component_path {
            match self.execute_wasm_summarizer(&path, &request.text).await {
                Ok(wasm_summary) => {
                    eprintln!("[summarizer] running via wasm component");
                    let mut summary = deterministic_summary(&request.text, request.source_path.clone());
                    summary.summary = wasm_summary;
                    summary.mode = "wasm-component".to_string();
                    return summary;
                }
                Err(e) => {
                    eprintln!("[summarizer] failed to execute wasm component: {e}. Falling back to deterministic fallback.");
                }
            }
        }

        eprintln!("[summarizer] running via deterministic fallback");
        let mut summary = deterministic_summary(&request.text, request.source_path.clone());
        summary.mode = "deterministic-local-fallback".to_string();
        summary
    }
}

fn deterministic_summary(text: &str, source_path: PathBuf) -> LogSummary {
    let mut line_count = 0usize;
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    let mut info_count = 0usize;
    let mut term_counts = std::collections::BTreeMap::<String, usize>::new();

    for line in text.lines() {
        line_count += 1;
        let lower = line.to_ascii_lowercase();
        if lower.contains("error") || lower.contains("[err") {
            error_count += 1;
        }
        if lower.contains("warn") {
            warning_count += 1;
        }
        if lower.contains("info") {
            info_count += 1;
        }
        for term in lower
            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .filter(|term| term.len() >= 4)
        {
            *term_counts.entry(term.to_string()).or_insert(0) += 1;
        }
    }

    let mut ranked_terms: Vec<(String, usize)> = term_counts.into_iter().collect();
    ranked_terms.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_terms = ranked_terms
        .into_iter()
        .take(6)
        .map(|(term, _)| term)
        .collect::<Vec<_>>();

    let summary = format!(
        "{} lines; {} errors; {} warnings; {} info entries; top terms: {}",
        line_count,
        error_count,
        warning_count,
        info_count,
        if top_terms.is_empty() {
            "none".to_string()
        } else {
            top_terms.join(", ")
        }
    );

    LogSummary {
        source_path,
        mode: "deterministic-local-fallback".to_string(),
        line_count,
        error_count,
        warning_count,
        info_count,
        top_terms,
        summary,
    }
}
