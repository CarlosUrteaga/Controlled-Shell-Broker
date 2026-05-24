# CLI Contract

## Status

This document defines the caller-facing CLI contract for version 0.

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

## Request And Response Boundary

The CLI should normalize raw input into canonical request types and serialize canonical result types returned by the broker.

The canonical command form should be a vector of arguments, not a shell string.

For valid requests, the adapter should generate an opaque `request_id` before handing the request to the broker.

Detailed request and response shapes live in [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md).

## Status Vocabulary

Version 0 uses:

```text
success
failed
timed_out
invalid_request
execution_error
```

## Responsibility Boundary

The CLI owns syntax, raw argument parsing, basic shape validation, request normalization, and final response serialization.

It does not own process spawning, policy enforcement, workspace authorization, timeout implementation, stdout/stderr capture, or log persistence.

## Version 0 Notes

- Version 0 defines one operation only: `run`.
- One request maps to one foreground command execution.
- Non-zero process exit is a command result, not a CLI parsing error.
- Evidence persistence is required by v0 but is not configured through the CLI contract yet.
