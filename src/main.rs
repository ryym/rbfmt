use std::process::ExitCode;

fn main() -> ExitCode {
    let result = rbfmt::run_command(
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        std::env::args().skip(1),
    );
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}
