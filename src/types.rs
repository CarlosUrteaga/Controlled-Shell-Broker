use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRequest {
    pub request_id: String,
    pub operation: String,
    pub cwd: PathBuf,
    pub timeout_seconds: u64,
    pub command: Vec<String>,
    pub mode: String,
    pub output_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidRequestResponse {
    pub request_id: Option<String>,
    pub operation: Option<String>,
    pub status: String,
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResponse {
    pub request_id: String,
    pub operation: String,
    pub status: String,
    pub cwd: PathBuf,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub error: Option<ErrorDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeniedExecution {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEvidence {
    pub event_type: String,
    pub request_id: String,
    pub operation: String,
    pub cwd: PathBuf,
    pub command: Vec<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub timestamp: String,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliOutput {
    Request(ExecutionRequest),
    Execution(ExecutionResponse),
    InvalidRequest(InvalidRequestResponse),
}

impl CliOutput {
    pub fn to_json(&self) -> String {
        match self {
            Self::Request(request) => request.to_json(),
            Self::Execution(response) => response.to_json(),
            Self::InvalidRequest(response) => response.to_json(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Request(_) => 0,
            Self::Execution(response) => match response.status.as_str() {
                "success" => 0,
                "failed" => response.exit_code.unwrap_or(1),
                "timed_out" | "denied" | "execution_error" => 1,
                _ => 1,
            },
            Self::InvalidRequest(_) => 2,
        }
    }
}

impl ExecutionRequest {
    pub fn to_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"operation\":{},",
                "\"cwd\":{},",
                "\"timeout_seconds\":{},",
                "\"command\":{},",
                "\"mode\":{},",
                "\"output_format\":{}",
                "}}"
            ),
            json_string(&self.request_id),
            json_string(&self.operation),
            json_string(&self.cwd.to_string_lossy()),
            self.timeout_seconds,
            json_array(&self.command),
            json_string(&self.mode),
            json_string(&self.output_format)
        )
    }
}

impl InvalidRequestResponse {
    pub fn to_json(&self) -> String {
        let request_id = self
            .request_id
            .as_ref()
            .map_or_else(|| "null".to_string(), |value| json_string(value));
        let operation = self
            .operation
            .as_ref()
            .map_or_else(|| "null".to_string(), |value| json_string(value));

        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"operation\":{},",
                "\"status\":{},",
                "\"error\":{{\"code\":{},\"message\":{}}}",
                "}}"
            ),
            request_id,
            operation,
            json_string(&self.status),
            json_string(&self.error.code),
            json_string(&self.error.message)
        )
    }
}

impl ExecutionResponse {
    pub fn to_json(&self) -> String {
        let exit_code = self
            .exit_code
            .map_or_else(|| "null".to_string(), |value| value.to_string());
        let error = self.error.as_ref().map_or_else(String::new, |detail| {
            format!(
                ",\"error\":{{\"code\":{},\"message\":{}}}",
                json_string(&detail.code),
                json_string(&detail.message)
            )
        });

        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"operation\":{},",
                "\"status\":{},",
                "\"cwd\":{},",
                "\"command\":{},",
                "\"exit_code\":{},",
                "\"stdout\":{},",
                "\"stderr\":{},",
                "\"duration_ms\":{},",
                "\"timed_out\":{}",
                "{}",
                "}}"
            ),
            json_string(&self.request_id),
            json_string(&self.operation),
            json_string(&self.status),
            json_string(&self.cwd.to_string_lossy()),
            json_array(&self.command),
            exit_code,
            json_string(&self.stdout),
            json_string(&self.stderr),
            self.duration_ms,
            self.timed_out,
            error
        )
    }
}

impl ExecutionEvidence {
    pub fn to_json(&self) -> String {
        let exit_code = self
            .exit_code
            .map_or_else(|| "null".to_string(), |value| value.to_string());
        let error_code = self.error_code.as_ref().map_or_else(
            || "\"error_code\":null".to_string(),
            |value| format!("\"error_code\":{}", json_string(value)),
        );

        format!(
            concat!(
                "{{",
                "\"event_type\":{},",
                "\"request_id\":{},",
                "\"operation\":{},",
                "\"cwd\":{},",
                "\"command\":{},",
                "\"status\":{},",
                "\"exit_code\":{},",
                "\"duration_ms\":{},",
                "\"timed_out\":{},",
                "\"timestamp\":{},",
                "{}",
                "}}"
            ),
            json_string(&self.event_type),
            json_string(&self.request_id),
            json_string(&self.operation),
            json_string(&self.cwd.to_string_lossy()),
            json_array(&self.command),
            json_string(&self.status),
            exit_code,
            self.duration_ms,
            self.timed_out,
            json_string(&self.timestamp),
            error_code
        )
    }
}

pub fn denied_execution(request: &ExecutionRequest, denied: DeniedExecution) -> ExecutionResponse {
    ExecutionResponse {
        request_id: request.request_id.clone(),
        operation: request.operation.clone(),
        status: "denied".to_string(),
        cwd: request.cwd.clone(),
        command: request.command.clone(),
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        duration_ms: 0,
        timed_out: false,
        error: Some(ErrorDetail {
            code: denied.code,
            message: denied.message,
        }),
    }
}

pub fn invalid_request(
    operation: Option<&str>,
    code: &str,
    message: &str,
) -> InvalidRequestResponse {
    InvalidRequestResponse {
        request_id: None,
        operation: operation.map(str::to_string),
        status: "invalid_request".to_string(),
        error: ErrorDetail {
            code: code.to_string(),
            message: message.to_string(),
        },
    }
}

pub(crate) fn json_array(items: &[String]) -> String {
    let parts = items
        .iter()
        .map(|item| json_string(item))
        .collect::<Vec<_>>();
    format!("[{}]", parts.join(","))
}

pub(crate) fn json_string(value: &str) -> String {
    format!("\"{}\"", escape_json(value))
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_json_uses_argument_vector() {
        let request = ExecutionRequest {
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from("/tmp/work"),
            timeout_seconds: 30,
            command: vec!["cargo".to_string(), "test".to_string(), "--all".to_string()],
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        };

        let json = request.to_json();

        assert!(json.contains("\"command\":[\"cargo\",\"test\",\"--all\"]"));
        assert!(!json.contains("cargo test --all"));
    }

    #[test]
    fn invalid_request_json_has_error_payload() {
        let response = invalid_request(
            Some("run"),
            "missing_cwd",
            "A working directory is required.",
        );
        let json = response.to_json();

        assert!(json.contains("\"status\":\"invalid_request\""));
        assert!(json.contains("\"code\":\"missing_cwd\""));
    }

    #[test]
    fn execution_json_includes_optional_error_only_when_present() {
        let response = ExecutionResponse {
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            status: "execution_error".to_string(),
            cwd: PathBuf::from("/tmp/work"),
            command: vec!["missing-command".to_string()],
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            timed_out: false,
            error: Some(ErrorDetail {
                code: "process_start_failed".to_string(),
                message: "The command could not be started.".to_string(),
            }),
        };

        let json = response.to_json();

        assert!(json.contains("\"status\":\"execution_error\""));
        assert!(json.contains("\"exit_code\":null"));
        assert!(json.contains("\"error\":{\"code\":\"process_start_failed\""));
    }

    #[test]
    fn denied_execution_json_uses_standard_envelope() {
        let request = ExecutionRequest {
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from("/tmp/work"),
            timeout_seconds: 30,
            command: vec!["rm".to_string(), "-rf".to_string(), ".".to_string()],
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        };
        let response = denied_execution(
            &request,
            DeniedExecution {
                code: "denied_executable".to_string(),
                message: "The request was denied by broker policy.".to_string(),
            },
        );

        let json = response.to_json();

        assert!(json.contains("\"status\":\"denied\""));
        assert!(json.contains("\"exit_code\":null"));
        assert!(json.contains("\"stdout\":\"\""));
        assert!(json.contains("\"stderr\":\"\""));
        assert!(json.contains("\"duration_ms\":0"));
        assert!(json.contains("\"timed_out\":false"));
        assert!(json.contains("\"error\":{\"code\":\"denied_executable\""));
    }

    #[test]
    fn denied_execution_preserves_workspace_root_denial_code() {
        let request = ExecutionRequest {
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from("/tmp/outside"),
            timeout_seconds: 30,
            command: vec!["pwd".to_string()],
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        };
        let response = denied_execution(
            &request,
            DeniedExecution {
                code: "cwd_outside_workspace_root".to_string(),
                message: "The request cwd is outside the broker startup workspace root."
                    .to_string(),
            },
        );

        assert_eq!(
            response.error.as_ref().map(|error| error.code.as_str()),
            Some("cwd_outside_workspace_root")
        );
    }

    #[test]
    fn timed_out_execution_json_uses_null_exit_code_and_true_flag() {
        let response = ExecutionResponse {
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            status: "timed_out".to_string(),
            cwd: PathBuf::from("/tmp/work"),
            command: vec!["sleep".to_string(), "2".to_string()],
            exit_code: None,
            stdout: "hello".to_string(),
            stderr: String::new(),
            duration_ms: 1000,
            timed_out: true,
            error: None,
        };

        let json = response.to_json();

        assert!(json.contains("\"status\":\"timed_out\""));
        assert!(json.contains("\"exit_code\":null"));
        assert!(json.contains("\"timed_out\":true"));
    }

    #[test]
    fn evidence_json_omits_stdout_and_stderr() {
        let evidence = ExecutionEvidence {
            event_type: "execution.completed".to_string(),
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from("/tmp/work"),
            command: vec!["echo".to_string(), "hello".to_string()],
            status: "success".to_string(),
            exit_code: Some(0),
            duration_ms: 12,
            timed_out: false,
            timestamp: "1716500000000".to_string(),
            error_code: None,
        };

        let json = evidence.to_json();

        assert!(json.contains("\"event_type\":\"execution.completed\""));
        assert!(json.contains("\"timestamp\":\"1716500000000\""));
        assert!(!json.contains("stdout"));
        assert!(!json.contains("stderr"));
    }

    #[test]
    fn denied_cli_output_uses_exit_code_one() {
        let request = ExecutionRequest {
            request_id: "req-123".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from("/tmp/work"),
            timeout_seconds: 30,
            command: vec!["rm".to_string()],
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        };
        let output = CliOutput::Execution(denied_execution(
            &request,
            DeniedExecution {
                code: "denied_executable".to_string(),
                message: "The request was denied by broker policy.".to_string(),
            },
        ));

        assert_eq!(output.exit_code(), 1);
    }
}
