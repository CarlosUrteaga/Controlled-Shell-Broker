use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

impl PolicyDecision {
    pub(crate) fn audit(&self) -> crate::types::PolicyAudit {
        match self {
            Self::Allow => crate::types::PolicyAudit::allow(),
            Self::Deny(denied) => crate::types::PolicyAudit::deny(denied.code.clone()),
        }
    }
}

pub(crate) fn build_context(startup_cwd: PathBuf) -> io::Result<PolicyContext> {
    Ok(PolicyContext {
        workspace_root: fs::canonicalize(startup_cwd)?,
    })
}

pub(crate) fn evaluate(request: &ExecutionRequest, context: &PolicyContext) -> PolicyDecision {
    if !request.cwd.starts_with(&context.workspace_root) {
        PolicyDecision::Deny(DeniedExecution {
            code: "cwd_outside_workspace_root".to_string(),
            message: "The request cwd is outside the broker startup workspace root.".to_string(),
        })
    } else if denied_executable(&request.command[0]) {
        PolicyDecision::Deny(DeniedExecution {
            code: "denied_executable".to_string(),
            message: "The request executable is denied by broker policy.".to_string(),
        })
    } else {
        PolicyDecision::Allow
    }
}

fn denied_executable(command_0: &str) -> bool {
    let Some(basename) = Path::new(command_0)
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return false;
    };

    matches!(
        basename,
        "rm" | "sudo" | "su" | "shutdown" | "reboot" | "mkfs" | "dd"
    )
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

    fn request(cwd: PathBuf) -> ExecutionRequest {
        ExecutionRequest {
            request_id: "req-policy-test".to_string(),
            operation: "run".to_string(),
            cwd,
            timeout_seconds: 30,
            command: vec!["echo".to_string(), "hello".to_string()],
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        }
    }

    fn request_with_command(cwd: PathBuf, command_0: &str) -> ExecutionRequest {
        let mut request = request(cwd);
        request.command = vec![command_0.to_string()];
        request
    }

    #[test]
    fn evaluate_allows_workspace_root_itself() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = PolicyContext {
            workspace_root: workspace_root.clone(),
        };

        assert_eq!(
            evaluate(&request(workspace_root), &context),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn evaluate_allows_descendant_of_workspace_root() {
        let root = temp_dir("descendant");
        let child = root.join("child");
        fs::create_dir_all(&child).expect("descendant directory should be created");
        let context = PolicyContext {
            workspace_root: root.canonicalize().expect("root should canonicalize"),
        };

        assert_eq!(
            evaluate(
                &request(child.canonicalize().expect("child should canonicalize")),
                &context
            ),
            PolicyDecision::Allow
        );

        fs::remove_dir_all(&root).expect("temp test directory cleanup should succeed");
    }

    #[test]
    fn evaluate_denies_cwd_outside_workspace_root() {
        let root = temp_dir("inside");
        let outside = temp_dir("outside");
        fs::create_dir_all(&root).expect("workspace root should be created");
        fs::create_dir_all(&outside).expect("outside directory should be created");
        let context = PolicyContext {
            workspace_root: root.canonicalize().expect("root should canonicalize"),
        };

        assert_eq!(
            evaluate(
                &request(outside.canonicalize().expect("outside should canonicalize")),
                &context
            ),
            PolicyDecision::Deny(DeniedExecution {
                code: "cwd_outside_workspace_root".to_string(),
                message: "The request cwd is outside the broker startup workspace root."
                    .to_string(),
            })
        );

        fs::remove_dir_all(&root).expect("temp test directory cleanup should succeed");
        fs::remove_dir_all(&outside).expect("temp test directory cleanup should succeed");
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

    #[test]
    fn deny_decision_produces_policy_audit_reason() {
        let audit = PolicyDecision::Deny(DeniedExecution {
            code: "denied_executable".to_string(),
            message: "The request was denied by broker policy.".to_string(),
        })
        .audit();

        assert_eq!(audit.decision, "deny");
        assert_eq!(audit.reason.as_deref(), Some("denied_executable"));
    }

    #[test]
    fn allow_decision_produces_allow_policy_audit() {
        let audit = PolicyDecision::Allow.audit();

        assert_eq!(audit.decision, "allow");
        assert_eq!(audit.reason, None);
    }

    #[test]
    fn evaluate_denies_exact_dangerous_executable_basenames() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = PolicyContext {
            workspace_root: workspace_root.clone(),
        };

        for command_0 in ["rm", "sudo", "su", "shutdown", "reboot", "mkfs", "dd"] {
            assert_eq!(
                evaluate(
                    &request_with_command(workspace_root.clone(), command_0),
                    &context
                ),
                PolicyDecision::Deny(DeniedExecution {
                    code: "denied_executable".to_string(),
                    message: "The request executable is denied by broker policy.".to_string(),
                })
            );
        }
    }

    #[test]
    fn evaluate_denies_absolute_path_when_basename_is_dangerous() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = PolicyContext {
            workspace_root: workspace_root.clone(),
        };

        assert_eq!(
            evaluate(&request_with_command(workspace_root, "/bin/rm"), &context),
            PolicyDecision::Deny(DeniedExecution {
                code: "denied_executable".to_string(),
                message: "The request executable is denied by broker policy.".to_string(),
            })
        );
    }

    #[test]
    fn evaluate_does_not_deny_substring_matches() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = PolicyContext {
            workspace_root: workspace_root.clone(),
        };

        assert_eq!(
            evaluate(&request_with_command(workspace_root, "rmdir"), &context),
            PolicyDecision::Allow
        );
    }
}
