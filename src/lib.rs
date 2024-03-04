use std::error::Error;

mod cli;
mod fmt;
mod parse;

#[cfg(test)]
mod test;

pub fn run() -> Result<(), Box<dyn Error>> {
    crate::cli::run(
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        std::env::args().skip(1),
    )
}

pub fn format_source(source: Vec<u8>) -> Result<String, parse::ParseError> {
    let result = parse::parse_into_fmt_node(source)?;
    let formatted = fmt::format(result.node, result.heredoc_map);
    Ok(formatted)
}
