# Product Specification

## Status

This document defines the intended product behavior.

It is not evidence that the system is implemented. At this stage, the repository is focused on refining the documentation and design contract for a future Rust command-line harness.

## Summary

The project will provide a Rust CLI harness for controlled command execution in LLM-assisted software development workflows.

The harness is not the agent.

It is the execution surface that an LLM-driven agent, script, or human developer can use to run commands in a controlled workspace and receive structured results.

## Product Positioning

The product should behave as a command execution broker, not as unrestricted terminal access.

A normal terminal accepts raw commands and executes them directly.

This harness should instead:

1. receive a command execution request;
2. validate the request shape;
3. normalize the request into a stable internal structure;
4. execute the command through the future broker pipeline;
5. capture stdout, stderr, exit code, duration, and status;
6. emit a structured JSON response;
7. record basic execution evidence for later review.

## Problem Statement

LLM-driven coding workflows often need to run commands such as tests, linters, build scripts, formatters, repository inspections, and local development tools.

However, giving an LLM unrestricted shell access is risky and difficult to review after the fact.

The product should provide a constrained command-line harness that allows command execution to be:

- explicit;
- structured;
- bounded by working directory;
- bounded by timeout;
- machine-readable;
- logged for later review;
- extensible toward policy and session management.

## Primary User

The primary user is an LLM-driven coding agent operating on behalf of a human developer inside a controlled workspace.

Secondary users include:

- human developers running the harness directly;
- reviewers inspecting command outputs and execution logs;
- operators defining future policy boundaries;
- automation tools consuming structured command results;
- future agent workflows that need repeatable command execution evidence.

## Product Goals

Version 0 should provide a minimal but stable command execution surface.

The product goals are:

1. Execute a foreground command within a specified working directory.
2. Accept command input through a clear CLI request interface.
3. Capture stdout and stderr separately.
4. Report exit code when available.
5. Report wall-clock duration.
6. Enforce a caller-specified timeout.
7. Return structured JSON output.
8. Produce a basic machine-readable execution log.
9. Establish a foundation for later policy, session, and long-lived process support.

## Non-Goals For Version 0

Version 0 does not aim to provide:

- autonomous planning;
- code generation;
- code editing;
- model prompting;
- model provider integration;
- unrestricted shell access;
- OS-level sandboxing;
- container orchestration;
- distributed job execution;
- long-lived terminal sessions;
- interactive terminal UI;
- streaming output;
- advanced approval workflows;
- cross-machine session coordination;
- MCP, JSON-RPC, HTTP, or agent-framework integration.

## Version 0 Scope

Version 0 should support a single foreground operation:

```bash
llm-shell run --cwd ./repo --timeout 30 -- cargo test
```

The `--` separator is part of the intended CLI contract.

It separates harness arguments from the command payload.

For example:

```bash
llm-shell run --cwd ./repo --timeout 60 -- cargo test --all
```

In this request:

- `llm-shell run` is the harness operation;
- `--cwd ./repo` defines the execution directory;
- `--timeout 60` defines the maximum execution duration;
- everything after `--` is the command and its arguments.

## Initial CLI Contract

### Command

```bash
llm-shell run [OPTIONS] -- <COMMAND> [ARGS...]
```

### Required Or Supported Options

```text
--cwd <PATH>        Working directory for command execution.
--timeout <SECONDS> Maximum command duration.
--output json       Output format. JSON should be the default.
```

The exact binary name may change before implementation. The product behavior should not depend on the final binary name.

## Canonical Request Model

The CLI should translate raw input into a canonical execution request.

Conceptual request shape:

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

The internal command representation should prefer an argument vector:

```json
{
  "command": ["cargo", "test", "--all"]
}
```

Rather than a shell string:

```json
{
  "command": "cargo test --all"
}
```

The vector form is preferred because it is easier to validate, inspect, log, and reason about.

Shell-string execution may be considered in the future, but it should be treated as a separate execution mode because it has different parsing and security implications.

## Functional Requirements

### Request Intake

- The tool must expose a `run` operation.
- The tool must accept harness options before the command separator `--`.
- The tool must treat arguments after `--` as the command payload.
- The tool must reject an empty command payload.
- The tool must accept an explicit working directory through `--cwd`.
- The tool must accept an explicit timeout through `--timeout`.
- The tool must default to JSON output unless another supported format is explicitly introduced.

### Request Validation

The tool must reject malformed requests before execution.

Malformed requests include:

- missing command payload;
- invalid timeout value;
- unsupported operation;
- unsupported output format;
- missing required option, if a required option is introduced;
- invalid argument shape.

Request validation does not replace future policy enforcement.

Version 0 request validation should answer:

> Is this a well-formed execution request?

It should not attempt to fully answer:

> Is this command safe?

That belongs to the future policy layer.

### Command Execution

- The tool must execute one foreground command per `run` request.
- The tool must run the command relative to the selected working directory.
- The tool must wait for the command to complete, fail, or time out.
- The tool must report whether execution completed successfully, failed, timed out, or could not start.

### Output Reporting

- The tool must capture stdout separately from stderr.
- The tool must include exit code when available.
- The tool must include wall-clock duration.
- The tool must include execution status.
- The tool must return the result in structured JSON.

### Timeout Handling

- The tool must allow a caller-specified timeout in seconds.
- The tool must terminate or stop waiting on commands that exceed the timeout.
- Timeout status must be visible in the structured result.
- Timeout behavior must be documented before implementation.

### Logging

- Each command execution must produce a basic log record.
- Logs must be machine-readable.
- Logs should include enough information to support later review and debugging workflows.
- Logs should not be treated as the same thing as user-facing command output.

## Expected Response Shape

A successful command response should resemble:

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

A failed command response should resemble:

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "status": "failed",
  "cwd": "./repo",
  "command": ["cargo", "test"],
  "exit_code": 101,
  "stdout": "...",
  "stderr": "...",
  "duration_ms": 1240,
  "timed_out": false
}
```

A timeout response should resemble:

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

An invalid request response should resemble:

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

## Execution Statuses

Version 0 should define a small status vocabulary:

```text
success
failed
timed_out
invalid_request
execution_error
```

Where:

- `success` means the command completed with exit code `0`;
- `failed` means the command completed with a non-zero exit code;
- `timed_out` means the timeout was reached before completion;
- `invalid_request` means the request was rejected before execution;
- `execution_error` means the command could not be started or completed due to harness-level failure.

## Basic Log Record

A basic log record should resemble:

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

The exact storage location and retention behavior may be defined later.

## Product Constraints

- The harness should not be coupled to a specific LLM provider.
- The harness should not be coupled to a specific coding agent.
- The harness should not interpret test failures or decide next actions.
- The harness should not mutate files except through the command it was explicitly asked to run.
- The harness should keep request intake, execution, output reporting, and logging conceptually separate.
- The harness should prefer stable, typed, machine-readable contracts over ad hoc text output.

## Expected Future Extensions

Future versions may add:

- long-lived process handles;
- multiple concurrent sessions;
- named terminal sessions;
- command allow/deny policies;
- approval hooks;
- richer structured event logs;
- workspace authorization rules;
- environment variable controls;
- output truncation policies;
- streaming output;
- interactive process management;
- JSON request input;
- protocol adapters such as MCP, JSON-RPC, or HTTP.

These future extensions should reuse the canonical request model where possible and should not bypass the broker architecture.

## Acceptance Standard For Future Implementation Tasks

Each implementation task derived from this spec must define:

- the exact requirement being implemented;
- the expected files to change;
- the validation commands to run;
- acceptance criteria tied to observable outcomes;
- a PR summary that matches the repository template.

## Documentation Obligation

Any future change that alters the product behavior, CLI contract, request shape, response shape, status vocabulary, logging expectations, or version scope must update this file in the same task.
