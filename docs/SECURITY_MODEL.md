# Security Model

## Status

This document defines the intended security boundary and assumptions for the tool.

It is not evidence that the protections described here are implemented.

## Summary

The tool exists to provide controlled workspace access for coding agents, not unrestricted shell access.

Version 0 should improve structure, visibility, and bounded execution. It should not be mistaken for a full sandbox.

## Security Goals

The tool should make agent workspace actions:

- explicit;
- bounded;
- inspectable;
- machine-readable;
- easier to review than raw terminal usage.

## What The Tool Should Protect Against

- malformed requests reaching execution;
- ambiguity between harness flags and command payload;
- unbounded command runtime;
- inconsistent output structures;
- silent execution without machine-readable logs;
- interface drift that bypasses canonical validation paths.

## What Version 0 Does Not Protect Against

- malicious commands intentionally requested by the caller;
- secret exposure already present in the workspace or environment;
- destructive file changes performed by an allowed command;
- network access by spawned processes;
- privilege escalation beyond the running user;
- OS or kernel-level escape.

## Workspace Boundary Assumptions

- Commands run relative to a selected working directory.
- The effective `cwd` should be explicit in requests and results.
- `cwd` alone is not a complete containment boundary.
- Future implementation must define whether paths outside an approved workspace root are allowed.

## Dangerous Command Assumptions

- CLI validation checks request shape, not command safety.
- Dangerous-command handling belongs to a later policy layer.
- Future policy should support allowlists, denylists, path restrictions, or approval hooks without changing canonical request shapes.

## Timeout Assumptions

- Timeouts bound runtime, not side effects.
- A timed-out command may already have changed files or spawned children.
- Future implementation must define child-process behavior on timeout.
- Timeout results must be visible in structured output and logs.

## Environment And Secret Handling

- Environment variables may contain secrets.
- Future implementation must define whether commands inherit the parent environment by default.
- Logs and structured results should avoid recording secrets unless explicitly required.
- Environment handling should become an explicit policy surface later.

## Logging Risks

- Logs may capture sensitive command arguments, paths, stdout, or stderr.
- Machine-readable logging improves traceability but increases retention risk.
- Future log design should separate operational metadata from high-risk payload content when practical.

## Human Approval Assumptions

- Version 0 does not define a full human approval workflow.
- Later policy layers may require approval for specific commands, paths, or modes.
- The architecture should preserve enough metadata for external review before execution.

## Documentation Obligation

Any future task that changes workspace boundaries, timeout behavior, environment handling, logging risk posture, or policy direction must update this file.
