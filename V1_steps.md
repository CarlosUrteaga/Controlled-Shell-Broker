# V1 Steps

This document sequences version 1 work for the Controlled Shell Broker.

Version 1 is defined as:

> Basic Policy / Admission Control

The goal is to add broker-layer policy after request validation and before command startup, without expanding scope into sessions, protocol adapters, approval UI, or command-semantic interpretation.

## Locked Defaults

- policy rejection is a new caller-visible `status: "denied"`
- the first workspace root is the broker startup cwd, canonicalized once
- denied requests produce machine-readable evidence
- `require_approval` remains documented but unimplemented in v1

## PR 1 — Close The V1 Policy Contract

**Type:** docs-only

**Requirement:** refine the existing policy model and related source-of-truth docs so the first implementation slice is fully specified.

**Expected files to change:**

- `docs/POLICY_MODEL.md`
- `docs/REQUEST_RESPONSE_SCHEMA.md`
- `docs/CLI_CONTRACT.md`
- `docs/SECURITY_MODEL.md`
- `docs/ARCHITECTURE.md`
- `docs/DECISIONS.md`

**Tests to run:**

- none
- manual doc consistency review across the files above

**Acceptance criteria:**

- `denied` is defined as a new status
- the denied result envelope is defined
- workspace-root restriction is defined as policy, not CLI validation
- denied evidence behavior is defined
- `require_approval` remains deferred

**PR summary:** define caller-visible denial semantics and policy audit expectations before code.

## PR 2 — Add Policy Seam With Allow-By-Default Behavior

**Type:** implementation

**Requirement:** add a broker-side policy module and route every valid request through it before execution, without changing behavior yet.

**Expected files to change:**

- `src/main.rs`
- `src/exec.rs`
- `src/policy.rs`
- `src/types.rs`
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- valid requests are evaluated by policy before subprocess spawn
- only `allow` is returned in this PR
- existing v0 execution behavior stays unchanged
- startup workspace root is resolved once and passed into policy context

**PR summary:** introduce the policy checkpoint without introducing denial yet.

## PR 3 — Add Structured Denied Responses

**Type:** implementation

**Requirement:** support policy rejection as a structured broker result without spawning a process.

**Expected files to change:**

- `src/exec.rs`
- `src/policy.rs`
- `src/types.rs`
- `docs/REQUEST_RESPONSE_SCHEMA.md`
- `docs/CLI_CONTRACT.md`
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- policy denial returns `status: "denied"`
- denied responses include `error.code` and `error.message`
- denied responses use `exit_code: null`, empty `stdout` and `stderr`, `duration_ms: 0`, `timed_out: false`
- denied CLI exit code is non-zero and fixed at `1`
- denied requests do not spawn a process

**PR summary:** make policy rejection caller-visible and distinct from `invalid_request` and `execution_error`.

## PR 4 — Add Workspace-Root Restriction

**Type:** implementation

**Requirement:** deny requests whose canonicalized `cwd` is outside the canonicalized startup workspace root.

**Expected files to change:**

- `src/main.rs`
- `src/exec.rs`
- `src/policy.rs`
- `src/types.rs`
- `docs/POLICY_MODEL.md`
- `docs/SECURITY_MODEL.md`
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- the root path itself is allowed
- descendant paths are allowed
- paths outside the root are denied before execution
- the denial code is `cwd_outside_workspace_root`
- CLI validation behavior for malformed `--cwd` is unchanged

**PR summary:** add the first real admission-control rule around workspace scope.

## PR 5 — Add Minimal Denied-Executable Rules

**Type:** implementation

**Requirement:** deny a small exact-basename set of dangerous executables.

**Expected files to change:**

- `src/policy.rs`
- `src/exec.rs`
- `src/types.rs`
- `docs/POLICY_MODEL.md`
- `docs/SECURITY_MODEL.md`
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- the denylist is exactly: `rm`, `sudo`, `su`, `shutdown`, `reboot`, `mkfs`, `dd`
- basename matching denies both `rm` and `/bin/rm`
- matching is exact, not substring-based
- the denial code is `denied_executable`
- allowed commands still follow the v0 execution path

**PR summary:** add the first static dangerous-command policy without inspecting command semantics beyond `command[0]`.

## PR 6 — Add Policy Metadata To Evidence

**Type:** implementation

**Requirement:** make allow/deny policy outcomes auditable in machine-readable evidence.

**Expected files to change:**

- `src/evidence.rs`
- `src/exec.rs`
- `src/policy.rs`
- `src/types.rs`
- `docs/REQUEST_RESPONSE_SCHEMA.md`
- `docs/DECISIONS.md`

**Tests to run:**

- `cargo fmt --check`
- `cargo test`

**Acceptance criteria:**

- executed requests persist `policy_decision: "allow"`
- denied requests persist evidence with `event_type: "execution.denied"`
- denied evidence includes `policy_decision: "deny"` and `policy_reason`
- no stdout or stderr persistence is added
- storage location and retention behavior stay unchanged

**PR summary:** extend evidence only enough to audit policy, without turning v1 into an evidence redesign.

## PR 7 — Add `docs/V1_ACCEPTANCE.md`

**Type:** docs-only

**Requirement:** add the canonical smoke checklist for v1 policy behavior.

**Expected files to change:**

- `docs/V1_ACCEPTANCE.md`
- `docs/ROADMAP.md` if it should reference the new checklist
- `docs/TESTING.md` only if needed for consistency

**Tests to run:**

- none
- manual validation command review

**Acceptance criteria:**

- the checklist includes success, failure, timeout, denied-executable, and outside-root examples
- expected statuses are explicit for each command
- denied commands are shown as broker rejections, not malformed input
- the checklist stays limited to v1 policy scope

**PR summary:** define the canonical verification path for v1 once implementation lands.

## Test Scenarios To Include Across The Sequence

- `cargo run -- run --cwd . --timeout 5 -- echo hello` -> `success`
- `cargo run -- run --cwd . --timeout 30 -- false` -> `failed`
- `cargo run -- run --cwd . --timeout 1 -- sleep 2` -> `timed_out`
- `cargo run -- run --cwd . --timeout 30 -- rm -rf .` -> `denied` / `denied_executable`
- `cargo run -- run --cwd . --timeout 30 -- /bin/rm -rf .` -> `denied` / `denied_executable`
- `cargo run -- run --cwd / --timeout 30 -- pwd` from repo root -> `denied` / `cwd_outside_workspace_root`

## Assumptions

- v1 does not add sessions, JSON adapters, MCP, approval hooks, environment shaping, or command-semantic interpretation
- workspace-root restriction is based on the broker startup cwd, not a new config source
- reason-specific denial codes are preferred over a single generic `policy_denied` code
