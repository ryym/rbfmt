use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("specify target file path");
            return Ok(());
        }
    };
    let source = fs::read(path)?;
    let result = rubyfmt::format(source);
    println!("{}", result);
    Ok(())
}
