# Decisions

This file records repository-level decisions and constraints. Update it when a decision is added, changed, or superseded.

## Decision Log

### D-0001: Documentation-First Repository Start

- Date: 2026-05-23
- Status: Accepted

Context:

The repository started without code and needed a stable operating contract for future coding-agent sessions.

Decision:

Create the product, architecture, testing, backlog, and workflow documentation before adding production code.

Consequences:

- future implementation work has a documented baseline;
- agent sessions can use consistent task framing;
- early effort is spent on requirements clarity instead of premature code.

### D-0002: Rust CLI Harness As The Initial Product Shape

- Date: 2026-05-23
- Status: Accepted

Context:

The project needs a controlled shell execution surface for LLM-assisted development workflows.

Decision:

The initial product will be a Rust-based CLI harness focused on controlled command execution.

Consequences:

- CLI ergonomics and structured outputs are first-class concerns;
- implementation should remain portable and typed;
- the repo should avoid coupling to a specific agent runtime.

### D-0003: Version 0 Focuses On `run`, Not Full Autonomy

- Date: 2026-05-23
- Status: Accepted

Context:

The full vision includes policy, sessions, and long-lived process management, but implementation needs a narrow first milestone.

Decision:

Version 0 will focus on a single `run` command with JSON output, timeout support, working directory support, output capture, exit code reporting, and basic logging.

Consequences:

- initial implementation scope stays small enough to test well;
- later features can extend from a simpler execution contract;
- backlog sequencing remains explicit.

### D-0004: Every Future Task Must Use A Reproducible Delivery Contract

- Date: 2026-05-23
- Status: Accepted

Context:

Agent-driven repository work becomes inconsistent when requirements, scope, validation, and final reporting are implied instead of explicit.

Decision:

Every future task must include:

- a clear requirement;
- expected files to change;
- tests to run;
- acceptance criteria;
- a PR summary format aligned to `.github/pull_request_template.md`.

Consequences:

- coding sessions become easier to review and resume;
- acceptance becomes more objective;
- diffs and validation are easier to audit.

### D-0005: The First Implementation Slice Is The CLI Request Boundary

- Date: 2026-05-23
- Status: Accepted

Context:

The harness will eventually include execution, policy, logging, and session management, but the first durable interface should be the boundary between external input and internal broker requests.

Decision:

The first architectural slice is the CLI / Request Interface. The CLI is an adapter that parses external input and normalizes it into a canonical typed `ExecutionRequest`. It should not directly own execution, policy, or logging concerns.

Consequences:

- the system is less tightly coupled to the CLI as the only interface;
- future adapters such as JSON, MCP, or RPC can reuse the same internal request model;
- validation, execution, and policy logic can evolve behind a stable request contract.

### D-0006: Command Input Uses `--` And Canonical Vector Arguments

- Date: 2026-05-23
- Status: Accepted

Context:

The harness needs a clear boundary between its own flags and the command payload it will later execute.

Decision:

The initial CLI should use `--` to separate harness arguments from command arguments, and the canonical internal command representation should be `Vec<String>` rather than a shell string.

Consequences:

- command payload parsing is more explicit and less ambiguous;
- request validation is simpler and safer;
- shell-string execution, if ever supported, can be treated as a distinct mode instead of the default behavior.
