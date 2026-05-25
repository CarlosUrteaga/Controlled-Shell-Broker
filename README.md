# Controlled Shell Broker

`Controlled Shell Broker` is a Rust-based controlled command-line execution tool with an implemented version 0 foreground execution primitive.

The tool is not the agent itself. It is the controlled environment through which an agent, script, or human can execute commands through a managed terminal interface.

## Summary

The long-term system should let an agent:

- run validation and build commands;
- manage foreground and long-lived processes;
- capture structured results and evidence;
- operate within working-directory, timeout, and policy boundaries.

Version 0 is intentionally smaller: one controlled foreground command execution primitive in an explicit working directory.

## Current Status

Implemented now:

- the Rust CLI for `run`;
- request validation for `--cwd`, `--timeout`, and `--`;
- one foreground command execution path;
- structured JSON results for `success`, `failed`, `execution_error`, and `timed_out`;
- metadata-only execution evidence stored outside the target workspace;
- product, architecture, CLI, schema, security, roadmap, testing, backlog, and decisions docs.

Not included yet:

- policy rules or approval workflows;
- sessions or background process management;
- MCP, JSON-RPC, or HTTP adapters;
- code-editing or diff-analysis features.

## Repository Guide

- [AGENTS.md](AGENTS.md): operating contract for future coding-agent sessions.
- [docs/PRODUCT_SPEC.md](docs/PRODUCT_SPEC.md): product behavior and v0 scope.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): system structure and boundaries.
- [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md): source of truth for caller-facing CLI behavior.
- [docs/REQUEST_RESPONSE_SCHEMA.md](docs/REQUEST_RESPONSE_SCHEMA.md): source of truth for request, response, error, event, and status shapes.
- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md): source of truth for v0 safety assumptions.
- [docs/V0_ACCEPTANCE.md](docs/V0_ACCEPTANCE.md): canonical smoke checks and expected outcomes for the implemented v0 primitive.
- [docs/ROADMAP.md](docs/ROADMAP.md): phased evolution.
- [docs/TESTING.md](docs/TESTING.md): validation expectations.
- [docs/BACKLOG.md](docs/BACKLOG.md): near-term design tasks.
- [docs/DECISIONS.md](docs/DECISIONS.md): recorded design decisions.

## Working Rule

Every future task must define:

- a clear requirement;
- expected files to change;
- tests to run;
- acceptance criteria;
- a PR summary in the repository format.

## Version 0 Contract

```bash
llm-shell run --cwd ./repo --timeout 30 -- cargo test
```

Version 0 means:

- one foreground command per request;
- explicit `--cwd` and `--timeout`;
- stdout and stderr captured separately;
- exit code, duration, and status returned as JSON;
- one machine-readable execution evidence record per run;
- no sessions, MCP, policy engine, or autonomous planning.

Use [docs/V0_ACCEPTANCE.md](docs/V0_ACCEPTANCE.md) for the canonical release-readiness checklist and smoke commands.
