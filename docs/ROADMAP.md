# Roadmap

This roadmap describes the intended phased evolution of the tool.

It is directional, not a delivery commitment.

## Phase 0: Documentation-Only Design

- define product scope and boundaries;
- define architecture, CLI, schema, and security docs;
- define testing and workflow standards.

## Phase 1: Version 0 Foreground Execution

- implement the Rust CLI entry point;
- parse `run` requests;
- validate basic request shape;
- normalize input into canonical request types;
- execute one foreground command per request;
- apply explicit working directory and timeout behavior;
- capture stdout and stderr;
- report exit code, duration, and status;
- return structured JSON;
- persist basic execution evidence.

## Phase 2: Basic Policy

- introduce a policy layer;
- define allow/deny behavior;
- define dangerous-command handling;
- add approval hooks where needed;
- define direct-command versus shell-mediated execution policy;
- define path and working-directory validation rules.

## Phase 3: Command Semantics And Evidence

- interpret command-specific exit semantics where non-zero does not always mean error;
- enrich execution evidence with normalized statuses and semantic notes;
- define evidence retention and truncation rules;
- add stronger validation-oriented workflows without changing the core execution contract.

## Phase 4: Sessions And Process Handles

- support long-lived processes;
- add session identifiers or handles;
- support stop and kill operations;
- support multiple managed sessions;
- define background execution lifecycle and cleanup behavior;
- define process-group handling and descendant cleanup semantics.

## Phase 5: Sandbox And Execution Modes

- add sandbox profile selection where applicable;
- define sandbox override rules;
- document sandbox-caused failure modes;
- keep execution evidence consistent across execution modes.

## Phase 6: Agent And Protocol Adapters

- add alternate request adapters;
- support JSON request input;
- support future MCP, RPC, or HTTP integrations;
- preserve canonical contracts across adapters;
- add optional editor or integration hooks without changing the core broker contract.

## Relationship To `BACKLOG.md`

`ROADMAP.md` defines phases. `BACKLOG.md` defines concrete next design tasks.
