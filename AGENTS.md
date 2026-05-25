# AGENTS.md

This repository is designed to support repeatable coding-agent sessions for building a Rust-based controlled command-line execution tool.

Treat this file as the operating contract for future work.

## Repository Purpose

This repository defines the controlled environment layer for agent-assisted software development.

The tool is not the intelligence. The agent decides what to do next.

The Rust tool should eventually provide:

- controlled command execution;
- process and session handling;
- structured observations and logs;
- safety boundaries and repeatable interfaces.

Version 0 is narrower: run one foreground command inside an explicit working directory, enforce a timeout, capture stdout and stderr, return structured JSON, and record basic execution evidence.

## Repository State

- The repo now includes an implemented version 0 Rust CLI and foreground execution path.
- The current phase is post-v0 hardening and phased expansion, not greenfield scaffolding.
- Do not assume broad new features are in scope; follow the current roadmap and backlog phase boundaries.
- Prefer updating docs alongside code when behavior, workflow, or architecture assumptions change.

## Product Boundary

The agent owns:

- planning;
- code generation;
- deciding which files to edit;
- interpreting failures;
- choosing the next action.

The Rust tool owns:

- controlled command execution in an explicit working directory;
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
7. `docs/ROADMAP.md`
8. `docs/TESTING.md`
9. `docs/BACKLOG.md`
10. `docs/DECISIONS.md`
11. `.github/pull_request_template.md`

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
- Treat `docs/CLI_CONTRACT.md` as the caller-facing CLI source of truth.
- Treat `docs/REQUEST_RESPONSE_SCHEMA.md` as the schema source of truth.
- Treat `docs/SECURITY_MODEL.md` as the v0 safety source of truth.
- Treat `docs/POLICY_MODEL.md` as the source of truth for the next post-v0 policy phase.

## Definition Of Done

A task is not complete until:

- the requirement is satisfied;
- scope changes are explained;
- listed tests were run or inability is explained;
- acceptance criteria are checked;
- the final summary matches the PR template structure.

## Out Of Scope For This Phase

Until explicitly requested, do not:

- add sessions or background process orchestration beyond the documented roadmap phase;
- add MCP, JSON-RPC, HTTP, or other adapter surfaces before the broker contract needs them;
- add approval workflows or interactive policy UI before the policy phase explicitly includes them;
- expand into code-editing, diff-analysis, or agent-planning behavior;
- introduce CI or automation changes unrelated to the stated task.
