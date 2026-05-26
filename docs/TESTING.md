# Testing Strategy

## Status

Version 0 production code and automated tests now exist. This document defines the validation standard for maintaining the implemented foreground execution path and for future post-v0 tasks.

## Testing Goals

Implementation and future changes should prove:

- commands execute in the intended working directory;
- stdout and stderr are captured correctly;
- exit codes are reported correctly;
- timeouts terminate long-running commands;
- structured JSON output is valid and complete;
- logging occurs for each command execution;
- policy enforcement behaves as documented.

## Test Pyramid For This Repository

### Unit Tests

Use for:

- argument parsing;
- request validation;
- timeout calculation;
- result serialization;
- policy decisions.

Expected command:

```bash
cargo test
```

### Integration Tests

Use for:

- end-to-end CLI invocation;
- real subprocess execution;
- cwd handling;
- timeout behavior;
- JSON output verification.

Expected command:

```bash
cargo test --test '*'
```

### Manual Validation

Use when needed for:

- checking CLI ergonomics;
- inspecting emitted logs;
- validating behavior with representative shell commands.

Representative commands:

```bash
cargo run -- run --cwd . --timeout 5 -- pwd
cargo run -- run --cwd . --timeout 5 -- echo hello
cargo run -- run --cwd . --timeout 1 -- sleep 5
```

For the implemented version 1 policy phase, use [docs/V1_ACCEPTANCE.md](V1_ACCEPTANCE.md) as the canonical manual smoke checklist covering success, timeout, and denial behavior.

For Phase 3A docs-only contract work, use manual cross-doc consistency review as the primary validation mode until [docs/PHASE3_ACCEPTANCE.md](PHASE3_ACCEPTANCE.md) exists.

## Task-Level Validation Contract

Every future task must explicitly list:

- the exact test commands to run;
- whether those commands are unit, integration, lint, format, or manual validation;
- what outcome is expected from each command.

If a task does not add or update automated tests, it must still provide:

- the reason no tests exist;
- the minimum manual validation commands required;
- the next follow-up task needed to add test coverage.

For docs-only Phase 3A contract tasks, the minimum manual validation is a source-of-truth consistency review across the listed docs, plus confirmation that no caller-visible schema or CLI behavior changed unintentionally.

## Minimum Validation Expectations

For future Rust implementation tasks, the default expectation is:

```bash
cargo fmt --check
cargo test
```

Additional commands should be listed when relevant, such as:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Acceptance Criteria Standard

Acceptance criteria should be observable and testable. Avoid vague criteria such as "works correctly."

Good examples:

- `llm-shell run` returns valid JSON containing `stdout`, `stderr`, `exit_code`, and `duration_ms`.
- A command exceeding the timeout is terminated and reported as timed out.
- A failing subprocess returns a non-zero exit code without crashing the CLI.

## Evidence Standard

Every completed task should record:

- commands executed;
- whether they passed or failed;
- any relevant notes on limitations or environment assumptions.

Use the PR template under `.github/pull_request_template.md` for this evidence.
