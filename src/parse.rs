use lib_ruby_parser::{Node, Parser};

use crate::fmt;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<fmt::Node> {
    let parser = Parser::new(source, Default::default());
    let result = parser.do_parse();
    let ast = match result.ast {
        None => return None,
        Some(ast) => ast,
    };
    let mut builder = FmtNodeBuilder {};
    let fmt_node = builder.build_fmt_node(*ast);
    Some(fmt_node)
}

struct FmtNodeBuilder {}

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: Node) -> fmt::Node {
        self.visit(node)
    }

    fn visit(&mut self, node: Node) -> fmt::Node {
        match node {
            Node::Ivar(node) => fmt::Node::Identifier(fmt::Identifier { name: node.name }),
            Node::Cvar(node) => fmt::Node::Identifier(fmt::Identifier { name: node.name }),
            Node::Gvar(node) => fmt::Node::Identifier(fmt::Identifier { name: node.name }),
            Node::Begin(node) => {
                let nodes = node.statements.into_iter().map(|n| self.visit(n)).collect();
                fmt::Node::Statements(fmt::Statements { nodes })
            }
            _ => {
                todo!("{}", format!("convert node {:?}", node));
            }
        }
    }
}
