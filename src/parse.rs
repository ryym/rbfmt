use lib_ruby_parser::Parser;

use crate::fmt;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<fmt::Node> {
    let parser = Parser::new(source, Default::default());
    let result = parser.do_parse();
    let _ast = match result.ast {
        None => return None,
        Some(ast) => ast,
    };
    todo!("parse into fmt node");
}
