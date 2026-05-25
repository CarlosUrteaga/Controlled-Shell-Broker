# Version 0 Acceptance Checklist

This checklist documents the canonical smoke checks for the implemented v0 execution primitive.

Version 0 is complete when the commands below produce the expected statuses and evidence behavior without adding any broader broker features.

## Scope

Version 0 proves:

- one foreground `run` command per request;
- required `--cwd` and `--timeout`;
- direct argument-vector execution after `--`;
- structured JSON results;
- metadata-only execution evidence stored outside the target workspace.

Version 0 does not add:

- policy rules or approval workflows;
- sessions or background process handles;
- MCP, JSON-RPC, or HTTP adapters;
- diff analysis or code-editing behavior.

## Canonical Smoke Commands

Run these commands from the repository root:

```bash
cargo test
cargo run -- run --cwd . --timeout 5 -- echo hello
cargo run -- run --cwd . --timeout 30 -- false
cargo run -- run --cwd . --timeout 30 -- missing-command-for-test
cargo run -- run --cwd . --timeout 1 -- sleep 2
```

## Expected Outcomes

### `cargo test`

- exits successfully;
- covers CLI parsing, execution mapping, timeout handling, and evidence persistence.

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
- `stdout` and `stderr` may both be empty;
- `timed_out` is `false`;
- the CLI exits non-zero.

### Execution Error Case

Command:

```bash
cargo run -- run --cwd . --timeout 30 -- missing-command-for-test
```

Expected result:

- JSON `status` is `execution_error`;
- `exit_code` is `null`;
- `timed_out` is `false`;
- `error.code` is `process_start_failed`;
- `stdout` and `stderr` are empty strings because the process never started;
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

## Evidence Location

Each execution that reaches the broker writes one evidence file under the harness-owned temp directory:

```text
<temp-dir>/llm-shell/evidence/<timestamp>_<request_id>.json
```

How to locate it:

1. Run one of the smoke commands and copy the returned `request_id`.
2. Resolve the temp root with `python3 -c 'import tempfile; print(tempfile.gettempdir())'`.
3. Inspect `<temp-dir>/llm-shell/evidence/` for the file whose name ends with `_<request_id>.json`.

On macOS, this commonly resolves under `/private/var/folders/.../T/llm-shell/evidence/`. On many Linux systems, it will resolve under `/tmp/llm-shell/evidence/`.

## Evidence Expectations

The evidence file is intentionally metadata-only in v0.

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
- `error_code` when applicable.

Version 0 evidence must not persist full `stdout` or `stderr` by default.

The evidence directory must stay outside the target workspace selected by `--cwd`. Running `--cwd .` for this repository must still write evidence to the temp-root path above, not into the repository tree.
