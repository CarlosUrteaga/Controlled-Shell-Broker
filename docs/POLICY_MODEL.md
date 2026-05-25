# Policy Model

## Status

This document defines the post-v0 admission-control model for the broker.

It is not evidence that policy enforcement is implemented.

## Purpose

Policy exists to decide whether a valid execution request may proceed to command startup.

The policy layer is not the agent, not the CLI parser, and not the execution harness.

Its purpose is to make admission control explicit, reviewable, and extensible before command execution begins.

## Scope

This document defines:

- the policy decision vocabulary;
- where policy sits in the broker path;
- which request inputs policy may inspect;
- what policy must not do;
- candidate v1 rule families for initial implementation.

This document does not change version 0 behavior.

## Policy Decision Vocabulary

Policy should eventually return one of these decisions:

- `allow`: the request may proceed to execution;
- `deny`: the request must be rejected before execution;
- `require_approval`: the request is valid but must wait for an external approval path before execution.

Only `allow` and `deny` are candidates for the first policy implementation slice.

`require_approval` is part of the model design now, but the approval hook itself is not implemented in the next code slice.

## Broker Placement

Policy sits inside the broker path after request validation and normalization, and before command startup.

Expected order:

1. adapter parses CLI input;
2. adapter validates basic request shape;
3. adapter normalizes a valid request into canonical types;
4. broker applies policy admission control;
5. broker either rejects the request or hands it to execution;
6. execution and evidence handling continue only for admitted requests.

Policy is therefore distinct from CLI validation and distinct from process spawning.

## Inputs Policy May Inspect

Policy may inspect only request metadata that exists before execution:

- canonical command vector;
- resolved or normalized `cwd`;
- requested operation such as `run`;
- requested timeout;
- environment assumptions, including inherited-environment posture or future explicit environment fields;
- future caller metadata such as caller identity, trust tier, source adapter, or approval context.

Policy decisions should be made from declared request data, not from speculative runtime behavior.

## What Policy Must Not Do

Policy must not:

- execute the command to see what happens;
- interpret command stdout or stderr;
- mutate files in the target workspace or broker-owned state;
- replace request validation performed by the adapter;
- redefine canonical request or result schemas on its own;
- silently rewrite the command into a different payload.

If a request is malformed, validation should reject it before policy runs.

If a request is valid but disallowed, policy should deny it before execution begins.

## Relationship To Version 0

Version 0 remains unchanged:

- no policy engine is required for v0 correctness;
- no approval workflow is introduced in v0;
- no new CLI flags are required to describe policy;
- no command-safety guarantee is added retroactively to v0.

This policy model is a post-v0 design document for the next architecture slice after the foreground execution primitive.

## Candidate Version 1 Rule Families

The first implementation slice after v0 may add basic deny rules such as:

- denied executables by exact binary name;
- denied path patterns for command targets or working directories;
- workspace-root restrictions that require `cwd` to stay within one approved root;
- operation restrictions if later broker operations extend beyond `run`;
- timeout ceilings for commands that exceed broker policy limits.

These are admission rules, not runtime interpretation rules.

## Denied Executable Candidates

Examples of future deny candidates include executables associated with destructive deletion, privilege escalation, or uncontrolled remote access when the product policy forbids them.

The exact list should be documented alongside implementation, not inferred ad hoc in code.

## Denied Path Pattern Candidates

Examples of future deny candidates include requests that target system directories, hidden harness state directories, or parent-directory traversals outside an approved workspace root.

Path policy should operate on normalized paths rather than raw string fragments where practical.

## Workspace-Root Restriction Candidates

Post-v0 policy may require the broker to resolve one approved workspace root and deny requests whose `cwd` escapes that root.

This is stricter than v0 `cwd` existence checks and should be introduced as a policy decision, not as a hidden change to request validation.

## Approval Hook Boundary

The model reserves a future `require_approval` outcome so the broker can pause before execution when a rule requires human or external approval.

That hook is designed at the model level now but is explicitly not implemented in the next code slice.

The next code slice should stop at documented allow or deny behavior unless a later task broadens scope.

## Result Mapping Expectation

A denied request should be reported as a broker-level rejection before command execution.

The exact schema fields and error codes for policy denial should be defined with implementation work, not in this design-only document.

## Documentation Obligation

Any future task that implements policy must update this document, the architecture, the schema, the security model, and the caller-facing contract if policy becomes visible to callers.
