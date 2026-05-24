# Request / Response Schema

## Status

This document defines the canonical machine-readable shapes intended for the tool.

It is not evidence that these types are implemented.

## Scope

This file is the source of truth for request, result, error, event, and status shapes shared across CLI adapters, broker components, and future protocol adapters.

## `ExecutionRequest`

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "cwd": "./repo",
  "timeout_seconds": 30,
  "command": ["cargo", "test"],
  "mode": "foreground",
  "output_format": "json"
}
```

Rules:

- `operation`: version 0 uses `run`
- `command`: canonical vector of command arguments
- `mode`: version 0 uses `foreground`
- `output_format`: version 0 uses `json`

## `ExecutionResult`

Successful execution:

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "status": "success",
  "cwd": "./repo",
  "command": ["cargo", "test"],
  "exit_code": 0,
  "stdout": "...",
  "stderr": "...",
  "duration_ms": 1240,
  "timed_out": false
}
```

Timed-out execution:

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "status": "timed_out",
  "cwd": "./repo",
  "command": ["cargo", "test"],
  "exit_code": null,
  "stdout": "...",
  "stderr": "...",
  "duration_ms": 30000,
  "timed_out": true
}
```

## `ExecutionError`

Invalid request:

```json
{
  "request_id": null,
  "operation": "run",
  "status": "invalid_request",
  "error": {
    "code": "missing_command",
    "message": "No command was provided after the command separator."
  }
}
```

Harness-level execution failure:

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "status": "execution_error",
  "error": {
    "code": "process_spawn_failed",
    "message": "The command could not be started."
  }
}
```

## `ExecutionEvent`

```json
{
  "event_type": "command_execution",
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "cwd": "./repo",
  "command": ["cargo", "test"],
  "status": "success",
  "exit_code": 0,
  "duration_ms": 1240,
  "timed_out": false,
  "timestamp": "2026-05-23T00:00:00Z"
}
```

## `PolicyDecision`

```json
{
  "allowed": true,
  "reason_code": null,
  "message": null
}
```

This is documented now to avoid future ad hoc policy-result shapes.

## Status Vocabulary

```text
success
failed
timed_out
invalid_request
execution_error
```

## Shape Stability Rules

- External adapters should reuse these shapes where possible.
- Future versions should extend fields compatibly rather than redefining core structures.
- Shell-string command input, if ever supported, should not replace the canonical vector form.
