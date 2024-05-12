use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    let result = rbfmt::run();
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}
