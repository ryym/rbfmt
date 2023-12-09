use lib_ruby_parser::{Loc, Node, Parser};

use crate::fmt;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<fmt::Node> {
    let parser = Parser::new(source.clone(), Default::default());
    let result = parser.do_parse();
    let ast = match result.ast {
        None => return None,
        Some(ast) => ast,
    };
    let mut builder = FmtNodeBuilder {
        src: source,
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(*ast);
    Some(fmt_node)
}

#[derive(Debug)]
struct FmtNodeBuilder {
    src: Vec<u8>,
    last_loc_end: usize,
}

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: Node) -> fmt::Node {
        self.visit(node)
    }

    fn visit(&mut self, node: Node) -> fmt::Node {
        let loc_end = node.expression().end;
        let fmt_node = match node {
            Node::Ivar(node) => {
                self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Identifier(fmt::Identifier { name: node.name })
            }
            Node::Cvar(node) => {
                self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Identifier(fmt::Identifier { name: node.name })
            }
            Node::Gvar(node) => {
                self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Identifier(fmt::Identifier { name: node.name })
            }
            Node::Begin(node) => {
                let nodes = node.statements.into_iter().map(|n| self.visit(n)).collect();
                fmt::Node::Statements(fmt::Statements { nodes })
            }
            _ => {
                todo!("{}", format!("convert node {:?}", node));
            }
        };
        self.last_loc_end = loc_end;
        fmt_node
    }

    fn consume_trivia_until(&mut self, end: usize) {
        self.consume_empty_lines_until(end);
    }

    fn consume_empty_lines_until(&mut self, end: usize) {
        let line_loc = self.last_empty_line_loc_within(self.last_loc_end, end);
        if let Some(line_loc) = line_loc {
            self.last_loc_end = line_loc.end;
        }
    }

    fn last_empty_line_loc_within(&self, begin: usize, end: usize) -> Option<Loc> {
        let mut line_begin: Option<usize> = None;
        let mut line_end: Option<usize> = None;
        for i in (begin..end).into_iter().rev() {
            let b = self.src[i];
            if b == b'\n' {
                if line_end.is_none() {
                    line_end = Some(i + 1);
                } else {
                    line_begin = Some(i);
                    break;
                }
            } else if line_end.is_some() && b != b' ' {
                line_end = None;
            }
        }
        match (line_begin, line_end) {
            (Some(begin), Some(end)) => Some(Loc { begin, end }),
            _ => None,
        }
    }
}
