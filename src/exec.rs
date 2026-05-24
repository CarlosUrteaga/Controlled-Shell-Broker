use std::process::Command;
use std::time::Instant;

use crate::types::{CliOutput, ErrorDetail, ExecutionRequest, ExecutionResponse};

pub fn execute(request: &ExecutionRequest) -> CliOutput {
    let mut command = Command::new(&request.command[0]);
    command.args(&request.command[1..]);
    command.current_dir(&request.cwd);

    let started_at = Instant::now();
    match command.output() {
        Ok(output) => {
            let exit_code = output.status.code();
            let status = match exit_code {
                Some(0) => "success",
                Some(_) | None => "failed",
            };

            CliOutput::Execution(ExecutionResponse {
                request_id: request.request_id.clone(),
                operation: request.operation.clone(),
                status: status.to_string(),
                cwd: request.cwd.clone(),
                command: request.command.clone(),
                exit_code,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                duration_ms: duration_ms(started_at),
                timed_out: false,
                error: None,
            })
        }
        Err(error) => CliOutput::Execution(ExecutionResponse {
            request_id: request.request_id.clone(),
            operation: request.operation.clone(),
            status: "execution_error".to_string(),
            cwd: request.cwd.clone(),
            command: request.command.clone(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            timed_out: false,
            error: Some(ErrorDetail {
                code: "process_start_failed".to_string(),
                message: format!("The command could not be started: {error}"),
            }),
        }),
    }
}

fn duration_ms(started_at: Instant) -> u64 {
    let millis = started_at.elapsed().as_millis();
    u64::try_from(millis).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn request(command: Vec<String>) -> ExecutionRequest {
        ExecutionRequest {
            request_id: "req-test".to_string(),
            operation: "run".to_string(),
            cwd: PathBuf::from(".")
                .canonicalize()
                .expect("current directory should resolve"),
            timeout_seconds: 30,
            command,
            mode: "foreground".to_string(),
            output_format: "json".to_string(),
        }
    }

    #[test]
    fn executes_direct_command_and_captures_stdout() {
        let output = execute(&request(vec!["echo".to_string(), "hello".to_string()]));

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "success");
                assert_eq!(response.exit_code, Some(0));
                assert_eq!(response.stdout, "hello\n");
                assert!(response.stderr.is_empty());
                assert!(!response.timed_out);
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn maps_non_zero_exit_to_failed() {
        let output = execute(&request(vec!["false".to_string()]));

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "failed");
                assert_eq!(response.exit_code, Some(1));
                assert!(response.stdout.is_empty());
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn maps_start_failures_to_execution_error() {
        let output = execute(&request(vec!["definitely-not-a-real-command".to_string()]));

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "execution_error");
                assert_eq!(response.exit_code, None);
                assert!(response.stdout.is_empty());
                assert!(response.stderr.is_empty());
                assert_eq!(
                    response.error.as_ref().map(|error| error.code.as_str()),
                    Some("process_start_failed")
                );
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }
}
