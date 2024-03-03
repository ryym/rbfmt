use std::{
    error::Error,
    io::{Read, Write},
};

mod cli;
mod fmt;
mod parse;

#[cfg(test)]
mod test;

pub fn run_command(
    input: &mut impl Read,
    output: &mut impl Write,
    args: impl Iterator<Item = String>,
) -> Result<(), Box<dyn Error>> {
    crate::cli::run(input, output, args)
}

pub fn format_source(source: Vec<u8>) -> Result<String, parse::ParseError> {
    let result = parse::parse_into_fmt_node(source)?;
    let formatted = fmt::format(result.node, result.heredoc_map);
    Ok(formatted)
}
