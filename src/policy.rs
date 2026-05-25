use std::fs;
use std::io;
use std::path::PathBuf;

use crate::types::{DeniedExecution, ExecutionRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyContext {
    pub(crate) workspace_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PolicyDecision {
    Allow,
    Deny(DeniedExecution),
}

pub(crate) fn build_context(startup_cwd: PathBuf) -> io::Result<PolicyContext> {
    Ok(PolicyContext {
        workspace_root: fs::canonicalize(startup_cwd)?,
    })
}

pub(crate) fn evaluate(_request: &ExecutionRequest, _context: &PolicyContext) -> PolicyDecision {
    PolicyDecision::Allow
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("llm-shell-policy-{name}-{}", std::process::id()))
    }

    #[test]
    fn build_context_canonicalizes_startup_cwd_once() {
        let root = temp_dir("canonicalize");
        let nested = root.join("subdir");
        fs::create_dir_all(&nested).expect("temp test directory should be created");

        let context = build_context(root.join("subdir").join(".."))
            .expect("policy context should canonicalize startup cwd");

        assert_eq!(
            context.workspace_root,
            root.canonicalize().expect("root should canonicalize")
        );

        fs::remove_dir_all(&root).expect("temp test directory cleanup should succeed");
    }

    #[test]
    fn evaluate_allows_valid_requests_in_pr_2() {
        let request = ExecutionRequest {
            request_id: "req-policy-test".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from(".")
                .canonicalize()
                .expect("current directory should resolve"),
            timeout_seconds: 30,
            command: vec!["echo".to_string(), "hello".to_string()],
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        };
        let context = PolicyContext {
            workspace_root: std::env::current_dir()
                .expect("current directory should resolve")
                .canonicalize()
                .expect("current directory should canonicalize"),
        };

        assert_eq!(evaluate(&request, &context), PolicyDecision::Allow);
    }

    #[test]
    fn deny_decision_can_carry_structured_reason() {
        let decision = PolicyDecision::Deny(DeniedExecution {
            code: "denied_executable".to_string(),
            message: "The request was denied by broker policy.".to_string(),
        });

        assert_eq!(
            decision,
            PolicyDecision::Deny(DeniedExecution {
                code: "denied_executable".to_string(),
                message: "The request was denied by broker policy.".to_string(),
            })
        );
    }
}
