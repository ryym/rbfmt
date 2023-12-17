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
                let exprs = self.visit_statements(Some(node.statements()), Some(self.src.len()));
                fmt::Node::new(pos, fmt::Kind::Exprs(exprs))
            }
            Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                let pos = self.next_pos();
                let exprs = self.visit_statements(Some(node), None);
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

            Node::StringNode { .. } => {
                let node = node.as_string_node().unwrap();
                let pos = self.next_pos();
                self.consume_and_store_decors_until(pos, node.location().start_offset());
                let str = self.visit_string(node);
                fmt::Node::new(pos, fmt::Kind::Str(str))
            }

            Node::IfNode { .. } => {
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
            Node::UnlessNode { .. } => {
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

            Node::CallNode { .. } => {
                let node = node.as_call_node().unwrap();
                let pos = self.next_pos();
                self.consume_and_store_decors_until(pos, node.location().start_offset());
                let chain = self.visit_call(node);
                fmt::Node::new(pos, fmt::Kind::MethodChain(chain))
            }

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

    fn visit_string(&mut self, node: prism::StringNode) -> fmt::Str {
        let value = Self::source_lossy_at(&node.content_loc());
        let open = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let close = node.closing_loc().as_ref().map(Self::source_lossy_at);
        fmt::Str {
            begin: open,
            value: value.into(),
            end: close,
        }
    }

    fn visit_statements(
        &mut self,
        node: Option<prism::StatementsNode>,
        end: Option<usize>,
    ) -> fmt::Exprs {
        let mut nodes = vec![];
        if let Some(node) = node {
            for n in node.body().iter() {
                nodes.push(self.visit(n));
            }
        }
        let mut exprs = fmt::Exprs(nodes);
        if let Some(end) = end {
            self.append_end_decors(&mut exprs, end);
        }
        exprs
    }

    fn visit_if_or_unless(&mut self, node: IfOrUnless) -> fmt::Node {
        let pos = self.next_pos();

        // Consume decors above the if expression.
        self.consume_and_store_decors_until(pos, node.loc.start_offset());

        // XXX: If we can move the responsibility of this decors merging to fmt.rs,
        // visit_if and the logic inside of visit_ifelse could be unified.
        //
        // Consume decors between "if" and the condition expression.
        // Then merge it to the decors of "if" itself.
        let predicate_start = node.predicate.location().start_offset();
        let decors_in_if_and_cond = self.consume_decors_until(predicate_start);
        if let Some((if_trailing, cond_leading)) = decors_in_if_and_cond {
            if let Some(c) = if_trailing {
                self.decor_store
                    .append_leading_decors(pos, vec![fmt::LineDecor::Comment(c)]);
            }
            if !cond_leading.is_empty() {
                self.decor_store.append_leading_decors(pos, cond_leading);
            }
        }

        let predicate = self.visit(node.predicate);
        let end_loc = node.end_loc.expect("end must exist in root if/unless");

        let ifexpr = match node.consequent {
            // if...(elsif...|else...)+end
            Some(conseq) => {
                let else_start = conseq.location().start_offset();
                let body = self.visit_statements(node.statements, Some(else_start));
                let if_part = fmt::IfPart::new(predicate, body);
                let mut ifexpr = fmt::IfExpr::new(!node.is_if, if_part);
                self.visit_ifelse(conseq, &mut ifexpr);
                ifexpr.end_pos = self.next_pos();
                self.consume_and_store_decors_until(ifexpr.end_pos, end_loc.start_offset());
                ifexpr
            }
            // if...end
            None => {
                let body = self.visit_statements(node.statements, Some(end_loc.start_offset()));
                let if_part = fmt::IfPart::new(predicate, body);
                fmt::IfExpr::new(!node.is_if, if_part)
            }
        };

        fmt::Node::new(pos, fmt::Kind::IfExpr(ifexpr))
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

                let body_end_loc = conseq.as_ref().map(|n| n.location().start_offset());
                let body = self.visit_statements(node.statements(), body_end_loc);

                ifexpr.elsifs.push(fmt::Elsif {
                    pos: elsif_pos,
                    part: fmt::IfPart::new(predicate, body),
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

                // XXX: ElseNode has "end" so we can use it.
                let body = self.visit_statements(node.statements(), None);
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
                    fmt::MethodChain {
                        receiver: Some(Box::new(recv)),
                        calls: vec![],
                    }
                }
            },
            None => fmt::MethodChain {
                receiver: None,
                calls: vec![],
            },
        };

        let call_pos = self.next_pos();
        if let Some(msg_loc) = call.message_loc() {
            self.consume_and_store_decors_until(call_pos, msg_loc.start_offset());
        } else {
            // foo.\n#hoge\n(2)
        }

        let args = if let Some(args) = call.arguments() {
            args.arguments().iter().map(|n| self.visit(n)).collect()
        } else {
            vec![]
        };
        let name = String::from_utf8_lossy(call.name().as_slice()).to_string();

        // XXX: We can just use call.call_operator_loc()
        let chain_type = if call.is_safe_navigation() {
            fmt::ChainType::SafeNav
        } else {
            fmt::ChainType::Normal
        };

        let block = call.block().map(|node| match node {
            prism::Node::BlockNode { .. } => {
                let node = node.as_block_node().unwrap();
                let block_pos = self.next_pos();
                self.last_pos = block_pos;
                let body = node.body().map(|n| self.visit(n));
                let body_end_loc = node.closing_loc().start_offset();
                let body = self.wrap_as_exprs(body, Some(body_end_loc));
                fmt::MethodBlock {
                    pos: block_pos,
                    body,
                }
            }
            _ => panic!("unexpected node for call block: {:?}", node),
        });

        chain.calls.push(fmt::MethodCall {
            pos: call_pos,
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
        let expr_nodes = match node {
            None => vec![],
            Some(node) => match node.kind {
                fmt::Kind::Exprs(fmt::Exprs(nodes)) => nodes,
                _ => vec![node],
            },
        };
        let mut exprs = fmt::Exprs(expr_nodes);
        if let Some(end) = end {
            self.append_end_decors(&mut exprs, end);
        }
        exprs
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
