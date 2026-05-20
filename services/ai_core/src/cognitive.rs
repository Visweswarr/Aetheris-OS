//! AI Core bridge to the deterministic cognitive planner in `services/ai`.
//!
//! This module intentionally depends on `aetheris-ai` instead of duplicating
//! its Phase 5 cognitive planner. Long-term memory is read once at session
//! construction so learning changes affect the next initialization, not the
//! current planning run.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::agent::{TaskPlan, TaskStep};
use crate::contracts::sha256_hex;
use crate::error::{AiCoreError, Result};

pub use aetheris_ai::learning::{
    append_ltm_event, summarize_ltm, LearnedState, LearningEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSessionConfig {
    pub ltm_path: PathBuf,
}

impl Default for CognitiveSessionConfig {
    fn default() -> Self {
        Self {
            ltm_path: PathBuf::from("data/aicore_ltm.ndjson"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveStepProjection {
    pub step_id: String,
    pub name: String,
    pub description: String,
    pub group: usize,
    pub order_in_group: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitivePlanProjection {
    pub plan_id: String,
    pub goal: String,
    pub learned_backup_drive_unavailable: bool,
    pub steps: Vec<CognitiveStepProjection>,
    pub replay_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitivePlanResult {
    pub plan: TaskPlan,
    pub projection: CognitivePlanProjection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayProof {
    pub identical: bool,
    pub first_hash: String,
    pub second_hash: String,
    pub plan: CognitivePlanProjection,
}

pub struct CognitiveSession {
    core: aetheris_ai::aicore::CognitiveCore,
    learned: LearnedState,
}

impl CognitiveSession {
    pub async fn new(config: CognitiveSessionConfig) -> Result<Self> {
        let ltm_path = config.ltm_path.to_string_lossy().to_string();
        let mut core_config = aetheris_ai::aicore::CognitiveCoreConfig::default();
        core_config.ltm_path = ltm_path.clone();
        let core = aetheris_ai::aicore::CognitiveCore::new(core_config)
            .await
            .map_err(ai_error)?;
        let learned = summarize_ltm(&ltm_path).await.map_err(ai_error)?;
        Ok(Self { core, learned })
    }

    pub fn plan_goal(&self, goal: &str) -> Result<CognitivePlanResult> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(AiCoreError::InvalidInput(
                "goal-plan requires a non-empty goal".to_string(),
            ));
        }

        let mut steps = self.core.decompose_goal(goal);
        if self.learned.backup_drive_unavailable && goal_mentions_backup(goal) {
            let has_preflight = steps
                .iter()
                .any(|step| step.name.contains("check backup drive availability"));
            if !has_preflight {
                steps.insert(
                    0,
                    aetheris_ai::aicore::DecomposedStep::seq(
                        0,
                        0,
                        "check backup drive availability",
                        "probe mount or path before any backup copy",
                    ),
                );
            }
        }

        for (idx, step) in steps.iter_mut().enumerate() {
            step.order_in_group = idx;
        }

        let plan_id = deterministic_plan_id(goal, &steps, &self.learned);
        let mut task_steps = Vec::with_capacity(steps.len());
        let mut projection_steps = Vec::with_capacity(steps.len());

        for (idx, step) in steps.iter().enumerate() {
            let step_id = format!("step-{:03}", idx + 1);
            projection_steps.push(CognitiveStepProjection {
                step_id: step_id.clone(),
                name: step.name.clone(),
                description: step.description.clone(),
                group: step.group,
                order_in_group: step.order_in_group,
            });

            task_steps.push(TaskStep {
                step_id: step_id.clone(),
                description: step.name.clone(),
                tool_name: "echo".to_string(),
                parameters: serde_json::json!({
                    "message": step.description,
                    "cognitive_step": step.name,
                    "group": step.group,
                    "order_in_group": step.order_in_group,
                    "requires_remote": false,
                }),
                required_capabilities: vec!["ai.plan.generate".to_string()],
                requires_approval: false,
                is_destructive: false,
                depends_on: if idx == 0 {
                    Vec::new()
                } else {
                    vec![format!("step-{:03}", idx)]
                },
                estimated_time_secs: 2,
            });
        }

        let replay_hash = projection_hash(goal, &projection_steps, &self.learned)?;
        let projection = CognitivePlanProjection {
            plan_id: plan_id.clone(),
            goal: goal.to_string(),
            learned_backup_drive_unavailable: self.learned.backup_drive_unavailable,
            steps: projection_steps,
            replay_hash,
        };

        let plan = TaskPlan {
            plan_id,
            original_intent: goal.to_string(),
            steps: task_steps,
            total_estimated_time_secs: projection.steps.len() as u64 * 2,
            metadata: HashMap::new(),
            created_at: "deterministic-cognitive-v1".to_string(),
        };

        Ok(CognitivePlanResult { plan, projection })
    }
}

pub async fn generate_cognitive_goal_plan(
    goal: &str,
    ltm_path: impl AsRef<Path>,
) -> Result<CognitivePlanResult> {
    let session = CognitiveSession::new(CognitiveSessionConfig {
        ltm_path: ltm_path.as_ref().to_path_buf(),
    })
    .await?;
    session.plan_goal(goal)
}

pub async fn replay_goal(goal: &str, ltm_path: impl AsRef<Path>) -> Result<ReplayProof> {
    let first = generate_cognitive_goal_plan(goal, ltm_path.as_ref()).await?;
    let second = generate_cognitive_goal_plan(goal, ltm_path.as_ref()).await?;
    Ok(ReplayProof {
        identical: first.projection == second.projection,
        first_hash: first.projection.replay_hash.clone(),
        second_hash: second.projection.replay_hash.clone(),
        plan: first.projection,
    })
}

pub async fn record_learning_event(
    goal: &str,
    outcome_code: &str,
    notes: Option<String>,
    ltm_path: impl AsRef<Path>,
) -> Result<LearningEvent> {
    let event = LearningEvent::new(goal, outcome_code, notes, None);
    append_ltm_event(&ltm_path.as_ref().to_string_lossy(), &event)
        .await
        .map_err(ai_error)?;
    crate::metrics::record_ltm_event();
    Ok(event)
}

fn projection_hash(
    goal: &str,
    steps: &[CognitiveStepProjection],
    learned: &LearnedState,
) -> Result<String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "goal": goal,
        "learned_backup_drive_unavailable": learned.backup_drive_unavailable,
        "steps": steps,
    }))
    .map_err(|error| AiCoreError::SerializationError(error.to_string()))?;
    Ok(sha256_hex(&bytes))
}

fn deterministic_plan_id(
    goal: &str,
    steps: &[aetheris_ai::aicore::DecomposedStep],
    learned: &LearnedState,
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(goal.as_bytes());
    bytes.push(u8::from(learned.backup_drive_unavailable));
    for step in steps {
        bytes.extend_from_slice(step.name.as_bytes());
        bytes.extend_from_slice(step.description.as_bytes());
    }
    let hash = sha256_hex(&bytes);
    format!("cognitive-{}", &hash[..16])
}

fn goal_mentions_backup(goal: &str) -> bool {
    let goal = goal.to_lowercase();
    (goal.contains("backup") || goal.contains("back up"))
        && (goal.contains("photo") || goal.contains("image") || goal.contains("file"))
}

fn ai_error(error: aetheris_ai::error::AiError) -> AiCoreError {
    AiCoreError::ExternalServiceError(format!("aetheris-ai cognitive core: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_goal_generates_byte_identical_projection() {
        let dir = tempfile::tempdir().unwrap();
        let ltm = dir.path().join("ltm.ndjson");
        let first = generate_cognitive_goal_plan("organize my files and back up photos", &ltm)
            .await
            .unwrap();
        let second = generate_cognitive_goal_plan("organize my files and back up photos", &ltm)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&first.projection).unwrap(),
            serde_json::to_vec(&second.projection).unwrap()
        );
    }

    #[tokio::test]
    async fn ltm_event_affects_next_initialization_only() {
        let dir = tempfile::tempdir().unwrap();
        let ltm = dir.path().join("ltm.ndjson");
        let session = CognitiveSession::new(CognitiveSessionConfig {
            ltm_path: ltm.clone(),
        })
        .await
        .unwrap();
        let before = session.plan_goal("back up photos").unwrap();

        record_learning_event(
            "back up photos",
            "backup_drive_unavailable",
            Some("drive offline".to_string()),
            &ltm,
        )
        .await
        .unwrap();

        let same_session = session.plan_goal("back up photos").unwrap();
        assert_eq!(before.projection, same_session.projection);

        let next = generate_cognitive_goal_plan("back up photos", &ltm)
            .await
            .unwrap();
        assert!(next
            .projection
            .steps
            .iter()
            .any(|step| step.name.contains("check backup drive availability")));
    }
}
