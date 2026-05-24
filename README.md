# Controlled Shell Broker

`Controlled Shell Broker` is a documentation-first repository for a Rust-based workspace execution tool for coding agents.

The tool is not the agent itself. It is the controlled environment through which an agent, script, or human can interact with a repository.

## Summary

The long-term system should let an agent:

- run validation and build commands;
- inspect repository state through controlled interfaces;
- manage foreground and long-lived processes;
- capture structured results and evidence;
- operate within workspace, timeout, and policy boundaries.

Version 0 starts with one primitive: controlled foreground command execution.

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
- [docs/PRODUCT_SPEC.md](docs/PRODUCT_SPEC.md): product behavior and scope.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): system structure and boundaries.
- [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md): caller-facing CLI behavior.
- [docs/REQUEST_RESPONSE_SCHEMA.md](docs/REQUEST_RESPONSE_SCHEMA.md): canonical shapes.
- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md): safety assumptions and limits.
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

## Initial CLI Goal

```bash
llm-shell run --cwd ./repo --timeout 30 -- cargo test
```

That command is the first slice of the broader workspace execution tool vision.
