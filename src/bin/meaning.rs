use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let file_path = args.next().expect("target path must be specified");
    rbfmt::print_meaning(&file_path)?;
    Ok(())
}
