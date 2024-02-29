mod fmt;
mod parse;

#[cfg(test)]
mod test;

pub fn format(source: Vec<u8>) -> Result<String, parse::ParseError> {
    let result = parse::parse_into_fmt_node(source)?;
    let formatted = fmt::format(result.node, result.heredoc_map);
    Ok(formatted)
}
