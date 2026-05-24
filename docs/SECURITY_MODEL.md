# Security Model

## Status

This document defines the intended security boundary and assumptions for the tool.

It is not evidence that the protections described here are implemented.

## Summary

The tool exists to provide controlled command-line access for coding agents, not unrestricted shell access.

Version 0 should improve structure, visibility, and bounded execution. It should not be mistaken for a full sandbox.

This file is the source of truth for v0 safety assumptions and limits.

## Security Goals

The tool should make command execution actions:

- explicit;
- bounded;
- inspectable;
- machine-readable;
- easier to review than raw terminal usage.

## What The Tool Should Protect Against

- malformed requests reaching execution;
- ambiguity between harness flags and command payload;
- accidental omission of execution boundaries such as `cwd` or timeout;
- unbounded command runtime;
- inconsistent output structures;
- silent execution without machine-readable logs;
- interface drift that bypasses canonical validation paths.

## What Version 0 Does Not Protect Against

- malicious commands intentionally requested by the caller;
- secret exposure already present in the selected working directory or inherited environment;
- destructive file changes performed by an allowed command;
- network access by spawned processes;
- privilege escalation beyond the running user;
- OS or kernel-level escape.

## Workspace Boundary Assumptions

- Commands run relative to a selected working directory.
- The selected working directory is explicit and required in v0.
- The effective `cwd` should be explicit in requests and results.
- `cwd` alone is not a complete containment boundary.
- Version 0 does not yet define working-directory-root allowlists or path jail behavior.
- In v0, "workspace" should be read narrowly as the current working directory context in which the command runs.

## Dangerous Command Assumptions

- CLI validation checks request shape, not command safety.
- Version 0 allows direct executable invocation only; it does not define shell-string execution.
- Dangerous-command handling belongs to a later policy layer.
- Future policy should support allowlists, denylists, path restrictions, or approval hooks without changing canonical request shapes.

## Timeout Assumptions

- Timeouts bound runtime, not side effects.
- A timed-out command may already have changed files or spawned children.
- Version 0 requires timeout reporting in both the result and the evidence record.
- Version 0 should attempt to terminate the spawned command path on timeout, but it does not promise full descendant cleanup semantics.
- Timeout results must be visible in structured output and logs.

## Version 0 Timeout Limitation

Version 0 only guarantees timeout detection and a best-effort attempt to terminate the direct child process.

It does not guarantee full process-tree cleanup, process-group termination, container isolation, or operating-system sandboxing.

Commands that spawn child processes may leave descendants behind.

This is a known v0 limitation and should be revisited before adding long-lived sessions or broader automation.

## Environment And Secret Handling

- Environment variables may contain secrets.
- Version 0 assumes the spawned command inherits the parent environment unless a later policy surface changes that behavior.
- Logs and structured results should avoid recording secrets unless explicitly required.
- Environment handling should become an explicit policy surface later.

## Logging Risks

- Logs may capture sensitive command arguments, paths, stdout, or stderr.
- Machine-readable logging improves traceability but increases retention risk.
- Version 0 requires one persisted machine-readable execution record per run.
- Version 0 evidence must be stored outside the target workspace in a tool-managed location.
- Future log design should separate operational metadata from high-risk payload content when practical.

## Evidence Logging Risk

Version 0 evidence records should avoid persisting full stdout and stderr by default because command output may contain secrets, tokens, file paths, or proprietary data.

The CLI response may return stdout and stderr to the caller, but persisted evidence should prioritize metadata unless a later logging policy explicitly allows output capture.

## Human Approval Assumptions

- Version 0 does not define a full human approval workflow.
- Later policy layers may require approval for specific commands, paths, or modes.
- The architecture should preserve enough metadata for external review before execution.

## Security Posture Summary

Version 0 provides:

- explicit request boundaries;
- explicit `cwd` and timeout inputs;
- direct argument-vector execution;
- structured JSON results;
- persisted machine-readable execution evidence.

Version 0 does not provide:

- command safety policy;
- secret isolation;
- network isolation;
- filesystem sandboxing;
- session governance.

## Documentation Obligation

Any future task that changes working-directory boundaries, timeout behavior, environment handling, logging risk posture, or policy direction must update this file.
