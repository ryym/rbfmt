mod fmt;
mod parse;

#[cfg(test)]
mod test;

pub fn format(source: Vec<u8>) -> String {
    let result = match parse::parse_into_fmt_node(source) {
        None => return String::new(),
        Some(result) => result,
    };
    fmt::format(result.node, result.decor_store, result.heredoc_map)
}
