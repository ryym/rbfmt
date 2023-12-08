use parse::parse_into_fmt_node;

mod fmt;
mod parse;

#[cfg(test)]
mod test;

pub fn format(source: Vec<u8>) -> String {
    let node = match parse_into_fmt_node(source) {
        None => return String::new(),
        Some(node) => node,
    };
    fmt::format(node)
}
