# Product Specification

## Summary

The project will provide a Rust CLI harness for controlled shell execution in LLM-assisted software development workflows.

The harness is not the agent. It is the execution surface the agent uses.

## Problem Statement

LLM-driven coding workflows need command execution, process management, and filesystem interaction, but unrestricted shell access is too risky and too hard to audit. The product should provide a constrained execution harness with structured outputs and policy controls.

## Primary User

The primary user is an LLM-driven coding agent operating on behalf of a human developer inside a controlled workspace.

Secondary users:

- developers reviewing command logs and results;
- operators defining policy boundaries;
- future automation tooling that consumes structured command results.

## Product Goals

1. Execute a single shell command within a specified working directory.
2. Capture stdout, stderr, exit code, and duration.
3. Enforce execution timeout.
4. Return structured JSON output.
5. Produce basic audit logs for executed commands.
6. Establish a foundation for later session, policy, and long-lived process support.

## Non-Goals For Version 0

- autonomous planning or code editing;
- sandboxing at the OS or container orchestration layer;
- distributed job execution;
- streaming terminal UI;
- advanced approval workflows;
- cross-machine session coordination.

## Version 0 Scope

The initial CLI contract is expected to resemble:

```bash
harness run --cwd ./repo --timeout 30 -- cargo test
```

Version 0 should support:

- a `run` command;
- `--cwd` for working directory selection;
- `--timeout` for maximum command duration;
- stdout capture;
- stderr capture;
- exit code reporting;
- execution duration reporting;
- structured JSON response;
- basic command logging.

The first implementation slice should prioritize the CLI/request interface boundary:

- parse CLI input;
- validate request shape;
- normalize into a typed `ExecutionRequest`;
- preserve a stable internal request contract for later broker components.

## Functional Requirements

### Command Execution

- The tool must execute a user-provided shell command.
- The tool must run the command relative to an explicit or default working directory.
- The tool must report whether the command completed, failed, or timed out.
- The canonical internal command representation should be a vector of arguments, not a shell string.

### Output Reporting

- The tool must capture stdout and stderr separately.
- The tool must include exit status when available.
- The tool must include wall-clock duration.
- The tool must return results in JSON.
- JSON should be the default machine-readable output format.

### Request Interface

- The CLI must use `--` to separate harness arguments from command arguments.
- The CLI must reject empty commands before they reach the broker.
- The CLI must validate basic request shape but not enforce execution policy.
- The CLI must convert external input into a stable typed request model.

### Timeout Handling

- The tool must allow a caller-specified timeout.
- The tool must terminate commands that exceed the timeout.
- Timeout should be visible in the structured result.

### Logging

- Each command execution must produce a basic log record.
- Log data should be machine-readable.
- Logs should be suitable for later audit and debugging workflows.

## Expected Future Extensions

- alternate request adapters such as JSON, MCP, or RPC;
- long-lived process handles;
- multiple concurrent sessions;
- command allow/deny policies;
- approval hooks;
- richer structured event logs;
- interactive process management.

## Acceptance Standard For Future Implementation Tasks

Each implementation task derived from this spec must define:

- the exact requirement being implemented;
- the files expected to change;
- the tests that prove the behavior;
- acceptance criteria tied to observable outcomes;
- a PR summary that matches the repo template.
