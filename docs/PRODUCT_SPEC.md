# Product Specification

## Status

This document defines intended product behavior and scope.

It is not evidence that the system is implemented.

## Summary

This project provides a Rust-based workspace execution tool for coding agents.

The tool gives an LLM-driven coding agent controlled access to repository operations needed to build features, validate changes, and return structured observations.

The tool is not the agent itself. It is the controlled environment through which an agent interacts with a repository.

## Core Product Concept

The tool should support an agent development loop:

1. receive a task or feature-building intention;
2. inspect repository state through controlled interfaces;
3. execute commands to understand, build, test, or validate the feature;
4. optionally manage supporting processes;
5. return structured observations to the agent;
6. record evidence for review and reproducibility.

Version 0 implements only the first execution primitive: controlled foreground command execution.

## Product Boundary

The agent owns planning, code generation, and next-step decisions.

The Rust tool owns controlled workspace access, command execution, process control, structured observations, evidence capture, and safety boundaries.

## Problem Statement

Coding agents need workspace capabilities such as command execution, validation runs, process handling, and structured evidence. Unrestricted shell access is too risky and too hard to review.

The product should provide a constrained workspace tool that makes these actions explicit, bounded, machine-readable, and extensible toward policy and sessions.

## Primary User

The primary user is an LLM-driven coding agent operating on behalf of a human developer inside a controlled workspace.

Secondary users include human developers, reviewers, operators, and automation tools consuming structured results.

## Version 0 Goals

1. Accept a well-formed foreground `run` request.
2. Execute one command within a selected working directory.
3. Capture stdout, stderr, exit code, duration, and status.
4. Enforce a caller-specified timeout.
5. Return structured JSON output.
6. Produce a machine-readable execution log.
7. Establish the foundation for later policy, sessions, and richer workspace operations.

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
- The tool must run the command relative to the selected working directory.
- The tool must capture stdout separately from stderr.
- The tool must report exit code when available.
- The tool must report wall-clock duration and execution status.
- The tool must expose timeout results in structured output.
- The tool must produce machine-readable logs distinct from user-facing output.

## Product Constraints

- The tool should not be coupled to a specific LLM provider or agent framework.
- The tool should not interpret failures or decide next actions.
- The tool should keep request intake, execution, output reporting, and logging separate.
- The tool should prefer stable, typed, machine-readable contracts over ad hoc text output.

## Expected Future Extensions

- workspace inspection operations;
- long-lived process handles;
- session management;
- policy and approval hooks;
- richer structured logs and evidence;
- alternate adapters such as JSON, MCP, RPC, or HTTP.

## Documentation Obligation

Any future change that alters product behavior, scope, or the agent/tool boundary must update this file in the same task.
