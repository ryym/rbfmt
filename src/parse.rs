use std::{collections::HashMap, iter::Peekable, ops::Range};

use crate::fmt;
use ruby_prism as prism;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<ParserResult> {
    let result = prism::parse(&source);

    let comments = result.comments().peekable();
    let decor_store = fmt::DecorStore::new();
    let heredoc_map = HashMap::new();

    let mut builder = FmtNodeBuilder {
        src: &source,
        comments,
        decor_store,
        heredoc_map,
        position_gen: 0,
        last_pos: fmt::Pos(0),
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(result.node());
    // dbg!(&fmt_node);
    // dbg!(&builder.heredoc_map);
    // dbg!(&builder.decor_store);
    Some(ParserResult {
        node: fmt_node,
        decor_store: builder.decor_store,
        heredoc_map: builder.heredoc_map,
    })
}

#[derive(Debug)]
pub(crate) struct ParserResult {
    pub node: fmt::Node,
    pub decor_store: fmt::DecorStore,
    pub heredoc_map: fmt::HeredocMap,
}

type MidDecors = (Option<fmt::Comment>, Vec<fmt::LineDecor>);

struct FmtNodeBuilder<'src> {
    src: &'src [u8],
    // comments: Vec<Comment>,
    // token_set: TokenSet,
    comments: Peekable<prism::Comments<'src>>,
    decor_store: fmt::DecorStore,
    heredoc_map: fmt::HeredocMap,
    position_gen: usize,
    last_pos: fmt::Pos,
    last_loc_end: usize,
}

impl FmtNodeBuilder<'_> {
    fn build_fmt_node(&mut self, node: prism::Node) -> fmt::Node {
        self.visit(node)
    }

    fn next_pos(&mut self) -> fmt::Pos {
        self.position_gen += 1;
        fmt::Pos(self.position_gen)
    }

    fn source_lossy_at(loc: &prism::Location) -> String {
        String::from_utf8_lossy(loc.as_slice()).to_string()
    }

    fn visit(&mut self, node: prism::Node) -> fmt::Node {
        use prism::Node;

        let loc_end = node.location().end_offset();
        let node = match node {
            Node::ProgramNode { .. } => {
                let node = node.as_program_node().unwrap();
                let pos = self.next_pos();
                let mut nodes = vec![];
                for n in node.statements().body().iter() {
                    nodes.push(self.visit(n));
                }
                let mut exprs = fmt::Exprs(nodes);
                self.append_end_decors(&mut exprs, self.src.len());
                fmt::Node::new(pos, fmt::Kind::Exprs(exprs))
            }

            Node::NilNode { .. } => self.parse_atom(node),
            Node::TrueNode { .. } => self.parse_atom(node),
            Node::FalseNode { .. } => self.parse_atom(node),
            Node::IntegerNode { .. } => self.parse_atom(node),
            Node::FloatNode { .. } => self.parse_atom(node),
            Node::RationalNode { .. } => self.parse_atom(node),
            Node::ImaginaryNode { .. } => self.parse_atom(node),
            Node::InstanceVariableReadNode { .. } => self.parse_atom(node),
            Node::ClassVariableReadNode { .. } => self.parse_atom(node),
            Node::GlobalVariableReadNode { .. } => self.parse_atom(node),

            _ => todo!("parse {:?}", node),
        };

        self.last_pos = node.pos;
        self.last_loc_end = loc_end;
        node
    }

    fn parse_atom(&mut self, node: prism::Node) -> fmt::Node {
        let pos = self.next_pos();
        self.consume_and_store_decors_until(pos, node.location().start_offset());
        let value = Self::source_lossy_at(&node.location());
        fmt::Node::new(pos, fmt::Kind::Atom(value))
    }

    fn append_end_decors(&mut self, exprs: &mut fmt::Exprs, end: usize) {
        if let Some(end_decors) = self.consume_decors_until(end) {
            let end_node = fmt::Node::new(self.next_pos(), fmt::Kind::EndDecors);
            self.store_decors_to(self.last_pos, end_node.pos, end_decors);
            exprs.0.push(end_node);
        }
    }

    fn consume_and_store_decors_until(&mut self, pos: fmt::Pos, end: usize) {
        if let Some(decors) = self.consume_decors_until(end) {
            self.store_decors_to(self.last_pos, pos, decors);
        }
    }

    fn store_decors_to(&mut self, last_pos: fmt::Pos, pos: fmt::Pos, decors: MidDecors) {
        let (trailing_comment, line_decors) = decors;
        if let Some(comment) = trailing_comment {
            self.decor_store.set_trailing_comment(last_pos, comment);
        }
        if !line_decors.is_empty() {
            self.decor_store.append_leading_decors(pos, line_decors);
        }
    }

    fn consume_decors_until(&mut self, end: usize) -> Option<MidDecors> {
        let mut line_decors = Vec::new();
        let mut trailing_comment = None;

        // Find the first comment. It may be a trailing comment of the last node.
        if let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if (self.last_loc_end..=end).contains(&loc.start_offset()) {
                let value = Self::source_lossy_at(&loc);
                let fmt_comment = fmt::Comment { value };
                if self.is_at_line_start(loc.start_offset()) {
                    self.consume_empty_lines_until(loc.start_offset(), &mut line_decors);
                    line_decors.push(fmt::LineDecor::Comment(fmt_comment));
                } else {
                    trailing_comment = Some(fmt_comment);
                }
                self.last_loc_end = loc.end_offset() - 1;
                self.comments.next();
            }
        }

        // Then find the other comments. They must not be a trailing comment.
        if !line_decors.is_empty() || trailing_comment.is_some() {
            while let Some(comment) = self.comments.peek() {
                let loc = comment.location();
                if !(self.last_loc_end..=end).contains(&loc.start_offset()) {
                    break;
                };
                let value = Self::source_lossy_at(&loc);
                let fmt_comment = fmt::Comment { value };
                self.consume_empty_lines_until(loc.start_offset(), &mut line_decors);
                line_decors.push(fmt::LineDecor::Comment(fmt_comment));
                self.last_loc_end = loc.end_offset() - 1;
                self.comments.next();
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

    fn consume_empty_lines_until(&mut self, end: usize, line_decors: &mut Vec<fmt::LineDecor>) {
        let range = self.last_empty_line_range_within(self.last_loc_end, end);
        if let Some(range) = range {
            line_decors.push(fmt::LineDecor::EmptyLine);
            self.last_loc_end = range.end;
        }
    }

    fn last_empty_line_range_within(&self, begin: usize, end: usize) -> Option<Range<usize>> {
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
            (Some(begin), Some(end)) => Some(begin..end),
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
