//! Polymera Supervisor — OTP-style supervision trees for Rust services.
//!
//! Initiative 6 of the polyglot roadmap. Provides the three classic Erlang/OTP
//! restart strategies (one-for-one, one-for-all, rest-for-one), per-child restart
//! policies (permanent, temporary, transient), and a sliding-window rate limit on
//! restarts so a flapping child does not loop forever.
//!
//! The existing `services/service_manager/service_manager.go` covers simple
//! lifecycle and idle suspension. This crate is the missing **structured fault
//! recovery** layer, callable from any Rust service (including via `Command::spawn`
//! shelling out to non-Rust binaries).

#![deny(missing_docs)]

pub mod child;
pub mod config;
pub mod error;
pub mod supervisor;

pub use child::{ChildSpec, RestartPolicy};
pub use config::SupervisorConfig;
pub use error::{SupError, SupResult};
pub use supervisor::{Strategy, Supervisor, SupervisorEvent, SupervisorHandle};
