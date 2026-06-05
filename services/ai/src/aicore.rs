//! Cognitive Core (Phase 5) - Reasoning, Memory, Planning, and Intent Orchestration
//! 
//! This module provides a deterministic Cognitive Core for Aetheris OS.
//! It composes:
//! - Reasoning Kernel: modular reasoning graph with pluggable nodes
//! - Working Memory: bounded, deterministic short-term store
//! - Long-Term Memory: NGFS-anchored knowledge interface (file-backed stub)
//! - Planner Bridge: orchestrates actions through Intent Kernel v0 via an Intent Bus adapter
//! - Meta-learning Hooks: metrics emitters for adaptive strategy selection (deterministic updates)
//!
//! Determinism:
//! - All collections that affect ordering use BTreeMap or sorted Vec
//! - Randomness is seeded via config.random_seed
//! - State transitions are snapshot-friendly via CBOR encoding

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{AiError, AiResult};
use crate::planner::{Plan, PlanRequest, PlanResult, StepPriority, TaskPlanner, PlannerConfig, PlanStep, ExecutionStrategy, PlanStatus};
use crate::plan_store::PlanStore;
use crate::policy::AiPolicy;
use crate::intent_client::{IntentClient, IntentResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveCoreConfig {
    pub working_mem_capacity: usize,
    pub random_seed: u64,
    pub enable_determinism: bool,
    pub ltm_path: String, // NGFS-anchored path or file fallback
    pub planner: PlannerConfig,
}

#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[tokio::test]
    async fn decompose_goal_is_deterministic() {
        let cfg = CognitiveCoreConfig::default();
        let core = CognitiveCore::new(cfg).await.unwrap();
        let goal = "organize my files and back up photos";
        let a = core.decompose_goal(goal);
        let b = core.decompose_goal(goal);
        assert_eq!(a, b, "Decomposition should be deterministic");
        assert!(!a.is_empty());
        // Ensure expected canonical steps appear
        let names: Vec<String> = a.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"scan file directories".to_string()));
        assert!(names.contains(&"identify photos".to_string()));
        assert!(names.contains(&"copy to backup drive".to_string()));
    }

    #[tokio::test]
    async fn plan_from_goal_builds_sequential_dependencies() {
        let cfg = CognitiveCoreConfig::default();
        let core = CognitiveCore::new(cfg).await.unwrap();
        let goal = "organize my files and back up photos";
        let res = core.plan_from_goal(goal).await.unwrap();
        let plan = res.plan.unwrap();
        // Expect at least 5 steps
        assert!(plan.steps.len() >= 5);
        // Check that within a group steps have dependencies in sequence
        // Find indexes of file organization steps by names
        let mut idx_scan = None;
        let mut idx_cat = None;
        for (i, s) in plan.steps.iter().enumerate() {
            if s.name.to_lowercase().contains("scan file directories") { idx_scan = Some((i, s.id.clone())); }
            if s.name.to_lowercase().contains("categorize files") { idx_cat = Some((i, s.id.clone())); }
        }
        let (scan_i, scan_id) = idx_scan.expect("scan step present");
        let (cat_i, _cat_id) = idx_cat.expect("categorize step present");
        assert!(cat_i > scan_i, "categorize should come after scan");
        let cat = plan.steps.get(cat_i).unwrap();
        assert!(cat.dependencies.contains(&scan_id));
    }

    #[tokio::test]
    async fn ltm_learning_affects_next_session() {
        // Use a temp NDJSON file under ./data for testing
        let ltm_path = "data/test_aicore_ltm.ndjson".to_string();
        let _ = std::fs::remove_file(&ltm_path);
        // Session 1
        let mut cfg1 = CognitiveCoreConfig::default();
        cfg1.ltm_path = ltm_path.clone();
        let core1 = CognitiveCore::new(cfg1).await.unwrap();
        core1.record_outcome("back up photos", "backup_drive_unavailable", Some("drive offline"), None)
            .await
            .unwrap();
        // Session 2
        let mut cfg2 = CognitiveCoreConfig::default();
        cfg2.ltm_path = ltm_path.clone();
        let core2 = CognitiveCore::new(cfg2).await.unwrap();
        let res = core2.plan_from_goal("back up photos").await.unwrap();
        let plan = res.plan.unwrap();
        let names: Vec<String> = plan.steps.iter().map(|s| s.name.to_lowercase()).collect();
        assert!(names.iter().any(|n| n.contains("check backup drive availability")));
    }
}

impl Default for CognitiveCoreConfig {
    fn default() -> Self {
        Self {
            working_mem_capacity: 256,
            random_seed: 42,
            enable_determinism: true,
            ltm_path: "data/aicore_ltm.ndjson".to_string(),
            planner: PlannerConfig::default(),
        }
    }
}

// 1) Reasoning Kernel
#[async_trait::async_trait]
pub trait ReasonerNode: Send + Sync {
    fn id(&self) -> &'static str;
    fn version(&self) -> u32 { 1 }

    // Deterministic evaluation: inputs are JSON values, output is JSON
    async fn eval(
        &self,
        input: &serde_json::Value,
        wm: &WorkingMemory,
    ) -> AiResult<serde_json::Value>;
}

/// A directed acyclic reasoning graph of nodes
#[derive(Default)]
pub struct ReasoningGraph {
    pub nodes: BTreeMap<String, Arc<dyn ReasonerNode>>,         // id -> node
    pub edges: BTreeMap<String, Vec<String>>,                    // from_id -> [to_id]
    pub entrypoints: Vec<String>,                                // deterministically ordered
}


impl ReasoningGraph {
    pub fn register_node<N: ReasonerNode + 'static>(&mut self, node: N) {
        self.nodes.insert(node.id().to_string(), Arc::new(node));
    }

    pub fn connect(&mut self, from: &str, to: &str) {
        self.edges.entry(from.to_string()).or_default();
        let mut v = self.edges.get(from).cloned().unwrap_or_default();
        if !v.iter().any(|x| x == to) { v.push(to.to_string()); v.sort(); }
        self.edges.insert(from.to_string(), v);
    }

    pub fn add_entrypoint(&mut self, id: &str) {
        if !self.entrypoints.iter().any(|e| e == id) { self.entrypoints.push(id.to_string()); self.entrypoints.sort(); }
    }

    /// Execute the graph deterministically from entrypoints
    pub async fn execute(
        &self,
        seed: u64,
        input: serde_json::Value,
        wm: &WorkingMemory,
    ) -> AiResult<BTreeMap<String, serde_json::Value>> {
        let mut outputs: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let mut frontier: Vec<String> = self.entrypoints.clone();
        // Deterministic pseudo-random: use seed only to pick tie-break order by hashing ids
        frontier.sort_by_key(|id| blake3::hash(format!("{}:{}", seed, id).as_bytes()).to_hex().to_string());

        while let Some(id) = frontier.first().cloned() {
            let _ = frontier.remove(0);
            if let Some(node) = self.nodes.get(&id) {
                let result = node.eval(&input, wm).await?;
                outputs.insert(id.clone(), result.clone());
                if let Some(nexts) = self.edges.get(&id) {
                    let mut nx = nexts.clone();
                    nx.sort();
                    for n in nx { if !frontier.contains(&n) { frontier.push(n); } }
                    frontier.sort();
                }
            }
        }
        Ok(outputs)
    }
}

// 2) Working Memory: deterministic bounded cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub ts_90khz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryState {
    pub capacity: usize,
    pub entries: Vec<WorkingEntry>, // maintained oldest->newest deterministically
}

#[derive(Clone)]
pub struct WorkingMemory {
    inner: Arc<RwLock<WorkingMemoryState>>,    // protected state
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self { inner: Arc::new(RwLock::new(WorkingMemoryState { capacity, entries: Vec::new() })) }
    }

    pub async fn put(&self, key: String, value: serde_json::Value, ts_90khz: u64) {
        let mut st = self.inner.write().await;
        // Remove any previous entry with same key
        st.entries.retain(|e| e.key != key);
        st.entries.push(WorkingEntry { key, value, ts_90khz });
        if st.entries.len() > st.capacity { st.entries.remove(0); }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let st = self.inner.read().await;
        st.entries.iter().rev().find(|e| e.key == key).map(|e| e.value.clone())
    }

    pub async fn snapshot(&self) -> WorkingMemoryState { self.inner.read().await.clone() }
    pub async fn restore(&self, state: WorkingMemoryState) { let mut st = self.inner.write().await; *st = state; }
}

// 3) Long-Term Memory: NGFS KV-backed store
use crate::long_term_memory::{LongTermMemory, NgfsKv};
use serde_cbor::value::to_value;

// 4) Intent Bus adapter (Event Fabric v0 / Intent Kernel v0)
#[async_trait::async_trait]
pub trait IntentBus: Send + Sync {
    async fn submit_goal(&self, goal: &str, caps: &[&str]) -> AiResult<String>;
    async fn publish_event(&self, topic: &str, payload: &serde_json::Value) -> AiResult<()>;
}

/// Real intent bus backed by IntentClient
pub struct RealIntentBus {
    client: IntentClient,
}

impl RealIntentBus {
    pub fn new(client: IntentClient) -> Self { Self { client } }
}

#[async_trait::async_trait]
impl IntentBus for RealIntentBus {
    async fn submit_goal(&self, goal: &str, _caps: &[&str]) -> AiResult<String> {
        let env = IntentClient::make_envelope("ai.plan.preview", serde_json::json!({"goal": goal}), None);
        let _resp: IntentResponse = self.client.preview(&env).await?;
        // Use a deterministic id based on the goal text for compatibility
        Ok(blake3::hash(goal.as_bytes()).to_hex().to_string())
    }

    async fn publish_event(&self, topic: &str, payload: &serde_json::Value) -> AiResult<()> {
        let env = IntentClient::make_envelope(&format!("event.{}", topic), payload.clone(), None);
        let _ = self.client.commit(&env).await?;
        Ok(())
    }
}

/// Stub intent bus that uses PlanStore and tracing as stand-ins
pub struct StubIntentBus {
    plan_store: Arc<PlanStore>,
}

impl StubIntentBus {
    pub fn new(plan_store: Arc<PlanStore>) -> Self { Self { plan_store } }
}

#[async_trait::async_trait]
impl IntentBus for StubIntentBus {
    async fn submit_goal(&self, goal: &str, _caps: &[&str]) -> AiResult<String> {
        tracing::info!("intent.submit goal={}", goal);
        // Generate deterministic plan id from a Plan placeholder for traceability
        let mut p = Plan::new("goal", goal, goal, "ai");
        let id = crate::plan_store::PlanStore::generate_plan_id(&p).map_err(|e| AiError::internal(e.to_string()))?;
        p.metadata.insert("intent_bus".to_string(), "stub".to_string());
        let _ = self.plan_store.create_plan(p, "aicore".to_string()).await; // ignore errors for stub
        Ok(id)
    }

    async fn publish_event(&self, topic: &str, payload: &serde_json::Value) -> AiResult<()> {
        tracing::info!(target: "aicore.intent", "publish topic={} payload={}", topic, payload);
        Ok(())
    }
}

// 5) Meta-learning hooks (deterministic metrics)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaMetrics {
    pub plans_generated: u64,
    pub steps_executed: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_generation_time_us: u64,
}

// 6) Cognitive Core main
pub struct CognitiveCore {
    cfg: CognitiveCoreConfig,
    reason: Arc<RwLock<ReasoningGraph>>,
    wm: WorkingMemory,
    ltm: Arc<NgfsKv>,
    planner: Arc<dyn TaskPlanner>,
    policy: Arc<AiPolicy>,
    intent_bus: Arc<dyn IntentBus>,
    metrics: Arc<RwLock<MetaMetrics>>, 
    learned: Arc<RwLock<LearnedState>>,
}

use crate::learning::{LearnedState, LearningEvent, append_ltm_event, summarize_ltm};

impl CognitiveCore {
    pub async fn new(cfg: CognitiveCoreConfig) -> AiResult<Self> {
        let policy = Arc::new(AiPolicy::new()?);
        let planner = Arc::new(crate::planner::StubTaskPlanner::new(policy.clone())) as Arc<dyn TaskPlanner>;
        planner.init(cfg.planner.clone()).await?;
        let plan_store = Arc::new(PlanStore::default().await.map_err(|e| AiError::internal(e.to_string()))?);
        // Prefer real IntentClient when INTENT_SOCKET is configured; otherwise, fallback to stub
        let intent_bus: Arc<dyn IntentBus> = if let Some(client) = IntentClient::from_env() {
            Arc::new(RealIntentBus::new(client))
        } else {
            Arc::new(StubIntentBus::new(plan_store))
        };
        // Load learned state from LTM NDJSON (deterministically at init)
        let learned = summarize_ltm(&cfg.ltm_path).await.unwrap_or_default();
        Ok(Self {
            cfg: cfg.clone(),
            reason: Arc::new(RwLock::new(ReasoningGraph::default())),
            wm: WorkingMemory::new(cfg.working_mem_capacity),
            ltm: Arc::new(NgfsKv::new(std::env::var("NGFS_ROOT").unwrap_or_else(|_| "./data/ngfs".to_string()))),
            planner,
            policy,
            intent_bus,
            metrics: Arc::new(RwLock::new(MetaMetrics::default())),
            learned: Arc::new(RwLock::new(learned)),
        })
    }

    pub fn working_memory(&self) -> WorkingMemory { self.wm.clone() }
    pub fn long_term_memory(&self) -> Arc<NgfsKv> { self.ltm.clone() }

    /// Register a reasoning node
    pub async fn register_node<N: ReasonerNode + 'static>(&self, node: N, entrypoint: bool) {
        let mut rg = self.reason.write().await;
        rg.register_node(node);
        if entrypoint {
            let id = rg.nodes.keys().next_back().cloned();
            if let Some(id) = id { rg.add_entrypoint(&id); }
        }
    }

    /// Deterministic goal decomposition into sub‑tasks (Agentic 2.0, in‑process)
    /// Returns a list of canonical steps with execution strategies and deps.
    pub fn decompose_goal(&self, goal: &str) -> Vec<DecomposedStep> {
        // Normalize goal and split into clauses deterministically
        let g = goal.to_lowercase();
        let clauses = Self::split_clauses(&g);
        // For each clause, map to a deterministic template of sub‑steps
        let mut out: Vec<DecomposedStep> = Vec::new();
        for (i, clause) in clauses.iter().enumerate() {
            let mut steps = Self::clause_to_steps(clause, i);
            out.append(&mut steps);
        }
        // Impose deterministic cap to prevent unbounded planning
        let max_steps = self.cfg.planner.max_steps_per_plan.max(1);
        if out.len() > max_steps { out.truncate(max_steps); }
        // Deterministic order by (group_index, order_in_group, name)
        out.sort_by_key(|a| (a.group, a.order_in_group, a.name.clone()));
        out
    }

    fn split_clauses(g: &str) -> Vec<String> {
        // Split by common conjunctions; keep deterministic trimming
        let mut parts: Vec<String> = g
            .split([',', ';'] )
            .flat_map(|p| p.split(" and "))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        // Deduplicate while preserving stable order
        let mut seen = BTreeMap::<String, ()>::new();
        let mut dedup = Vec::new();
        for p in parts.drain(..) {
            if !seen.contains_key(&p) { seen.insert(p.clone(), ()); dedup.push(p); }
        }
        if dedup.is_empty() { vec![g.to_string()] } else { dedup }
    }

    fn clause_to_steps(clause: &str, group_idx: usize) -> Vec<DecomposedStep> {
        // Static templates for common intents; fallback to analyze→plan→execute
        // Deterministic mapping via sorted keyword checks
        let mut templates: Vec<DecomposedStep> = Vec::new();
        // File organization
        if clause.contains("organize") && clause.contains("file") {
            templates.push(DecomposedStep::seq(group_idx, 0, "scan file directories", "enumerate files"));
            templates.push(DecomposedStep::seq(group_idx, 1, "categorize files", "by type/date/size"));
            templates.push(DecomposedStep::seq(group_idx, 2, "apply file rules", "move/rename per policy"));
            return templates;
        }
        // Photos backup
        if (clause.contains("backup") || clause.contains("back up")) && (clause.contains("photo") || clause.contains("image")) {
            templates.push(DecomposedStep::seq(group_idx, 0, "identify photos", "find image assets"));
            templates.push(DecomposedStep::seq(group_idx, 1, "copy to backup drive", "mirror to target"));
            templates.push(DecomposedStep::seq(group_idx, 2, "verify checksums", "ensure integrity"));
            return templates;
        }
        // Generic web/app/task patterns
        if clause.contains("build") || clause.contains("create") || clause.contains("develop") {
            templates.push(DecomposedStep::seq(group_idx, 0, "analyze requirements", "clarify constraints"));
            templates.push(DecomposedStep::seq(group_idx, 1, "draft plan", "break into tasks"));
            templates.push(DecomposedStep::seq(group_idx, 2, "implement core", "baseline deliverable"));
            templates.push(DecomposedStep::seq(group_idx, 3, "test and verify", "validate outputs"));
            return templates;
        }
        // Fallback deterministic trio
        templates.push(DecomposedStep::seq(group_idx, 0, "analyze goal", "understand input"));
        templates.push(DecomposedStep::seq(group_idx, 1, "plan steps", "structure tasks"));
        templates.push(DecomposedStep::seq(group_idx, 2, "execute plan", "run actions"));
        templates
    }

    /// Build a Plan from the deterministic decomposition (in‑process, no LLM)
    pub async fn plan_from_goal(&self, goal: &str) -> AiResult<PlanResult> {
        self.policy.check_capability("ai:plan.generate")?;
        // Decompose
        let steps = self.decompose_goal(goal);
        tracing::debug!(target="aicore.plan", "decomposition goal='{}' steps={:?}", goal, steps);
        // Construct Plan with sequential/parallel semantics via dependencies
        let mut plan = Plan::new(
            &format!("Plan for: {}", goal),
            &format!("Deterministic decomposition for: {}", goal),
            goal,
            "aicore",
        );
        // Group steps by group index to allow parallel between groups
        // Optionally adjust based on learned state captured at init
        let mut steps = steps;
        let learned = self.learned.read().await.clone();
        if learned.backup_drive_unavailable {
            // For any group containing a backup copy step, insert a preflight check
            let mut groups: BTreeMap<usize, bool> = BTreeMap::new();
            for s in &steps {
                if s.name.contains("copy to backup drive") { groups.insert(s.group, true); }
            }
            for g in groups.keys() {
                let exists = steps.iter().any(|s| s.group == *g && s.name.contains("check backup drive availability"));
                if !exists {
                    steps.push(DecomposedStep::seq(*g, 0, "check backup drive availability", "probe mount or path"));
                }
            }
        }
        let mut by_group: BTreeMap<usize, Vec<DecomposedStep>> = BTreeMap::new();
        for s in steps { by_group.entry(s.group).or_default().push(s); }
        for (_g, mut group_steps) in by_group.into_iter() {
            group_steps.sort_by_key(|s| s.order_in_group);
            let mut prev_step_id: Option<String> = None;
            for s in group_steps {
                let mut ps = PlanStep::new(&capitalize(&s.name), &s.description, "action");
                ps.execution_strategy = match s.strategy {
                    DecompStrategy::Sequential => ExecutionStrategy::Sequential,
                    DecompStrategy::Parallel => ExecutionStrategy::Parallel,
                };
                if let Some(dep) = &prev_step_id { ps.add_dependency(dep); }
                plan.add_step(ps.clone());
                prev_step_id = Some(ps.id.clone());
            }
        }
        // If goal requests a direct intent call (special format: "intent:<name> {json}"),
        // fire-and-forget to avoid blocking the reasoning thread.
        if goal.starts_with("intent:") {
            if let Some(_client) = crate::intent_client::IntentClient::from_env() {
                let parts: Vec<&str> = goal.splitn(2, ' ').collect();
                let intent_name = parts[0].trim_start_matches("intent:").to_string();
                let params_json = if parts.len() > 1 { parts[1] } else { "{}" };
                let params_val: serde_json::Value = serde_json::from_str(params_json).unwrap_or(serde_json::json!({}));
                let params_cbor = to_value(&params_val).unwrap_or(serde_cbor::Value::Map(BTreeMap::new()));
                tokio::spawn(async move {
                    let _ = crate::intent_client::send_intent(&intent_name, &params_cbor).await;
                });
            }
        }
        plan.status = PlanStatus::Ready;
        Ok(PlanResult { plan: Some(plan), success: true, generation_time_us: 0, error: None,
            llm_stats: crate::planner::LlmUsageStats{ input_tokens: 0, output_tokens: 0, inference_time_us: 0, model: "deterministic".into() } })
    }

    /// Submit a goal via Intent Bus and generate a deterministic plan
    pub async fn submit_goal(&self, goal: &str) -> AiResult<PlanResult> {
        self.policy.check_capability("ai:plan.generate")?;
        let _intent_id = self.intent_bus.submit_goal(goal, &["ai:plan.generate"]).await?;
        let req = PlanRequest {
            goal: goal.to_string(),
            context: None,
            required_tools: vec![],
            priority: StepPriority::Normal,
            max_execution_time: Some(self.cfg.planner.plan_timeout_secs),
            tags: vec!["aicore".to_string()],
            metadata: HashMap::new(),
        };
        let res = self.planner.generate_plan(req).await?;
        {
            let mut m = self.metrics.write().await;
            m.plans_generated += 1;
            m.avg_generation_time_us = if m.plans_generated == 0 { res.generation_time_us } else { (m.avg_generation_time_us + res.generation_time_us) / 2 };
        }
        Ok(res)
    }

    /// Execute a reasoning pass over the graph
    pub async fn reason_once(&self, input: serde_json::Value) -> AiResult<BTreeMap<String, serde_json::Value>> {
        let rg = self.reason.read().await;
        rg.execute(self.cfg.random_seed, input, &self.wm).await
    }

    /// Append outcome to NDJSON LTM (applied on next init only)
    pub async fn record_outcome(&self, goal: &str, outcome_code: &str, notes: Option<&str>, facts: Option<serde_json::Value>) -> AiResult<()> {
        let ev = LearningEvent::new(goal, outcome_code, notes.map(|s| s.to_string()), facts);
        append_ltm_event(&self.cfg.ltm_path, &ev).await
    }

    /// Persist a summary to LTM deterministically
    pub async fn persist_summary(&self, plan: &Plan) -> AiResult<()> {
        // Store a compact CBOR summary under kv/plans/<id>
        #[derive(serde::Serialize)]
        struct Summary<'a> { plan_id: &'a str, name: &'a str, version: u32, created_at: u64 }
        let s = Summary { plan_id: &plan.id, name: &plan.name, version: plan.version, created_at: plan.created_at };
        self.ltm.put(&format!("plans/{}", plan.id), &s).await
    }

    /// Export and write a full snapshot into NGFS; returns snapshot id
    pub async fn snapshot_write(&self) -> AiResult<String> {
        #[derive(Serialize)]
        struct Snapshot<'a> {
            cfg: &'a CognitiveCoreConfig,
            wm: WorkingMemoryState,
            metrics: &'a MetaMetrics,
        }
        let wm = self.wm.snapshot().await;
        let metrics = self.metrics.read().await.clone();
        let snap = Snapshot { cfg: &self.cfg, wm, metrics: &metrics };
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&snap, &mut buf).map_err(|e| AiError::serialization(e.to_string()))?;
        let id = blake3::hash(&buf).to_hex().to_string();
        let ngfs = NgfsKv::new(std::env::var("NGFS_ROOT").unwrap_or_else(|_| "./data/ngfs".to_string()));
        ngfs.write_snapshot_bytes(&id, &buf).await?;
        Ok(id)
    }

    /// Restore from snapshot id
    pub async fn snapshot_replay(&self, id: &str) -> AiResult<()> {
        let ngfs = NgfsKv::new(std::env::var("NGFS_ROOT").unwrap_or_else(|_| "./data/ngfs".to_string()));
        let data = ngfs.read_snapshot_bytes(id).await?;
        #[derive(Serialize, Deserialize)]
        struct Snapshot {
            cfg: CognitiveCoreConfig,
            wm: WorkingMemoryState,
            metrics: MetaMetrics,
        }
        let snap: Snapshot = ciborium::de::from_reader(data.as_slice()).map_err(|e| AiError::deserialization(e.to_string()))?;
        // Verify hash determinism
        let mut rebuf = Vec::new();
        ciborium::ser::into_writer(&snap, &mut rebuf).map_err(|e| AiError::serialization(e.to_string()))?;
        let rehash = blake3::hash(&rebuf).to_hex().to_string();
        if rehash != *id { tracing::warn!("replay divergence: expected {} got {}", id, rehash); }
        self.wm.restore(snap.wm).await;
        *self.metrics.write().await = snap.metrics;
        Ok(())
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new(), }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompStrategy { Sequential, Parallel }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecomposedStep {
    pub group: usize,
    pub order_in_group: usize,
    pub name: String,
    pub description: String,
    pub strategy: DecompStrategy,
}

impl DecomposedStep {
    pub fn seq(group: usize, order: usize, name: &str, desc: &str) -> Self {
        Self { group, order_in_group: order, name: name.to_string(), description: desc.to_string(), strategy: DecompStrategy::Sequential }
    }
    pub fn par(group: usize, order: usize, name: &str, desc: &str) -> Self {
        Self { group, order_in_group: order, name: name.to_string(), description: desc.to_string(), strategy: DecompStrategy::Parallel }
    }
}

// --- Example builtin nodes ---
pub struct EchoNode;
#[async_trait::async_trait]
impl ReasonerNode for EchoNode {
    fn id(&self) -> &'static str { "echo" }
    async fn eval(&self, input: &serde_json::Value, wm: &WorkingMemory) -> AiResult<serde_json::Value> {
let ts = crate::time::get_time_90khz();
        wm.put("echo.last".to_string(), input.clone(), ts).await;
        Ok(serde_json::json!({ "echo": input }))
    }
}

// --- Minimal C FFI (synchronous wrappers for async core) ---
#[cfg(feature = "phase5")]
lazy_static::lazy_static! {
    static ref AICORE_RT: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("aicore runtime");
    static ref AICORE_CORE: std::sync::Mutex<Option<Arc<CognitiveCore>>> = std::sync::Mutex::new(None);
}

#[cfg(feature = "phase5")]
#[no_mangle]
pub extern "C" fn aicore_init_default() -> i32 {
    AICORE_RT.block_on(async {
        match CognitiveCore::new(CognitiveCoreConfig::default()).await {
            Ok(core) => {
                *AICORE_CORE.lock().unwrap() = Some(Arc::new(core));
                0
            }
            Err(_) => 1,
        }
    })
}

#[cfg(feature = "phase5")]
#[no_mangle]
pub extern "C" fn aicore_snapshot(out_id_buf: *mut u8, out_len: usize) -> i32 {
    if out_id_buf.is_null() || out_len == 0 { return 2; }
    let id = AICORE_RT.block_on(async {
        let guard = AICORE_CORE.lock().unwrap();
        let core = match &*guard { Some(c) => c.clone(), None => return Err(AiError::internal("not initialized")) };
        core.snapshot_write().await
    });
    match id {
        Ok(id) => {
            let bytes = id.as_bytes();
            let n = bytes.len().min(out_len);
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_id_buf, n); }
            0
        }
        Err(_) => 1,
    }
}

#[cfg(feature = "phase5")]
#[no_mangle]
pub extern "C" fn aicore_replay(id_cstr: *const std::os::raw::c_char) -> i32 {
    use std::ffi::CStr;
    if id_cstr.is_null() { return 2; }
    let cstr = unsafe { CStr::from_ptr(id_cstr) };
    let id = match cstr.to_str() { Ok(s) => s, Err(_) => return 3 };
    let rc = AICORE_RT.block_on(async {
        let guard = AICORE_CORE.lock().unwrap();
        let core = match &*guard { Some(c) => c.clone(), None => return Err(AiError::internal("not initialized")) };
        core.snapshot_replay(id).await
    });
    if rc.is_ok() { 0 } else { 1 }
}

#[cfg(feature = "phase5")]
#[no_mangle]
pub extern "C" fn aicore_submit_goal_json(goal_json: *const std::os::raw::c_char, out_buf: *mut u8, out_len: usize) -> i32 {
    use std::ffi::CStr;
    if goal_json.is_null() || out_buf.is_null() || out_len == 0 { return 2; }
    let cstr = unsafe { CStr::from_ptr(goal_json) };
    let goal = match cstr.to_str() { Ok(s) => s, Err(_) => return 3 };
    let r = AICORE_RT.block_on(async {
        let guard = AICORE_CORE.lock().unwrap();
        let core = match &*guard { Some(c) => c.clone(), None => return Err(AiError::internal("not initialized")) };
        core.submit_goal(goal).await
    });
    match r {
        Ok(res) => {
            let json = serde_json::to_string(&res).unwrap_or("{}".to_string());
            let bytes = json.as_bytes();
            let n = bytes.len().min(out_len);
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, n); }
            0
        },
        Err(_) => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cognitive_core_init() {
        let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
        let wm = core.working_memory();
        wm.put("k".to_string(), serde_json::json!(1), 123).await;
        assert_eq!(wm.get("k").await, Some(serde_json::json!(1)));
    }

    #[tokio::test]
    async fn test_reasoning_graph_exec() {
        let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
        core.register_node(EchoNode, true).await;
        let out = core.reason_once(serde_json::json!({"x":1})).await.unwrap();
        assert!(out.contains_key("echo"));
    }
}
