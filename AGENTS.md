# AGENTS.md

This repository is designed to support repeatable coding-agent sessions for building a Rust-based workspace execution tool for coding agents.

Treat this file as the operating contract for future work.

## Repository Purpose

This repository defines the controlled environment layer for agent-assisted software development.

The tool is not the intelligence. The agent decides what to do next.

The Rust tool should eventually provide:

- controlled command execution;
- repository and workspace context access;
- process and session handling;
- structured observations and logs;
- safety boundaries and repeatable interfaces.

Version 0 is narrower: a controlled foreground command runner with typed requests and structured results.

## Repository State

- The repo is documentation-only until implementation work is explicitly requested.
- The current phase is design, not production implementation.
- Do not add Rust source code, dependencies, CLI scaffolding, or CI unless the task explicitly requests it.
- Prefer updating docs first when behavior, workflow, or architecture assumptions change.

## Product Boundary

The agent owns:

- planning;
- code generation;
- deciding which files to edit;
- interpreting failures;
- choosing the next action.

The Rust tool owns:

- controlled workspace access;
- command execution;
- process lifecycle control;
- structured observations;
- evidence capture;
- safety boundaries;
- repeatable interfaces.

## Doc Structure Rule

- `AGENTS.md` should stay between roughly 50 and 200 lines.
- Topic docs should stay between roughly 50 and 150 lines.
- If a topic grows too large, split it by subject into a subdirectory such as `docs/security/` or `docs/architecture/`.
- Avoid numbered overflow files such as `_1` or `_2` unless no clearer subject split exists.

## Required Reading Order

Before starting implementation work, read:

1. `README.md`
2. `docs/PRODUCT_SPEC.md`
3. `docs/ARCHITECTURE.md`
4. `docs/CLI_CONTRACT.md`
5. `docs/REQUEST_RESPONSE_SCHEMA.md`
6. `docs/SECURITY_MODEL.md`
7. `docs/TESTING.md`
8. `docs/BACKLOG.md`
9. `docs/DECISIONS.md`
10. `.github/pull_request_template.md`

## Task Intake Contract

Every future task must include:

- `Requirement`
- `Expected files to change`
- `Tests to run`
- `Acceptance criteria`
- `PR summary`

If any item is missing, the agent should propose a concrete version before coding.

## Required Task Template

```md
## Task

- Requirement:
- Expected files to change:
  - `path/to/file`
- Tests to run:
  - `cargo test`
- Acceptance criteria:
  - [ ] Behavior is implemented as described
  - [ ] Tests pass
  - [ ] No unrelated files changed
- PR summary:
  - What changed:
  - Why this approach:
  - Evidence:
```

## Execution Rules

- Keep changes scoped to the stated requirement.
- Avoid touching unrelated files.
- Update docs when behavior, interfaces, workflow, or architecture changes.
- Prefer small, reviewable diffs.
- Add tests with implementation work whenever practical.
- Update `docs/DECISIONS.md` when a design choice is added or reversed.

## Definition Of Done

A task is not complete until:

- the requirement is satisfied;
- scope changes are explained;
- listed tests were run or inability is explained;
- acceptance criteria are checked;
- the final summary matches the PR template structure.

## Out Of Scope For This Phase

Until explicitly requested, do not:

- add production Rust code;
- add dependencies;
- scaffold the CLI;
- implement the broker;
- introduce CI beyond documentation changes.
