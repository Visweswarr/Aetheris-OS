# Sprint 01 (2 weeks)

Objectives
- Solidify planning artifacts (this roadmap set) and security model rollup.
- Ship assistant v1 skeleton interfaces and CBOR transport smoke tests (reuse existing infra).
- Prepare stubs for app specs under /docs/specs/apps.

Scope & Deliverables
- MASTER_ROADMAP.md, backlog.csv, TRACKING.md (Kanban) committed under /docs.
- SECURITY_MODEL.md baseline created and reviewed (docs/security).
- Ensure intent client skeleton present and CBOR transport smoke tests green on Windows + Ubuntu.

Exit Criteria
- All above artifacts merged; tests pass in CI where applicable; owners assigned for Sprint 02.

Risks/Mitigations
- Toolchain gaps on Windows: use WSL/containers in CI; keep local dev instructions minimal.
- Scope creep: keep to planning + skeletons; push feature builds to Sprint 02.
