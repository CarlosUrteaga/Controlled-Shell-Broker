# Phase 3A Steps

This document sequences Phase 3A work for the Controlled Shell Broker.

Phase 3A is defined in the roadmap as:

> Durable Evidence And Retention

The goal is to strengthen evidence storage and retention behavior after version 1 policy without changing the caller-facing execution contract.

Phase 3A should not expand into open inspection mode, command-semantic reinterpretation, sessions, sandbox profiles, or protocol adapters.

## Locked Defaults

- evidence remains machine-readable and separate from caller-facing JSON
- evidence storage stays broker-owned and outside the target workspace
- no stdout or stderr persistence is added unless explicitly documented later
- denied and allowed requests continue to use the existing version 1 result contract
- evidence changes must not silently redefine CLI behavior

## PR 1 — Close The Phase 3A Evidence Contract

**Type:** docs-only

**Requirement:** define the Phase 3A source-of-truth contract for durable evidence storage, retention expectations, and unchanged caller-visible execution behavior.

**Expected files to change:**

- `docs/REQUEST_RESPONSE_SCHEMA.md`
- `docs/CLI_CONTRACT.md`
- `docs/SECURITY_MODEL.md`
- `docs/ARCHITECTURE.md`
- `docs/TESTING.md`
- `docs/DECISIONS.md`

**Tests to run:**

- none
- manual doc consistency review across the files above

**Acceptance criteria:**

- the evidence storage contract is explicit about replacing temp-directory storage or keeping it temporarily
- retention expectations are defined clearly enough for implementation
- evidence metadata boundaries are explicit without requiring stdout or stderr persistence
- caller-visible behavior that remains unchanged is stated explicitly
- open inspection mode and command semantics remain out of scope for Phase 3A

**PR summary:** define the durable evidence contract before implementation.

## PR 2 — Move Evidence Storage To A Durable Broker-Owned State Directory

**Type:** implementation

**Requirement:** replace temporary evidence storage with a durable broker-owned state directory without changing the caller-facing execution-result schema.

**Expected files to change:**

- `src/evidence.rs`
- `src/exec.rs`
- `src/types.rs` only if implementation needs additional evidence metadata
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- evidence is no longer written under the operating-system temporary directory by default
- evidence stays outside the target workspace
- one evidence record is still written per execution attempt that reaches the broker
- execution-result JSON stays unchanged unless Phase 3A docs explicitly require otherwise
- evidence-write failures still map to structured `execution_error`

**PR summary:** make evidence durable without changing the execution contract.

## PR 3 — Add Retention Structure And Evidence Lifecycle Rules

**Type:** implementation

**Requirement:** implement the minimum retention and layout behavior required by the Phase 3A docs without turning the broker into a full log-management system.

**Expected files to change:**

- `src/evidence.rs`
- `src/types.rs` only if required by evidence metadata changes
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- the on-disk evidence layout matches the documented directory and naming strategy
- retention behavior is deterministic and testable
- cleanup rules, if introduced, apply only to broker-owned evidence paths
- no target-workspace files are mutated as part of evidence retention
- no stdout or stderr persistence is added unless separately documented and approved

**PR summary:** implement the minimum evidence lifecycle needed for durable storage to stay reviewable and bounded.

## PR 4 — Add `docs/PHASE3_ACCEPTANCE.md`

**Type:** docs-only

**Requirement:** add the canonical smoke checklist for Phase 3A evidence behavior after the storage and retention work lands.

**Expected files to change:**

- `docs/PHASE3_ACCEPTANCE.md`
- `docs/ROADMAP.md` if it should reference the new checklist
- `docs/TESTING.md` only if needed for consistency

**Tests to run:**

- none
- manual validation command review

**Acceptance criteria:**

- the checklist covers success, failure, timeout, denied, and evidence-write expectations
- the expected evidence location and persistence behavior are explicit
- the checklist stays focused on Phase 3A evidence scope
- Phase 3B open inspection mode is not implied by the checklist

**PR summary:** define the canonical verification path for durable evidence behavior.

## Test Scenarios To Include Across The Sequence

- `cargo run -- run --cwd . --timeout 5 -- echo hello` -> `success` and evidence persists in the durable broker-owned location
- `cargo run -- run --cwd . --timeout 30 -- false` -> `failed` and evidence persists in the durable broker-owned location
- `cargo run -- run --cwd . --timeout 1 -- sleep 2` -> `timed_out` and evidence persists in the durable broker-owned location
- `cargo run -- run --cwd . --timeout 30 -- rm -rf .` -> `denied` and denied evidence persists in the durable broker-owned location
- broker evidence-path failure injection -> `execution_error` / `evidence_write_failed`

## Assumptions

- Phase 3A begins after version 1 policy behavior and evidence audit metadata are already implemented
- durable evidence storage should be solved before open inspection mode is attempted
- command semantics remain unchanged throughout Phase 3A
- sessions, sandbox profiles, JSON adapters, MCP, JSON-RPC, and HTTP remain out of scope
- if durable evidence remains sufficient without broader logging changes, Phase 3A ends without adding output persistence
