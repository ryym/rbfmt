use lib_ruby_parser::{source::Comment, Loc, Node, Parser};

use crate::fmt;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<fmt::Node> {
    let parser = Parser::new(source.clone(), Default::default());

    let mut result = parser.do_parse();
    // Sort the comments by their locations, because they are unordered when there is a heredoc.
    result.comments.sort_by_key(|c| c.location.begin);
    let reversed_comments = result.comments.into_iter().rev().collect();

    let mut builder = FmtNodeBuilder {
        src: source,
        comments: reversed_comments,
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(result.ast);
    Some(fmt_node)
}

#[derive(Debug)]
struct FmtNodeBuilder {
    src: Vec<u8>,
    comments: Vec<Comment>,
    last_loc_end: usize,
}

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: Option<Box<Node>>) -> fmt::Node {
        let mut stmts = Vec::with_capacity(2);
        if let Some(node) = node {
            let fmt_node = self.visit(*node);
            stmts.push(fmt_node);
        }
        if let Some(trivia) = self.consume_trivia_until(self.src.len()) {
            let eof = fmt::Node::None(trivia);
            stmts.push(eof);
        }
        fmt::Node::Statements(fmt::Statements { nodes: stmts })
    }

    fn visit(&mut self, node: Node) -> fmt::Node {
        let loc_end = node.expression().end;
        let fmt_node = match node {
            Node::Nil(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Nil(trivia)
            }
            Node::True(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Boolean(trivia, fmt::Boolean { is_true: true })
            }
            Node::False(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Boolean(trivia, fmt::Boolean { is_true: false })
            }
            Node::Int(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Number(trivia, fmt::Number { value: node.value })
            }
            Node::Float(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Number(trivia, fmt::Number { value: node.value })
            }
            Node::Rational(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Number(trivia, fmt::Number { value: node.value })
            }
            Node::Complex(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Number(trivia, fmt::Number { value: node.value })
            }
            Node::Ivar(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Identifier(trivia, fmt::Identifier { name: node.name })
            }
            Node::Cvar(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Identifier(trivia, fmt::Identifier { name: node.name })
            }
            Node::Gvar(node) => {
                let trivia = self.consume_trivia_until(node.expression_l.begin);
                fmt::Node::Identifier(trivia, fmt::Identifier { name: node.name })
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

    fn consume_trivia_until(&mut self, end: usize) -> Option<fmt::Trivia> {
        let mut trivia = fmt::Trivia::new();

        // Find the first comment. It may be a trailing comment of the last node.
        let first_comment_found = match self.comments.last() {
            Some(comment) if comment.location.begin <= end => {
                let (comment_begin, comment_end) = (comment.location.begin, comment.location.end);
                let fmt_comment = self.get_comment_content(comment);
                if self.is_at_line_start(comment_begin) {
                    self.consume_empty_lines_until(comment_begin, &mut trivia);
                    trivia
                        .leading_trivia
                        .push(fmt::TriviaNode::LineComment(fmt_comment));
                } else {
                    trivia.last_trailing_comment = Some(fmt_comment);
                }
                self.last_loc_end = comment_end - 1;
                self.comments.pop();
                true
            }
            _ => false,
        };

        if first_comment_found {
            // Then find the other comments. They must not be a trailing comment.
            loop {
                let comment = match self.comments.last() {
                    Some(comment) if comment.location.begin <= end => comment,
                    _ => break,
                };
                let fmt_comment = self.get_comment_content(comment);
                let comment_end = comment.location.end;
                self.consume_empty_lines_until(comment.location.begin, &mut trivia);
                trivia
                    .leading_trivia
                    .push(fmt::TriviaNode::LineComment(fmt_comment));
                self.last_loc_end = comment_end - 1;
                self.comments.pop();
            }
        }

        // Finally consume the remaining empty lines.
        self.consume_empty_lines_until(end, &mut trivia);
        if trivia.is_empty() {
            None
        } else {
            Some(trivia)
        }
    }

    fn get_comment_content(&self, comment: &Comment) -> fmt::Comment {
        let comment_bytes = &self.src[comment.location.begin..comment.location.end];
        // Ignore non-UTF8 source code for now.
        let comment_str = String::from_utf8_lossy(comment_bytes)
            .trim_end()
            .to_string();
        fmt::Comment { value: comment_str }
    }

    fn consume_empty_lines_until(&mut self, end: usize, trivia: &mut fmt::Trivia) {
        let line_loc = self.last_empty_line_loc_within(self.last_loc_end, end);
        if let Some(line_loc) = line_loc {
            trivia.leading_trivia.push(fmt::TriviaNode::EmptyLine);
            self.last_loc_end = line_loc.end;
        }
    }

    fn last_empty_line_loc_within(&self, begin: usize, end: usize) -> Option<Loc> {
        let mut line_begin: Option<usize> = None;
        let mut line_end: Option<usize> = None;
        for i in (begin..end).rev() {
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
        while let Some(b) = self.src.get(idx) {
            match b {
                b' ' => {
                    idx -= 1;
                    continue;
                }
                b'\n' => break,
                _ => {
                    has_char_between_last_newline = true;
                    break;
                }
            }
        }
        !has_char_between_last_newline
    }
}
