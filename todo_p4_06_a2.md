# P4-06-A2: Avatar DID Binding + CapToken Gating + DAO-Governed Scene Edits

## Status: In Progress
## Phase: Phase 4 — XR/Metaverse Surface (Step 2 of 4: A1 → A2 → A3 → A4)

### Deliverables

#### Rust Core Service
- [ ] `services/xr/scene/session.rs` - DID session lifecycle management
- [ ] `services/xr/scene/cap.rs` - Extend CapToken v2 scopes & checks
- [ ] `services/xr/scene/policy.rs` - Extend DAO hook + policy attach/list/get
- [ ] `services/xr/scene/dao_hook.rs` - Adapter to services/dao
- [ ] `services/xr/scene/rpc.rs` - New RPCs + authz path + events
- [ ] `services/xr/scene/tests/*.rs` - Unit + integration tests

#### C++ (Godot Module)
- [ ] `ui/xr/godot_modules/aetheris/src/aetheris_auth.cpp` - Bind session/cap to runtime
- [ ] `ui/xr/godot_modules/aetheris/include/aetheris_auth.hpp` - Auth header
- [ ] `ui/xr/godot_app/scripts/AetherisScene.gd` - Update to call auth paths

#### TypeScript Bridge
- [ ] `tooling/ts/xr_bridge.ts` - Add beginSession/endSession/issueCap + gated ops

#### Go CLI
- [ ] `go/tooling/xrctl/main.go` - Add session/cap commands + policy apply/check

#### Python Validator
- [ ] `tooling/python/xr_validator.py` - New validations for auth/caps/dao/snapshot

#### Documentation & CI
- [ ] `docs/phase-4/P4-06-A2-AVATAR-CAPS-DAO.md` - Comprehensive documentation
- [ ] `.github/workflows/p4-06-xr-a2.yml` - CI workflow

### Key Requirements
- ✅ All mutating RPCs: MUST verify active DID session + CapToken scope + DAO decision
- ✅ Deny-by-default; explicit allow via policy or DAO vote
- ✅ Audit every decision (who/what/why/policy_ref) with monotonic ticks
- ✅ Deterministic tick loop; same inputs → identical snapshot bytes
- ✅ p95 'spawn node → visible' ≤ 120ms (headless)
- ✅ No >10% regression against P4-06-A1 baselines
- ✅ Polyglot parity across Rust/C++/TS/Go/Python
- ✅ CI runs unit + integration + validator in < 15 min

### New API Changes
- `BeginSession{did, proof, nonce} → SessionId`
- `EndSession{session_id} → Ack`
- `IssueCap{session_id, scopes, ttl} → CapToken`
- `SpawnNode{session_id, cap, spec} → NodeId`
- `MoveNode{session_id, cap, node_id, transform} → Ack`
- `DeleteNode{session_id, cap, node_id} → Ack`
- `AttachPolicy{session_id, cap, scope, rego_bundle_cbor} → PolicyId`
- `Simulate{session_id, ops[]} → DecisionReport{per-op allow/deny reasons}`
- `SnapshotSave{label, anchor?:bool} → SnapshotId`
- `SnapshotLoad{snapshot_id} → Ack`

### Event Stream Events
- `policy_denied{op, reason, policy_ref}`
- `avatar_session_started/ended`
- `cap_issued/expired`
