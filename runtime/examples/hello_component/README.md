# hello_component

A minimal Component Model "hello world" for Polymera OS. Demonstrates the
Initiative 3 path: source code in any language → `.wasm` component →
loaded by `services/wasm_driver/src/host.rs`.

## Build

Requires `cargo-component` (`cargo install cargo-component`).

```bash
cd runtime/examples/hello_component
cargo component build --release
```

Output: `target/wasm32-wasip2/release/hello_component.wasm`

## Run via the Polymera host (manual smoke test)

```rust
use wasm_driver_host::host::Host;
let host = Host::new()?;
let component = host.load_component(
    "runtime/examples/hello_component/target/wasm32-wasip2/release/hello_component.wasm".as_ref()
)?;
let (mut store, instance) = host.instantiate(&component).await?;
// Look up `greeter.greet` and call it — see services/wasm_driver/tests/ for full example.
```

## Same interface, other languages

Per the polyglot roadmap Initiative 3, the same `wit/world.wit` works for components
authored in Go (TinyGo), Python (`componentize-py`), or JS (`jco componentize`).
The host doesn't care about the source language.
