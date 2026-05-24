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
--cwd <PATH>         Working directory for command execution
--timeout <SECONDS>  Maximum command duration
--output json        Output format; JSON is the default in v0
```

## Input Rules

- Harness flags must appear before `--`.
- Command arguments must appear after `--`.
- The command payload must not be empty.
- `--timeout` must be numeric and positive.
- Unknown harness flags should produce a structured invalid-request response.
- JSON should be the default output mode.

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

- missing command payload;
- invalid timeout value;
- unsupported operation;
- unsupported output format;
- unknown harness flag;
- invalid argument shape.

These are request-validation failures, not policy decisions.

## Request And Response Boundary

The CLI should normalize raw input into canonical request types and serialize canonical result types returned by the broker.

The canonical command form should be a vector of arguments, not a shell string.

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
