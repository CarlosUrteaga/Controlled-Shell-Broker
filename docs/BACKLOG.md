# Backlog

This backlog is ordered by recommended implementation sequence. It is a planning artifact, not a commitment.

## Phase 0: Repository Setup

1. Initialize the Rust crate for the CLI harness.
2. Define core request and response types.
3. Establish formatting, lint, and test commands.
4. Add baseline integration test scaffolding.

## Phase 1: `run` Command

1. Implement CLI argument parsing for `run`.
2. Implement single-command execution.
3. Add working directory support.
4. Add timeout handling.
5. Capture stdout and stderr separately.
6. Return structured JSON results.
7. Add basic execution logging.

## Phase 2: Safety And Policy

1. Define a command policy model.
2. Implement allow/deny checks.
3. Add dangerous-command handling rules.
4. Document approval and escalation boundaries.

## Phase 3: Session Management

1. Introduce long-lived process support.
2. Add process handles or session IDs.
3. Support stop/kill operations.
4. Support multiple concurrent managed sessions.

## Phase 4: Observability

1. Improve structured log schema.
2. Add log destinations and retention strategy.
3. Expose richer execution metadata.
4. Add traceability across multi-step workflows.

## Cross-Cutting Work

1. Keep docs aligned with implementation.
2. Add tests with each feature.
3. Record design decisions in `docs/DECISIONS.md`.
4. Keep task summaries aligned with the PR template.

## Task Readiness Rule

An item should be considered ready for implementation only when the task statement includes:

- a clear requirement;
- expected files to change;
- tests to run;
- acceptance criteria;
- a PR summary format.
