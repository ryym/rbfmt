use crate::fmt;
use ruby_prism as prism;
use std::{collections::HashMap, iter::Peekable, ops::Range};

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

#[derive(Debug)]
struct Decors {
    leading: Vec<fmt::LineDecor>,
    trailing: Option<fmt::Comment>,
    width: fmt::Width,
}

impl Decors {
    fn new() -> Self {
        Self {
            leading: vec![],
            trailing: None,
            width: fmt::Width::Flat(0),
        }
    }

    fn append_leading(&mut self, decor: fmt::LineDecor) {
        if matches!(decor, fmt::LineDecor::Comment(_)) {
            self.width = fmt::Width::NotFlat;
        }
        self.leading.push(decor);
    }
    fn set_trailing(&mut self, comment: Option<fmt::Comment>) {
        if comment.is_some() {
            self.width = fmt::Width::NotFlat;
        }
        self.trailing = comment;
    }
}

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
    last_loc_end: usize,
}

impl FmtNodeBuilder<'_> {
    fn build_fmt_node(&mut self, node: prism::Node) -> fmt::Node {
        self.visit(node, Some(self.src.len()))
    }

    fn next_pos(&mut self) -> fmt::Pos {
        self.position_gen += 1;
        fmt::Pos(self.position_gen)
    }

    fn source_lossy_at(loc: &prism::Location) -> String {
        String::from_utf8_lossy(loc.as_slice()).to_string()
    }

    fn each_node_with_next_start(
        mut nodes: prism::NodeListIter,
        next_loc_start: Option<usize>,
        mut f: impl FnMut(prism::Node, Option<usize>),
    ) {
        if let Some(node) = nodes.next() {
            let mut prev = node;
            for next in nodes {
                let next_start = Some(next.location().start_offset());
                f(prev, next_start);
                prev = next;
            }
            f(prev, next_loc_start);
        }
    }

    // XXX: Probably `next_loc_start` should not be Option.
    fn visit(&mut self, node: prism::Node, next_loc_start: Option<usize>) -> fmt::Node {
        let loc_end = node.location().end_offset();
        let node = match node {
            prism::Node::ProgramNode { .. } => {
                let node = node.as_program_node().unwrap();
                let exprs = self.visit_statements(Some(node.statements()), next_loc_start);
                fmt::Node {
                    pos: fmt::Pos::none(),
                    width: exprs.width(),
                    kind: fmt::Kind::Exprs(exprs),
                }
            }
            prism::Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                let exprs = self.visit_statements(Some(node), next_loc_start);
                fmt::Node {
                    pos: fmt::Pos::none(),
                    width: exprs.width(),
                    kind: fmt::Kind::Exprs(exprs),
                }
            }

            prism::Node::NilNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::TrueNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::FalseNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::IntegerNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::FloatNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::RationalNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ImaginaryNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::InstanceVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ClassVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::GlobalVariableReadNode { .. } => self.parse_atom(node, next_loc_start),

            prism::Node::StringNode { .. } => {
                let node = node.as_string_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let mut decors = self.take_leading_decors(loc.start_offset());
                let mut fmt_node = if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let opening_len = self.visit_simple_heredoc(pos, node);
                    fmt::Node {
                        pos,
                        width: fmt::Width::Flat(opening_len),
                        kind: fmt::Kind::HeredocOpening,
                    }
                } else {
                    let (str, width) = self.visit_string(node);
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::Str(str),
                    }
                };
                if let Some(next_loc_start) = next_loc_start {
                    decors.set_trailing(self.take_trailing_comment(next_loc_start));
                }
                fmt_node.width.append(&decors.width);
                self.store_decors_to(pos, decors);
                fmt_node
            }
            prism::Node::InterpolatedStringNode { .. } => {
                let node = node.as_interpolated_string_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let mut decors = self.take_leading_decors(loc.start_offset());
                let mut fmt_node = if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let opening_len = self.visit_complex_heredoc(pos, node);
                    let width = fmt::Width::Flat(opening_len);
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::HeredocOpening,
                    }
                } else {
                    let (str, width) = self.visit_interpolated_string(node);
                    fmt::Node {
                        pos,
                        width,
                        kind: fmt::Kind::DynStr(str),
                    }
                };
                if let Some(next_loc_start) = next_loc_start {
                    decors.set_trailing(self.take_trailing_comment(next_loc_start));
                }
                fmt_node.width.append(&decors.width);
                self.store_decors_to(pos, decors);
                fmt_node
            }

            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();
                self.visit_if_or_unless(
                    IfOrUnless {
                        is_if: true,
                        loc: node.location(),
                        predicate: node.predicate(),
                        statements: node.statements(),
                        consequent: node.consequent(),
                        end_loc: node.end_keyword_loc(),
                    },
                    next_loc_start,
                )
            }
            prism::Node::UnlessNode { .. } => {
                let node = node.as_unless_node().unwrap();
                self.visit_if_or_unless(
                    IfOrUnless {
                        is_if: false,
                        loc: node.location(),
                        predicate: node.predicate(),
                        statements: node.statements(),
                        consequent: node.consequent().map(|n| n.as_node()),
                        end_loc: node.end_keyword_loc(),
                    },
                    next_loc_start,
                )
            }

            prism::Node::CallNode { .. } => {
                let node = node.as_call_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let mut decors = self.take_leading_decors(loc.start_offset());
                let chain = self.visit_call(node, None);
                if let Some(next_loc_start) = next_loc_start {
                    decors.set_trailing(self.take_trailing_comment(next_loc_start))
                }
                let whole_width = chain.body_width().add(&decors.width);
                self.store_decors_to(pos, decors);
                fmt::Node {
                    pos,
                    width: whole_width,
                    kind: fmt::Kind::MethodChain(chain),
                }
            }

            _ => todo!("parse {:?}", node),
        };

        // XXX: We should take trailing comment after setting the last location end.
        self.last_loc_end = loc_end;
        node
    }

    fn parse_atom(&mut self, node: prism::Node, next_loc_start: Option<usize>) -> fmt::Node {
        let pos = self.next_pos();
        let loc = node.location();
        let mut decors = self.take_leading_decors(loc.start_offset());
        let value = Self::source_lossy_at(&loc);
        let mut node = fmt::Node {
            pos,
            width: fmt::Width::Flat(value.len()),
            kind: fmt::Kind::Atom(value),
        };
        if let Some(next_loc_start) = next_loc_start {
            decors.set_trailing(self.take_trailing_comment(next_loc_start));
        }
        node.width.append(&decors.width);
        self.store_decors_to(pos, decors);
        node
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
                    self.last_loc_end = node.opening_loc().end_offset();

                    let exprs = self.visit_statements(node.statements(), Some(loc.end_offset()));
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    width.append_value(opening.len());
                    width.append_value(closing.len());
                    width.append(&exprs.width());
                    parts.push(fmt::DynStrPart::Exprs(fmt::EmbeddedExprs {
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
        width.append_value(opening.as_ref().map_or(0, |s| s.len()));
        width.append_value(closing.as_ref().map_or(0, |s| s.len()));
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

                    let exprs = self.visit_statements(node.statements(), Some(loc.end_offset()));
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    parts.push(fmt::HeredocPart::Exprs(fmt::EmbeddedExprs {
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
            Self::each_node_with_next_start(node.body().iter(), end, |prev, next_start| {
                let fmt_node = self.visit(prev, next_start);
                exprs.append_node(fmt_node);
            });
        }
        let virtual_end = self.take_end_decors_as_virtual_end(end);
        exprs.set_virtual_end(virtual_end);
        exprs
    }

    fn visit_if_or_unless(&mut self, node: IfOrUnless, next_loc_start: Option<usize>) -> fmt::Node {
        let pos = self.next_pos();
        let mut if_decors = self.take_leading_decors(node.loc.start_offset());

        let if_pos = self.next_pos();

        let end_loc = node.end_loc.expect("end must exist in root if/unless");
        let end_start = end_loc.start_offset();

        let if_next_loc = node.predicate.location().start_offset();
        let mut decors = Decors::new();
        decors.set_trailing(self.take_trailing_comment(if_next_loc));
        self.store_decors_to(if_pos, decors);

        let conseq = node.consequent;
        let next_pred_loc_start = node
            .statements
            .as_ref()
            .map(|s| s.location())
            .or_else(|| conseq.as_ref().map(|c| c.location()))
            .or(Some(end_loc))
            .map(|l| l.start_offset());
        let predicate = self.visit(node.predicate, next_pred_loc_start);

        let ifexpr = match conseq {
            // if...(elsif...|else...)+end
            Some(conseq) => {
                // take trailing of else/elsif
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
                let body = self.visit_statements(node.statements, Some(end_start));
                let if_first = fmt::Conditional {
                    pos: if_pos,
                    cond: Box::new(predicate),
                    body,
                };
                fmt::IfExpr::new(node.is_if, if_first)
            }
        };

        if let Some(next_loc_start) = next_loc_start {
            if_decors.set_trailing(self.take_trailing_comment(next_loc_start));
        }
        self.store_decors_to(pos, if_decors);

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

                let elsif_next_loc = node
                    .statements()
                    .as_ref()
                    .map(|s| s.location())
                    .or_else(|| node.end_keyword_loc())
                    .map(|l| l.start_offset());
                if let Some(elsif_next_loc) = elsif_next_loc {
                    let mut decors = Decors::new();
                    decors.set_trailing(self.take_trailing_comment(elsif_next_loc));
                    self.store_decors_to(elsif_pos, decors);
                }

                // XXX: We cannot find the case where the `end` keyword is None.
                let conseq = node.consequent();
                let next_loc_start = node
                    .statements()
                    .map(|s| s.location())
                    .or_else(|| conseq.as_ref().map(|c| c.location()))
                    .or_else(|| node.end_keyword_loc())
                    .map(|l| l.start_offset());
                let predicate = self.visit(node.predicate(), next_loc_start);

                let body_end_loc = conseq
                    .as_ref()
                    .map(|n| n.location().start_offset())
                    .or_else(|| node.end_keyword_loc().map(|l| l.start_offset()));
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

                let else_next_loc = node
                    .statements()
                    .as_ref()
                    .map(|s| s.location())
                    .or_else(|| node.end_keyword_loc())
                    .map(|l| l.start_offset());
                if let Some(else_next_loc) = else_next_loc {
                    let mut decors = Decors::new();
                    decors.set_trailing(self.take_trailing_comment(else_next_loc));
                    self.store_decors_to(else_pos, decors);
                }

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

    fn visit_call(
        &mut self,
        call: prism::CallNode,
        next_msg_start: Option<usize>,
    ) -> fmt::MethodChain {
        let mut chain = match call.receiver() {
            Some(receiver) => match receiver {
                prism::Node::CallNode { .. } => {
                    let node = receiver.as_call_node().unwrap();
                    let msg_end = call.message_loc().as_ref().map(|l| l.start_offset());
                    self.visit_call(node, msg_end)
                }
                _ => {
                    // XXX: message_loc || opening loc || args loc || block loc || next loc
                    let next_loc_start = call.message_loc().map(|l| l.start_offset());
                    let recv = self.visit(receiver, next_loc_start);
                    fmt::MethodChain::new(Some(recv))
                }
            },
            None => fmt::MethodChain::new(None),
        };

        let mut call_width = fmt::Width::Flat(0);

        let call_pos = self.next_pos();
        let mut decors = if let Some(msg_loc) = call.message_loc() {
            self.take_leading_decors(msg_loc.start_offset())
        } else {
            Decors::new()
            // foo.\n#hoge\n(2)
        };

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

        let args = match call.arguments() {
            None => {
                if let Some(closing_loc) = call.closing_loc().map(|l| l.start_offset()) {
                    let virtual_end = self.take_end_decors_as_virtual_end(Some(closing_loc));
                    virtual_end.map(|end| {
                        let mut args = fmt::Arguments::new();
                        args.set_virtual_end(Some(end));
                        args
                    })
                } else {
                    None
                }
            }
            Some(args_node) => {
                let mut args = fmt::Arguments::new();
                let closing_loc = call.closing_loc().as_ref().map(|l| l.start_offset());
                Self::each_node_with_next_start(
                    args_node.arguments().iter(),
                    closing_loc,
                    |prev, next_start| {
                        let fmt_node = self.visit(prev, next_start);
                        args.append_node(fmt_node);
                    },
                );

                let virtual_end = self.take_end_decors_as_virtual_end(closing_loc);
                args.set_virtual_end(virtual_end);

                Some(args)
            }
        };
        if let Some(args) = &args {
            // For now surround the arguments by parentheses always.
            call_width.append_value("()".len());
            call_width.append(&args.width());
        }

        let block = call.block().map(|node| match node {
            prism::Node::BlockNode { .. } => {
                let node = node.as_block_node().unwrap();
                let block_pos = self.next_pos();

                let block_next_loc = node
                    .body()
                    .map(|b| b.location())
                    .unwrap_or(node.closing_loc())
                    .start_offset();
                let mut decors = Decors::new();
                decors.set_trailing(self.take_trailing_comment(block_next_loc));
                call_width.append(&decors.width);
                self.store_decors_to(block_pos, decors);

                let body_end_loc = node.closing_loc().start_offset();
                let body = node.body().map(|n| self.visit(n, Some(body_end_loc)));
                // XXX: Is this necessary? I cannot find the case where the body is not a StatementNode.
                let body = self.wrap_as_exprs(body, Some(body_end_loc));

                let loc = node.location();
                let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());

                if was_flat {
                    call_width.append_value(" {  }".len());
                    call_width.append(&body.width());
                } else {
                    call_width.append(&fmt::Width::NotFlat);
                }

                fmt::MethodBlock {
                    pos: block_pos,
                    body,
                    was_flat,
                }
            }
            _ => panic!("unexpected node for call block: {:?}", node),
        });

        if let Some(next_msg_start) = next_msg_start {
            decors.set_trailing(self.take_trailing_comment(next_msg_start));
        }
        call_width.append(&decors.width);
        self.store_decors_to(call_pos, decors);

        chain.append_call(fmt::MethodCall {
            pos: call_pos,
            width: call_width,
            chain_type,
            name,
            args,
            block,
        });

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
        let virtual_end = self.take_end_decors_as_virtual_end(end);
        exprs.set_virtual_end(virtual_end);
        exprs
    }

    fn take_end_decors_as_virtual_end(&mut self, end: Option<usize>) -> Option<fmt::VirtualEnd> {
        if let Some(end) = end {
            let decors = self.take_leading_decors(end);
            if !decors.leading.is_empty() {
                let end_pos = self.next_pos();
                let width = decors.width;
                self.store_decors_to(end_pos, decors);
                return Some(fmt::VirtualEnd {
                    pos: end_pos,
                    width,
                });
            }
        }
        None
    }

    fn store_decors_to(&mut self, pos: fmt::Pos, decors: Decors) {
        if !decors.leading.is_empty() {
            self.decor_store.append_leading_decors(pos, decors.leading);
        }
        if let Some(comment) = decors.trailing {
            self.decor_store.set_trailing_comment(pos, comment);
        }
    }

    fn take_leading_decors(&mut self, loc_start: usize) -> Decors {
        let mut decors = Decors::new();

        while let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if !(self.last_loc_end..=loc_start).contains(&loc.start_offset()) {
                break;
            };
            // We treat the found comment as line comment always.
            let value = Self::source_lossy_at(&loc);
            let fmt_comment = fmt::Comment { value };
            self.take_empty_lines_until(loc.start_offset(), &mut decors);
            decors.append_leading(fmt::LineDecor::Comment(fmt_comment));
            self.last_loc_end = loc.end_offset() - 1;
            self.comments.next();
        }

        self.take_empty_lines_until(loc_start, &mut decors);
        decors
    }

    fn take_empty_lines_until(&mut self, end: usize, decors: &mut Decors) {
        let range = self.last_empty_line_range_within(self.last_loc_end, end);
        if let Some(range) = range {
            decors.append_leading(fmt::LineDecor::EmptyLine);
            self.last_loc_end = range.end;
        }
    }

    fn take_trailing_comment(&mut self, next_loc_start: usize) -> Option<fmt::Comment> {
        if let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if (self.last_loc_end..=next_loc_start).contains(&loc.start_offset())
                && !self.is_at_line_start(loc.start_offset())
            {
                self.last_loc_end = loc.end_offset() - 1;
                self.comments.next();
                let value = Self::source_lossy_at(&loc);
                return Some(fmt::Comment { value });
            }
        }
        None
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

    fn does_line_break_exist_in(&self, start: usize, end: usize) -> bool {
        let end = end.min(self.src.len());
        for i in start..end {
            if self.src[i] == b'\n' {
                return true;
            }
        }
        false
    }
}
