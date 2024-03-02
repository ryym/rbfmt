use std::process::ExitCode;

fn main() -> ExitCode {
    let result = rbf::run_command(std::env::args());
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}
