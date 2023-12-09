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
        let mut root = fmt::Statements { nodes: vec![] };
        self.visit(node, &mut root);
        fmt::Node::Statements(root)
    }

    fn visit<G: fmt::GroupNodeEntity>(&mut self, node: Node, group: &mut G) {
        let loc_end = node.expression().end;
        let fmt_node = match node {
            Node::Ivar(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Identifier(fmt::Identifier { name: node.name })
            }
            Node::Cvar(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Identifier(fmt::Identifier { name: node.name })
            }
            Node::Gvar(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Identifier(fmt::Identifier { name: node.name })
            }
            Node::Begin(node) => {
                let mut stmts = fmt::Statements { nodes: vec![] };
                for n in node.statements {
                    self.visit(n, &mut stmts);
                }
                fmt::Node::Statements(stmts)
            }
            _ => {
                todo!("{}", format!("convert node {:?}", node));
            }
        };
        group.append_node(fmt_node);
        self.last_loc_end = loc_end;
    }

    fn consume_trivia_until<G: fmt::GroupNodeEntity>(&mut self, end: usize, group: &mut G) {
        self.consume_empty_lines_until(end, group);
    }

    fn consume_empty_lines_until<G: fmt::GroupNodeEntity>(&mut self, end: usize, group: &mut G) {
        let line_loc = self.last_empty_line_loc_within(self.last_loc_end, end);
        if let Some(line_loc) = line_loc {
            group.append_node(fmt::Node::EmptyLine);
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
