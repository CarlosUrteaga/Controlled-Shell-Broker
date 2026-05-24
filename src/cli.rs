use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{invalid_request, CliOutput, ExecutionRequest};

pub fn parse_env() -> CliOutput {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    parse_args(args)
}

pub fn parse_args(args: Vec<OsString>) -> CliOutput {
    let Some(operation) = args.first() else {
        return CliOutput::InvalidRequest(invalid_request(
            None,
            "unsupported_operation",
            "An operation is required. Version 0 supports only `run`.",
        ));
    };

    match operation.to_string_lossy().as_ref() {
        "run" => parse_run_args(&args[1..]),
        _ => CliOutput::InvalidRequest(invalid_request(
            None,
            "unsupported_operation",
            "Unsupported operation. Version 0 supports only `run`.",
        )),
    }
}

fn parse_run_args(args: &[OsString]) -> CliOutput {
    let separator_index = match args.iter().position(|arg| arg == "--") {
        Some(index) => index,
        None => {
            return CliOutput::InvalidRequest(invalid_request(
                Some("run"),
                "invalid_argument_shape",
                "The `--` separator is required before the command payload.",
            ))
        }
    };

    let mut cwd: Option<PathBuf> = None;
    let mut timeout_seconds: Option<u64> = None;
    let mut output_format = "json".to_string();

    let harness_args = &args[..separator_index];
    let mut index = 0;
    while index < harness_args.len() {
        let current = harness_args[index].to_string_lossy();
        match current.as_ref() {
            "--cwd" => {
                let Some(value) = harness_args.get(index + 1) else {
                    return CliOutput::InvalidRequest(invalid_request(
                        Some("run"),
                        "invalid_cwd",
                        "The `--cwd` flag requires a directory path.",
                    ));
                };
                cwd = Some(PathBuf::from(value));
                index += 2;
            }
            "--timeout" => {
                let Some(value) = harness_args.get(index + 1) else {
                    return CliOutput::InvalidRequest(invalid_request(
                        Some("run"),
                        "invalid_timeout",
                        "The `--timeout` flag requires a positive integer value.",
                    ));
                };

                match parse_timeout(value) {
                    Ok(parsed) => timeout_seconds = Some(parsed),
                    Err(response) => return CliOutput::InvalidRequest(response),
                }

                index += 2;
            }
            "--output" => {
                let Some(value) = harness_args.get(index + 1) else {
                    return CliOutput::InvalidRequest(invalid_request(
                        Some("run"),
                        "invalid_argument_shape",
                        "The `--output` flag requires a value.",
                    ));
                };

                if value != "json" {
                    return CliOutput::InvalidRequest(invalid_request(
                        Some("run"),
                        "unsupported_output_format",
                        "Version 0 supports only `--output json`.",
                    ));
                }

                output_format = "json".to_string();
                index += 2;
            }
            value if value.starts_with("--") => {
                return CliOutput::InvalidRequest(invalid_request(
                    Some("run"),
                    "unknown_flag",
                    &format!("Unknown harness flag `{value}`."),
                ))
            }
            _ => {
                return CliOutput::InvalidRequest(invalid_request(
                    Some("run"),
                    "invalid_argument_shape",
                    "Harness flags must appear before `--`, and command arguments must appear after it.",
                ))
            }
        }
    }

    let cwd = match cwd {
        Some(path) => match validate_cwd(path) {
            Ok(path) => path,
            Err(response) => return CliOutput::InvalidRequest(response),
        },
        None => {
            return CliOutput::InvalidRequest(invalid_request(
                Some("run"),
                "missing_cwd",
                "The `run` command requires `--cwd <PATH>`.",
            ))
        }
    };

    let timeout_seconds = match timeout_seconds {
        Some(timeout) => timeout,
        None => {
            return CliOutput::InvalidRequest(invalid_request(
                Some("run"),
                "missing_timeout",
                "The `run` command requires `--timeout <SECONDS>`.",
            ))
        }
    };

    let command = args[separator_index + 1..]
        .iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if command.is_empty() {
        return CliOutput::InvalidRequest(invalid_request(
            Some("run"),
            "missing_command",
            "The command payload after `--` must not be empty.",
        ));
    }

    CliOutput::Request(ExecutionRequest {
        request_id: generate_request_id(),
        operation: "run".to_string(),
        cwd,
        timeout_seconds,
        command,
        mode: "foreground".to_string(),
        output_format,
    })
}

fn validate_cwd(path: PathBuf) -> Result<PathBuf, crate::types::InvalidRequestResponse> {
    if path.as_os_str().is_empty() {
        return Err(invalid_request(
            Some("run"),
            "invalid_cwd",
            "The working directory path must not be empty.",
        ));
    }

    let resolved = fs::canonicalize(&path).map_err(|_| {
        invalid_request(
            Some("run"),
            "invalid_cwd",
            "The working directory could not be resolved to an existing directory.",
        )
    })?;

    let metadata = fs::metadata(&resolved).map_err(|_| {
        invalid_request(
            Some("run"),
            "invalid_cwd",
            "The working directory could not be inspected.",
        )
    })?;

    if !metadata.is_dir() {
        return Err(invalid_request(
            Some("run"),
            "invalid_cwd",
            "The working directory must refer to a directory.",
        ));
    }

    Ok(resolved)
}

fn parse_timeout(value: &OsString) -> Result<u64, crate::types::InvalidRequestResponse> {
    let raw = value.to_string_lossy();

    match raw.parse::<u64>() {
        Ok(0) | Err(_) => Err(invalid_request(
            Some("run"),
            "invalid_timeout",
            "The timeout must be a positive integer number of seconds.",
        )),
        Ok(parsed) => Ok(parsed),
    }
}

fn generate_request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);

    format!("req-{}-{nanos}", process::id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CliOutput;

    fn parse(parts: &[&str]) -> CliOutput {
        parse_args(parts.iter().map(OsString::from).collect())
    }

    #[test]
    fn accepts_valid_run_request() {
        let output = parse(&[
            "run",
            "--cwd",
            ".",
            "--timeout",
            "30",
            "--",
            "cargo",
            "test",
        ]);

        match output {
            CliOutput::Request(request) => {
                assert_eq!(request.operation, "run");
                assert_eq!(request.timeout_seconds, 30);
                assert_eq!(
                    request.command,
                    vec!["cargo".to_string(), "test".to_string()]
                );
                assert_eq!(request.mode, "foreground");
                assert_eq!(request.output_format, "json");
                assert!(request.cwd.is_absolute());
                assert!(request.request_id.starts_with("req-"));
            }
            other => panic!("expected valid request, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_cwd() {
        let output = parse(&["run", "--timeout", "30", "--", "cargo", "test"]);

        match output {
            CliOutput::InvalidRequest(response) => assert_eq!(response.error.code, "missing_cwd"),
            other => panic!("expected invalid request, got {other:?}"),
        }
    }

    #[test]
    fn rejects_nonexistent_cwd() {
        let output = parse(&[
            "run",
            "--cwd",
            "./definitely-not-a-real-directory",
            "--timeout",
            "30",
            "--",
            "cargo",
            "test",
        ]);

        match output {
            CliOutput::InvalidRequest(response) => assert_eq!(response.error.code, "invalid_cwd"),
            other => panic!("expected invalid request, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_timeout() {
        let output = parse(&["run", "--cwd", ".", "--", "cargo", "test"]);

        match output {
            CliOutput::InvalidRequest(response) => {
                assert_eq!(response.error.code, "missing_timeout")
            }
            other => panic!("expected invalid request, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_timeout_values() {
        for value in ["0", "-1", "nope"] {
            let output = parse(&[
                "run",
                "--cwd",
                ".",
                "--timeout",
                value,
                "--",
                "cargo",
                "test",
            ]);

            match output {
                CliOutput::InvalidRequest(response) => {
                    assert_eq!(response.error.code, "invalid_timeout")
                }
                other => panic!("expected invalid request, got {other:?}"),
            }
        }
    }

    #[test]
    fn rejects_missing_command_payload() {
        let output = parse(&["run", "--cwd", ".", "--timeout", "30", "--"]);

        match output {
            CliOutput::InvalidRequest(response) => {
                assert_eq!(response.error.code, "missing_command")
            }
            other => panic!("expected invalid request, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_separator() {
        let output = parse(&["run", "--cwd", ".", "--timeout", "30", "cargo", "test"]);

        match output {
            CliOutput::InvalidRequest(response) => {
                assert_eq!(response.error.code, "invalid_argument_shape")
            }
            other => panic!("expected invalid request, got {other:?}"),
        }
    }
}
