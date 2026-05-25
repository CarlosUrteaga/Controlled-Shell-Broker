use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::evidence;
use crate::policy::{self, PolicyContext, PolicyDecision};
use crate::types::{denied_execution, CliOutput, ErrorDetail, ExecutionRequest, ExecutionResponse};

pub fn execute(request: &ExecutionRequest, policy_context: &PolicyContext) -> CliOutput {
    execute_with_policy(request, policy_context, policy::evaluate)
}

fn execute_with_policy<F>(
    request: &ExecutionRequest,
    policy_context: &PolicyContext,
    evaluate: F,
) -> CliOutput
where
    F: FnOnce(&ExecutionRequest, &PolicyContext) -> PolicyDecision,
{
    match evaluate(request, policy_context) {
        PolicyDecision::Allow => persist_execution(execute_once(request)),
        PolicyDecision::Deny(denied) => CliOutput::Execution(denied_execution(request, denied)),
    }
}

fn persist_execution(response: ExecutionResponse) -> CliOutput {
    match evidence::persist(&response) {
        Ok(_) => CliOutput::Execution(response),
        Err(error) => CliOutput::Execution(evidence_write_failed(response, error)),
    }
}

fn execute_once(request: &ExecutionRequest) -> ExecutionResponse {
    let mut command = Command::new(&request.command[0]);
    command.args(&request.command[1..]);
    command.current_dir(&request.cwd);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let started_at = Instant::now();
    match command.spawn() {
        Ok(mut child) => match observe_child(&mut child, request.timeout_seconds, started_at) {
            Ok(observed) => ExecutionResponse {
                request_id: request.request_id.clone(),
                operation: request.operation.clone(),
                status: observed.status,
                cwd: request.cwd.clone(),
                command: request.command.clone(),
                exit_code: observed.exit_code,
                stdout: observed.stdout,
                stderr: observed.stderr,
                duration_ms: observed.duration_ms,
                timed_out: observed.timed_out,
                error: None,
            },
            Err(error) => ExecutionResponse {
                request_id: request.request_id.clone(),
                operation: request.operation.clone(),
                status: "execution_error".to_string(),
                cwd: request.cwd.clone(),
                command: request.command.clone(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                duration_ms: duration_ms(started_at),
                timed_out: false,
                error: Some(ErrorDetail {
                    code: "process_start_failed".to_string(),
                    message: format!("The command could not be observed: {error}"),
                }),
            },
        },
        Err(error) => ExecutionResponse {
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
        },
    }
}

fn evidence_write_failed(response: ExecutionResponse, error: std::io::Error) -> ExecutionResponse {
    ExecutionResponse {
        status: "execution_error".to_string(),
        error: Some(ErrorDetail {
            code: "evidence_write_failed".to_string(),
            message: format!(
                "The command ran, but required evidence could not be written: {error}"
            ),
        }),
        ..response
    }
}

struct ObservedExecution {
    status: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration_ms: u64,
    timed_out: bool,
}

fn observe_child(
    child: &mut Child,
    timeout_seconds: u64,
    started_at: Instant,
) -> Result<ObservedExecution, std::io::Error> {
    let stdout_handle = child.stdout.take().map(read_stream);
    let stderr_handle = child.stderr.take().map(read_stream);
    let timeout = Duration::from_secs(timeout_seconds);

    let timed_out = loop {
        if child.try_wait()?.is_some() {
            break false;
        }

        if started_at.elapsed() >= timeout {
            child.kill()?;
            break true;
        }

        thread::sleep(Duration::from_millis(10));
    };

    let exit_code = child.wait()?.code();
    let stdout = join_reader(stdout_handle)?;
    let stderr = join_reader(stderr_handle)?;
    let status = if timed_out {
        "timed_out".to_string()
    } else if exit_code == Some(0) {
        "success".to_string()
    } else {
        "failed".to_string()
    };

    Ok(ObservedExecution {
        status,
        exit_code: if timed_out { None } else { exit_code },
        stdout,
        stderr,
        duration_ms: duration_ms(started_at),
        timed_out,
    })
}

fn read_stream<R>(mut stream: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        Ok(buffer)
    })
}

fn join_reader(
    handle: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
) -> Result<String, std::io::Error> {
    let Some(handle) = handle else {
        return Ok(String::new());
    };

    let bytes = handle
        .join()
        .map_err(|_| std::io::Error::other("reader thread panicked"))??;

    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn duration_ms(started_at: Instant) -> u64 {
    let millis = started_at.elapsed().as_millis();
    u64::try_from(millis).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

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

    fn policy_context() -> PolicyContext {
        PolicyContext {
            workspace_root: std::env::current_dir()
                .expect("current directory should resolve")
                .canonicalize()
                .expect("current directory should canonicalize"),
        }
    }

    #[test]
    fn executes_direct_command_and_captures_stdout() {
        let output = execute(
            &request(vec!["echo".to_string(), "hello".to_string()]),
            &policy_context(),
        );

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
        let output = execute(&request(vec!["false".to_string()]), &policy_context());

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
    fn returns_timed_out_when_command_exceeds_timeout() {
        let mut request = request(vec!["sleep".to_string(), "2".to_string()]);
        request.timeout_seconds = 1;

        let output = execute(&request, &policy_context());

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "timed_out");
                assert_eq!(response.exit_code, None);
                assert!(response.timed_out);
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn preserves_partial_output_when_timeout_is_enforced() {
        let mut request = request(vec![
            "sh".to_string(),
            "-c".to_string(),
            "printf hello; printf world >&2; sleep 2".to_string(),
        ]);
        request.timeout_seconds = 1;

        let output = execute(&request, &policy_context());

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "timed_out");
                assert_eq!(response.exit_code, None);
                assert_eq!(response.stdout, "hello");
                assert_eq!(response.stderr, "world");
                assert!(response.timed_out);
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn evaluates_policy_before_spawning_process() {
        let workspace_root = std::env::current_dir()
            .expect("current directory should resolve")
            .canonicalize()
            .expect("current directory should canonicalize");
        let context = PolicyContext {
            workspace_root: workspace_root.clone(),
        };
        let was_called = Rc::new(RefCell::new(false));
        let captured_root = Rc::new(RefCell::new(None::<PathBuf>));
        let called_ref = Rc::clone(&was_called);
        let captured_ref = Rc::clone(&captured_root);

        let output = execute_with_policy(
            &request(vec!["echo".to_string(), "hello".to_string()]),
            &context,
            move |_request, context| {
                *called_ref.borrow_mut() = true;
                *captured_ref.borrow_mut() = Some(context.workspace_root.clone());
                PolicyDecision::Allow
            },
        );

        assert!(
            *was_called.borrow(),
            "policy evaluator should run before spawn"
        );
        assert_eq!(*captured_root.borrow(), Some(workspace_root));

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "success");
                assert_eq!(response.stdout, "hello\n");
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn maps_start_failures_to_execution_error() {
        let output = execute(
            &request(vec!["definitely-not-a-real-command".to_string()]),
            &policy_context(),
        );

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

    #[test]
    fn returns_structured_denied_response_without_spawning() {
        let output = execute_with_policy(
            &request(vec!["definitely-not-a-real-command".to_string()]),
            &policy_context(),
            |_request, _context| {
                PolicyDecision::Deny(crate::types::DeniedExecution {
                    code: "denied_executable".to_string(),
                    message: "The request was denied by broker policy.".to_string(),
                })
            },
        );

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "denied");
                assert_eq!(response.exit_code, None);
                assert!(response.stdout.is_empty());
                assert!(response.stderr.is_empty());
                assert_eq!(response.duration_ms, 0);
                assert!(!response.timed_out);
                assert_eq!(
                    response.error.as_ref().map(|error| error.code.as_str()),
                    Some("denied_executable")
                );
                assert_eq!(
                    response.error.as_ref().map(|error| error.message.as_str()),
                    Some("The request was denied by broker policy.")
                );
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn writes_one_evidence_record_for_successful_execution() {
        let request = request(vec!["echo".to_string(), "hello".to_string()]);
        let evidence_root =
            std::env::temp_dir().join(format!("llm-shell-exec-test-{}", request.request_id));

        let response = execute_once(&request);
        let output = match evidence::persist_in_root(&response, &evidence_root) {
            Ok(_) => CliOutput::Execution(response),
            Err(error) => CliOutput::Execution(evidence_write_failed(response, error)),
        };

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "success");
                let evidence_dir = evidence_root.join("evidence");
                let entries = std::fs::read_dir(&evidence_dir)
                    .expect("evidence directory should exist")
                    .collect::<Result<Vec<_>, _>>()
                    .expect("evidence directory should be readable");
                assert_eq!(entries.len(), 1);

                let file_name = entries[0]
                    .file_name()
                    .into_string()
                    .expect("file name should be valid unicode");
                assert!(file_name.contains(&request.request_id));
                assert!(!entries[0].path().starts_with(&request.cwd));

                std::fs::remove_dir_all(&evidence_root)
                    .expect("evidence directory cleanup should succeed");
            }
            other => panic!("expected execution output, got {other:?}"),
        }
    }

    #[test]
    fn maps_evidence_write_failures_to_execution_error() {
        let request = request(vec!["echo".to_string(), "hello".to_string()]);
        let evidence_root =
            std::env::temp_dir().join(format!("llm-shell-exec-blocked-{}", request.request_id));
        std::fs::write(&evidence_root, "not a directory")
            .expect("test fixture file should be created");

        let response = execute_once(&request);
        let output = match evidence::persist_in_root(&response, &evidence_root) {
            Ok(_) => CliOutput::Execution(response),
            Err(error) => CliOutput::Execution(evidence_write_failed(response, error)),
        };

        match output {
            CliOutput::Execution(response) => {
                assert_eq!(response.status, "execution_error");
                assert_eq!(response.exit_code, Some(0));
                assert_eq!(
                    response.error.as_ref().map(|error| error.code.as_str()),
                    Some("evidence_write_failed")
                );
            }
            other => panic!("expected execution output, got {other:?}"),
        }

        std::fs::remove_file(&evidence_root).expect("fixture cleanup should succeed");
    }
}
