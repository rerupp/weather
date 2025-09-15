use std::process::ExitCode;
mod cli;

/// The weather cli entry point.
fn main() -> ExitCode {
    let args = cli::command().get_matches();
    if let Err(error) = cli::initialize_and_run(args) {
        eprintln!("Error: {}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
