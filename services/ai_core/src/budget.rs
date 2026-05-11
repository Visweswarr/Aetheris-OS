//! CPU, memory, and power budgeting for on-device AI work.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::contracts::{BudgetDecision, BudgetRequest, Priority};

#[derive(Debug, Clone, Copy)]
pub struct BudgetLimits {
    pub max_cpu_percent: u8,
    pub max_memory_mb: u64,
    pub max_power_mw: u64,
    pub max_concurrent: usize,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 85,
            max_memory_mb: 3_300,
            max_power_mw: 15_000,
            max_concurrent: 8,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BudgetSnapshot {
    pub active_reservations: usize,
    pub cpu_percent: u8,
    pub memory_mb: u64,
    pub power_mw: u64,
}

#[derive(Debug, Clone)]
pub struct BudgetEngine {
    limits: BudgetLimits,
    active: Arc<RwLock<HashMap<String, BudgetRequest>>>,
}

impl BudgetEngine {
    pub fn new(limits: BudgetLimits) -> Self {
        Self {
            limits,
            active: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn reserve(&self, request: BudgetRequest) -> BudgetDecision {
        let mut active = self.active.write().await;
        let snapshot = Self::snapshot_locked(&active);

        let next_cpu = snapshot.cpu_percent.saturating_add(request.cpu_percent);
        let next_memory = snapshot.memory_mb.saturating_add(request.memory_mb);
        let next_power = snapshot.power_mw.saturating_add(request.power_mw);

        let denied_reason = if active.len() >= self.limits.max_concurrent {
            Some(format!("concurrency limit {} reached", self.limits.max_concurrent))
        } else if next_cpu > self.limits.max_cpu_percent {
            Some(format!("cpu budget {}% exceeds limit {}%", next_cpu, self.limits.max_cpu_percent))
        } else if next_memory > self.limits.max_memory_mb {
            Some(format!("memory budget {} MB exceeds limit {} MB", next_memory, self.limits.max_memory_mb))
        } else if next_power > self.limits.max_power_mw {
            Some(format!("power budget {} mW exceeds limit {} mW", next_power, self.limits.max_power_mw))
        } else {
            None
        };

        if let Some(reason) = denied_reason {
            return BudgetDecision {
                request_id: request.request_id,
                granted: false,
                reason,
                queue_position: Some(Self::queue_position(request.priority, active.values())),
                active_cpu_percent: snapshot.cpu_percent,
                active_memory_mb: snapshot.memory_mb,
                active_power_mw: snapshot.power_mw,
            };
        }

        let request_id = request.request_id.clone();
        active.insert(request_id.clone(), request);

        BudgetDecision {
            request_id,
            granted: true,
            reason: "budget reserved".to_string(),
            queue_position: None,
            active_cpu_percent: next_cpu,
            active_memory_mb: next_memory,
            active_power_mw: next_power,
        }
    }

    pub async fn release(&self, request_id: &str) -> bool {
        self.active.write().await.remove(request_id).is_some()
    }

    pub async fn report(&self) -> BudgetSnapshot {
        let active = self.active.read().await;
        Self::snapshot_locked(&active)
    }

    pub fn order_by_priority<T, F>(items: &mut [T], priority: F)
    where
        F: Fn(&T) -> Priority,
    {
        items.sort_by_key(|item| std::cmp::Reverse(priority(item)));
    }

    fn snapshot_locked(active: &HashMap<String, BudgetRequest>) -> BudgetSnapshot {
        BudgetSnapshot {
            active_reservations: active.len(),
            cpu_percent: active
                .values()
                .fold(0u8, |acc, request| acc.saturating_add(request.cpu_percent)),
            memory_mb: active.values().map(|request| request.memory_mb).sum(),
            power_mw: active.values().map(|request| request.power_mw).sum(),
        }
    }

    fn queue_position<'a>(priority: Priority, active: impl Iterator<Item = &'a BudgetRequest>) -> usize {
        active.filter(|request| request.priority >= priority).count() + 1
    }
}

impl Default for BudgetEngine {
    fn default() -> Self {
        Self::new(BudgetLimits::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str, priority: Priority, memory_mb: u64) -> BudgetRequest {
        BudgetRequest {
            request_id: id.to_string(),
            priority,
            cpu_percent: 10,
            memory_mb,
            power_mw: 100,
            expected_duration_ms: 10,
        }
    }

    #[tokio::test]
    async fn enforces_memory_budget() {
        let engine = BudgetEngine::new(BudgetLimits {
            max_cpu_percent: 100,
            max_memory_mb: 64,
            max_power_mw: 1_000,
            max_concurrent: 2,
        });

        assert!(engine.reserve(request("a", Priority::Normal, 32)).await.granted);
        let denied = engine.reserve(request("b", Priority::High, 40)).await;
        assert!(!denied.granted);
        assert!(denied.reason.contains("memory budget"));
    }

    #[test]
    fn orders_urgent_first() {
        let mut items = vec![
            request("low", Priority::Low, 1),
            request("urgent", Priority::Urgent, 1),
            request("normal", Priority::Normal, 1),
        ];
        BudgetEngine::order_by_priority(&mut items, |item| item.priority);
        assert_eq!(items[0].request_id, "urgent");
    }
}
