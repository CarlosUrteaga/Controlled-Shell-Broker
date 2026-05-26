use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{ExecutionEvidence, ExecutionResponse, PolicyAudit};

pub fn persist(response: &ExecutionResponse, policy_audit: &PolicyAudit) -> io::Result<PathBuf> {
    persist_in_root(response, policy_audit, &default_state_dir())
}

pub fn default_state_dir() -> PathBuf {
    std::env::temp_dir().join("llm-shell")
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
}
