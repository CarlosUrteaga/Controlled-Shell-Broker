mod cli;
mod exec;
mod types;

fn main() {
    let output = match cli::parse_env() {
        types::CliOutput::Request(request) => exec::execute(&request),
        other => other,
    };
    println!("{}", output.to_json());
    std::process::exit(output.exit_code());
}
