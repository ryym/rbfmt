use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("specify target file path");
            return Ok(());
        }
    };
    let write_to_file = env::args().nth(2).map_or(false, |a| a == "-w");
    let source = fs::read(&path)?;
    let result = rbf::format(source);
    if write_to_file {
        fs::write(&path, result)?;
    } else {
        print!("{}", result);
    }
    Ok(())
}
