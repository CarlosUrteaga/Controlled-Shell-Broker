# Backlog

This backlog starts after the completed version 0 execution primitive.

Version 0 is no longer an open implementation target. The canonical proof of completion is [docs/V0_ACCEPTANCE.md](V0_ACCEPTANCE.md).

## Version 0 Completion Status

Completed in the current implementation:

- `run --cwd <PATH> --timeout <SECONDS> -- <COMMAND> [ARGS...]`;
- request validation for required flags and argument shape;
- one foreground command execution path;
- structured JSON result mapping for `success`, `failed`, `execution_error`, and `timed_out`;
- metadata-only evidence persistence outside the target workspace.

Version 0 follow-up for release readiness:

- keep the smoke checks in [docs/V0_ACCEPTANCE.md](V0_ACCEPTANCE.md) passing;
- keep caller-facing behavior aligned with [docs/CLI_CONTRACT.md](CLI_CONTRACT.md);
- keep canonical result and evidence shapes aligned with [docs/REQUEST_RESPONSE_SCHEMA.md](REQUEST_RESPONSE_SCHEMA.md).

## Post-v0 Backlog

The remaining backlog begins after the execution primitive.

### Policy And Admission Control

The next concrete post-v0 work item is the policy model defined in [docs/POLICY_MODEL.md](POLICY_MODEL.md).

Near-term follow-up:

- implement documented `allow` and `deny` admission behavior without changing v0 request validation;
- preserve `require_approval` as a designed future outcome without implementing approval hooks in the next slice;
- specify working-directory root restrictions beyond simple `cwd` validation;
- document environment-shaping rules if v1 changes inherited environment behavior.

### Phase 3A: Durable Evidence And Retention

- decide whether evidence moves from the temp directory to a durable state directory;
- define retention and cleanup expectations;
- document any future output-capture policy beyond metadata-only evidence.

### Phase 3B: Open Inspection Mode

- define a read-oriented inspection profile that preserves broker policy, timeout, workspace-root, and evidence boundaries;
- start with repository discovery commands such as listing paths, finding files, searching text, and bounded file reading;
- deny mutation, privilege escalation, network-oriented commands, shell escape hatches, and out-of-workspace access in the inspection profile;
- add inspection-oriented evidence metrics without redefining the core execution result shape;
- use observed command usage to decide whether higher-level inspection primitives are justified later.

### Command Semantics Decision

- keep generic process semantics as the default result model;
- require a concrete documented insufficiency before introducing command-family-specific result interpretation.

### Process Lifecycle

- add explicit process-group handling if descendant cleanup becomes required;
- design long-lived sessions and stop or kill semantics only after the foreground path stays stable.

### Adapter Expansion

- evaluate JSON request input only after the CLI contract remains stable;
- defer MCP, JSON-RPC, and HTTP adapters until the broker contract needs them.

## Excluded From This Backlog Slice

Do not reopen version 0 scope by adding:

- sessions before post-v0 lifecycle design exists;
- policy engine behavior without a documented phase change;
- MCP, JSON-RPC, or HTTP transport work;
- diff analysis, editor integration, or code-editing behavior.

Those remain outside the completed v0 execution primitive unless a later documented phase expands the product boundary explicitly.
