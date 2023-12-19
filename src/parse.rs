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
    // dbg!(&builder.decor_store);
    // dbg!(&builder.heredoc_map);
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

struct IfOrUnless<'src> {
    is_if: bool,
    loc: prism::Location<'src>,
    predicate: prism::Node<'src>,
    statements: Option<prism::StatementsNode<'src>>,
    consequent: Option<prism::Node<'src>>,
    end_loc: Option<prism::Location<'src>>,
}

struct FmtNodeBuilder<'src> {
    src: &'src [u8],
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
        let loc_end = node.location().end_offset();
        let node = match node {
            prism::Node::ProgramNode { .. } => {
                let node = node.as_program_node().unwrap();
                let pos = self.next_pos();
                let exprs = self.visit_statements(Some(node.statements()), Some(self.src.len()));
                fmt::Node {
                    pos,
                    width: exprs.width(),
                    kind: fmt::Kind::Exprs(exprs),
                }
            }
            prism::Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                let exprs = self.visit_statements(Some(node), None);
                fmt::Node {
                    pos: fmt::Pos::none(),
                    width: exprs.width(),
                    kind: fmt::Kind::Exprs(exprs),
                }
            }

            prism::Node::NilNode { .. } => self.parse_atom(node),
            prism::Node::TrueNode { .. } => self.parse_atom(node),
            prism::Node::FalseNode { .. } => self.parse_atom(node),
            prism::Node::IntegerNode { .. } => self.parse_atom(node),
            prism::Node::FloatNode { .. } => self.parse_atom(node),
            prism::Node::RationalNode { .. } => self.parse_atom(node),
            prism::Node::ImaginaryNode { .. } => self.parse_atom(node),
            prism::Node::InstanceVariableReadNode { .. } => self.parse_atom(node),
            prism::Node::ClassVariableReadNode { .. } => self.parse_atom(node),
            prism::Node::GlobalVariableReadNode { .. } => self.parse_atom(node),

            prism::Node::StringNode { .. } => {
                let node = node.as_string_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let has_decors = self.consume_and_store_decors_until(pos, loc.start_offset());
                if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let opening_len = self.visit_simple_heredoc(pos, node);
                    let width = if has_decors {
                        fmt::Width::NotFlat
                    } else {
                        fmt::Width::Flat(opening_len)
                    };
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::HeredocOpening,
                    }
                } else {
                    let (str, mut width) = self.visit_string(node);
                    if has_decors {
                        width = fmt::Width::NotFlat
                    }
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::Str(str),
                    }
                }
            }
            prism::Node::InterpolatedStringNode { .. } => {
                let node = node.as_interpolated_string_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let has_decors = self.consume_and_store_decors_until(pos, loc.start_offset());
                if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let opening_len = self.visit_complex_heredoc(pos, node);
                    let width = if has_decors {
                        fmt::Width::NotFlat
                    } else {
                        fmt::Width::Flat(opening_len)
                    };
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::HeredocOpening,
                    }
                } else {
                    let (str, mut width) = self.visit_interpolated_string(node);
                    if has_decors {
                        width = fmt::Width::NotFlat
                    }
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::DynStr(str),
                    }
                }
            }

            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();
                self.visit_if_or_unless(IfOrUnless {
                    is_if: true,
                    loc: node.location(),
                    predicate: node.predicate(),
                    statements: node.statements(),
                    consequent: node.consequent(),
                    end_loc: node.end_keyword_loc(),
                })
            }
            prism::Node::UnlessNode { .. } => {
                let node = node.as_unless_node().unwrap();
                self.visit_if_or_unless(IfOrUnless {
                    is_if: false,
                    loc: node.location(),
                    predicate: node.predicate(),
                    statements: node.statements(),
                    consequent: node.consequent().map(|n| n.as_node()),
                    end_loc: node.end_keyword_loc(),
                })
            }

            prism::Node::CallNode { .. } => {
                let node = node.as_call_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let has_decors = self.consume_and_store_decors_until(pos, loc.start_offset());
                let chain = self.visit_call(node);
                let width = if has_decors {
                    fmt::Width::NotFlat
                } else {
                    chain.width()
                };
                fmt::Node {
                    pos,
                    width,
                    kind: fmt::Kind::MethodChain(chain),
                }
            }

            _ => todo!("parse {:?}", node),
        };

        self.last_pos = node.pos;
        self.last_loc_end = loc_end;
        node
    }

    fn parse_atom(&mut self, node: prism::Node) -> fmt::Node {
        let pos = self.next_pos();
        let loc = node.location();
        let has_decors = self.consume_and_store_decors_until(pos, loc.start_offset());
        let value = Self::source_lossy_at(&loc);
        let flat_width = if has_decors {
            fmt::Width::NotFlat
        } else {
            fmt::Width::Flat(value.len())
        };
        fmt::Node {
            pos,
            kind: fmt::Kind::Atom(value),
            width: flat_width,
        }
    }

    fn visit_string(&mut self, node: prism::StringNode) -> (fmt::Str, fmt::Width) {
        let value = Self::source_lossy_at(&node.content_loc());
        let opening = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = node.closing_loc().as_ref().map(Self::source_lossy_at);
        let str = fmt::Str {
            opening,
            value: value.into(),
            closing,
        };
        let width = fmt::Width::Flat(str.len());
        (str, width)
    }

    fn visit_interpolated_string(
        &mut self,
        itp_str: prism::InterpolatedStringNode,
    ) -> (fmt::DynStr, fmt::Width) {
        let mut parts = vec![];
        let mut width = fmt::Width::Flat(0);
        for part in itp_str.parts().iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let (str, str_width) = self.visit_string(node);
                    parts.push(fmt::DynStrPart::Str(str));
                    width.append(&str_width);
                    self.last_loc_end = node_end;
                }
                prism::Node::InterpolatedStringNode { .. } => {
                    let node = part.as_interpolated_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let (str, str_width) = self.visit_interpolated_string(node);
                    parts.push(fmt::DynStrPart::DynStr(str));
                    width.append(&str_width);
                    self.last_loc_end = node_end;
                }
                prism::Node::EmbeddedStatementsNode { .. } => {
                    let node = part.as_embedded_statements_node().unwrap();
                    let loc = node.location();
                    let embedded_pos = self.next_pos();
                    self.last_pos = embedded_pos;
                    let exprs = self.visit_statements(node.statements(), Some(loc.end_offset()));
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    width.append(&exprs.width());
                    parts.push(fmt::DynStrPart::Exprs(fmt::EmbeddedExprs {
                        pos: embedded_pos,
                        exprs,
                        opening,
                        closing,
                    }));
                }
                _ => panic!("unexpected string interpolation node: {:?}", part),
            }
        }
        let opening = itp_str.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = itp_str.closing_loc().as_ref().map(Self::source_lossy_at);
        let str = fmt::DynStr {
            opening,
            parts,
            closing,
        };
        (str, width)
    }

    fn is_heredoc(str_opening_loc: Option<&prism::Location>) -> bool {
        if let Some(loc) = str_opening_loc {
            let bytes = loc.as_slice();
            bytes.len() > 2 && bytes[0] == b'<' && bytes[1] == b'<'
        } else {
            false
        }
    }

    fn visit_simple_heredoc(&mut self, pos: fmt::Pos, node: prism::StringNode) -> usize {
        let open = node.opening_loc().unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let id = String::from_utf8_lossy(id).to_string();
        let (str, _) = self.visit_string(node);
        let heredoc = fmt::Heredoc {
            id,
            indent_mode,
            parts: vec![fmt::HeredocPart::Str(str)],
        };
        self.heredoc_map.insert(pos, heredoc);
        open.len()
    }

    fn visit_complex_heredoc(
        &mut self,
        pos: fmt::Pos,
        node: prism::InterpolatedStringNode,
    ) -> usize {
        let open = node.opening_loc().unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let id = String::from_utf8_lossy(id).to_string();
        let mut parts = vec![];
        let mut last_str_end: Option<usize> = None;
        for part in node.parts().iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let (str, _) = self.visit_string(node);
                    parts.push(fmt::HeredocPart::Str(str));
                    self.last_loc_end = node_end;
                    last_str_end = Some(node_end);
                }
                prism::Node::EmbeddedStatementsNode { .. } => {
                    let node = part.as_embedded_statements_node().unwrap();
                    let loc = node.location();
                    if let Some(last_str_end) = last_str_end {
                        // I don't know why but ruby-prism ignores spaces before an interpolation in some cases.
                        if last_str_end < loc.start_offset() {
                            let value = self.src[last_str_end..loc.start_offset()].to_vec();
                            let str = fmt::Str {
                                opening: None,
                                value,
                                closing: None,
                            };
                            parts.push(fmt::HeredocPart::Str(str))
                        }
                    }
                    let embedded_pos = self.next_pos();
                    self.last_pos = embedded_pos;
                    let exprs = self.visit_statements(node.statements(), Some(loc.end_offset()));
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    parts.push(fmt::HeredocPart::Exprs(fmt::EmbeddedExprs {
                        pos: embedded_pos,
                        exprs,
                        opening,
                        closing,
                    }));
                }
                _ => panic!("unexpected heredoc part: {:?}", part),
            }
        }
        let heredoc = fmt::Heredoc {
            id,
            indent_mode,
            parts,
        };
        self.heredoc_map.insert(pos, heredoc);
        open.len()
    }

    fn visit_statements(
        &mut self,
        node: Option<prism::StatementsNode>,
        end: Option<usize>,
    ) -> fmt::Exprs {
        let mut exprs = fmt::Exprs::new();
        if let Some(node) = node {
            for n in node.body().iter() {
                let node = self.visit(n);
                exprs.append_node(node);
            }
        }
        if let Some(end) = end {
            self.append_end_decors(&mut exprs, end);
        }
        exprs
    }

    fn visit_if_or_unless(&mut self, node: IfOrUnless) -> fmt::Node {
        let pos = self.next_pos();
        let _ = self.consume_and_store_decors_until(pos, node.loc.start_offset());

        let if_pos = self.next_pos();
        self.last_pos = if_pos;
        let predicate = self.visit(node.predicate);
        let end_loc = node.end_loc.expect("end must exist in root if/unless");

        let ifexpr = match node.consequent {
            // if...(elsif...|else...)+end
            Some(conseq) => {
                let else_start = conseq.location().start_offset();
                let body = self.visit_statements(node.statements, Some(else_start));
                let if_first = fmt::Conditional {
                    pos: if_pos,
                    cond: Box::new(predicate),
                    body,
                };
                let mut ifexpr = fmt::IfExpr::new(node.is_if, if_first);
                self.visit_ifelse(conseq, &mut ifexpr);
                ifexpr
            }
            // if...end
            None => {
                let body = self.visit_statements(node.statements, Some(end_loc.start_offset()));
                let if_first = fmt::Conditional {
                    pos: if_pos,
                    cond: Box::new(predicate),
                    body,
                };
                fmt::IfExpr::new(node.is_if, if_first)
            }
        };

        fmt::Node {
            pos,
            kind: fmt::Kind::IfExpr(ifexpr),
            width: fmt::Width::NotFlat,
        }
    }

    fn visit_ifelse(&mut self, node: prism::Node, ifexpr: &mut fmt::IfExpr) {
        match node {
            // elsif ("if" only, "unles...elsif" is syntax error)
            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();
                let elsif_pos = self.next_pos();
                self.last_pos = elsif_pos;
                let predicate = self.visit(node.predicate());
                let conseq = node.consequent();

                let body_end_loc = conseq
                    .as_ref()
                    .map(|n| n.location().start_offset())
                    .or_else(|| node.end_keyword_loc().map(|l| l.end_offset()));
                let body = self.visit_statements(node.statements(), body_end_loc);

                ifexpr.elsifs.push(fmt::Conditional {
                    pos: elsif_pos,
                    cond: Box::new(predicate),
                    body,
                });
                if let Some(conseq) = conseq {
                    self.visit_ifelse(conseq, ifexpr);
                }
            }
            // else
            prism::Node::ElseNode { .. } => {
                let node = node.as_else_node().unwrap();
                let else_pos = self.next_pos();
                self.last_pos = else_pos;

                let body_end_loc = node.end_keyword_loc().map(|l| l.end_offset());
                let body = self.visit_statements(node.statements(), body_end_loc);
                ifexpr.if_last = Some(fmt::Else {
                    pos: else_pos,
                    body,
                });
            }
            _ => {
                panic!("unexpected node in IfNode: {:?}", node);
            }
        }
    }

    fn visit_call(&mut self, call: prism::CallNode) -> fmt::MethodChain {
        let mut chain = match call.receiver() {
            Some(receiver) => match receiver {
                prism::Node::CallNode { .. } => {
                    let node = receiver.as_call_node().unwrap();
                    self.visit_call(node)
                }
                _ => {
                    let recv = self.visit(receiver);
                    fmt::MethodChain::new(Some(recv))
                }
            },
            None => fmt::MethodChain::new(None),
        };

        let mut call_width = fmt::Width::Flat(0);

        let call_pos = self.next_pos();
        if let Some(msg_loc) = call.message_loc() {
            let has_decors = self.consume_and_store_decors_until(call_pos, msg_loc.start_offset());
            if has_decors {
                call_width = fmt::Width::NotFlat;
            }
        } else {
            // foo.\n#hoge\n(2)
        }

        let chain_type = if call.is_safe_navigation() {
            fmt::ChainType::SafeNav
        } else {
            fmt::ChainType::Normal
        };
        let name = String::from_utf8_lossy(call.name().as_slice()).to_string();

        if let Some(loc) = call.call_operator_loc() {
            let op_len = loc.end_offset() - loc.start_offset();
            call_width.append_value(op_len);
        }
        call_width.append_value(name.len());

        let mut args = vec![];
        if let Some(args_node) = call.arguments() {
            // For now surround the arguments by parentheses always.
            call_width.append_value("()".len());
            for (i, arg) in args_node.arguments().iter().enumerate() {
                if i > 0 {
                    call_width.append_value(", ".len());
                }
                let node = self.visit(arg);
                call_width.append(&node.width);
                args.push(node);
            }
        }

        let block = call.block().map(|node| match node {
            prism::Node::BlockNode { .. } => {
                let node = node.as_block_node().unwrap();
                let block_pos = self.next_pos();

                self.last_pos = block_pos;
                let body = node.body().map(|n| self.visit(n));
                let body_end_loc = node.closing_loc().start_offset();
                let body = self.wrap_as_exprs(body, Some(body_end_loc));

                call_width.append_value(" {  }".len());
                call_width.append(&body.width());

                fmt::MethodBlock {
                    pos: block_pos,
                    body,
                }
            }
            _ => panic!("unexpected node for call block: {:?}", node),
        });

        chain.append_call(fmt::MethodCall {
            pos: call_pos,
            width: call_width,
            chain_type,
            name,
            args,
            block,
        });

        self.last_pos = call_pos;
        self.last_loc_end = call.location().end_offset();
        chain
    }

    fn wrap_as_exprs(&mut self, node: Option<fmt::Node>, end: Option<usize>) -> fmt::Exprs {
        let mut exprs = match node {
            None => fmt::Exprs::new(),
            Some(node) => match node.kind {
                fmt::Kind::Exprs(exprs) => exprs,
                _ => {
                    let mut exprs = fmt::Exprs::new();
                    exprs.append_node(node);
                    exprs
                }
            },
        };
        if let Some(end) = end {
            self.append_end_decors(&mut exprs, end);
        }
        exprs
    }

    fn append_end_decors(&mut self, exprs: &mut fmt::Exprs, end: usize) {
        if let Some(end_decors) = self.consume_decors_until(end) {
            let end_node = fmt::Node {
                pos: self.next_pos(),
                kind: fmt::Kind::EndDecors,
                width: fmt::Width::NotFlat,
            };
            self.store_decors_to(self.last_pos, end_node.pos, end_decors);
            exprs.append_node(end_node);
        }
    }

    #[must_use = "you need to check deocrs existence for flat width calculation"]
    fn consume_and_store_decors_until(&mut self, pos: fmt::Pos, end: usize) -> bool {
        if let Some(decors) = self.consume_decors_until(end) {
            let has_leading_decors = !decors.1.is_empty();
            self.store_decors_to(self.last_pos, pos, decors);
            has_leading_decors
        } else {
            false
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

    fn last_empty_line_range_within(&self, start: usize, end: usize) -> Option<Range<usize>> {
        let mut line_start: Option<usize> = None;
        let mut line_end: Option<usize> = None;
        for i in (start..end).rev() {
            let b = self.src[i];
            if b == b'\n' {
                if line_end.is_none() {
                    line_end = Some(i + 1);
                } else {
                    line_start = Some(i);
                    break;
                }
            } else if line_end.is_some() && b != b' ' {
                line_end = None;
            }
        }
        match (line_start, line_end) {
            (Some(start), Some(end)) => Some(start..end),
            _ => None,
        }
    }

    fn is_at_line_start(&self, start: usize) -> bool {
        if start == 0 {
            return true;
        }
        let mut idx = start - 1;
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
