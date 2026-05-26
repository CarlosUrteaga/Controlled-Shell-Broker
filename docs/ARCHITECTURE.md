# Architecture

## Status

This document defines intended system structure and component boundaries.

It is not evidence that the system is implemented.

## Summary

The system should be a Rust-based controlled command-line execution tool that sits between an agent or caller and the operating system command line.

The first implemented slice is a single foreground execution path. The next slice adds broker-layer admission control without expanding into sessions or adapter sprawl.

Later phases may add a read-oriented inspection profile above the broker, but the broker remains the execution and evidence boundary.

## Core Pattern

The intended pattern is:

> Adapter + Canonical Request Model + Broker Pipeline

```text
Human CLI / Script / Agent Adapter
              |
              v
      Canonical Request Types
              |
              v
        Execution Broker
   +----------+----------+----------+
   |          |          |          |
 policy   working-dir execution   logging
 layer    rules       harness     layer
              |
              v
      Canonical Result Types
```

The CLI is an adapter, not the execution engine.

## Component Responsibilities

### Adapter Layer

- parse caller input;
- validate basic request shape;
- normalize input into canonical request types;
- serialize canonical results for the caller.

### Domain Types

- define stable request, response, error, and event structures;
- separate adapters from broker internals;
- support later policy and session workflows.

### Execution Broker

- coordinate policy checks, working-directory rules, execution, and logging;
- route canonical requests to the correct path;
- return canonical results.

### Execution Harness

- spawn commands;
- apply working directory and timeout behavior;
- capture stdout and stderr;
- collect exit status and duration.

### Policy Layer

- evaluate valid canonical requests before subprocess spawn;
- return `allow` or `deny` in version 1;
- preserve a clean seam for future approval hooks without implementing them yet.

### Logging Layer

- record machine-readable execution events;
- support traceability and later evidence workflows.

### Optional Inspection Layer

- may provide higher-level repository inspection workflows on top of the broker;
- should reuse broker policy, timeout, and evidence behavior rather than bypass them;
- should stay read-oriented unless a later product change explicitly broadens scope.

## Interface Boundary

The request interface owns parsing, basic validation, request normalization, request IDs if used, and response serialization.

It does not own process spawning, policy decisions, working-directory authorization, timeout enforcement, stdout/stderr capture, or log persistence.

## Expected Data Flow

1. An adapter receives a request.
2. The adapter validates basic shape.
3. The adapter generates a request ID for valid requests and normalizes input into canonical request types.
4. The broker applies policy admission control using canonical request data and the canonicalized startup workspace root.
5. Denied requests return a canonical `denied` result without subprocess spawn.
6. Allowed requests continue into the execution harness.
7. The logging layer persists one machine-readable evidence record for either the denial or the completed execution outside the target working directory.
8. The broker returns canonical result types.
9. The adapter serializes the result for the caller.

No downstream component should need raw CLI tokens after canonicalization.

If an inspection layer is added later, it should consume canonical broker operations and evidence rather than invent a separate execution path.

## Source Of Truth Boundaries

- Caller-facing CLI details: [docs/CLI_CONTRACT.md](CLI_CONTRACT.md)
- Canonical shapes: [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md)
- Security assumptions: [docs/SECURITY_MODEL.md](SECURITY_MODEL.md)
- Product scope: [docs/PRODUCT_SPEC.md](PRODUCT_SPEC.md)

## Constraints

- Keep canonical request and response shapes stable once introduced.
- Avoid mixing policy checks directly into process-spawning code.
- Keep logging independent from display formatting.
- Prefer typed contracts over ad hoc maps.
- Keep the architecture reusable across future adapters.
- Do not require session management, streaming, or shell-string parsing in v0.
- Keep any later inspection profile layered above the broker instead of coupling it directly to a specific agent runtime.

## Out Of Scope For This Slice

- protocol adapter implementations;
- session internals;
- streaming output;
- detailed evidence-retention policy;
- OS sandboxing internals.

## Version Boundary Notes

Version 0 is intentionally narrow:

- one adapter operation: `run`
- one broker path: foreground command execution
- one canonical command representation: argument vector
- one result family: structured JSON success, failure, timeout, or execution error
- one evidence family: machine-readable execution events

Version 1 keeps the same adapter surface and adds:

- one policy checkpoint between validation and execution;
- one new caller-visible status: `denied`;
- one denial evidence event family for broker rejections before spawn.

This document defines structure and responsibility boundaries. Exact field names and status values belong in [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md).

## Documentation Obligation

Any future change that alters component boundaries or the adapter-to-broker separation must update this file in the same task.
