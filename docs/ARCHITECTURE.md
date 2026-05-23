# Architecture

## Status

This document defines intended architecture only. It is not evidence that the system is implemented.

## High-Level Intent

The system should be a Rust CLI harness that sits between an LLM agent, human user, script, or external controller and the operating system shell.

The harness is responsible for:

- validating execution requests;
- applying policy;
- running commands in a controlled context;
- collecting outputs and metadata;
- emitting structured results and logs.

The first architectural slice should focus on the request boundary, not on process execution internals.

## Architectural Principles

- Least authority: the harness should expose only the minimum execution capability needed.
- Auditability: all command executions should be observable and reconstructable.
- Determinism where possible: inputs and outputs should be explicit and structured.
- Separation of concerns: execution, policy, logging, and CLI parsing should remain distinct.
- Extensibility: `v0` should not block later support for sessions and long-lived processes.

## Planned Logical Components

### CLI / Request Interface

Responsibilities:

- parse command-line arguments;
- validate basic input shape;
- normalize external input into a canonical request type;
- reject malformed requests early;
- pass valid requests to the broker pipeline;
- serialize the final broker response.

Non-responsibilities:

- process spawning;
- policy enforcement;
- logging persistence;
- workspace authorization;
- timeout enforcement;
- stdout/stderr capture.

Likely future files:

- `src/main.rs`
- `src/cli.rs`
- `src/request.rs`

### Domain Types

Responsibilities:

- define stable canonical request and response structures;
- separate external adapters from broker internals;
- provide typed contracts for future execution, policy, and session workflows.

Likely future files:

- `src/types.rs`

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

## CLI / Request Interface Architecture

### Purpose

The CLI / Request Interface is the outer boundary of the harness.

Its purpose is to receive command execution intentions from a human user, script, LLM agent, or future protocol adapter and convert them into a stable typed request that the internal broker pipeline can process.

The CLI layer should not own command execution.

The CLI layer should not own policy enforcement.

The CLI layer should not own logging.

The CLI layer should only:

- parse user-facing input;
- validate basic input shape;
- normalize input into a typed request;
- reject malformed requests early;
- pass valid requests to the internal broker;
- serialize the final response returned by the broker.

### Architectural Pattern

The intended pattern for this layer is:

> Adapter + Canonical Request Model

The CLI is only one adapter.

All external interfaces should eventually map into the same internal request model.

```text
Human CLI
   |
Scripted CLI
   |
LLM Agent Tool Call
   |
Future JSON / MCP / RPC Adapter
   |
   v
Canonical ExecutionRequest
   |
   v
Execution Broker
```

This prevents the system from coupling execution behavior to a specific interface.

The CLI is a user-facing adapter.

The `ExecutionRequest` is the internal contract.

### Interface Boundary

The request interface owns:

- command-line argument parsing;
- command input collection;
- basic required-field validation;
- conversion into typed request structures;
- request IDs or correlation IDs, if applicable;
- response serialization format selection.

The request interface does not own:

- process spawning;
- command safety decisions;
- allowlist or blocklist evaluation;
- workspace authorization;
- timeout enforcement;
- stdout/stderr capture;
- log persistence;
- long-lived process lifecycle management.

Those concerns belong to later broker components.

### Initial CLI Surface

The first CLI surface should be intentionally small.

Recommended first command:

```bash
harness run --cwd ./repo --timeout 30 -- cargo test
```

The `--` separator is important because it clearly separates harness arguments from the command that should be executed.

For example:

```bash
harness run --cwd ./repo --timeout 60 -- cargo test --all
```

In this example:

- `harness run` belongs to the harness CLI;
- `--cwd ./repo` configures the execution context;
- `--timeout 60` configures the maximum execution duration;
- everything after `--` is the command payload.

### Initial Request Shape

The CLI should convert raw arguments into a typed request similar to:

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "command": ["cargo", "test", "--all"],
  "cwd": "./repo",
  "timeout_seconds": 60,
  "mode": "foreground",
  "output": {
    "format": "json"
  }
}
```

This structure is only the intended contract. It does not imply implementation yet.

### Canonical Request Type

The future internal type may be conceptually represented as:

```rust
struct ExecutionRequest {
    request_id: RequestId,
    operation: ExecutionOperation,
    command: Vec<String>,
    cwd: WorkspacePath,
    timeout_seconds: u64,
    mode: ExecutionMode,
    output_format: OutputFormat,
}
```

Possible supporting enums:

```rust
enum ExecutionOperation {
    Run,
}

enum ExecutionMode {
    Foreground,
}

enum OutputFormat {
    Json,
}
```

Future versions may extend these enums without changing the basic architecture.

### Why Command Should Be A Vector

The command should ideally be represented internally as a vector of arguments:

```json
{
  "command": ["cargo", "test", "--all"]
}
```

Rather than as a single shell string:

```json
{
  "command": "cargo test --all"
}
```

The vector form is preferable because it is more explicit, easier to inspect, easier to validate, and avoids unnecessary shell parsing ambiguity.

A later version may support shell-string execution, but that should be treated as a distinct mode because it has different security and parsing implications.

### CLI Input Rules

The CLI should follow these input rules:

- harness flags must appear before the command separator `--`;
- command arguments must appear after `--`;
- `cwd` should be explicit or default to the current directory;
- `timeout` should have a safe default;
- output should default to JSON;
- malformed requests should fail before reaching the execution broker;
- unknown harness flags should produce structured errors;
- empty commands should be rejected.

Example invalid request:

```bash
harness run --cwd ./repo --timeout 30
```

Reason:

```text
No command was provided after the command separator.
```

### Request Validation Responsibilities

The CLI / Request Interface may validate:

- required arguments are present;
- timeout is numeric and positive;
- command is not empty;
- output format is supported;
- operation name is valid;
- `cwd` was provided or defaulted.

The CLI / Request Interface should not validate:

- whether the command is dangerous;
- whether the command is allowed by policy;
- whether the workspace is authorized;
- whether the command can actually execute;
- whether the command modifies files.

Those checks belong to the policy and broker layers.

### Expected Request Flow

The intended request flow is:

```text
Raw CLI arguments
   |
   v
CLI parser
   |
   v
Basic shape validation
   |
   v
Request normalization
   |
   v
Canonical ExecutionRequest
   |
   v
Execution broker
```

The CLI should pass only a normalized request into the broker.

No downstream component should need to inspect raw command-line arguments.

### Response Boundary

The CLI should serialize the broker response, not construct execution results itself.

The expected response shape may eventually look like:

```json
{
  "request_id": "generated-or-provided-id",
  "operation": "run",
  "status": "success",
  "exit_code": 0,
  "stdout": "...",
  "stderr": "...",
  "duration_ms": 1240
}
```

For failed request validation, the CLI may return:

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

### Future Interface Extensions

The architecture should leave room for additional interfaces that produce the same canonical request type.

Possible future extensions:

```bash
harness run-json < request.json
```

```bash
harness validate-request < request.json
```

```bash
harness session start api --cwd ./repo -- npm run dev
```

```bash
harness session stop api
```

```bash
harness session logs api
```

These future interfaces should not bypass the canonical request model.

They should map into either `ExecutionRequest` or future sibling request types such as `SessionRequest`.

### Design Decisions For This Slice

- The CLI is an adapter, not the execution engine.
- The canonical request model is the internal contract.
- The initial operation should be `run`.
- The first execution mode should be `foreground`.
- The command should be represented internally as `Vec<String>`.
- Shell-string execution should not be the default internal model.
- The CLI should use `--` to separate harness flags from command arguments.
- JSON should be the default output format for machine-readability.
- Future protocol adapters should reuse the same request model.

### Out Of Scope For This Slice

This architecture slice does not define:

- command execution internals;
- process spawning implementation;
- policy engine internals;
- background process management;
- long-lived sessions;
- streaming output;
- MCP or RPC adapters;
- log storage;
- workspace authorization rules.

Those belong to later architecture slices.

## Expected Data Flow

1. CLI receives a `run` request.
2. CLI validates arguments and creates a canonical `ExecutionRequest`.
3. Broker components evaluate policy and workspace constraints.
4. Execution harness runs the command with timeout and cwd settings.
5. Harness captures outputs and metadata.
6. Logging layer records the execution event.
7. CLI emits the structured JSON response returned by the broker.

## Boundary Decisions

- The shell broker owns command execution policy and result formatting.
- The LLM agent owns planning, interpretation, and follow-up actions.
- The repository should avoid coupling execution logic to any specific model provider or agent framework.
- The CLI must remain an adapter around a stable canonical request model.

## Implementation Constraints For Future Tasks

- Keep the public response shape stable once introduced.
- Keep the canonical request shape stable once introduced.
- Avoid mixing policy checks directly into process-spawning code.
- Keep logging independent from display formatting.
- Prefer typed request and response structures over ad hoc maps.

## Documentation Obligation

Any future change that alters the architecture, boundaries, or component responsibilities must update this file in the same task.
