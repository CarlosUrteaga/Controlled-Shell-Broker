# Product Specification

## Status

This document defines intended product behavior and scope.

It is not evidence that the system is implemented.

## Summary

This project provides a Rust-based controlled command-line execution tool.

The tool gives an LLM-driven coding agent, script, or human caller controlled access to command execution needed to build, test, validate, or inspect software behavior from the terminal.

The tool is not the agent itself. It is the controlled execution layer through which an agent interacts with the command line.

## Core Product Concept

The long-term tool should support an agent development loop:

1. receive a task or feature-building intention;
2. execute commands to understand, build, test, or validate the feature;
3. optionally manage supporting processes;
4. return structured observations to the agent;
5. record evidence for review and reproducibility.

Version 0 implements only the first execution primitive: controlled foreground command execution.

## Product Boundary

The agent owns planning, code generation, and next-step decisions.

The Rust tool owns controlled command execution, process control, structured observations, evidence capture, and safety boundaries.

## Problem Statement

Coding agents need controlled terminal capabilities such as command execution, validation runs, process handling, and structured evidence. Unrestricted shell access is too risky and too hard to review.

The product should provide a constrained command-line tool that makes these actions explicit, bounded, machine-readable, and extensible toward policy and sessions.

## Primary User

The primary user is an LLM-driven coding agent operating on behalf of a human developer inside an explicit working-directory context.

Secondary users include human developers, reviewers, operators, and automation tools consuming structured results.

## Version 0 Goals

1. Accept a well-formed foreground `run` request.
2. Execute exactly one command within an explicit working directory.
3. Capture stdout, stderr, exit code, duration, and status.
4. Enforce an explicit caller-specified timeout.
5. Return structured JSON output.
6. Persist one machine-readable execution evidence record per run.
7. Establish the foundation for later policy, sessions, and richer command-line operations.

## Non-Goals For Version 0

- autonomous planning;
- code generation or editing;
- unrestricted shell access;
- OS-level sandboxing;
- long-lived sessions;
- interactive terminal UI;
- streaming output;
- advanced approval workflows;
- protocol adapters beyond the initial CLI.

## Source Of Truth Boundaries

- CLI syntax and examples: [docs/CLI_CONTRACT.md](CLI_CONTRACT.md)
- Canonical request and response shapes: [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md)
- Security assumptions and limits: [docs/SECURITY_MODEL.md](SECURITY_MODEL.md)
- System structure and boundaries: [docs/ARCHITECTURE.md](ARCHITECTURE.md)
- Phase evolution: [docs/ROADMAP.md](ROADMAP.md)

## Functional Requirements

- The tool must validate malformed requests before execution.
- The tool must execute one foreground command per run request.
- The tool must require an explicit working directory and explicit timeout in the CLI contract.
- The tool must run the command relative to the selected working directory.
- The tool must invoke the command directly from a canonical argument vector rather than a shell string.
- The tool must capture stdout separately from stderr.
- The tool must report exit code when available.
- The tool must report wall-clock duration and execution status.
- The tool must expose timeout results in structured output.
- The tool must persist machine-readable execution evidence distinct from user-facing output.

## Product Constraints

- The tool should not be coupled to a specific LLM provider or agent framework.
- The tool should not interpret failures or decide next actions.
- The tool should keep request intake, execution, output reporting, and logging separate.
- The tool should prefer stable, typed, machine-readable contracts over ad hoc text output.

## Expected Future Extensions

- long-lived process handles;
- session management;
- policy and approval hooks;
- richer structured logs and evidence;
- alternate adapters such as JSON, MCP, RPC, or HTTP.

## Version 0 Contract Summary

Version 0 is closed around one operation:

- CLI: `run`
- command model: argument vector after `--`
- execution mode: foreground only
- working directory: explicit and required
- timeout: explicit and required
- output: JSON result
- evidence: one persisted machine-readable event per execution

This document defines product intent and boundaries. It does not redefine the exact CLI, schema, or safety contract from the source-of-truth documents.

## Documentation Obligation

Any future change that alters product behavior, scope, or the agent/tool boundary must update this file in the same task.
