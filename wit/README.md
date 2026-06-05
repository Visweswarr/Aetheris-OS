# Polymera OS — WIT Interfaces

This directory is the **canonical home** for WebAssembly Interface Type definitions used by
Polymera OS. It is the IDL referenced by `LANGUAGES.md` § 1 and by the polyglot roadmap
(`docs/polyglot/POLYGLOT-ROADMAP-TRACKING.md` § Initiative 2).

## Status

**Phase 1 — foundation.** `services/wasm_driver` now has a pinned Wasmtime Component
Model host scaffold, and this directory contains the first component-facing AI contract.
No generated bindings are committed yet. This directory exists so that:

1. Future PRs have a single place to land new WIT files.
2. The roadmap's "no new IDL outside this directory" rule has somewhere to point.
3. CI can wire in repo-pinned `wit-bindgen` and `wasm-tools` against a fixed root.

## Layout

```
wit/
├── README.md                    # this file
└── polymera/
    ├── ai/
    │   └── 0.1.0/
    │       └── world.wit        # AI planning/tool/memory facade over ai_core.proto
    └── fs/
        └── 0.1.0/
            ├── world.wit        # 'fs-service' world export
            └── filesystem.wit   # interface definition
```

Package naming follows the WIT convention `polymera:<area>@<semver>`. One package per
service-shaped surface. Version directories make breaking changes auditable — additions go
in the same version, breaking changes bump.

## Adding a new interface

1. Pick a package name `polymera:<area>` (e.g. `polymera:net`, `polymera:display`).
2. Create `wit/polymera/<area>/<semver>/` with at least one `world.wit` and one `*.wit`
   defining the interface.
3. Update `docs/polyglot/POLYGLOT-ROADMAP-TRACKING.md` § Initiative 2 with the new entry.
4. Wire bindings into the consuming service via `wit-bindgen` (target — Phase 1 milestone 2).

## Tooling expectations

- `scripts/bootstrap-polyglot-tools.*` installs repo-local pinned copies of
  `wasm-tools`, `wit-bindgen-cli`, `cargo-component`, `jco`, and `componentize-py`.
- TinyGo + `wasm-tools component new` — Go side.

Do **not** install these in the kernel build path — components are built and shipped
separately from kernel artifacts.
