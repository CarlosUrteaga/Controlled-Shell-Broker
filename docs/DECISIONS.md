# Decisions

This file records repository-level decisions and constraints. Update it when a decision is added, changed, or superseded.

## Decision Log

### D-0001: Start Documentation-First

- Date: 2026-05-23
- Status: Accepted

The repository started without code and needed a stable operating contract before implementation.

Consequence:

- future work begins from explicit requirements instead of ad hoc assumptions.

### D-0002: Build A Rust Workspace Execution Tool

- Date: 2026-05-23
- Status: Accepted

The product is a Rust-based workspace execution tool for coding agents, not the agent itself.

Consequence:

- the tool owns controlled workspace capabilities while the agent owns planning and interpretation.

### D-0003: Version 0 Starts With Foreground `run`

- Date: 2026-05-23
- Status: Accepted

Version 0 focuses on one foreground `run` operation with structured results, timeout handling, and logging.

Consequence:

- the first implementation slice stays small enough to validate well.

### D-0004: Use A Reproducible Task Contract

- Date: 2026-05-23
- Status: Accepted

Every future task must include requirement, expected files, tests, acceptance criteria, and PR summary.

Consequence:

- coding-agent sessions are easier to review and resume.

### D-0005: Make The CLI An Adapter Around Canonical Requests

- Date: 2026-05-23
- Status: Accepted

The first architectural slice is the request boundary. The CLI parses external input and normalizes it into canonical typed requests instead of owning execution logic.

Consequence:

- future adapters can reuse the same internal request model.

### D-0006: Use `--` And Canonical Vector Commands

- Date: 2026-05-23
- Status: Accepted

The CLI uses `--` to separate harness arguments from the command payload, and the canonical command representation is `Vec<String>`.

Consequence:

- command parsing stays explicit and less ambiguous than shell-string defaults.

### D-0007: Split Docs By Concern

- Date: 2026-05-23
- Status: Accepted

Product behavior, architecture, CLI contract, schema, security model, and roadmap each live in separate source-of-truth documents.

Consequence:

- future sessions are less likely to introduce conflicting behavior across docs.

### D-0008: Keep Docs Small And Split By Subject

- Date: 2026-05-23
- Status: Accepted

`AGENTS.md` should stay around 50-200 lines, topic docs around 50-150 lines, and oversized topics should be split by subject into subdirectories instead of numbered overflow files.

Consequence:

- docs stay easier to navigate, review, and maintain as the feature set grows.
