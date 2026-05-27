mod cli;
mod evidence;
mod exec;
mod policy;
mod types;

fn main() {
    let output = match cli::parse_env() {
        types::CliOutput::Request(request) => match build_policy_context(&request) {
            Ok(context) => exec::execute(&request, &context),
            Err(response) => types::CliOutput::Execution(*response),
        },
        other => other,
    };
    println!("{}", output.to_json());
    std::process::exit(output.exit_code());
}

fn build_policy_context(
    request: &types::ExecutionRequest,
) -> Result<policy::PolicyContext, Box<types::ExecutionResponse>> {
    let startup_cwd =
        std::env::current_dir().map_err(|error| startup_context_error(request, error))?;
    policy::build_context(startup_cwd).map_err(|error| startup_context_error(request, error))
}

fn startup_context_error(
    request: &types::ExecutionRequest,
    error: std::io::Error,
) -> Box<types::ExecutionResponse> {
    Box::new(types::ExecutionResponse {
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
        error: Some(types::ErrorDetail {
            code: "process_start_failed".to_string(),
            message: format!("The broker startup working directory could not be resolved: {error}"),
        }),
    })
}
