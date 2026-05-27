# Phase 3A Acceptance Checklist

This checklist documents the canonical smoke checks for Phase 3A durable evidence and retention behavior.

Phase 3A is complete when the commands below preserve the version 1 execution contract while writing metadata-only evidence to durable broker-owned date buckets with bounded retention.

## Scope

Phase 3A proves:

- durable broker-owned per-user evidence storage;
- UTC date-bucket evidence layout;
- metadata-only evidence for success, failure, timeout, and denial outcomes;
- retention cleanup scoped to date-named broker-owned evidence directories;
- structured evidence failure for invalid retention configuration.

Phase 3A does not add CLI flags, stdout or stderr evidence persistence, sessions, open inspection mode, MCP, JSON-RPC, HTTP, or other adapter surfaces.

## Canonical Smoke Commands

Run these commands from the repository root:

```bash
cargo test
cargo run -- run --cwd . --timeout 5 -- echo hello
cargo run -- run --cwd . --timeout 30 -- false
cargo run -- run --cwd . --timeout 1 -- sleep 2
cargo run -- run --cwd . --timeout 5 -- rm -f /tmp/llm-shell-phase3a-acceptance
LLM_SHELL_EVIDENCE_RETENTION_DAYS=0 cargo run -- run --cwd . --timeout 5 -- echo hello
```

Expected result summary:

- `cargo test` exits successfully.
- `echo hello` returns `status: "success"`, `exit_code: 0`, `stdout: "hello\n"`, empty `stderr`, `timed_out: false`, and CLI exit code `0`.
- `false` returns `status: "failed"`, `exit_code: 1`, `timed_out: false`, and a non-zero CLI exit.
- `sleep 2` with timeout `1` returns `status: "timed_out"`, `exit_code: null`, `timed_out: true`, a duration near the timeout window, and a non-zero CLI exit.
- denied `rm` returns `status: "denied"`, `error.code: "denied_executable"`, `exit_code: null`, empty `stdout` and `stderr`, `duration_ms: 0`, `timed_out: false`, and CLI exit code `1`.
- invalid `LLM_SHELL_EVIDENCE_RETENTION_DAYS=0` returns `status: "execution_error"` with `error.code: "evidence_write_failed"` and a non-zero CLI exit.

Each valid request that reaches the broker should write one evidence file, including denied requests. Invalid retention configuration is a harness-owned evidence lifecycle failure, not command failure.

## Evidence Location

Phase 3A writes evidence under the platform-specific broker-owned state directory documented in [CLI_CONTRACT.md](CLI_CONTRACT.md):

- macOS: `$HOME/Library/Application Support/llm-shell`;
- Windows: `%LOCALAPPDATA%/llm-shell`, falling back to `%APPDATA%/llm-shell`;
- other Unix platforms: absolute `$XDG_STATE_HOME/llm-shell`, falling back to `$HOME/.local/state/llm-shell`.

Evidence files use this layout:

```text
<state-dir>/evidence/YYYY-MM-DD/<timestamp>_<request_id>.json
```

How to locate an evidence file:

1. Run one smoke command and copy the returned `request_id`.
2. Resolve the platform state root using the rules above.
3. Inspect `<state-dir>/evidence/YYYY-MM-DD/` for the file whose name ends with `_<request_id>.json`.

## Evidence Expectations

Expected fields include:

- `event_type`;
- `request_id`;
- `operation`;
- `cwd`;
- `command`;
- `status`;
- `exit_code`;
- `duration_ms`;
- `timed_out`;
- `timestamp`;
- `error_code` when applicable;
- `policy_decision`;
- `policy_reason` for denied requests.

Expected behavior:

- allowed executions persist `event_type: "execution.completed"` with `policy_decision: "allow"`;
- denied requests persist `event_type: "execution.denied"` with `policy_decision: "deny"` and a denial reason in `policy_reason`;
- persisted evidence does not include full `stdout` or `stderr`;
- evidence is written outside the target workspace selected by `--cwd`.

## Retention Cleanup Smoke

Use a disposable broker-owned evidence root before running this check, and do not create fixtures in the target workspace.

Suggested manual flow:

1. Resolve the current platform state root.
2. Create `<state-dir>/evidence/2000-01-01/manual-old-event.json`.
3. Create `<state-dir>/evidence/evidence-old/manual-non-date-event.json`.
4. Run `cargo run -- run --cwd . --timeout 5 -- echo hello`.
5. Confirm `2000-01-01` was removed by retention cleanup.
6. Confirm `evidence-old` was ignored.
7. Confirm the new evidence file exists in the current UTC date bucket.

Retention defaults to 30 UTC date buckets. `LLM_SHELL_EVIDENCE_RETENTION_DAYS` may override the window when set to a positive integer.

## Evidence Write Failure Expectations

Evidence write or lifecycle failures map to `execution_error` with error code `evidence_write_failed`.

Representative failure modes include invalid `LLM_SHELL_EVIDENCE_RETENTION_DAYS`, missing required platform state-directory environment, and inability to create, write, or clean broker-owned evidence paths.

These failures do not add caller-visible status values and do not change command-specific success or failure semantics.
