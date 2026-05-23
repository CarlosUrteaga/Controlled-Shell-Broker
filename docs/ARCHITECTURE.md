# Architecture

## Status

This document defines intended architecture only. It is not evidence that the system is implemented.

## High-Level Intent

The system should be a Rust CLI harness that sits between an LLM agent and the operating system shell.

The harness is responsible for:

- validating execution requests;
- applying policy;
- running commands in a controlled context;
- collecting outputs and metadata;
- emitting structured results and logs.

## Architectural Principles

- Least authority: the harness should expose only the minimum execution capability needed.
- Auditability: all command executions should be observable and reconstructable.
- Determinism where possible: inputs and outputs should be explicit and structured.
- Separation of concerns: execution, policy, logging, and CLI parsing should remain distinct.
- Extensibility: `v0` should not block later support for sessions and long-lived processes.

## Planned Logical Components

### CLI Layer

Responsibilities:

- parse command-line arguments;
- validate user input shape;
- construct an execution request;
- serialize the final result as JSON.

Likely future files:

- `src/main.rs`
- `src/cli.rs`

### Execution Harness

Responsibilities:

- spawn shell commands;
- apply working directory and timeout configuration;
- capture stdout and stderr;
- collect exit status and duration.

Likely future files:

- `src/exec/mod.rs`
- `src/exec/run.rs`

### Policy Layer

Responsibilities:

- enforce command restrictions;
- gate dangerous actions;
- support future approval integration.

Likely future files:

- `src/policy.rs`

### Logging Layer

Responsibilities:

- record execution metadata;
- emit machine-readable logs;
- support later audit workflows.

Likely future files:

- `src/logging.rs`

### Domain Types

Responsibilities:

- define stable request and response structures;
- separate internal execution state from CLI presentation.

Likely future files:

- `src/types.rs`

## Expected Data Flow

1. CLI receives a `run` request.
2. CLI validates arguments and creates a typed execution request.
3. Policy layer evaluates whether the request is allowed.
4. Execution harness runs the command with timeout and cwd settings.
5. Harness captures outputs and metadata.
6. Logging layer records the execution event.
7. CLI emits the structured JSON response.

## Boundary Decisions

- The shell broker owns command execution policy and result formatting.
- The LLM agent owns planning, interpretation, and follow-up actions.
- The repository should avoid coupling execution logic to any specific model provider or agent framework.

## Implementation Constraints For Future Tasks

- Keep the public response shape stable once introduced.
- Avoid mixing policy checks directly into process-spawning code.
- Keep logging independent from display formatting.
- Prefer typed request and response structures over ad hoc maps.

## Documentation Obligation

Any future change that alters the architecture, boundaries, or component responsibilities must update this file in the same task.
