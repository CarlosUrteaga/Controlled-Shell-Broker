# Backlog

This backlog is focused on Version 0 of the Rust-based agent workspace execution tool.

Version 0 should provide the smallest useful execution primitive for a coding agent:

> Run one foreground command inside a controlled workspace, capture the result, return structured JSON, and record basic execution evidence.

The project is still documentation-first. Implementation work should begin only after the v0 contract is stable enough to guide a coding-agent session without requiring architectural invention.

## Version 0 Goal

Enable a coding agent, human developer, or script to call the tool with a command such as:

```bash
llm-shell run --cwd ./repo --timeout 30 -- cargo test
```

The tool should eventually:

- parse the request;
- validate the request shape;
- run one foreground command;
- apply the selected working directory;
- enforce a timeout;
- capture stdout and stderr separately;
- report exit code and duration;
- return structured JSON;
- produce a basic machine-readable execution log.

Version 0 is not the full agent workspace tool. It is the first execution primitive required for that tool.

## Closed Version 0 Contract

The v0 documentation contract is closed on the following points:

1. `--cwd` is required.
2. `--timeout` is required and must be positive.
3. The command must appear after the `--` separator.
4. The canonical command model is an argument vector.
5. Shell-string execution is out of scope for v0.
6. The adapter generates `request_id` for valid CLI requests.
7. Environment inheritance is the default v0 assumption.
8. Non-zero exit is `failed`; startup or harness failure is `execution_error`.
9. Timeout is represented by `status: "timed_out"` and `timed_out: true`.
10. One machine-readable execution evidence record is persisted per run outside the target workspace.

## Recommended First Implementation Task

The first implementation task should be intentionally small.

```md
## Task

- Requirement:
  Scaffold the Rust CLI and implement request parsing plus structured invalid-request responses for the v0 `run` command without executing commands yet.

- Expected files to change:
  - `Cargo.toml`
  - `src/main.rs`
  - `src/cli.rs`
  - `src/types.rs`
  - `docs/DECISIONS.md`

- Tests to run:
  - `cargo test`
  - `cargo run -- run --cwd . --timeout 30 -- cargo test`

- Acceptance criteria:
  - [ ] The CLI accepts `run --cwd <PATH> --timeout <SECONDS> -- <COMMAND> [ARGS...]`.
  - [ ] The CLI requires both `--cwd` and `--timeout`.
  - [ ] The CLI rejects missing command payloads.
  - [ ] The CLI rejects nonexistent working directories.
  - [ ] The CLI rejects invalid timeout values.
  - [ ] The CLI converts valid input into a typed v0 request structure.
  - [ ] The parsed command is observable through a test or placeholder structured response.
  - [ ] No command execution occurs yet.
  - [ ] Documentation is updated if the implemented CLI shape differs from the contract.

- PR summary:
  - What changed:
  - Why this approach:
  - Evidence:
```

The second implementation task should add foreground command execution, timeout handling, and structured command results.

The third implementation task should add persisted execution evidence.

## Deferred Until After Version 0

The backlog should not expand beyond the v0 execution primitive. Broader future phases belong in [docs/ROADMAP.md](ROADMAP.md), not here.

## Readiness Focus

Until implementation begins, backlog work should stay constrained to doc consistency across the v0 source-of-truth files, implementation-ready task shaping, explicit decision capture in `docs/DECISIONS.md`, and manual conflict checks across README, product, architecture, CLI, schema, and security docs.

## Task Readiness Rule

An item is ready for implementation only when the task statement includes:

- a clear requirement;
- expected files to change;
- tests to run;
- acceptance criteria;
- a PR summary format.

## Not In Scope Yet

Until the v0 documentation contract is stable, do not add:

- Rust crate setup;
- CLI implementation;
- process management code;
- policy enforcement code;
- CI or workflow automation beyond documentation checks.
