use std::error::Error;

use config::FormatConfig;

mod cli;
mod config;
mod fmt;
mod parse;

#[cfg(test)]
mod test;

pub fn run() -> Result<(), Box<dyn Error>> {
    cli::run(
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        std::env::args().skip(1),
    )
}

pub fn format_source(source: Vec<u8>, config: FormatConfig) -> Result<String, parse::ParseError> {
    let result = parse::parse_into_fmt_node(source)?;
    let formatted = fmt::format(config, result.node, result.heredoc_map);
    Ok(formatted)
}
