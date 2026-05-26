# Policy Model

## Status

This document defines the version 1 admission-control model for the broker.

It is not evidence that policy enforcement is implemented.

## Purpose

Policy decides whether a valid execution request may proceed to command startup.

The policy layer is not the agent, not the CLI parser, and not the execution harness.

Its purpose is to make admission control explicit, reviewable, and extensible before command execution begins.

## Version 1 Contract

Version 1 adds broker-layer policy after request validation and before subprocess spawn.

Locked defaults:

- caller-visible policy rejection uses `status: "denied"`;
- the first approved workspace root is the broker startup `cwd`, canonicalized once;
- denied requests produce machine-readable evidence;
- `require_approval` remains documented but unimplemented in v1.

Policy does not change CLI syntax or request-validation behavior.

## Policy Decision Vocabulary

Policy defines these decisions:

- `allow`: the request may proceed to execution;
- `deny`: the request must be rejected before execution;
- `require_approval`: the request is valid but must wait for an external approval path before execution.

Version 1 implements only `allow` and `deny`.

`require_approval` remains part of the model design, but no approval hook or caller-visible approval result is implemented in v1.

## Broker Placement

Policy sits inside the broker path after request validation and normalization, and before command startup.

Expected order:

1. adapter parses CLI input;
2. adapter validates basic request shape;
3. adapter normalizes a valid request into canonical types;
4. broker applies policy admission control;
5. broker either denies the request or hands it to execution;
6. the broker persists evidence for either the denial or the completed execution;
7. the broker returns a canonical result to the adapter.

Policy is distinct from CLI validation and distinct from process spawning.

## Inputs Policy May Inspect

Policy may inspect only request metadata that exists before execution:

- canonical command vector;
- resolved `cwd`;
- requested operation such as `run`;
- requested timeout;
- the canonicalized broker startup workspace root;
- environment assumptions, including inherited-environment posture or future explicit environment fields;
- future caller metadata such as caller identity, trust tier, source adapter, or approval context.

Policy decisions should be made from declared request data, not from speculative runtime behavior.

## What Policy Must Not Do

Policy must not:

- execute the command to see what happens;
- interpret command stdout or stderr;
- mutate files in the target workspace or broker-owned state;
- replace request validation performed by the adapter;
- silently rewrite the command into a different payload.

If a request is malformed, validation rejects it before policy runs.

If a request is valid but disallowed, policy returns `deny` before execution begins.

## Caller-Visible Denial Semantics

A policy denial is a broker-level rejection, not a malformed request and not a harness failure.

Denied responses use the standard execution result envelope with:

- `status: "denied"`;
- `exit_code: null`;
- empty `stdout` and `stderr`;
- `duration_ms: 0`;
- `timed_out: false`;
- `error.code` and `error.message` describing the denial reason.

Denied requests must not spawn a subprocess.

Reason-specific denial codes are preferred over a single generic policy code.

## Initial Version 1 Rules

Version 1 defines these first rule families:

- workspace-root restriction on the canonicalized request `cwd`;
- denied executables by exact basename match.

The initial workspace-root rule treats the broker startup `cwd`, canonicalized once, as the approved root.

The root path itself is allowed. Descendant paths are allowed. Requests outside that root are denied with `cwd_outside_workspace_root`.

This workspace-root restriction is the first implemented deny rule in version 1.

The initial denied-executable rule family uses reason-specific code `denied_executable`.

The concrete exact-basename denylist is: `rm`, `sudo`, `su`, `shutdown`, `reboot`, `mkfs`, `dd`.

Basename matching is exact and applies to both bare names such as `rm` and path forms such as `/bin/rm`.

These are admission rules, not runtime interpretation rules.

## Evidence Expectation

A denied request must still produce one machine-readable evidence record.

Denied evidence is separate from caller JSON and is written to the same broker-managed evidence location used for executed requests.

The denied evidence record uses event type `execution.denied` and includes the request metadata, denied status, and denial error code. Version 1 does not add persisted stdout or stderr for denied requests.

## Approval Hook Boundary

The model reserves a future `require_approval` outcome so the broker can pause before execution when a rule requires human or external approval.

That hook is intentionally deferred beyond version 1.

## Documentation Obligation

Any future task that changes policy decisions, denial result semantics, workspace-root rules, or evidence behavior must update this document, the architecture, the schema, the security model, and the caller-facing contract in the same task.
