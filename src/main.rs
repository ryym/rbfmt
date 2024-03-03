use std::process::ExitCode;

fn main() -> ExitCode {
    let result = rbfmt::run_command(&mut std::io::stdout(), std::env::args());
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}
