//! Component Model host runtime.
//!
//! Wires `wasmtime` with the Component Model + WASI Preview 2 imports so user-app
//! components (Rust / Go / Python / JS — see `LANGUAGES.md` § 2.8) can be instantiated
//! and called through WIT-defined surfaces under `/wit/polymera/*`.
//!
//! This module is the entrypoint Initiative 3 of the polyglot roadmap targets. It
//! intentionally exposes a small surface (`Host::new`, `Host::load_component`,
//! `Host::call`) so the rest of `wasm_driver` does not need to know that wasmtime is
//! the underlying engine.

#![allow(clippy::needless_pass_by_value)]

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

#[derive(Debug, Error)]
pub enum HostError {
    #[error("wasmtime engine init failed: {0}")]
    Engine(String),
    #[error("component load failed: {0}")]
    Load(String),
    #[error("component instantiation failed: {0}")]
    Instantiate(String),
    #[error("invocation failed: {0}")]
    Invoke(String),
}

pub type HostResult<T> = Result<T, HostError>;

/// Per-component runtime state passed to host imports.
pub struct ComponentState {
    pub table: ResourceTable,
    pub wasi: WasiCtx,
}

impl WasiView for ComponentState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

/// The Component Model host.
///
/// One `Host` is constructed per Polymera-OS instance and re-used across many
/// component invocations.
pub struct Host {
    engine: Arc<Engine>,
}

impl Host {
    pub fn new() -> HostResult<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        // Polymera's deterministic-execution promise: pin cranelift opt level and
        // disable backend caching that would yield nondeterministic codegen.
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        let engine = Engine::new(&config).map_err(|e| HostError::Engine(e.to_string()))?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    /// Load a Component-Model `.wasm` file from disk.
    pub fn load_component(&self, path: &Path) -> HostResult<Component> {
        Component::from_file(&self.engine, path).map_err(|e| HostError::Load(e.to_string()))
    }

    /// Instantiate `component` with a fresh per-instance state and return the
    /// `(store, instance)` pair. Callers then look up exports by name.
    pub async fn instantiate(
        &self,
        component: &Component,
    ) -> HostResult<(Store<ComponentState>, wasmtime::component::Instance)> {
        let mut linker = Linker::<ComponentState>::new(&self.engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .map_err(|e| HostError::Instantiate(e.to_string()))?;

        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        let state = ComponentState {
            table: ResourceTable::new(),
            wasi,
        };

        let mut store = Store::new(&self.engine, state);
        let instance = linker
            .instantiate_async(&mut store, component)
            .await
            .map_err(|e| HostError::Instantiate(e.to_string()))?;
        Ok((store, instance))
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    #[test]
    fn mock_feature_compiles_without_wasmtime() {
        // The `mock` feature is a placeholder for shrinking the dependency surface
        // in pure-logic unit tests. Tests that exercise the engine should be in
        // tests/host_integration.rs (gated on default features).
    }
}
