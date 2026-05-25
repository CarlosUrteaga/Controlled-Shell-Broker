# CLI Contract

## Status

This document defines the caller-facing CLI contract for version 0 and version 1 policy admission control.

It is not evidence that the CLI is implemented.

## Scope

This file is the source of truth for CLI syntax, flags, examples, invalid requests, and caller-visible status expectations.

Canonical schemas live in [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md).

## Binary Name

The current placeholder binary name is `llm-shell`.

The final binary name may change before implementation. The semantics should not.

## Supported Operation

Version 0 defines one caller-facing operation:

- `run`

## `run` Syntax

```bash
llm-shell run [OPTIONS] -- <COMMAND> [ARGS...]
```

The `--` separator is required.

Everything before `--` belongs to the harness CLI. Everything after `--` belongs to the command payload.

## Supported Flags

```text
--cwd <PATH>         Required working directory for command execution
--timeout <SECONDS>  Required maximum command duration
--output json        Output format; JSON is the only supported v0 value
```

## Input Rules

- Harness flags must appear before `--`.
- Command arguments must appear after `--`.
- `--cwd` is required in v0.
- `--timeout` is required in v0.
- The command payload must not be empty.
- `--timeout` must be numeric and positive.
- `--cwd` must refer to an existing directory.
- Unknown harness flags should produce a structured invalid-request response.
- `json` is the default and only supported v0 output mode.
- The command payload is passed as an argument vector, not a shell string.
- Version 0 does not support shell-mediated execution modes.

## Valid Examples

```bash
llm-shell run --cwd ./repo --timeout 30 -- cargo test
llm-shell run --cwd ./repo --timeout 60 -- cargo test --all
llm-shell run --cwd . --timeout 5 -- echo hello
```

## Invalid Examples

```bash
llm-shell run --cwd ./repo --timeout 30
llm-shell run --cwd ./repo --timeout nope -- cargo test
llm-shell run --cwd ./repo --timeout 30 cargo test
llm-shell run --unknown-flag --cwd ./repo --timeout 30 -- cargo test
```

## Error Expectations

The CLI should reject malformed requests before execution.

Malformed requests include:

- missing `--cwd`;
- missing `--timeout`;
- missing command payload;
- nonexistent or invalid `cwd`;
- invalid timeout value;
- unsupported operation;
- unsupported output format;
- unknown harness flag;
- invalid argument shape.

These are request-validation failures, not policy decisions.

Valid requests may still be rejected later by broker policy with `status: "denied"`.

## Working Directory Semantics

`--cwd` is required in v0.

The CLI adapter must reject a request as `invalid_request` if `--cwd`:

- is missing;
- is empty;
- cannot be resolved;
- does not exist;
- does not refer to a directory.

Invalid working-directory requests must be rejected before command execution.

The rejection happens during request validation and normalization, before the request is handed to the execution broker.

A request rejected for invalid `cwd` must not execute the command payload.

Expected status:

```text
invalid_request
```

Expected error code:

```text
invalid_cwd
```

Version 1 adds a separate policy rule for workspace authorization.

If `--cwd` resolves successfully but falls outside the canonicalized broker startup workspace root, the request is not malformed. The broker must reject it as `denied` with error code `cwd_outside_workspace_root`.

This workspace-root restriction is policy, not CLI validation.

## Timeout Semantics

`--timeout` is required in v0.

The timeout value is expressed in seconds and must be a positive integer.

Version 0 timeout behavior is defined as follows:

- the timeout starts when the execution backend attempts to start the command;
- if the command completes before the timeout, the result is reported normally;
- if the timeout is reached, the harness reports `timed_out`;
- the harness must attempt to terminate the direct child process;
- full process-tree or process-group termination is not guaranteed in v0.

Commands that spawn additional child processes may leave descendants behind if the operating system does not terminate them when the direct child is killed.

Full process-group management is deferred until after v0.

Expected timeout status:

```text
timed_out
```

Expected timeout fields:

```json
{
  "status": "timed_out",
  "exit_code": null,
  "timed_out": true
}
```

## Execution Error Semantics

`execution_error` is reserved for harness-level failures where the command could not be started or the harness could not complete execution handling.

Examples include:

- executable not found;
- permission denied when starting the process;
- failure to create the process;
- failure to access required harness-owned state;
- failure to write required execution evidence.

`execution_error` is different from `failed`.

A command that starts and exits with a non-zero exit code is `failed`, not `execution_error`.

For schema stability, `execution_error` responses should keep the normal result fields when possible:

```json
{
  "request_id": "opaque-request-id",
  "operation": "run",
  "status": "execution_error",
  "cwd": "./repo",
  "command": ["missing-command"],
  "exit_code": null,
  "stdout": "",
  "stderr": "",
  "duration_ms": 0,
  "timed_out": false,
  "error": {
    "code": "process_start_failed",
    "message": "The command could not be started."
  }
}
```

If partial stdout or stderr exists, it may be included. If no process was started, stdout and stderr should be empty strings.

## Policy Denial Semantics

`denied` is reserved for broker-level policy rejection after request validation and before command startup.

Examples include:

- a valid `cwd` outside the approved workspace root;
- a denied executable selected by policy.

`denied` is different from `invalid_request` and `execution_error`.

Denied responses keep the standard execution result envelope:

```json
{
  "request_id": "opaque-request-id",
  "operation": "run",
  "status": "denied",
  "cwd": "./repo",
  "command": ["rm", "-rf", "."],
  "exit_code": null,
  "stdout": "",
  "stderr": "",
  "duration_ms": 0,
  "timed_out": false,
  "error": {
    "code": "denied_executable",
    "message": "The request was denied by broker policy."
  }
}
```

Denied requests must not spawn a subprocess.

The CLI exit code for `denied` is fixed at `1`.

## Evidence Persistence Semantics

Version 0 requires one persisted execution-evidence record per execution attempt that reaches the broker.

Version 1 extends this requirement to policy denials that reach the broker and are rejected before process spawn.

Evidence records must be written outside the target working directory selected by `--cwd`.

The default evidence location should be a harness-owned state directory.

Recommended default strategy:

```text
<harness-state-dir>/evidence/YYYY-MM-DD/<timestamp>_<request_id>.json
```

The exact platform-specific state directory may be defined during implementation, but it must not be inside the target working directory by default.

Evidence persistence is not configured through the v0 CLI contract.

If the harness cannot write required execution evidence for a request that otherwise reached execution, the result should be reported as `execution_error`.

The evidence record should prioritize metadata over full output persistence to reduce accidental secret exposure.

The v0 evidence record should include:

- request id;
- operation;
- command vector;
- resolved cwd;
- status;
- exit code when available;
- duration;
- timeout flag;
- timestamp;
- error code when applicable.

Full stdout and stderr persistence is not required in v0 or in version 1 denial evidence.

## Request And Response Boundary

The CLI should normalize raw input into canonical request types and serialize canonical result types returned by the broker.

The canonical command form should be a vector of arguments, not a shell string.

For valid requests, the adapter should generate an opaque `request_id` before handing the request to the broker.

Detailed request and response shapes live in [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md).

## Status Vocabulary

Version 1 uses:

```text
success
failed
timed_out
denied
invalid_request
execution_error
```

## Command Exit Semantics

Version 0 does not interpret command-specific behavior.

The harness must not infer that a test run, linter, formatter, build tool, or package manager succeeded or failed based on command-specific output.

Version 0 uses only generic process semantics:

- exit code `0` maps to `success`;
- non-zero exit code maps to `failed`;
- timeout maps to `timed_out`;
- policy rejection maps to `denied`;
- malformed request maps to `invalid_request`;
- harness-level process or evidence failure maps to `execution_error`.

Command-specific semantics are deferred until after v0.

## Responsibility Boundary

The CLI owns syntax, raw argument parsing, basic shape validation, request normalization, and final response serialization.

It does not own process spawning, policy enforcement, workspace authorization, timeout implementation, stdout/stderr capture, or log persistence.

## Version 0 Notes

- Version 0 defines one operation only: `run`.
- One request maps to one foreground command execution.
- Non-zero process exit is a command result, not a CLI parsing error.
- Evidence persistence is required by v0 but is not configured through the CLI contract yet.
- Version 1 does not add new CLI flags for policy or approval.
