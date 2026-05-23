# AGENTS.md

This repository is designed to support repeatable coding-agent sessions for building a Rust-based command-line execution harness.

Treat this file as the operating contract for future work.

## Repository Purpose

This repository implements the environment layer for LLM-assisted development workflows.

The goal is to build a Rust-based command-line harness that can eventually allow an LLM-driven agent or external controller to:

- execute shell commands;
- spawn and manage terminal/process sessions;
- capture stdout, stderr, exit codes, and durations;
- enforce timeouts and execution policies;
- manage working directories and repository boundaries;
- produce structured logs for every action.

This repository is not the final agent.

It is the execution substrate that future agents, external controllers, or coding workflows may use.

## Repository State

- The repo is documentation-only until implementation work is explicitly requested.
- The current phase is harness design, not production implementation.
- Do not add Rust source code, dependencies, CLI scaffolding, or CI unless the task explicitly requests it.
- Do not invent production behavior that conflicts with `docs/PRODUCT_SPEC.md` or `docs/ARCHITECTURE.md`.
- Prefer updating the docs first when a requirement, workflow, or architecture assumption changes.

## Design Boundary

The harness should be designed as a controlled execution broker, not as unrestricted shell access.

A normal terminal executes whatever the user types.

This harness must eventually validate, constrain, execute, capture, and log commands in a structured way.

The intended future architecture is:

```text
LLM / Agent / External Controller
   |
   v
Rust CLI Harness
   |
   +-- command runner
   +-- process/session manager
   +-- policy layer
   +-- workspace boundary manager
   +-- stdout/stderr capture
   +-- structured logs
   +-- timeout/kill controls
   |
   v
Operating System / Repository / Terminal
```

## Required Reading Order

Before starting any implementation task, read:

1. `README.md`
2. `docs/PRODUCT_SPEC.md`
3. `docs/ARCHITECTURE.md`
4. `docs/TESTING.md`
5. `docs/BACKLOG.md`
6. `docs/DECISIONS.md`
7. `.github/pull_request_template.md`

## Task Intake Contract

Every future task must be stated in a form that includes all of the following:

- `Requirement`: one concise statement of the desired behavior or change.
- `Expected files to change`: exact files or directories expected to be modified.
- `Tests to run`: concrete validation commands or a justified statement that no automated tests exist yet.
- `Acceptance criteria`: verifiable outcomes.
- `PR summary`: a summary written in the repository PR format.

If any item is missing, the agent should supply a proposed version before coding and use it as the working contract for the task.

## Required Task Template

Use this template for all future implementation tasks:

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

## Execution Rules For Agents

- Keep changes scoped to the stated requirement.
- Avoid touching unrelated files.
- Update documentation when behavior, interfaces, workflow, or architecture changes.
- Prefer small, reviewable diffs.
- Add tests with implementation work whenever practical.
- If a task conflicts with an existing documented decision, update `docs/DECISIONS.md` as part of the change.

## Definition Of Done

A task is not complete until all of the following are true:

- the requirement is satisfied;
- expected files were the only files changed, or the scope expansion is documented;
- all listed tests were run, or inability to run them is explained;
- acceptance criteria are explicitly checked;
- the final summary matches the PR template structure.

## PR Summary Format

Use the existing repository template in `.github/pull_request_template.md`. At minimum, every final task summary should cover:

- `Plan`
- `Implementation summary`
- `Evidence`
- `Security / governance review`
- `Review checklist`

## When Docs Must Be Updated

Update the following files when applicable:

- `README.md`: repo purpose or setup expectations changed.
- `docs/PRODUCT_SPEC.md`: user-facing requirements or scope changed.
- `docs/ARCHITECTURE.md`: component boundaries, data flow, or module contracts changed.
- `docs/TESTING.md`: required validation strategy changed.
- `docs/BACKLOG.md`: priority or sequencing changed.
- `docs/DECISIONS.md`: a design or workflow decision was made, reversed, or superseded.

## Out Of Scope For This Phase

Until explicitly requested, do not:

- add production Rust code;
- add dependencies;
- scaffold the CLI;
- implement the execution engine;
- introduce CI beyond documentation changes.
