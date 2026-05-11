//! Learning (LTM) — Append-only NDJSON memory and deterministic summaries

use serde::{Deserialize, Serialize};
use crate::error::{AiError, AiResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearnedState {
    pub backup_drive_unavailable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub ts90k: u64,
    pub goal: String,
    pub goal_hash: String,
    pub outcome_code: String,
    #[serde(skip_serializing_if="Option::is_none")] pub notes: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub facts: Option<serde_json::Value>,
}

impl LearningEvent {
    pub fn new(goal: &str, outcome_code: &str, notes: Option<String>, facts: Option<serde_json::Value>) -> Self {
        let ts90k = crate::time::get_time_90khz();
        let goal_hash = blake3::hash(goal.as_bytes()).to_hex().to_string();
        Self { ts90k, goal: goal.to_string(), goal_hash, outcome_code: outcome_code.to_string(), notes, facts }
    }
}

/// Append one event line to NDJSON file. Creates parent dirs if needed.
pub async fn append_ltm_event(path: &str, ev: &LearningEvent) -> AiResult<()> {
    use tokio::io::AsyncWriteExt;
    use tokio::fs::{OpenOptions, create_dir_all};
    use std::path::Path;
    let p = Path::new(path);
    if let Some(parent) = p.parent() { create_dir_all(parent).await.ok(); }
    let mut f = OpenOptions::new().create(true).append(true).open(p).await
        .map_err(|e| AiError::io(e.to_string()))?;
    let line = serde_json::to_string(ev).map_err(|e| AiError::serialization(e.to_string()))?;
    f.write_all(line.as_bytes()).await.map_err(|e| AiError::io(e.to_string()))?;
    f.write_all(b"\n").await.map_err(|e| AiError::io(e.to_string()))?;
    f.flush().await.map_err(|e| AiError::io(e.to_string()))
}

/// Deterministically summarize NDJSON events into a LearnedState.
pub async fn summarize_ltm(path: &str) -> AiResult<LearnedState> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::fs::File;
    let file = match File::open(path).await { Ok(f) => f, Err(_) => return Ok(LearnedState::default()) };
    let mut rd = BufReader::new(file);
    let mut buf = String::new();
    let mut backup_unavail = false;
    loop {
        buf.clear();
        let n = rd.read_line(&mut buf).await.map_err(|e| AiError::io(e.to_string()))?;
        if n == 0 { break; }
        if let Ok(ev) = serde_json::from_str::<LearningEvent>(buf.trim_end()) {
            if ev.outcome_code == "backup_drive_unavailable" { backup_unavail = true; }
        }
    }
    Ok(LearnedState { backup_drive_unavailable: backup_unavail })
}
