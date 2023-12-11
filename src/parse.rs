use crate::fmt;
use lib_ruby_parser::{source::Comment, Loc, Node, Parser};

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<ParserResult> {
    let parser = Parser::new(source.clone(), Default::default());
    let mut result = parser.do_parse();

    // Sort the comments by their locations, because they are unordered when there is a heredoc.
    result.comments.sort_by_key(|c| c.location.begin);
    let reversed_comments = result.comments.into_iter().rev().collect();

    let decor_store = fmt::DecorStore::new();

    let mut builder = FmtNodeBuilder {
        src: source,
        comments: reversed_comments,
        decor_store,
        node_id_gen: 0,
        last_node_id: 0,
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(result.ast);
    // dbg!(&fmt_node);
    // dbg!(&builder.decor_store);
    Some(ParserResult {
        node: fmt_node,
        decor_store: builder.decor_store,
    })
}

#[derive(Debug)]
pub(crate) struct ParserResult {
    pub node: fmt::Node,
    pub decor_store: fmt::DecorStore,
}

#[derive(Debug)]
struct FmtNodeBuilder {
    src: Vec<u8>,
    comments: Vec<Comment>,
    decor_store: fmt::DecorStore,
    node_id_gen: usize,
    last_node_id: usize,
    last_loc_end: usize,
}

type MidDecors = (Option<fmt::Comment>, Vec<fmt::LineDecor>);

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: Option<Box<Node>>) -> fmt::Node {
        let fmt_node = node.map(|n| self.visit(*n));
        self.wrap_as_exprs(fmt_node, self.src.len())
    }

    fn next_node_id(&mut self) -> usize {
        self.node_id_gen += 1;
        self.node_id_gen
    }

    fn visit(&mut self, node: Node) -> fmt::Node {
        let node_id = self.next_node_id();
        let node_end = node.expression().end;
        let fmt_node = match node {
            Node::Nil(node) => self.parse_atom(&node.expression_l, node_id, "nil".to_string()),
            Node::True(node) => self.parse_atom(&node.expression_l, node_id, "true".to_string()),
            Node::False(node) => self.parse_atom(&node.expression_l, node_id, "false".to_string()),
            Node::Int(node) => self.parse_atom(&node.expression_l, node_id, node.value),
            Node::Float(node) => self.parse_atom(&node.expression_l, node_id, node.value),
            Node::Rational(node) => self.parse_atom(&node.expression_l, node_id, node.value),
            Node::Complex(node) => self.parse_atom(&node.expression_l, node_id, node.value),
            Node::Ivar(node) => self.parse_atom(&node.expression_l, node_id, node.name),
            Node::Cvar(node) => self.parse_atom(&node.expression_l, node_id, node.name),
            Node::Gvar(node) => self.parse_atom(&node.expression_l, node_id, node.name),
            Node::Begin(node) => {
                let nodes = node.statements.into_iter().map(|n| self.visit(n)).collect();
                fmt::Node::new(node_id, fmt::Kind::Exprs(nodes))
            }
            Node::If(node) => {
                self.consume_and_store_decors_until(node_id, node.expression_l.begin);
                let decors_in_if_and_cond = self.consume_decors_until(node.cond.expression().begin);
                if let Some((if_trailing, cond_leading)) = decors_in_if_and_cond {
                    if let Some(c) = if_trailing {
                        self.decor_store
                            .append_leading_decors(node_id, vec![fmt::LineDecor::Comment(c)]);
                    }
                    if !cond_leading.is_empty() {
                        self.decor_store
                            .append_leading_decors(node_id, cond_leading);
                    }
                }
                let cond = self.visit(*node.cond);
                let body = node.if_true.map(|n| self.visit(*n));
                let body = self.wrap_as_exprs(body, node.end_l.unwrap().begin);
                fmt::Node::new(
                    node_id,
                    fmt::Kind::IfExpr(fmt::IfExpr {
                        cond: Box::new(cond),
                        body: Box::new(body),
                    }),
                )
            }
            _ => {
                todo!("{}", format!("convert node {:?}", node));
            }
        };
        self.last_node_id = node_id;
        self.last_loc_end = node_end;
        fmt_node
    }

    fn parse_atom(&mut self, loc: &Loc, node_id: usize, value: String) -> fmt::Node {
        self.consume_and_store_decors_until(node_id, loc.begin);
        fmt::Node::new(node_id, fmt::Kind::Atom(value))
    }

    // Wrap the given node as Exprs to handle decors around it.
    // If the given node is Exprs, just add the EndDecors to it if necessary.
    fn wrap_as_exprs(&mut self, orig_node: Option<fmt::Node>, end: usize) -> fmt::Node {
        let (node_id, mut expr_nodes) = match orig_node {
            None => (self.next_node_id(), vec![]),
            Some(node) => match node.kind {
                fmt::Kind::Exprs(nodes) => (node.id, nodes),
                _ => (self.next_node_id(), vec![node]),
            },
        };

        if let Some(end_decors) = self.consume_decors_until(end) {
            let end_node = fmt::Node::new(self.next_node_id(), fmt::Kind::EndDecors);
            self.store_decors_to(self.last_node_id, end_node.id, end_decors);
            expr_nodes.push(end_node);
        }

        fmt::Node::new(node_id, fmt::Kind::Exprs(expr_nodes))
    }

    fn consume_and_store_decors_until(&mut self, node_id: usize, end: usize) {
        if let Some(decors) = self.consume_decors_until(end) {
            self.store_decors_to(self.last_node_id, node_id, decors);
        }
    }

    fn store_decors_to(&mut self, last_node_id: usize, node_id: usize, decors: MidDecors) {
        let (trailing_comment, line_decors) = decors;
        if let Some(comment) = trailing_comment {
            self.decor_store.set_trailing_comment(last_node_id, comment);
        }
        if !line_decors.is_empty() {
            self.decor_store.append_leading_decors(node_id, line_decors);
        }
    }

    fn consume_decors_until(&mut self, end: usize) -> Option<MidDecors> {
        let mut line_decors = Vec::new();
        let mut trailing_comment = None;

        // Find the first comment. It may be a trailing comment of the last node.
        match self.comments.last() {
            Some(comment) if comment.location.begin <= end => {
                let (comment_begin, comment_end) = (comment.location.begin, comment.location.end);
                let fmt_comment = self.get_comment_content(comment);
                if self.is_at_line_start(comment_begin) {
                    self.consume_empty_lines_until(comment_begin, &mut line_decors);
                    line_decors.push(fmt::LineDecor::Comment(fmt_comment));
                } else {
                    trailing_comment = Some(fmt_comment);
                }
                self.last_loc_end = comment_end - 1;
                self.comments.pop();
            }
            _ => {}
        };

        // Then find the other comments. They must not be a trailing comment.
        if !line_decors.is_empty() || trailing_comment.is_some() {
            loop {
                let comment = match self.comments.last() {
                    Some(comment) if comment.location.begin <= end => comment,
                    _ => break,
                };
                let fmt_comment = self.get_comment_content(comment);
                let comment_end = comment.location.end;
                self.consume_empty_lines_until(comment.location.begin, &mut line_decors);
                line_decors.push(fmt::LineDecor::Comment(fmt_comment));
                self.last_loc_end = comment_end - 1;
                self.comments.pop();
            }
        }

        // Finally consume the remaining empty lines.
        self.consume_empty_lines_until(end, &mut line_decors);

        if line_decors.is_empty() && trailing_comment.is_none() {
            None
        } else {
            Some((trailing_comment, line_decors))
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

    fn consume_empty_lines_until(&mut self, end: usize, line_decors: &mut Vec<fmt::LineDecor>) {
        let line_loc = self.last_empty_line_loc_within(self.last_loc_end, end);
        if let Some(line_loc) = line_loc {
            line_decors.push(fmt::LineDecor::EmptyLine);
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
