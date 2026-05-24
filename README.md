# Controlled Shell Broker

`Controlled Shell Broker` is a documentation-first repository for a Rust-based controlled command-line execution tool.

The tool is not the agent itself. It is the controlled environment through which an agent, script, or human can execute commands through a managed terminal interface.

## Summary

The long-term system should let an agent:

- run validation and build commands;
- manage foreground and long-lived processes;
- capture structured results and evidence;
- operate within working-directory, timeout, and policy boundaries.

Version 0 is intentionally smaller: one controlled foreground command execution primitive in an explicit working directory.

## Current Status

The repository is still documentation-only.

Included now:

- product and architecture docs;
- CLI and schema contracts;
- security, roadmap, testing, backlog, and decisions;
- workflow rules for future coding-agent sessions.

Not included yet:

- Rust crate scaffolding;
- CLI implementation;
- production code for execution, policy, logging, or sessions.

## Repository Guide

- [AGENTS.md](AGENTS.md): operating contract for future coding-agent sessions.
- [docs/PRODUCT_SPEC.md](docs/PRODUCT_SPEC.md): product behavior and v0 scope.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): system structure and boundaries.
- [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md): source of truth for caller-facing CLI behavior.
- [docs/REQUEST_RESPONSE_SCHEMA.md](docs/REQUEST_RESPONSE_SCHEMA.md): source of truth for request, response, error, event, and status shapes.
- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md): source of truth for v0 safety assumptions.
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
