//! LLM session management stubs

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::schema::{AdapterConfigV1, CompletionChunkV1, PromptV1};

/// LLM session
#[derive(Debug, Clone)]
pub struct LlmSession {
    pub id: u64,
    pub backend: u8,
    pub active: bool,
    pub ring: CompletionRing,
    pub request_count: u64,
    pub created_at: u64,
    pub last_activity: u64,
}

impl LlmSession {
    pub fn new(id: u64, config: AdapterConfigV1) -> Result<Self, &'static str> {
        Ok(Self {
            id,
            backend: config.backend,
            active: true,
            ring: CompletionRing::new(),
            request_count: 0,
            created_at: 0,
            last_activity: 0,
        })
    }

    pub fn send_prompt(&mut self, prompt: &PromptV1) -> Result<(), &'static str> {
        self.request_count = self.request_count.saturating_add(1);
        self.last_activity = self.last_activity.saturating_add(1);

        let text = prompt
            .messages
            .last()
            .map(|message| message.content.clone())
            .unwrap_or_else(|| String::from(""));

        self.ring.push(CompletionChunkV1 {
            text: format!("echo: {}", text),
            is_final: true,
            seq: self.request_count,
            token: Some(text),
            tool: None,
            finish: Some(true),
        });

        Ok(())
    }

    pub fn receive_chunks(&mut self, max_chunks: usize) -> Vec<CompletionChunkV1> {
        self.ring.get_all().into_iter().take(max_chunks).collect()
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn get_info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id,
            backend: format!("{}", self.backend),
            active: self.active,
            request_count: self.request_count,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompletionRing {
    chunks: Vec<CompletionChunkV1>,
}

impl CompletionRing {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn push(&mut self, chunk: CompletionChunkV1) {
        self.chunks.push(chunk);
    }

    pub fn get_all(&self) -> Vec<CompletionChunkV1> {
        self.chunks.clone()
    }
}

/// Session info
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: u64,
    pub backend: String,
    pub active: bool,
    pub request_count: u64,
}

/// LLM statistics
#[derive(Debug, Clone, Default)]
pub struct LlmStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub sessions_opened: u64,
    pub sessions_closed: u64,
}
