# Decisions

This file records repository-level decisions and constraints. Update it when a decision is added, changed, or superseded.

## Decision Log

### D-0001: Start Documentation-First

- Date: 2026-05-23
- Status: Accepted
The repository started without code and needed a stable operating contract before implementation.
- future work begins from explicit requirements instead of ad hoc assumptions.

### D-0002: Build A Rust Controlled Command-Line Execution Tool

- Date: 2026-05-23
- Status: Accepted
The product is a Rust-based controlled command-line execution tool, not the agent itself.
- the tool owns controlled command-line capabilities while the agent owns planning and interpretation.

### D-0003: Version 0 Starts With Foreground `run`

- Date: 2026-05-23
- Status: Accepted
Version 0 focuses on one foreground `run` operation with structured results, timeout handling, and logging.
- the first implementation slice stays small enough to validate well.

### D-0004: Use A Reproducible Task Contract

- Date: 2026-05-23
- Status: Accepted
Every future task must include requirement, expected files, tests, acceptance criteria, and PR summary.
- coding-agent sessions are easier to review and resume.

### D-0005: Make The CLI An Adapter Around Canonical Requests

- Date: 2026-05-23
- Status: Accepted
The first architectural slice is the request boundary. The CLI parses external input and normalizes it into canonical typed requests instead of owning execution logic.
- future adapters can reuse the same internal request model.

### D-0006: Use `--` And Canonical Vector Commands

- Date: 2026-05-23
- Status: Accepted
The CLI uses `--` to separate harness arguments from the command payload, and the canonical command representation is `Vec<String>`.
- command parsing stays explicit and less ambiguous than shell-string defaults.

### D-0007: Split Docs By Concern

- Date: 2026-05-23
- Status: Accepted
Product behavior, architecture, CLI contract, schema, security model, and roadmap each live in separate source-of-truth documents.
- future sessions are less likely to introduce conflicting behavior across docs.

### D-0008: Keep Docs Small And Split By Subject

- Date: 2026-05-23
- Status: Accepted
`AGENTS.md` should stay around 50-200 lines, topic docs around 50-150 lines, and oversized topics should be split by subject into subdirectories instead of numbered overflow files.
- docs stay easier to navigate, review, and maintain as the feature set grows.

### D-0009: Require Explicit `--cwd` And `--timeout` In Version 0

- Date: 2026-05-23
- Status: Accepted
Version 0 requires the caller to provide both `--cwd <PATH>` and `--timeout <SECONDS>` for every `run` request.
- execution boundaries stay explicit in the caller contract and do not depend on hidden defaults.

### D-0010: Use Direct Argument-Vector Execution Only In Version 0

- Date: 2026-05-23
- Status: Accepted
Version 0 accepts commands only after the `--` separator and represents them canonically as an argument vector. Shell-string execution is excluded.
- parsing stays explicit and the broker avoids shell-specific ambiguity in the first implementation slice.

### D-0011: Distinguish Command Failure From Harness Failure

- Date: 2026-05-23
- Status: Accepted
If the command starts and exits non-zero, the result status is `failed`. If the harness cannot start or observe execution in the normal path, the result status is `execution_error`.
- callers can separate command outcomes from infrastructure failures without inferring from text.

### D-0012: Generate Request IDs For Valid CLI Requests

- Date: 2026-05-23
- Status: Accepted
For CLI v0, the adapter generates an opaque `request_id` for each valid request before broker execution. Invalid requests may return `null` when no valid request object exists.
- result correlation and evidence logging work consistently without requiring extra caller flags in v0.

### D-0013: Persist One Execution Evidence Record Outside The Target Workspace

- Date: 2026-05-23
- Status: Accepted
Each execution that reaches the broker must produce one machine-readable evidence record persisted to a tool-managed location outside the target working directory.
- v0 provides durable execution evidence without mutating the target working directory by default.

### D-0014: Inherit Parent Environment In Version 0

- Date: 2026-05-23
- Status: Accepted
Version 0 assumes spawned commands inherit the parent process environment. Environment shaping is deferred to later policy work.
- the first implementation avoids premature environment-contract design while documenting the security tradeoff explicitly.

### D-0015: Use `futurework/BashTool` As Reference Material, Not A Port Target

- Date: 2026-05-24
- Status: Accepted
The TypeScript implementation under `futurework/BashTool/` is a useful reference source for concepts, but Rust v0 should not aim for feature parity with it.
- v0 stays focused on one foreground execution path instead of inheriting background tasks, permission engines, sandbox flows, or UI-driven orchestration from the TypeScript tool.

### D-0016: Version 0 Timeout Termination Scope

- Date: 2026-05-24
- Status: Accepted
Version 0 timeout behavior guarantees timeout detection and a best-effort attempt to terminate the direct child process.
- full process-tree or process-group termination is not guaranteed in v0 and remains deferred.

### D-0017: Reject Invalid `--cwd` Before Execution

- Date: 2026-05-24
- Status: Accepted
Invalid `--cwd` values are rejected during request validation and normalization before execution.
- invalid `cwd` maps to `invalid_request` with error code `invalid_cwd`, and no command is executed.

### D-0018: Store Version 0 Evidence Outside The Target Working Directory

- Date: 2026-05-24
- Status: Accepted
Version 0 writes execution evidence outside the target working directory in a harness-owned state directory using a per-request file strategy such as `<timestamp>_<request_id>.json`.
- evidence persistence is required for requests that reach execution, and evidence-write failure maps to `execution_error`.

### D-0019: Use Generic Process Exit Semantics In Version 0

- Date: 2026-05-24
- Status: Accepted
Version 0 uses generic process semantics only: exit code `0` is `success`, non-zero exit is `failed`, timeout is `timed_out`, malformed request is `invalid_request`, and harness-level failures are `execution_error`.
- command-specific semantics such as grep, diff, or test interpretation are deferred until after v0.

### D-0020: Preserve The Standard Result Envelope For `execution_error`

- Date: 2026-05-24
- Status: Accepted
`execution_error` responses preserve the standard result envelope when possible and include an `error` object; `stdout` and `stderr` are empty strings when no process started.
- callers can consume one stable result shape across success, failure, timeout, and harness-level execution failure cases.

### D-0021: Expose Parsed v0 Requests As Structured JSON Before Broker Execution Exists

- Date: 2026-05-24
- Status: Accepted
The first Rust implementation slice stops at adapter parsing and validation. Valid `run` inputs are normalized into the canonical `ExecutionRequest` shape and printed as structured JSON instead of being executed.
- the request boundary becomes testable immediately while execution, timeout enforcement, and evidence persistence remain separate follow-on tasks.

### D-0022: Use A Temporary Harness-Owned Directory For Initial v0 Evidence Storage

- Date: 2026-05-24
- Status: Accepted
Version 0 stores execution evidence outside the target working directory under a harness-owned state directory, and the initial implementation uses the operating system temporary directory under `llm-shell/evidence`.
- this satisfies the v0 boundary requirement that evidence must not be written into the target workspace while deferring durable, user-configurable retention until after v0.
- v0 evidence is available after execution but may be removed by operating-system temporary-directory cleanup.
- future versions may move evidence to a platform-specific state directory or support explicit configuration.
