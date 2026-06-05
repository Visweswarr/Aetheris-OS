//! Assistant kernel for Phase 5-A task graph planning and execution.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::contracts::{timestamp_90khz, Priority, TaskEdge, TaskGraph, TaskGraphStatus, TaskNode};
use crate::error::{AiCoreError, Result};
use crate::ipc::ToolCallRequest;
use crate::memory::MemoryStore;
use crate::tools::ToolRegistry;

#[derive(Clone)]
pub struct AssistantKernel {
    tool_registry: Arc<ToolRegistry>,
    memory_store: Option<Arc<MemoryStore>>,
    graphs: Arc<RwLock<HashMap<String, TaskGraph>>>,
}

impl AssistantKernel {
    pub fn new(tool_registry: Arc<ToolRegistry>, memory_store: Option<Arc<MemoryStore>>) -> Self {
        Self {
            tool_registry,
            memory_store,
            graphs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn plan(&self, goal: impl Into<String>, priority: Priority) -> Result<TaskGraph> {
        let goal = goal.into();
        if goal.trim().is_empty() {
            return Err(AiCoreError::InvalidInput(
                "assistant goal cannot be empty".to_string(),
            ));
        }

        let graph = TaskGraph {
            graph_id: format!("graph-{}", blake3::hash(goal.as_bytes()).to_hex()),
            goal: goal.clone(),
            nodes: vec![TaskNode {
                node_id: "node-echo".to_string(),
                tool_name: "echo".to_string(),
                parameters: serde_json::json!({ "message": goal }),
                required_capabilities: Vec::new(),
                priority,
                timeout_ms: 30_000,
            }],
            edges: Vec::new(),
            status: TaskGraphStatus::Ready,
            created_at: timestamp_90khz(),
            metadata: HashMap::new(),
        };

        self.submit(graph.clone()).await?;
        Ok(graph)
    }

    pub async fn submit(&self, graph: TaskGraph) -> Result<TaskGraph> {
        self.validate_graph(&graph).await?;
        self.graphs
            .write()
            .await
            .insert(graph.graph_id.clone(), graph.clone());
        Ok(graph)
    }

    pub async fn execute(
        &self,
        graph_id: &str,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<TaskGraph> {
        let mut graph = self
            .get(graph_id)
            .await
            .ok_or_else(|| AiCoreError::NotFound(format!("task graph not found: {}", graph_id)))?;

        if graph.status == TaskGraphStatus::Cancelled {
            return Err(AiCoreError::InvalidInput(format!(
                "task graph {} is cancelled",
                graph_id
            )));
        }

        graph.status = TaskGraphStatus::Running;
        self.graphs
            .write()
            .await
            .insert(graph.graph_id.clone(), graph.clone());

        let order = self.execution_order(&graph)?;
        let nodes_by_id: HashMap<_, _> = graph
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node.clone()))
            .collect();

        for node_id in order {
            let node = nodes_by_id.get(&node_id).ok_or_else(|| {
                AiCoreError::InvalidInput(format!("missing task node: {}", node_id))
            })?;
            let request = ToolCallRequest {
                tool_name: node.tool_name.clone(),
                call_id: format!("tool-{}-{}", graph.graph_id, node.node_id),
                parameters: node.parameters.clone(),
                arguments: String::new(),
                cap_token: None,
                user_id: user_id.clone(),
                session_id: session_id.clone(),
            };
            let result = self.tool_registry.execute_tool(&request).await?;
            if !result.success {
                graph.status = TaskGraphStatus::Failed;
                self.graphs
                    .write()
                    .await
                    .insert(graph.graph_id.clone(), graph.clone());
                return Err(AiCoreError::ToolError(
                    result
                        .error_message
                        .unwrap_or_else(|| "tool execution failed".to_string()),
                ));
            }
        }

        graph.status = TaskGraphStatus::Completed;
        if let Some(memory) = &self.memory_store {
            memory
                .put(
                    format!(
                        "assistant:{}:last_graph",
                        user_id.as_deref().unwrap_or("anonymous")
                    ),
                    graph.graph_id.clone(),
                    vec!["assistant".to_string(), "task_graph".to_string()],
                    None,
                    user_id.clone().unwrap_or_else(|| "anonymous".to_string()),
                    session_id,
                )
                .await?;
        }
        self.graphs
            .write()
            .await
            .insert(graph.graph_id.clone(), graph.clone());
        Ok(graph)
    }

    pub async fn cancel(&self, graph_id: &str) -> Result<TaskGraph> {
        let mut graphs = self.graphs.write().await;
        let graph = graphs
            .get_mut(graph_id)
            .ok_or_else(|| AiCoreError::NotFound(format!("task graph not found: {}", graph_id)))?;

        if graph.status == TaskGraphStatus::Completed {
            return Err(AiCoreError::InvalidInput(format!(
                "task graph {} already completed",
                graph_id
            )));
        }

        graph.status = TaskGraphStatus::Cancelled;
        Ok(graph.clone())
    }

    pub async fn get(&self, graph_id: &str) -> Option<TaskGraph> {
        self.graphs.read().await.get(graph_id).cloned()
    }

    pub async fn list(&self) -> Vec<TaskGraph> {
        self.graphs.read().await.values().cloned().collect()
    }

    async fn validate_graph(&self, graph: &TaskGraph) -> Result<()> {
        if graph.graph_id.trim().is_empty() {
            return Err(AiCoreError::InvalidInput(
                "task graph id cannot be empty".to_string(),
            ));
        }
        if graph.nodes.is_empty() {
            return Err(AiCoreError::InvalidInput(
                "task graph must contain at least one node".to_string(),
            ));
        }

        for node in &graph.nodes {
            if self.tool_registry.get_tool(&node.tool_name).await.is_none() {
                return Err(AiCoreError::ToolNotFound(node.tool_name.clone()));
            }
        }

        self.execution_order(graph).map(|_| ())
    }

    fn execution_order(&self, graph: &TaskGraph) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = graph
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), 0))
            .collect();
        let mut outgoing: HashMap<String, Vec<String>> = graph
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), Vec::new()))
            .collect();

        for TaskEdge { from, to } in &graph.edges {
            if !in_degree.contains_key(from) || !in_degree.contains_key(to) {
                return Err(AiCoreError::InvalidInput(format!(
                    "task edge references unknown node: {} -> {}",
                    from, to
                )));
            }
            outgoing.get_mut(from).unwrap().push(to.clone());
            *in_degree.get_mut(to).unwrap() += 1;
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|&(_node_id, degree)| *degree == 0)
            .map(|(node_id, _degree)| node_id.clone())
            .collect();
        let mut order = Vec::with_capacity(graph.nodes.len());

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id.clone());
            if let Some(next_nodes) = outgoing.get(&node_id) {
                for next in next_nodes {
                    let degree = in_degree.get_mut(next).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(next.clone());
                    }
                }
            }
        }

        if order.len() != graph.nodes.len() {
            return Err(AiCoreError::InvalidInput(
                "task graph contains a cycle".to_string(),
            ));
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn kernel() -> AssistantKernel {
        let registry = Arc::new(ToolRegistry::new_mock());
        registry.initialize().await.unwrap();
        AssistantKernel::new(registry, None)
    }

    #[tokio::test]
    async fn plans_and_executes_task_graph() {
        let kernel = kernel().await;
        let graph = kernel
            .plan("remember this workflow", Priority::High)
            .await
            .unwrap();
        let executed = kernel
            .execute(
                &graph.graph_id,
                Some("user-1".to_string()),
                Some("session-1".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(executed.status, TaskGraphStatus::Completed);
    }

    #[tokio::test]
    async fn rejects_unknown_tools() {
        let kernel = kernel().await;
        let graph = TaskGraph {
            graph_id: "bad-graph".to_string(),
            goal: "bad".to_string(),
            nodes: vec![TaskNode {
                node_id: "node-1".to_string(),
                tool_name: "missing".to_string(),
                parameters: serde_json::json!({}),
                required_capabilities: Vec::new(),
                priority: Priority::Normal,
                timeout_ms: 1_000,
            }],
            edges: Vec::new(),
            status: TaskGraphStatus::Ready,
            created_at: timestamp_90khz(),
            metadata: HashMap::new(),
        };

        assert!(kernel.submit(graph).await.is_err());
    }
}
