use lib_ruby_parser::{source::Comment, Loc, Node, Parser};

use crate::fmt;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<fmt::Node> {
    let parser = Parser::new(source.clone(), Default::default());

    let mut result = parser.do_parse();
    let ast = match result.ast {
        None => return None,
        Some(ast) => ast,
    };
    // Sort the comments by their locations, because they are unordered when there is a heredoc.
    result.comments.sort_by_key(|c| c.location.begin);
    let reversed_comments = result.comments.into_iter().rev().collect();

    let mut builder = FmtNodeBuilder {
        src: source,
        comments: reversed_comments,
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(*ast);
    Some(fmt_node)
}

#[derive(Debug)]
struct FmtNodeBuilder {
    src: Vec<u8>,
    comments: Vec<Comment>,
    last_loc_end: usize,
}

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: Node) -> fmt::Node {
        let mut root = fmt::Statements { nodes: vec![] };
        self.visit(node, &mut root);
        self.consume_trivia_until(self.src.len(), &mut root);
        fmt::Node::Statements(root)
    }

    fn visit<G: fmt::GroupNodeEntity>(&mut self, node: Node, group: &mut G) {
        let loc_end = node.expression().end;
        let fmt_node = match node {
            Node::Int(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Number(fmt::Number { value: node.value })
            }
            Node::Float(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Number(fmt::Number { value: node.value })
            }
            Node::Rational(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Number(fmt::Number { value: node.value })
            }
            Node::Complex(node) => {
                self.consume_trivia_until(node.expression_l.begin, group);
                fmt::Node::Number(fmt::Number { value: node.value })
            }
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
        // Find comments and empty lines between the last parsed location and the given end.
        loop {
            let (comment_begin, comment_end) = {
                match self.comments.last() {
                    Some(comment) if comment.location.begin <= end => {
                        (comment.location.begin, comment.location.end)
                    }
                    _ => {
                        self.consume_empty_lines_until(end, group);
                        break;
                    }
                }
            };
            self.consume_empty_lines_until(comment_begin, group);

            // Ignore non-UTF8 source code for now.
            let comment_bytes = &self.src[comment_begin..comment_end];
            let comment_str = String::from_utf8_lossy(comment_bytes).to_string();
            let comment_node = fmt::Comment { value: comment_str };

            if self.is_at_line_start(comment_begin) {
                group.append_node(fmt::Node::LineComment(comment_node))
            } else {
                group.append_node(fmt::Node::TrailingComment(comment_node))
            }

            // Set the location of newline like other actual syntax nodes.
            self.last_loc_end = comment_end - 1;
            self.comments.pop();
        }
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

    fn is_at_line_start(&self, begin: usize) -> bool {
        if begin == 0 {
            return true;
        }
        let mut idx = begin - 1;
        let mut has_char_between_last_newline = false;
        loop {
            match self.src.get(idx) {
                Some(b) => match b {
                    b' ' => {
                        idx -= 1;
                        continue;
                    }
                    b'\n' => break,
                    _ => {
                        has_char_between_last_newline = true;
                        break;
                    }
                },
                None => break,
            }
        }
        !has_char_between_last_newline
    }
}
