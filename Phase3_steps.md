# Phase 3 Steps

This document sequences Phase 3 work for the Controlled Shell Broker.

Phase 3 is split into:

> Phase 3A: Durable Evidence And Retention
>
> Phase 3B: Open Inspection Mode

The Phase 3 goal is to strengthen evidence storage and retention behavior after version 1 policy, then add a read-oriented inspection mode that remains broker-governed and measurable.

Command semantics remain a separate decision after Phase 3A and Phase 3B produce enough evidence to justify any reinterpretation beyond generic process semantics.

Phase 3 should not expand into sessions, sandbox profiles, or protocol adapters.

## Locked Defaults

- evidence remains machine-readable and separate from caller-facing JSON
- evidence storage stays broker-owned and outside the target workspace
- open inspection mode stays read-oriented and policy-bounded
- open inspection mode reuses the existing broker execution contract instead of replacing it
- command semantics remain generic unless a specific insufficiency is demonstrated
- no CLI transport expansion is part of this phase
- no shell-string execution is added in this phase

## PR 1 — Close The Phase 3A Evidence Contract

**Type:** docs-only

**Requirement:** define the Phase 3A source-of-truth contract for durable evidence storage, retention expectations, and the decision boundary for later inspection work.

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

- the evidence storage contract is explicit about whether temp-directory storage is replaced
- retention expectations are defined clearly enough for implementation
- evidence metadata boundaries are defined without requiring stdout or stderr persistence
- the inspection-mode boundary is explicit: later inspection workflows remain layered above the broker contract
- caller-visible behavior that remains unchanged is stated explicitly

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
- execution-result JSON stays unchanged unless Phase 3 docs explicitly require otherwise
- evidence-write failures still map to structured `execution_error`

**PR summary:** make evidence durable without changing the execution contract.

## PR 3 — Add Retention Structure And Evidence Lifecycle Rules

**Type:** implementation

**Requirement:** implement the minimum retention and layout behavior required by the Phase 3 docs without turning the broker into a full log-management system.

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
- open inspection mode is not implied by the checklist unless a later PR adds it

**PR summary:** define the canonical verification path for durable evidence behavior.

## PR 5 — Close The Phase 3B Open Inspection Contract

**Type:** docs-only

**Requirement:** define the read-oriented inspection profile, command boundary, and evidence goals for open repository inspection without changing the base `run` result contract.

**Expected files to change:**

- `docs/PRODUCT_SPEC.md`
- `docs/ARCHITECTURE.md`
- `docs/CLI_CONTRACT.md` only if caller-visible behavior would change
- `docs/REQUEST_RESPONSE_SCHEMA.md` only if result shapes or evidence fields would change
- `docs/SECURITY_MODEL.md`
- `docs/TESTING.md`
- `docs/BACKLOG.md`
- `docs/DECISIONS.md`

**Tests to run:**

- none
- manual design review against current broker and policy semantics

**Acceptance criteria:**

- the inspection profile is explicitly read-oriented and broker-governed
- the initial exploration scope is documented in terms of policy and evidence rather than ad hoc shell freedom
- unchanged base execution behavior is still documented for ordinary `run` requests
- higher-level inspection primitives remain deferred unless open inspection evidence later justifies them

**PR summary:** define open inspection mode as a layered product capability rather than a silent expansion of the shell contract.

## PR 6 — Add Inspection Metrics To Evidence

**Type:** implementation

**Requirement:** extend evidence just enough to compare agent inspection behavior during read-oriented repository exploration without redesigning the execution result schema.

**Expected files to change:**

- `src/exec.rs`
- `src/evidence.rs`
- `src/types.rs`
- `docs/DECISIONS.md`
- any source-of-truth docs approved in PR 5

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- the added evidence fields are limited to inspection-oriented metrics approved in the docs
- ordinary execution results remain stable unless the docs explicitly changed them
- denied and non-inspection executions still produce valid evidence
- automated tests cover the new evidence metadata without reopening unrelated execution behavior

**PR summary:** make open inspection mode measurable without redefining the base broker result contract.

## PR 7 — Add A Read-Oriented Inspection Policy Profile

**Type:** implementation

**Requirement:** implement the initial open inspection mode policy boundary so repository exploration stays read-oriented, bounded, and auditable.

**Expected files to change:**

- `src/policy.rs`
- `src/exec.rs`
- `src/types.rs` only if needed for policy or evidence metadata
- `docs/DECISIONS.md`
- any source-of-truth docs approved in PR 5

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- the documented read-oriented inspection command profile is enforced before subprocess spawn
- destructive, privilege-escalating, or network-oriented commands remain denied in this profile
- inspection requests stay within the workspace-root boundary and existing timeout rules
- automated tests cover both allowed inspection commands and denied out-of-profile commands

**PR summary:** add a controlled open inspection mode without replacing the broker execution path.

## PR 8 — Decide Whether Derived Inspection Primitives Are Needed

**Type:** docs-only

**Requirement:** review open inspection evidence and determine whether repeated agent behavior justifies introducing higher-level inspection primitives.

**Expected files to change:**

- `docs/PRODUCT_SPEC.md`
- `docs/ARCHITECTURE.md`
- `docs/BACKLOG.md`
- `docs/TESTING.md`
- `docs/DECISIONS.md`

**Tests to run:**

- none
- manual design review against collected inspection evidence

**Acceptance criteria:**

- at least one concrete repeated insufficiency is documented if derived primitives are approved
- unchanged broker execution behavior remains the default for commands outside the approved primitive scope
- if no sufficient justification exists, the docs explicitly defer derived primitives and command semantics again

**PR summary:** require empirical justification before adding higher-level inspection operations or command reinterpretation.

## Test Scenarios To Include Across The Sequence

- `cargo run -- run --cwd . --timeout 5 -- echo hello` -> `success` and evidence persists in the durable broker-owned location
- `cargo run -- run --cwd . --timeout 30 -- false` -> `failed` and evidence persists in the durable broker-owned location
- `cargo run -- run --cwd . --timeout 1 -- sleep 2` -> `timed_out` and evidence persists in the durable broker-owned location
- `cargo run -- run --cwd . --timeout 30 -- rm -rf .` -> `denied` and denied evidence persists in the durable broker-owned location
- broker evidence-path failure injection -> `execution_error` / `evidence_write_failed`
- if open inspection mode is implemented, include at least one allowed repository-search command and one denied out-of-profile command
- if derived primitives or command semantics are later approved, include one explicit before-and-after scenario proving why generic exit behavior or open inspection mode was insufficient

## Assumptions

- Phase 3 begins after version 1 policy behavior and evidence audit metadata are already implemented
- durable evidence storage should be solved before any command-semantics work is attempted
- open inspection mode should be solved before any higher-level inspection primitives are attempted
- command semantics are optional after Phase 3B and require explicit justification
- sessions, sandbox profiles, JSON adapters, MCP, JSON-RPC, and HTTP remain out of scope
- if Phase 3 concludes that open inspection mode is sufficient, derived primitives and command semantics remain deferred
