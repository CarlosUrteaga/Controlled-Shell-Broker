use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{ExecutionEvidence, ExecutionResponse, PolicyAudit};

const APP_DIR_NAME: &str = "llm-shell";

pub fn persist(response: &ExecutionResponse, policy_audit: &PolicyAudit) -> io::Result<PathBuf> {
    let state_dir = default_state_dir()?;
    persist_in_root(response, policy_audit, &state_dir)
}

pub fn default_state_dir() -> io::Result<PathBuf> {
    resolve_state_dir_for(std::env::consts::OS, |key| {
        std::env::var_os(key).map(PathBuf::from)
    })
}

pub fn persist_in_root(
    response: &ExecutionResponse,
    policy_audit: &PolicyAudit,
    state_dir: &Path,
) -> io::Result<PathBuf> {
    let timestamp = unix_timestamp_millis();
    let evidence_dir = state_dir.join("evidence");
    fs::create_dir_all(&evidence_dir)?;

    let file_name = format!("{}_{}.json", timestamp, response.request_id);
    let path = evidence_dir.join(file_name);
    let evidence = build_evidence(response, policy_audit, timestamp);

    fs::write(&path, evidence.to_json())?;

    Ok(path)
}

fn resolve_state_dir_for<F>(os: &str, mut env: F) -> io::Result<PathBuf>
where
    F: FnMut(&str) -> Option<PathBuf>,
{
    match os {
        "macos" => require_env_path(&mut env, "HOME").map(|home| {
            home.join("Library")
                .join("Application Support")
                .join(APP_DIR_NAME)
        }),
        "windows" => env("LOCALAPPDATA")
            .filter(|path| !path.as_os_str().is_empty())
            .or_else(|| env("APPDATA").filter(|path| !path.as_os_str().is_empty()))
            .map(|root| root.join(APP_DIR_NAME))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing LOCALAPPDATA/APPDATA")),
        _ => {
            if let Some(xdg_state_home) = env("XDG_STATE_HOME")
                .filter(|path| !path.as_os_str().is_empty())
                .filter(|path| path.is_absolute())
            {
                return Ok(xdg_state_home.join(APP_DIR_NAME));
            }

            require_env_path(&mut env, "HOME")
                .map(|home| home.join(".local").join("state").join(APP_DIR_NAME))
        }
    }
}

fn require_env_path<F>(env: &mut F, key: &str) -> io::Result<PathBuf>
where
    F: FnMut(&str) -> Option<PathBuf>,
{
    env(key)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("missing {key}")))
}

fn build_evidence(
    response: &ExecutionResponse,
    policy_audit: &PolicyAudit,
    timestamp: u128,
) -> ExecutionEvidence {
    ExecutionEvidence {
        event_type: if policy_audit.decision == "deny" {
            "execution.denied".to_string()
        } else {
            "execution.completed".to_string()
        },
        request_id: response.request_id.clone(),
        operation: response.operation.clone(),
        cwd: response.cwd.clone(),
        command: response.command.clone(),
        status: response.status.clone(),
        exit_code: response.exit_code,
        duration_ms: response.duration_ms,
        timed_out: response.timed_out,
        timestamp: timestamp.to_string(),
        error_code: response.error.as_ref().map(|error| error.code.clone()),
        policy_decision: policy_audit.decision.clone(),
        policy_reason: policy_audit.reason.clone(),
    }
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::types::{ErrorDetail, ExecutionResponse, PolicyAudit};

    fn response() -> ExecutionResponse {
        ExecutionResponse {
            request_id: "req-evidence-test".to_string(),
            operation: "run".to_string(),
            status: "success".to_string(),
            cwd: PathBuf::from(".")
                .canonicalize()
                .expect("current directory should resolve"),
            command: vec!["echo".to_string(), "hello".to_string()],
            exit_code: Some(0),
            stdout: "hello\n".to_string(),
            stderr: "debug\n".to_string(),
            duration_ms: 5,
            timed_out: false,
            error: None,
        }
    }

    fn denied_response() -> ExecutionResponse {
        ExecutionResponse {
            request_id: "req-denied-test".to_string(),
            operation: "run".to_string(),
            status: "denied".to_string(),
            cwd: PathBuf::from(".")
                .canonicalize()
                .expect("current directory should resolve"),
            command: vec!["rm".to_string(), "-rf".to_string(), ".".to_string()],
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            timed_out: false,
            error: Some(ErrorDetail {
                code: "denied_executable".to_string(),
                message: "The request executable is denied by broker policy.".to_string(),
            }),
        }
    }

    #[test]
    fn persists_metadata_only_evidence_with_timestamped_file_name() {
        let temp_root =
            std::env::temp_dir().join(format!("llm-shell-test-{}", unix_timestamp_millis()));
        let response = response();

        let path = persist_in_root(&response, &PolicyAudit::allow(), &temp_root)
            .expect("evidence should persist");
        let contents = fs::read_to_string(&path).expect("evidence file should be readable");

        assert!(path.starts_with(temp_root.join("evidence")));
        assert!(path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_req-evidence-test.json")));
        assert!(contents.contains("\"request_id\":\"req-evidence-test\""));
        assert!(contents.contains("\"timestamp\":\""));
        assert!(contents.contains("\"policy_decision\":\"allow\""));
        assert!(!contents.contains("stdout"));
        assert!(!contents.contains("stderr"));

        fs::remove_file(&path).expect("evidence file cleanup should succeed");
        fs::remove_dir_all(&temp_root).expect("evidence directory cleanup should succeed");
    }

    #[test]
    fn persists_denied_evidence_with_policy_reason() {
        let temp_root =
            std::env::temp_dir().join(format!("llm-shell-denied-test-{}", unix_timestamp_millis()));
        let response = denied_response();

        let path = persist_in_root(
            &response,
            &PolicyAudit::deny("denied_executable".to_string()),
            &temp_root,
        )
        .expect("denied evidence should persist");
        let contents = fs::read_to_string(&path).expect("evidence file should be readable");

        assert!(contents.contains("\"event_type\":\"execution.denied\""));
        assert!(contents.contains("\"status\":\"denied\""));
        assert!(contents.contains("\"policy_decision\":\"deny\""));
        assert!(contents.contains("\"policy_reason\":\"denied_executable\""));
        assert!(!contents.contains("stdout"));
        assert!(!contents.contains("stderr"));

        fs::remove_file(&path).expect("evidence file cleanup should succeed");
        fs::remove_dir_all(&temp_root).expect("evidence directory cleanup should succeed");
    }

    #[test]
    fn resolve_state_dir_uses_absolute_xdg_state_home_on_unix() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "XDG_STATE_HOME" => Some(PathBuf::from("/xdg-state")),
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(resolved, PathBuf::from("/xdg-state").join(APP_DIR_NAME));
    }

    #[test]
    fn resolve_state_dir_falls_back_to_home_on_unix_when_xdg_missing() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/home/tester")
                .join(".local")
                .join("state")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_ignores_relative_xdg_state_home() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "XDG_STATE_HOME" => Some(PathBuf::from("relative/state")),
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/home/tester")
                .join(".local")
                .join("state")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_ignores_empty_xdg_state_home() {
        let resolved = resolve_state_dir_for("linux", |key| match key {
            "XDG_STATE_HOME" => Some(PathBuf::new()),
            "HOME" => Some(PathBuf::from("/home/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/home/tester")
                .join(".local")
                .join("state")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_uses_application_support_on_macos() {
        let resolved = resolve_state_dir_for("macos", |key| match key {
            "HOME" => Some(PathBuf::from("/Users/tester")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/Users/tester")
                .join("Library")
                .join("Application Support")
                .join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_errors_when_home_missing_on_macos() {
        let error =
            resolve_state_dir_for("macos", |_key| None).expect_err("missing HOME should error");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn resolve_state_dir_prefers_localappdata_on_windows() {
        let resolved = resolve_state_dir_for("windows", |key| match key {
            "LOCALAPPDATA" => Some(PathBuf::from(r"C:\Users\tester\AppData\Local")),
            "APPDATA" => Some(PathBuf::from(r"C:\Users\tester\AppData\Roaming")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from(r"C:\Users\tester\AppData\Local").join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_falls_back_to_appdata_on_windows() {
        let resolved = resolve_state_dir_for("windows", |key| match key {
            "APPDATA" => Some(PathBuf::from(r"C:\Users\tester\AppData\Roaming")),
            _ => None,
        })
        .expect("state dir should resolve");

        assert_eq!(
            resolved,
            PathBuf::from(r"C:\Users\tester\AppData\Roaming").join(APP_DIR_NAME)
        );
    }

    #[test]
    fn resolve_state_dir_errors_when_windows_env_missing() {
        let error = resolve_state_dir_for("windows", |_key| None)
            .expect_err("missing windows env should error");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn default_state_dir_is_not_the_v0_temp_directory_baseline() {
        let resolved = default_state_dir().expect("default state dir should resolve");

        assert_ne!(resolved, std::env::temp_dir().join(APP_DIR_NAME));
        assert!(resolved.ends_with(APP_DIR_NAME));
    }
}
