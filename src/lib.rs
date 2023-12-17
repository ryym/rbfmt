use parse_old::parse_into_fmt_node;

mod fmt;
mod parse;
mod parse_old;

#[cfg(test)]
mod test;

pub fn format_old(source: Vec<u8>) -> String {
    let result = match parse_into_fmt_node(source) {
        None => return String::new(),
        Some(result) => result,
    };
    fmt::format(result.node, result.decor_store, result.heredoc_map)
}

pub fn format(source: Vec<u8>) -> String {
    let result = match parse::parse_into_fmt_node(source) {
        None => return String::new(),
        Some(result) => result,
    };
    fmt::format(result.node, result.decor_store, result.heredoc_map)
}
