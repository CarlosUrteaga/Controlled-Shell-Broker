# Roadmap

This roadmap describes phased evolution after the implemented version 0 execution primitive.

Version 0 is complete: the tool can validate and execute one foreground command in an explicit working directory, enforce a timeout, return structured JSON, and persist metadata-only execution evidence outside the target workspace.

Use [docs/V0_ACCEPTANCE.md](V0_ACCEPTANCE.md) as the canonical v0 completion checklist.

## Completed Phase 0: Documentation And Contract Design

- defined product scope and agent/tool boundaries;
- defined architecture, CLI, schema, security, testing, and workflow docs.

## Completed Phase 1: Version 0 Foreground Execution

- implemented the Rust CLI entry point;
- parse `run` requests;
- validate basic request shape;
- normalize input into canonical request types;
- execute one foreground command per request;
- apply explicit working directory and timeout behavior;
- capture stdout and stderr;
- report exit code, duration, and status;
- return structured JSON;
- persist basic metadata-only execution evidence.

## Next Phase 2: Basic Policy

- use [docs/POLICY_MODEL.md](POLICY_MODEL.md) as the source of truth for broker-layer admission control;
- introduce a policy seam with documented allow and deny behavior;
- define dangerous-command handling expectations;
- define path and working-directory restrictions beyond simple existence checks;
- keep approval hooks designed but unimplemented until a later phase expands scope explicitly.

## Future Phase 3: Evidence And Command Semantics

- enrich evidence retention and storage rules;
- decide whether durable state storage replaces temp-directory evidence;
- add command-semantic interpretation only if generic process semantics become insufficient.

## Future Phase 4: Sessions And Process Handles

- support long-lived processes;
- add session identifiers or handles;
- support stop and kill operations;
- define descendant cleanup semantics beyond the current direct-child timeout behavior.

## Future Phase 5: Sandbox And Execution Modes

- add sandbox profile selection where applicable;
- define sandbox override rules and failure modes;
- preserve consistent evidence across execution modes.

## Future Phase 6: Agent And Protocol Adapters

- add alternate request adapters when justified by stable broker contracts;
- evaluate JSON request input;
- defer MCP, JSON-RPC, and HTTP until they are needed without changing the v0 execution contract.

## Relationship To `BACKLOG.md`

`ROADMAP.md` defines the phase sequence after v0. `BACKLOG.md` tracks concrete work that starts after the completed execution primitive.
