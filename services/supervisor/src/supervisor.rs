//! Core supervisor loop.

use crate::{ChildSpec, RestartPolicy, SupError, SupResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, warn};

/// OTP restart strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Strategy {
    /// Restart only the failed child.
    OneForOne,
    /// Restart all children when any one fails.
    OneForAll,
    /// Restart the failed child and every child started after it.
    RestForOne,
}

/// Runtime event emitted by a supervisor loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisorEvent {
    /// A child process started.
    Started {
        /// Child name from its `ChildSpec`.
        name: String,
    },
    /// A child process exited.
    Exited {
        /// Child name from its `ChildSpec`.
        name: String,
        /// Whether the process exited successfully.
        success: bool,
        /// Platform exit code when available.
        code: Option<i32>,
    },
    /// A child was restarted after exit.
    Restarted {
        /// Child name from its `ChildSpec`.
        name: String,
    },
    /// A child was intentionally not restarted by its policy.
    RestartSkipped {
        /// Child name from its `ChildSpec`.
        name: String,
        /// Restart policy that skipped the restart.
        policy: RestartPolicy,
    },
    /// The restart intensity limit was exceeded.
    RestartLimitExceeded {
        /// Number of restart attempts observed in the active window.
        restarts: usize,
        /// Restart intensity window in seconds.
        within_seconds: u64,
    },
    /// The supervisor is shutting down.
    Shutdown,
}

/// Handle returned by `Supervisor::start`. Drop to begin shutdown.
#[derive(Debug)]
pub struct SupervisorHandle {
    shutdown_tx: mpsc::Sender<()>,
    events_rx: mpsc::Receiver<SupervisorEvent>,
    join: tokio::task::JoinHandle<()>,
}

impl SupervisorHandle {
    /// Receive the next supervisor event.
    pub async fn next_event(&mut self) -> Option<SupervisorEvent> {
        self.events_rx.recv().await
    }

    /// Request graceful shutdown of the supervisor.
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(()).await;
        let _ = self.join.await;
    }
}

/// Supervisor specification — what to run and how to react to failure.
#[derive(Debug, Clone)]
pub struct Supervisor {
    /// Logical name used in logs.
    pub name: String,
    /// Restart strategy.
    pub strategy: Strategy,
    /// Maximum restarts inside the window.
    pub max_restarts: u32,
    /// Window length, seconds.
    pub within_seconds: u64,
    /// Children, in start order. Order matters for `RestForOne`.
    pub children: Vec<ChildSpec>,
}

impl Supervisor {
    /// Construct a supervisor with all children starting `Permanent` by default.
    pub fn new(name: impl Into<String>, strategy: Strategy) -> Self {
        Self {
            name: name.into(),
            strategy,
            max_restarts: 5,
            within_seconds: 60,
            children: Vec::new(),
        }
    }

    /// Builder: append a child.
    pub fn child(mut self, spec: ChildSpec) -> Self {
        self.children.push(spec);
        self
    }

    /// Builder: tune the restart window.
    pub fn max_restarts(mut self, count: u32, within_seconds: u64) -> Self {
        self.max_restarts = count;
        self.within_seconds = within_seconds;
        self
    }

    /// Start the supervisor loop. Returns a handle; drop or shutdown when done.
    pub fn start(self) -> SupResult<SupervisorHandle> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        let (events_tx, events_rx) = mpsc::channel::<SupervisorEvent>(128);
        let join = tokio::spawn(async move {
            run_loop(self, shutdown_rx, events_tx).await;
        });
        Ok(SupervisorHandle { shutdown_tx, events_rx, join })
    }
}

struct ChildState {
    spec: ChildSpec,
    process: Option<Child>,
}

fn spawn_child(spec: &ChildSpec) -> SupResult<Child> {
    let mut cmd = Command::new(&spec.command);
    cmd.args(&spec.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }
    cmd.spawn().map_err(|e| SupError::StartFailed {
        name: spec.name.clone(),
        reason: e.to_string(),
    })
}

async fn run_loop(
    sup: Supervisor,
    mut shutdown_rx: mpsc::Receiver<()>,
    events_tx: mpsc::Sender<SupervisorEvent>,
) {
    let mut children: Vec<ChildState> = Vec::with_capacity(sup.children.len());
    for spec in &sup.children {
        match spawn_child(spec) {
            Ok(process) => {
                info!(supervisor = %sup.name, child = %spec.name, "started");
                let _ = events_tx.send(SupervisorEvent::Started { name: spec.name.clone() }).await;
                children.push(ChildState {
                    spec: spec.clone(),
                    process: Some(process),
                });
            }
            Err(e) => {
                warn!(supervisor = %sup.name, child = %spec.name, error = %e, "failed to start");
                children.push(ChildState {
                    spec: spec.clone(),
                    process: None,
                });
            }
        }
    }

    let mut restart_history: VecDeque<Instant> = VecDeque::new();
    let window = Duration::from_secs(sup.within_seconds);
    let mut poll = tokio::time::interval(Duration::from_millis(250));

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!(supervisor = %sup.name, "shutting down");
                let _ = events_tx.send(SupervisorEvent::Shutdown).await;
                for c in &mut children {
                    if let Some(mut p) = c.process.take() {
                        let _ = p.kill();
                    }
                }
                break;
            }
            _ = poll.tick() => {
                let mut failed_indices = Vec::new();
                for (idx, c) in children.iter_mut().enumerate() {
                    if let Some(p) = c.process.as_mut() {
                        match p.try_wait() {
                            Ok(Some(status)) => {
                                let normal_exit = status.success();
                                let should_restart = match c.spec.restart {
                                    RestartPolicy::Permanent => true,
                                    RestartPolicy::Temporary => false,
                                    RestartPolicy::Transient => !normal_exit,
                                };
                                info!(
                                    supervisor = %sup.name,
                                    child = %c.spec.name,
                                    exit_code = ?status.code(),
                                    restart = should_restart,
                                    "child exited"
                                );
                                let _ = events_tx.send(SupervisorEvent::Exited {
                                    name: c.spec.name.clone(),
                                    success: normal_exit,
                                    code: status.code(),
                                }).await;
                                c.process = None;
                                if should_restart {
                                    failed_indices.push(idx);
                                } else {
                                    let _ = events_tx.send(SupervisorEvent::RestartSkipped {
                                        name: c.spec.name.clone(),
                                        policy: c.spec.restart,
                                    }).await;
                                }
                            }
                            Ok(None) => {} // still running
                            Err(e) => {
                                warn!(supervisor = %sup.name, child = %c.spec.name, error = %e, "try_wait failed");
                            }
                        }
                    }
                }

                if failed_indices.is_empty() {
                    continue;
                }

                // Apply strategy.
                let indices_to_restart: Vec<usize> = match sup.strategy {
                    Strategy::OneForOne => failed_indices.clone(),
                    Strategy::OneForAll => (0..children.len()).collect(),
                    Strategy::RestForOne => {
                        let first = *failed_indices.iter().min().unwrap();
                        (first..children.len()).collect()
                    }
                };

                // OneForAll/RestForOne require stopping non-failed siblings first.
                if !matches!(sup.strategy, Strategy::OneForOne) {
                    for &idx in &indices_to_restart {
                        if let Some(p) = children[idx].process.as_mut() {
                            let _ = p.kill();
                            children[idx].process = None;
                        }
                    }
                }

                // Rate-limit check.
                let now = Instant::now();
                restart_history.push_back(now);
                while let Some(front) = restart_history.front() {
                    if now.duration_since(*front) > window {
                        restart_history.pop_front();
                    } else {
                        break;
                    }
                }
                if restart_history.len() as u32 > sup.max_restarts {
                    warn!(
                        supervisor = %sup.name,
                        restarts = restart_history.len(),
                        window_s = sup.within_seconds,
                        "rate limit exceeded — supervisor giving up"
                    );
                    let _ = events_tx.send(SupervisorEvent::RestartLimitExceeded {
                        restarts: restart_history.len(),
                        within_seconds: sup.within_seconds,
                    }).await;
                    break;
                }

                for idx in indices_to_restart {
                    match spawn_child(&children[idx].spec) {
                        Ok(p) => {
                            info!(supervisor = %sup.name, child = %children[idx].spec.name, "restarted");
                            children[idx].process = Some(p);
                            let _ = events_tx.send(SupervisorEvent::Restarted {
                                name: children[idx].spec.name.clone(),
                            }).await;
                        }
                        Err(e) => {
                            warn!(
                                supervisor = %sup.name,
                                child = %children[idx].spec.name,
                                error = %e,
                                "restart failed"
                            );
                        }
                    }
                }
            }
        }
    }
}
