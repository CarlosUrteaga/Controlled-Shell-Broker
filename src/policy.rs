use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::types::{DeniedExecution, ExecutionRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyContext {
    pub(crate) workspace_root: PathBuf,
    pub(crate) profile: PolicyProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PolicyProfile {
    DefaultRun,
    OpenInspection,
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
        profile: PolicyProfile::DefaultRun,
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
    } else if context.profile == PolicyProfile::OpenInspection {
        evaluate_open_inspection(request, context)
    } else {
        PolicyDecision::Allow
    }
}

fn evaluate_open_inspection(request: &ExecutionRequest, context: &PolicyContext) -> PolicyDecision {
    if !inspection_executable(&request.command[0]) {
        return PolicyDecision::Deny(DeniedExecution {
            code: "inspection_command_not_allowed".to_string(),
            message: "The request executable is outside the open inspection profile.".to_string(),
        });
    }

    if inspection_path_outside_workspace(request, context) {
        return PolicyDecision::Deny(DeniedExecution {
            code: "inspection_path_outside_workspace".to_string(),
            message: "The inspection request references a path outside the workspace root."
                .to_string(),
        });
    }

    PolicyDecision::Allow
}

fn denied_executable(command_0: &str) -> bool {
    let Some(basename) = executable_basename(command_0) else {
        return false;
    };

    matches!(
        basename,
        "rm" | "sudo" | "su" | "shutdown" | "reboot" | "mkfs" | "dd"
    )
}

fn inspection_executable(command_0: &str) -> bool {
    let Some(basename) = executable_basename(command_0) else {
        return false;
    };

    matches!(
        basename,
        "ls" | "find" | "fd" | "rg" | "grep" | "cat" | "head" | "tail" | "sed"
    )
}

fn executable_basename(command_0: &str) -> Option<&str> {
    Path::new(command_0)
        .file_name()
        .and_then(|name| name.to_str())
}

fn inspection_path_outside_workspace(request: &ExecutionRequest, context: &PolicyContext) -> bool {
    request.command.iter().skip(1).any(|arg| {
        if arg == "--" || arg.starts_with('-') {
            return false;
        }

        let path = Path::new(arg);

        if path
            .components()
            .any(|component| component == Component::ParentDir)
        {
            return true;
        }

        path.is_absolute() && !path.starts_with(&context.workspace_root)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("llm-shell-policy-{name}-{}", std::process::id()))
    }

    fn context(workspace_root: PathBuf) -> PolicyContext {
        PolicyContext {
            workspace_root,
            profile: PolicyProfile::DefaultRun,
        }
    }

    fn inspection_context(workspace_root: PathBuf) -> PolicyContext {
        PolicyContext {
            workspace_root,
            profile: PolicyProfile::OpenInspection,
        }
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

    fn request_with_args(cwd: PathBuf, command: Vec<&str>) -> ExecutionRequest {
        let mut request = request(cwd);
        request.command = command.into_iter().map(str::to_string).collect();
        request
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
        assert_eq!(context.profile, PolicyProfile::DefaultRun);

        fs::remove_dir_all(&root).expect("temp test directory cleanup should succeed");
    }

    #[test]
    fn evaluate_allows_workspace_root_itself() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = context(workspace_root.clone());

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
        let context = context(root.canonicalize().expect("root should canonicalize"));

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
        let context = context(root.canonicalize().expect("root should canonicalize"));

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
        fs::remove_dir_all(&outside).expect("outside directory cleanup should succeed");
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
        let context = context(workspace_root.clone());

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
        let context = context(workspace_root.clone());

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
        let context = context(workspace_root.clone());

        assert_eq!(
            evaluate(&request_with_command(workspace_root, "rmdir"), &context),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn open_inspection_allows_documented_discovery_commands() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = inspection_context(workspace_root.clone());

        for command in [
            vec!["ls", "src"],
            vec!["find", "."],
            vec!["fd", "policy"],
            vec!["rg", "Phase 3B", "docs"],
            vec!["grep", "Phase 3B", "docs/ROADMAP.md"],
            vec!["cat", "README.md"],
            vec!["head", "README.md"],
            vec!["tail", "README.md"],
            vec!["sed", "1,5p", "README.md"],
        ] {
            assert_eq!(
                evaluate(
                    &request_with_args(workspace_root.clone(), command),
                    &context
                ),
                PolicyDecision::Allow
            );
        }
    }

    #[test]
    fn open_inspection_denies_out_of_profile_commands_before_spawn() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = inspection_context(workspace_root.clone());

        for command_0 in ["cargo", "curl", "wget", "python", "sh"] {
            assert_eq!(
                evaluate(
                    &request_with_command(workspace_root.clone(), command_0),
                    &context
                ),
                PolicyDecision::Deny(DeniedExecution {
                    code: "inspection_command_not_allowed".to_string(),
                    message: "The request executable is outside the open inspection profile."
                        .to_string(),
                })
            );
        }
    }

    #[test]
    fn open_inspection_keeps_existing_dangerous_executable_denials() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = inspection_context(workspace_root.clone());

        assert_eq!(
            evaluate(&request_with_command(workspace_root, "rm"), &context),
            PolicyDecision::Deny(DeniedExecution {
                code: "denied_executable".to_string(),
                message: "The request executable is denied by broker policy.".to_string(),
            })
        );
    }

    #[test]
    fn open_inspection_denies_obvious_out_of_workspace_path_args() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = inspection_context(workspace_root.clone());

        for command in [vec!["cat", "../outside"], vec!["cat", "/etc/passwd"]] {
            assert_eq!(
                evaluate(
                    &request_with_args(workspace_root.clone(), command),
                    &context
                ),
                PolicyDecision::Deny(DeniedExecution {
                    code: "inspection_path_outside_workspace".to_string(),
                    message: "The inspection request references a path outside the workspace root."
                        .to_string(),
                })
            );
        }
    }

    #[test]
    fn open_inspection_does_not_change_default_run_policy() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = context(workspace_root.clone());

        assert_eq!(
            evaluate(&request_with_command(workspace_root, "cargo"), &context),
            PolicyDecision::Allow
        );
    }
}
