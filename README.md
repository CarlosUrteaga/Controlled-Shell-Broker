# Controlled Shell Broker

`Controlled Shell Broker` is a documentation-first repository for a Rust-based CLI harness that gives an LLM agent structured, auditable, and policy-controlled access to terminal execution.

The repository currently defines the product, architecture, workflow, and delivery standards. It does not yet contain production Rust code.

## Purpose

The harness is intended to be the execution layer for LLM-assisted software development, not the autonomous agent itself.

The long-term system should allow an agent to:

- execute shell commands inside an approved workspace;
- capture stdout, stderr, exit codes, and duration;
- manage long-lived processes and terminal sessions;
- enforce timeout and policy controls;
- log every action in a structured and reviewable format.

## Current Status

This repository is in pre-implementation mode.

Included now:

- product requirements;
- architecture intent;
- testing strategy;
- backlog and decisions;
- agent-session workflow rules for reproducible future tasks.

Not included yet:

- Rust crate scaffolding;
- CLI implementation;
- production code for execution, policy, logging, or session management.

## Repository Guide

- [AGENTS.md](AGENTS.md): required workflow for future coding-agent sessions.
- [docs/PRODUCT_SPEC.md](docs/PRODUCT_SPEC.md): product scope and version goals.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): planned system structure and boundaries.
- [docs/TESTING.md](docs/TESTING.md): validation strategy and task-level test expectations.
- [docs/BACKLOG.md](docs/BACKLOG.md): ordered implementation backlog.
- [docs/DECISIONS.md](docs/DECISIONS.md): recorded design and process decisions.
- [.github/pull_request_template.md](.github/pull_request_template.md): PR template that all future tasks should satisfy.

## Working Rule

Future implementation work should start by reading `AGENTS.md` and the documents in `docs/`. Every task must define:

- a clear requirement;
- expected files to change;
- tests to run;
- acceptance criteria;
- a PR summary in the repository format.

## Initial CLI Goal

The first production milestone remains:

```bash
llm-shell run --cwd ./repo --timeout 30 "cargo test"
```

Expected `v0` behavior is documented in [docs/PRODUCT_SPEC.md](docs/PRODUCT_SPEC.md). No implementation has been started yet.
