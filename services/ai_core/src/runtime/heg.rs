//! Deterministic Heterogeneous Execution Graph (HEG) planning.
//!
//! This module models where AI runtime work should run across CPU, iGPU,
//! and NPU resources. It does not execute on accelerators yet; it produces a
//! stable placement plan and metrics that the runtime can audit and expose.

use serde::{Deserialize, Serialize};

use crate::contracts::sha256_hex;
use crate::error::Result;

use super::RuntimeRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorKind {
    Cpu,
    Igpu,
    Npu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorKind {
    Prefill,
    Decode,
    Embedding,
    ToolCall,
    MemoryLookup,
    BrowserAssist,
    LogSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub compute_weight: u32,
    pub memory_weight: u32,
    pub latency_budget_ms: u64,
    pub batchable: bool,
    pub deterministic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HegNode {
    pub node_id: String,
    pub operator: OperatorKind,
    pub profile: ResourceProfile,
    pub preferred: Option<AcceleratorKind>,
    pub assigned: AcceleratorKind,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HegEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementDecision {
    pub node_id: String,
    pub assigned: AcceleratorKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HegPlan {
    pub graph_id: String,
    pub nodes: Vec<HegNode>,
    pub edges: Vec<HegEdge>,
    pub decisions: Vec<PlacementDecision>,
    pub estimated_latency_ms: u64,
    pub ddr_pressure_score: u32,
    pub deterministic_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HegPolicy {
    pub npu_enabled: bool,
    pub igpu_enabled: bool,
    pub max_ddr_pressure: u32,
    pub prefer_local: bool,
    pub deterministic_seed: u64,
    pub preferred_accelerator: Option<AcceleratorKind>,
}

impl Default for HegPolicy {
    fn default() -> Self {
        Self {
            npu_enabled: true,
            igpu_enabled: true,
            max_ddr_pressure: 120,
            prefer_local: true,
            deterministic_seed: 0,
            preferred_accelerator: None,
        }
    }
}

pub fn plan_request(request: &RuntimeRequest, policy: &HegPolicy) -> Result<HegPlan> {
    let mut nodes = request_nodes(request);
    let mut decisions = Vec::with_capacity(nodes.len());

    for node in &mut nodes {
        let (assigned, reason) = initial_assignment(node.operator, policy);
        node.preferred = preferred_for(node.operator, policy);
        node.assigned = assigned;
        decisions.push(PlacementDecision {
            node_id: node.node_id.clone(),
            assigned,
            reason,
        });
    }

    let mut ddr_pressure_score = ddr_pressure(&nodes);
    if ddr_pressure_score > policy.max_ddr_pressure {
        let decode_indexes: Vec<usize> = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                (node.operator == OperatorKind::Decode && node.assigned != AcceleratorKind::Cpu)
                    .then_some(index)
            })
            .collect();
        for index in decode_indexes {
            let node_id = nodes[index].node_id.clone();
            nodes[index].assigned = AcceleratorKind::Cpu;
            decisions.push(PlacementDecision {
                node_id,
                assigned: AcceleratorKind::Cpu,
                reason: format!(
                    "ddr pressure {} exceeded cap {}; moved memory-bound decode to CPU",
                    ddr_pressure_score, policy.max_ddr_pressure
                ),
            });
            ddr_pressure_score = ddr_pressure(&nodes);
            if ddr_pressure_score <= policy.max_ddr_pressure {
                break;
            }
        }
    }

    let edges = request_edges(&nodes);
    let estimated_latency_ms = nodes.iter().map(estimated_node_latency).sum();
    let graph_id = graph_id(request, policy)?;
    let deterministic_hash = deterministic_hash(
        &graph_id,
        &nodes,
        &edges,
        &decisions,
        estimated_latency_ms,
        ddr_pressure_score,
    )?;

    Ok(HegPlan {
        graph_id,
        nodes,
        edges,
        decisions,
        estimated_latency_ms,
        ddr_pressure_score,
        deterministic_hash,
    })
}

pub fn audit_event_for_plan(plan: &HegPlan) -> String {
    format!(
        "runtime.heg.plan_generated graph_id={} hash={} ddr_pressure={}",
        plan.graph_id, plan.deterministic_hash, plan.ddr_pressure_score
    )
}

fn request_nodes(request: &RuntimeRequest) -> Vec<HegNode> {
    let prompt = request.prompt.to_ascii_lowercase();
    let workload = request
        .metadata
        .get("workload")
        .map(String::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut nodes = vec![
        node("embedding", OperatorKind::Embedding, 15, 12, 4, true, Vec::new()),
        node(
            "prefill",
            OperatorKind::Prefill,
            80,
            35,
            12,
            true,
            vec!["embedding".to_string()],
        ),
        node(
            "decode",
            OperatorKind::Decode,
            45,
            55,
            20,
            false,
            vec!["prefill".to_string()],
        ),
    ];

    if workload.contains("memory") {
        nodes.insert(
            1,
            node(
                "memory_lookup",
                OperatorKind::MemoryLookup,
                20,
                20,
                6,
                false,
                vec!["embedding".to_string()],
            ),
        );
    }
    if workload.contains("tool") || prompt.contains("tool:") {
        nodes.push(node(
            "tool_call",
            OperatorKind::ToolCall,
            20,
            10,
            15,
            false,
            vec!["decode".to_string()],
        ));
    }
    if workload.contains("browser") || prompt.contains("browser") || prompt.contains("page") {
        nodes.push(node(
            "browser_assist",
            OperatorKind::BrowserAssist,
            25,
            15,
            18,
            false,
            vec!["decode".to_string()],
        ));
    }
    if workload.contains("log") || prompt.contains("log") {
        nodes.push(node(
            "log_summary",
            OperatorKind::LogSummary,
            30,
            30,
            25,
            false,
            vec!["decode".to_string()],
        ));
    }

    nodes
}

fn node(
    node_id: &str,
    operator: OperatorKind,
    compute_weight: u32,
    memory_weight: u32,
    latency_budget_ms: u64,
    batchable: bool,
    dependencies: Vec<String>,
) -> HegNode {
    HegNode {
        node_id: node_id.to_string(),
        operator,
        profile: ResourceProfile {
            compute_weight,
            memory_weight,
            latency_budget_ms,
            batchable,
            deterministic: true,
        },
        preferred: None,
        assigned: AcceleratorKind::Cpu,
        dependencies,
    }
}

fn initial_assignment(operator: OperatorKind, policy: &HegPolicy) -> (AcceleratorKind, String) {
    if matches!(
        operator,
        OperatorKind::ToolCall
            | OperatorKind::MemoryLookup
            | OperatorKind::BrowserAssist
            | OperatorKind::LogSummary
    ) {
        return (
            AcceleratorKind::Cpu,
            "local/capability-gated work stays on CPU".to_string(),
        );
    }

    if let Some(preferred) = policy.preferred_accelerator {
        if accelerator_enabled(preferred, policy) {
            return (
                preferred,
                format!("caller preferred {:?}", preferred).to_ascii_lowercase(),
            );
        }
    }

    match operator {
        OperatorKind::Prefill | OperatorKind::Embedding => {
            if policy.npu_enabled {
                (
                    AcceleratorKind::Npu,
                    "compute-heavy prefill/embedding prefers NPU".to_string(),
                )
            } else if policy.igpu_enabled {
                (
                    AcceleratorKind::Igpu,
                    "NPU disabled; compute-heavy work falls back to iGPU".to_string(),
                )
            } else {
                (
                    AcceleratorKind::Cpu,
                    "accelerators disabled; compute-heavy work falls back to CPU".to_string(),
                )
            }
        }
        OperatorKind::Decode => {
            if policy.igpu_enabled {
                (
                    AcceleratorKind::Igpu,
                    "memory-bound decode prefers iGPU".to_string(),
                )
            } else {
                (
                    AcceleratorKind::Cpu,
                    "iGPU disabled; decode falls back to CPU".to_string(),
                )
            }
        }
        _ => (AcceleratorKind::Cpu, "default CPU placement".to_string()),
    }
}

fn preferred_for(operator: OperatorKind, policy: &HegPolicy) -> Option<AcceleratorKind> {
    if matches!(
        operator,
        OperatorKind::ToolCall
            | OperatorKind::MemoryLookup
            | OperatorKind::BrowserAssist
            | OperatorKind::LogSummary
    ) {
        return Some(AcceleratorKind::Cpu);
    }
    policy.preferred_accelerator
}

fn accelerator_enabled(kind: AcceleratorKind, policy: &HegPolicy) -> bool {
    match kind {
        AcceleratorKind::Cpu => true,
        AcceleratorKind::Igpu => policy.igpu_enabled,
        AcceleratorKind::Npu => policy.npu_enabled,
    }
}

fn ddr_pressure(nodes: &[HegNode]) -> u32 {
    nodes
        .iter()
        .filter(|node| node.assigned != AcceleratorKind::Cpu)
        .map(|node| node.profile.memory_weight)
        .sum()
}

fn request_edges(nodes: &[HegNode]) -> Vec<HegEdge> {
    let mut edges = Vec::new();
    for node in nodes {
        for dependency in &node.dependencies {
            edges.push(HegEdge {
                from: dependency.clone(),
                to: node.node_id.clone(),
            });
        }
    }
    edges
}

fn estimated_node_latency(node: &HegNode) -> u64 {
    let divisor = match node.assigned {
        AcceleratorKind::Cpu => 1,
        AcceleratorKind::Igpu => 2,
        AcceleratorKind::Npu => 4,
    };
    let weighted = (node.profile.compute_weight as u64 + node.profile.memory_weight as u64)
        / divisor;
    weighted.max(node.profile.latency_budget_ms)
}

fn graph_id(request: &RuntimeRequest, policy: &HegPolicy) -> Result<String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "prompt": request.prompt,
        "max_tokens": request.max_tokens,
        "metadata": request.metadata,
        "policy": policy,
    }))?;
    Ok(format!("heg-{}", &sha256_hex(&bytes)[..16]))
}

fn deterministic_hash(
    graph_id: &str,
    nodes: &[HegNode],
    edges: &[HegEdge],
    decisions: &[PlacementDecision],
    estimated_latency_ms: u64,
    ddr_pressure_score: u32,
) -> Result<String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "graph_id": graph_id,
        "nodes": nodes,
        "edges": edges,
        "decisions": decisions,
        "estimated_latency_ms": estimated_latency_ms,
        "ddr_pressure_score": ddr_pressure_score,
    }))?;
    Ok(sha256_hex(&bytes))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::audit::TamperEvidentAuditLog;

    use super::*;

    fn request(prompt: &str, workload: Option<&str>) -> RuntimeRequest {
        let mut metadata = HashMap::new();
        if let Some(workload) = workload {
            metadata.insert("workload".to_string(), workload.to_string());
        }
        RuntimeRequest {
            prompt: prompt.to_string(),
            max_tokens: Some(64),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
            metadata,
        }
    }

    #[test]
    fn same_request_policy_produces_byte_identical_plan() {
        let req = request("summarize this browser page", Some("browser"));
        let policy = HegPolicy::default();
        let plan_a = plan_request(&req, &policy).unwrap();
        let plan_b = plan_request(&req, &policy).unwrap();
        assert_eq!(serde_json::to_vec(&plan_a).unwrap(), serde_json::to_vec(&plan_b).unwrap());
        assert_eq!(plan_a.deterministic_hash, plan_b.deterministic_hash);
    }

    #[test]
    fn prefill_routes_to_npu_then_cpu_fallback() {
        let req = request("answer", None);
        let plan = plan_request(&req, &HegPolicy::default()).unwrap();
        assert_eq!(assigned(&plan, "prefill"), AcceleratorKind::Npu);

        let policy = HegPolicy {
            npu_enabled: false,
            igpu_enabled: false,
            ..HegPolicy::default()
        };
        let fallback = plan_request(&req, &policy).unwrap();
        assert_eq!(assigned(&fallback, "prefill"), AcceleratorKind::Cpu);
    }

    #[test]
    fn decode_routes_to_igpu_then_cpu_fallback() {
        let req = request("answer", None);
        let plan = plan_request(&req, &HegPolicy::default()).unwrap();
        assert_eq!(assigned(&plan, "decode"), AcceleratorKind::Igpu);

        let policy = HegPolicy {
            igpu_enabled: false,
            ..HegPolicy::default()
        };
        let fallback = plan_request(&req, &policy).unwrap();
        assert_eq!(assigned(&fallback, "decode"), AcceleratorKind::Cpu);
    }

    #[test]
    fn browser_log_and_tool_work_stay_cpu_local() {
        let req = request("browser page log tool: summarize", Some("browser log tool"));
        let plan = plan_request(&req, &HegPolicy::default()).unwrap();
        assert_eq!(assigned(&plan, "browser_assist"), AcceleratorKind::Cpu);
        assert_eq!(assigned(&plan, "log_summary"), AcceleratorKind::Cpu);
        assert_eq!(assigned(&plan, "tool_call"), AcceleratorKind::Cpu);
    }

    #[test]
    fn ddr_pressure_cap_moves_decode_deterministically() {
        let req = request("answer", None);
        let policy = HegPolicy {
            max_ddr_pressure: 10,
            ..HegPolicy::default()
        };
        let plan = plan_request(&req, &policy).unwrap();
        assert_eq!(assigned(&plan, "decode"), AcceleratorKind::Cpu);
        assert!(plan.ddr_pressure_score <= 47);
    }

    #[test]
    fn audit_chain_detects_modified_heg_event() {
        let req = request("answer", None);
        let plan = plan_request(&req, &HegPolicy::default()).unwrap();
        let mut audit = TamperEvidentAuditLog::new();
        audit.append("runtime", audit_event_for_plan(&plan)).unwrap();
        audit.verify().unwrap();
        audit.entries[0].event.push_str(" tampered");
        assert!(audit.verify().is_err());
    }

    fn assigned(plan: &HegPlan, node_id: &str) -> AcceleratorKind {
        plan.nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .unwrap()
            .assigned
    }
}
