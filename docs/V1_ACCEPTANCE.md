# Version 1 Acceptance Checklist

This checklist documents the canonical smoke checks for version 1 policy behavior.

Version 1 is complete when the commands below produce the expected broker statuses, policy denials, and evidence behavior without expanding scope into approval workflows, sessions, or adapter changes.

## Scope

Version 1 proves:

- broker-layer policy runs after request validation and before subprocess spawn;
- allowed commands still follow the version 0 execution path;
- denied executable requests return structured broker rejections;
- outside-root `cwd` requests return structured broker rejections;
- allow and deny outcomes both persist machine-readable evidence with policy metadata.

Version 1 does not add:

- `require_approval` behavior;
- sessions or background process handles;
- MCP, JSON-RPC, or HTTP adapters;
- richer evidence retention or stdout/stderr persistence.

## Canonical Smoke Commands

Run these commands from the repository root:

```bash
cargo test
cargo run -- run --cwd . --timeout 5 -- echo hello
cargo run -- run --cwd . --timeout 30 -- false
cargo run -- run --cwd . --timeout 1 -- sleep 2
cargo run -- run --cwd . --timeout 5 -- rm -f /tmp/llm-shell-v1-acceptance
cargo run -- run --cwd .. --timeout 5 -- pwd
```

## Expected Outcomes

### `cargo test`

- exits successfully;
- covers the execution path, denial mapping, and evidence persistence.

### Success Case

Command:

```bash
cargo run -- run --cwd . --timeout 5 -- echo hello
```

Expected result:

- JSON `status` is `success`;
- `exit_code` is `0`;
- `stdout` is `hello\n`;
- `stderr` is empty;
- `timed_out` is `false`;
- the CLI exits with code `0`.

### Failed Case

Command:

```bash
cargo run -- run --cwd . --timeout 30 -- false
```

Expected result:

- JSON `status` is `failed`;
- `exit_code` is `1`;
- `timed_out` is `false`;
- the CLI exits non-zero.

### Timed-Out Case

Command:

```bash
cargo run -- run --cwd . --timeout 1 -- sleep 2
```

Expected result:

- JSON `status` is `timed_out`;
- `exit_code` is `null`;
- `timed_out` is `true`;
- `duration_ms` is roughly the requested timeout window;
- the CLI exits non-zero.

### Denied Executable Case

Command:

```bash
cargo run -- run --cwd . --timeout 5 -- rm -f /tmp/llm-shell-v1-acceptance
```

Expected result:

- JSON `status` is `denied`;
- `error.code` is `denied_executable`;
- `error.message` describes the executable rejection;
- `exit_code` is `null`;
- `stdout` and `stderr` are empty strings;
- `duration_ms` is `0`;
- `timed_out` is `false`;
- the CLI exits with code `1`;
- the broker rejects the request before subprocess spawn.

### Outside-Root Case

Command:

```bash
cargo run -- run --cwd .. --timeout 5 -- pwd
```

Expected result:

- JSON `status` is `denied`;
- `error.code` is `cwd_outside_workspace_root`;
- `error.message` describes the workspace-root restriction;
- `exit_code` is `null`;
- `stdout` and `stderr` are empty strings;
- `duration_ms` is `0`;
- `timed_out` is `false`;
- the CLI exits with code `1`;
- the broker rejects the request before subprocess spawn.

This is a policy denial, not malformed input. The request is valid, but its canonicalized `cwd` is outside the canonicalized startup workspace root.

## Evidence Location

Each request that reaches the broker writes one evidence file under the harness-owned temp directory:

```text
<temp-dir>/llm-shell/evidence/<timestamp>_<request_id>.json
```

How to locate it:

1. Run one of the smoke commands and copy the returned `request_id`.
2. Resolve the temp root with `python3 -c 'import tempfile; print(tempfile.gettempdir())'`.
3. Inspect `<temp-dir>/llm-shell/evidence/` for the file whose name ends with `_<request_id>.json`.

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
- `policy_decision`;
- `policy_reason` for denied requests;
- `timestamp`.

Expected event behavior:

- allowed executions persist `event_type: "execution.completed"` with `policy_decision: "allow"`;
- denied requests persist `event_type: "execution.denied"` with `policy_decision: "deny"` and the denial reason code in `policy_reason`;
- version 1 still does not persist full `stdout` or `stderr`.
