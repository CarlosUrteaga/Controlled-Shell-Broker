# Roadmap

This roadmap describes the intended phased evolution of the tool.

It is directional, not a delivery commitment.

## Phase 0: Documentation-Only Design

- define product scope and boundaries;
- define architecture, CLI, schema, and security docs;
- define testing and workflow standards.

## Phase 1: CLI Request Interface

- implement the Rust CLI entry point;
- parse `run` requests;
- validate basic request shape;
- normalize input into canonical request types;
- return structured invalid-request errors.

## Phase 2: Foreground Command Execution

- execute one foreground command per request;
- apply working directory and timeout behavior;
- capture stdout and stderr;
- report exit code, duration, and status.

## Phase 3: Logging And Evidence

- emit machine-readable execution events;
- record enough metadata for review and traceability;
- establish the initial evidence model.

## Phase 4: Basic Policy

- introduce a policy layer;
- define allow/deny behavior;
- define dangerous-command handling;
- add approval hooks where needed.

## Phase 5: Workspace Operations

- add controlled workspace inspection capabilities;
- add diff or state-observation primitives;
- support validation-oriented workflows beyond raw command runs.

## Phase 6: Sessions And Process Handles

- support long-lived processes;
- add session identifiers or handles;
- support stop and kill operations;
- support multiple managed sessions.

## Phase 7: Agent And Protocol Adapters

- add alternate request adapters;
- support JSON request input;
- support future MCP, RPC, or HTTP integrations;
- preserve canonical contracts across adapters.

## Relationship To `BACKLOG.md`

`ROADMAP.md` defines phases. `BACKLOG.md` defines concrete next design tasks.
