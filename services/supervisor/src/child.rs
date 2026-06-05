//! Child-process specification.

use serde::{Deserialize, Serialize};

/// Restart policy for a single child. Mirrors Erlang/OTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RestartPolicy {
    /// Always restart on exit, regardless of exit reason.
    Permanent,
    /// Never restart on exit.
    Temporary,
    /// Restart only on abnormal exit (nonzero exit code or panic).
    Transient,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::Permanent
    }
}

/// Declarative description of a child managed by a supervisor.
///
/// The `command` is shelled out via `std::process::Command` — a child does not
/// need to be Rust. This is how the supervisor can supervise the Go services in
/// `services/net_go/`, `services/fs_go/`, etc., per `LANGUAGES.md` § 2.3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildSpec {
    /// Stable identifier shown in logs and metrics.
    pub name: String,
    /// Command to spawn (argv[0]) — absolute path recommended.
    pub command: String,
    /// Arguments passed after `command`.
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment-variable overrides.
    #[serde(default)]
    pub env: Vec<(String, String)>,
    /// Restart policy. Defaults to `Permanent`.
    #[serde(default)]
    pub restart: RestartPolicy,
    /// Soft shutdown timeout (seconds). After this, the child is killed.
    #[serde(default = "default_shutdown_secs")]
    pub shutdown_secs: u64,
}

fn default_shutdown_secs() -> u64 {
    5
}

impl ChildSpec {
    /// Construct a permanent child with no env overrides.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            restart: RestartPolicy::Permanent,
            shutdown_secs: default_shutdown_secs(),
        }
    }

    /// Builder: set restart policy.
    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart = policy;
        self
    }

    /// Builder: set args.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }
}
