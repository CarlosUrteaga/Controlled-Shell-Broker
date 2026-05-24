mod cli;
mod types;

fn main() {
    let output = cli::parse_env();
    println!("{}", output.to_json());
    std::process::exit(output.exit_code());
}
