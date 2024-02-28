mod atoms;
mod calls;
mod cases;
mod ifs;
mod loops;
mod postmodifiers;
mod regexps;
mod src;
mod strings;
mod symbols;

use crate::fmt;
use std::{collections::HashMap, iter::Peekable, ops::Range};

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<ParserResult> {
    let result = prism::parse(&source);

    let comments = result.comments().peekable();
    let heredoc_map = HashMap::new();

    let mut parser = Parser {
        src: &source,
        comments,
        heredoc_map,
        position_gen: 0,
        last_loc_end: 0,
    };
    let fmt_node = parser.parse_from_prism_node(result.node());
    // dbg!(&fmt_node);
    // dbg!(&builder.heredoc_map);
    Some(ParserResult {
        node: fmt_node,
        heredoc_map: parser.heredoc_map,
    })
}

#[derive(Debug)]
pub(crate) struct ParserResult {
    pub node: fmt::Node,
    pub heredoc_map: fmt::HeredocMap,
}

struct Parser<'src> {
    src: &'src [u8],
    comments: Peekable<prism::Comments<'src>>,
    heredoc_map: fmt::HeredocMap,
    position_gen: usize,
    last_loc_end: usize,
}

impl Parser<'_> {
    fn parse_from_prism_node(&mut self, node: prism::Node) -> fmt::Node {
        self.visit(node, Some(self.src.len()))
    }

    fn next_pos(&mut self) -> fmt::Pos {
        self.position_gen += 1;
        fmt::Pos(self.position_gen)
    }

    fn source_lossy_at(loc: &prism::Location) -> String {
        String::from_utf8_lossy(loc.as_slice()).to_string()
    }

    fn each_node_with_trailing_end<'a>(
        mut nodes: impl Iterator<Item = prism::Node<'a>>,
        last_trailing_end: Option<usize>,
        mut f: impl FnMut(prism::Node<'a>, Option<usize>),
    ) {
        if let Some(node) = nodes.next() {
            let mut prev = node;
            for next in nodes {
                let trailing_end = next.location().start_offset();
                f(prev, Some(trailing_end));
                prev = next;
            }
            f(prev, last_trailing_end);
        }
    }

    fn visit(&mut self, node: prism::Node, trailing_end: Option<usize>) -> fmt::Node {
        let loc = node.location();
        let loc_end = loc.end_offset();

        let leading = match node {
            prism::Node::ProgramNode { .. } | prism::Node::StatementsNode { .. } => {
                fmt::LeadingTrivia::new()
            }
            _ => self.take_leading_trivia(loc.start_offset()),
        };
        let mut node = self.parse_node(node, trailing_end);
        node.prepend_leading_trivia(leading);

        self.last_loc_end = loc_end;

        if let Some(trailing_end) = trailing_end {
            let trailing = self.take_trailing_comment(trailing_end);
            node.set_trailing_trivia(trailing);
        }

        node
    }

    fn parse_node(&mut self, node: prism::Node, trailing_end: Option<usize>) -> fmt::Node {
        match node {
            prism::Node::ProgramNode { .. } => {
                let node = node.as_program_node().unwrap();
                let statements = self.visit_statements(Some(node.statements()), trailing_end);
                let kind = fmt::Kind::Statements(statements);
                fmt::Node::new(kind)
            }
            prism::Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                let statements = self.visit_statements(Some(node), trailing_end);
                let kind = fmt::Kind::Statements(statements);
                fmt::Node::new(kind)
            }

            prism::Node::SelfNode { .. } => self.parse_as_atom(node),
            prism::Node::NilNode { .. } => self.parse_as_atom(node),
            prism::Node::TrueNode { .. } => self.parse_as_atom(node),
            prism::Node::FalseNode { .. } => self.parse_as_atom(node),
            prism::Node::IntegerNode { .. } => self.parse_as_atom(node),
            prism::Node::FloatNode { .. } => self.parse_as_atom(node),
            prism::Node::RationalNode { .. } => self.parse_as_atom(node),
            prism::Node::ImaginaryNode { .. } => self.parse_as_atom(node),
            prism::Node::LocalVariableReadNode { .. } => self.parse_as_atom(node),
            prism::Node::InstanceVariableReadNode { .. } => self.parse_as_atom(node),
            prism::Node::ClassVariableReadNode { .. } => self.parse_as_atom(node),
            prism::Node::GlobalVariableReadNode { .. } => self.parse_as_atom(node),
            prism::Node::BackReferenceReadNode { .. } => self.parse_as_atom(node),
            prism::Node::NumberedReferenceReadNode { .. } => self.parse_as_atom(node),
            prism::Node::ConstantReadNode { .. } => self.parse_as_atom(node),
            prism::Node::BlockLocalVariableNode { .. } => self.parse_as_atom(node),
            prism::Node::ForwardingArgumentsNode { .. } => self.parse_as_atom(node),
            prism::Node::RedoNode { .. } => self.parse_as_atom(node),
            prism::Node::RetryNode { .. } => self.parse_as_atom(node),
            prism::Node::SourceFileNode { .. } => self.parse_as_atom(node),
            prism::Node::SourceLineNode { .. } => self.parse_as_atom(node),
            prism::Node::SourceEncodingNode { .. } => self.parse_as_atom(node),

            prism::Node::ConstantPathNode { .. } => {
                let node = node.as_constant_path_node().unwrap();
                let const_path = self.visit_constant_path(node.parent(), node.child());
                fmt::Node::new(fmt::Kind::ConstantPath(const_path))
            }

            prism::Node::StringNode { .. } => {
                let node = node.as_string_node().unwrap();
                self.parse_string_or_heredoc(
                    node.opening_loc(),
                    node.content_loc(),
                    node.closing_loc(),
                )
            }
            prism::Node::InterpolatedStringNode { .. } => {
                let node = node.as_interpolated_string_node().unwrap();
                self.parse_interpolated_string_or_heredoc(
                    node.opening_loc(),
                    node.parts(),
                    node.closing_loc(),
                )
            }

            prism::Node::XStringNode { .. } => {
                let node = node.as_x_string_node().unwrap();
                self.parse_string_or_heredoc(
                    Some(node.opening_loc()),
                    node.content_loc(),
                    Some(node.closing_loc()),
                )
            }
            prism::Node::InterpolatedXStringNode { .. } => {
                let node = node.as_interpolated_x_string_node().unwrap();
                self.parse_interpolated_string_or_heredoc(
                    Some(node.opening_loc()),
                    node.parts(),
                    Some(node.closing_loc()),
                )
            }

            prism::Node::SymbolNode { .. } => {
                let node = node.as_symbol_node().unwrap();
                self.parse_symbol(node)
            }
            prism::Node::InterpolatedSymbolNode { .. } => {
                let node = node.as_interpolated_symbol_node().unwrap();
                self.parse_interpolated_symbol(node)
            }

            prism::Node::RegularExpressionNode { .. } => {
                let node = node.as_regular_expression_node().unwrap();
                self.parse_regexp(node)
            }
            prism::Node::InterpolatedRegularExpressionNode { .. } => {
                let node = node.as_interpolated_regular_expression_node().unwrap();
                self.parse_interpolated_regexp(node)
            }
            prism::Node::MatchLastLineNode { .. } => {
                let node = node.as_match_last_line_node().unwrap();
                self.parse_match_last_line(node)
            }
            prism::Node::InterpolatedMatchLastLineNode { .. } => {
                let node = node.as_interpolated_match_last_line_node().unwrap();
                self.parse_interpolated_match_last_line(node)
            }
            prism::Node::MatchWriteNode { .. } => {
                let node = node.as_match_write_node().unwrap();
                self.visit(node.call().as_node(), None)
            }

            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();
                self.parse_if_or_ternary(node)
            }
            prism::Node::UnlessNode { .. } => {
                let node = node.as_unless_node().unwrap();
                self.parse_unless(node)
            }

            prism::Node::CaseNode { .. } => {
                let node = node.as_case_node().unwrap();
                self.parse_case(node)
            }

            prism::Node::WhileNode { .. } => {
                let node = node.as_while_node().unwrap();
                self.parse_while(node)
            }
            prism::Node::UntilNode { .. } => {
                let node = node.as_until_node().unwrap();
                self.parse_until(node)
            }

            prism::Node::ForNode { .. } => {
                let node = node.as_for_node().unwrap();
                self.parse_for(node)
            }

            prism::Node::RescueModifierNode { .. } => {
                let node = node.as_rescue_modifier_node().unwrap();
                self.parse_rescue_modifier(node)
            }

            prism::Node::CallNode { .. } => {
                let node = node.as_call_node().unwrap();
                self.parse_call(node)
            }
            prism::Node::ForwardingSuperNode { .. } => {
                let node = node.as_forwarding_super_node().unwrap();
                let chain = self.parse_call_root(&node);
                fmt::Node::new(fmt::Kind::MethodChain(chain))
            }
            prism::Node::SuperNode { .. } => {
                let node = node.as_super_node().unwrap();
                let chain = self.parse_call_root(&node);
                fmt::Node::new(fmt::Kind::MethodChain(chain))
            }
            prism::Node::YieldNode { .. } => {
                let node = node.as_yield_node().unwrap();
                let call_like = self.visit_yield(node);
                fmt::Node::new(fmt::Kind::CallLike(call_like))
            }

            prism::Node::BreakNode { .. } => {
                let node = node.as_break_node().unwrap();
                let call_like = self.parse_call_like(node.keyword_loc(), node.arguments());
                fmt::Node::new(fmt::Kind::CallLike(call_like))
            }
            prism::Node::NextNode { .. } => {
                let node = node.as_next_node().unwrap();
                let call_like = self.parse_call_like(node.keyword_loc(), node.arguments());
                fmt::Node::new(fmt::Kind::CallLike(call_like))
            }
            prism::Node::ReturnNode { .. } => {
                let node = node.as_return_node().unwrap();
                let call_like = self.parse_call_like(node.keyword_loc(), node.arguments());
                fmt::Node::new(fmt::Kind::CallLike(call_like))
            }

            prism::Node::AndNode { .. } => {
                let node = node.as_and_node().unwrap();
                let chain = self.visit_infix_op(node.left(), node.operator_loc(), node.right());
                fmt::Node::new(fmt::Kind::InfixChain(chain))
            }
            prism::Node::OrNode { .. } => {
                let node = node.as_or_node().unwrap();
                let chain = self.visit_infix_op(node.left(), node.operator_loc(), node.right());
                fmt::Node::new(fmt::Kind::InfixChain(chain))
            }

            prism::Node::LocalVariableWriteNode { .. } => {
                let node = node.as_local_variable_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::LocalVariableAndWriteNode { .. } => {
                let node = node.as_local_variable_and_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::LocalVariableOrWriteNode { .. } => {
                let node = node.as_local_variable_or_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::LocalVariableOperatorWriteNode { .. } => {
                let node = node.as_local_variable_operator_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::InstanceVariableWriteNode { .. } => {
                let node = node.as_instance_variable_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::InstanceVariableAndWriteNode { .. } => {
                let node = node.as_instance_variable_and_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::InstanceVariableOrWriteNode { .. } => {
                let node = node.as_instance_variable_or_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::InstanceVariableOperatorWriteNode { .. } => {
                let node = node.as_instance_variable_operator_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::ClassVariableWriteNode { .. } => {
                let node = node.as_class_variable_write_node().unwrap();
                let assign = self.visit_variable_assign(
                    node.name_loc(),
                    // XXX: When does the operator becomes None?
                    node.operator_loc().expect("must have operator"),
                    node.value(),
                );
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ClassVariableAndWriteNode { .. } => {
                let node = node.as_class_variable_and_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ClassVariableOrWriteNode { .. } => {
                let node = node.as_class_variable_or_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ClassVariableOperatorWriteNode { .. } => {
                let node = node.as_class_variable_operator_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::GlobalVariableWriteNode { .. } => {
                let node = node.as_global_variable_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::GlobalVariableAndWriteNode { .. } => {
                let node = node.as_global_variable_and_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::GlobalVariableOrWriteNode { .. } => {
                let node = node.as_global_variable_or_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::GlobalVariableOperatorWriteNode { .. } => {
                let node = node.as_global_variable_operator_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::ConstantWriteNode { .. } => {
                let node = node.as_constant_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantAndWriteNode { .. } => {
                let node = node.as_constant_and_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantOrWriteNode { .. } => {
                let node = node.as_constant_or_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantOperatorWriteNode { .. } => {
                let node = node.as_constant_operator_write_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::ConstantPathWriteNode { .. } => {
                let node = node.as_constant_path_write_node().unwrap();
                let assign = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                );
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantPathAndWriteNode { .. } => {
                let node = node.as_constant_path_and_write_node().unwrap();
                let assign = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                );
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantPathOrWriteNode { .. } => {
                let node = node.as_constant_path_or_write_node().unwrap();
                let assign = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                );
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantPathOperatorWriteNode { .. } => {
                let node = node.as_constant_path_operator_write_node().unwrap();
                let assign = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                );
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::CallAndWriteNode { .. } => {
                let node = node.as_call_and_write_node().unwrap();
                let assign = self.visit_call_assign(&node, node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::CallOrWriteNode { .. } => {
                let node = node.as_call_or_write_node().unwrap();
                let assign = self.visit_call_assign(&node, node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::CallOperatorWriteNode { .. } => {
                let node = node.as_call_operator_write_node().unwrap();
                let assign = self.visit_call_assign(&node, node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::IndexAndWriteNode { .. } => {
                let node = node.as_index_and_write_node().unwrap();
                let assign = self.visit_call_assign(&node, node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::IndexOrWriteNode { .. } => {
                let node = node.as_index_or_write_node().unwrap();
                let assign = self.visit_call_assign(&node, node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::IndexOperatorWriteNode { .. } => {
                let node = node.as_index_operator_write_node().unwrap();
                let assign = self.visit_call_assign(&node, node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }

            prism::Node::LocalVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::InstanceVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::ClassVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::GlobalVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::ConstantTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::ConstantPathTargetNode { .. } => {
                let node = node.as_constant_path_target_node().unwrap();
                let const_path = self.visit_constant_path(node.parent(), node.child());
                fmt::Node::new(fmt::Kind::ConstantPath(const_path))
            }
            prism::Node::CallTargetNode { .. } => {
                let node = node.as_call_target_node().unwrap();
                let chain = self.parse_call_root(&node);
                fmt::Node::new(fmt::Kind::MethodChain(chain))
            }
            prism::Node::IndexTargetNode { .. } => {
                let node = node.as_index_target_node().unwrap();
                let chain = self.parse_call_root(&node);
                fmt::Node::new(fmt::Kind::MethodChain(chain))
            }

            prism::Node::MultiWriteNode { .. } => {
                let node = node.as_multi_write_node().unwrap();
                let assign = self.visit_multi_assign(node);
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::MultiTargetNode { .. } => {
                let node = node.as_multi_target_node().unwrap();
                let target = self.visit_multi_assign_target(
                    node.lefts(),
                    node.rest(),
                    node.rights(),
                    node.lparen_loc(),
                    node.rparen_loc(),
                );
                fmt::Node::new(fmt::Kind::MultiAssignTarget(target))
            }
            prism::Node::ImplicitRestNode { .. } => {
                let atom = fmt::Atom("".to_string());
                fmt::Node::new(fmt::Kind::Atom(atom))
            }

            prism::Node::SplatNode { .. } => {
                let node = node.as_splat_node().unwrap();
                let operator = Self::source_lossy_at(&node.operator_loc());
                let expr = node.expression().map(|expr| self.visit(expr, None));
                let splat = fmt::Prefix::new(operator, expr);
                fmt::Node::new(fmt::Kind::Prefix(splat))
            }
            prism::Node::AssocSplatNode { .. } => {
                let node = node.as_assoc_splat_node().unwrap();
                let operator = Self::source_lossy_at(&node.operator_loc());
                let value = node.value().map(|v| self.visit(v, None));
                let splat = fmt::Prefix::new(operator, value);
                fmt::Node::new(fmt::Kind::Prefix(splat))
            }
            prism::Node::BlockArgumentNode { .. } => {
                let node = node.as_block_argument_node().unwrap();
                let prefix = self.visit_block_arg(node);
                fmt::Node::new(fmt::Kind::Prefix(prefix))
            }

            prism::Node::ArrayNode { .. } => {
                let node = node.as_array_node().unwrap();
                let opening_loc = node.opening_loc();
                let closing_loc = node.closing_loc();
                let opening = opening_loc.as_ref().map(Self::source_lossy_at);
                let closing = closing_loc.as_ref().map(Self::source_lossy_at);
                let mut array = fmt::Array::new(opening, closing);
                let closing_start = closing_loc.map(|l| l.start_offset());
                Self::each_node_with_trailing_end(
                    node.elements().iter(),
                    closing_start,
                    |node, trailing_end| match node {
                        prism::Node::KeywordHashNode { .. } => {
                            let node = node.as_keyword_hash_node().unwrap();
                            self.each_keyword_hash_element(node, trailing_end, |element| {
                                array.append_element(element);
                            });
                        }
                        _ => {
                            let element = self.visit(node, trailing_end);
                            array.append_element(element);
                        }
                    },
                );
                if let Some(closing_start) = closing_start {
                    let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
                    array.set_virtual_end(virtual_end);
                }
                fmt::Node::new(fmt::Kind::Array(array))
            }

            prism::Node::HashNode { .. } => {
                let node = node.as_hash_node().unwrap();
                let opening_loc = node.opening_loc();
                let closing_loc = node.closing_loc();
                let opening = Self::source_lossy_at(&opening_loc);
                let closing = Self::source_lossy_at(&closing_loc);
                let should_be_inline = if let Some(first_element) = node.elements().iter().next() {
                    !self.does_line_break_exist_in(
                        opening_loc.start_offset(),
                        first_element.location().start_offset(),
                    )
                } else {
                    true
                };
                let mut hash = fmt::Hash::new(opening, closing, should_be_inline);
                let closing_start = closing_loc.start_offset();
                Self::each_node_with_trailing_end(
                    node.elements().iter(),
                    Some(closing_start),
                    |node, trailing_end| {
                        let element = self.visit(node, trailing_end);
                        hash.append_element(element);
                    },
                );
                let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
                hash.set_virtual_end(virtual_end);
                fmt::Node::new(fmt::Kind::Hash(hash))
            }
            prism::Node::AssocNode { .. } => {
                let node = node.as_assoc_node().unwrap();
                let key = node.key();
                let key = self.visit(key, None);
                let operator = node.operator_loc().map(|l| Self::source_lossy_at(&l));
                let value = self.visit(node.value(), None);
                let assoc = fmt::Assoc::new(key, operator, value);
                fmt::Node::new(fmt::Kind::Assoc(assoc))
            }
            prism::Node::ImplicitNode { .. } => {
                fmt::Node::new(fmt::Kind::Atom(fmt::Atom("".to_string())))
            }

            prism::Node::ParenthesesNode { .. } => {
                let node = node.as_parentheses_node().unwrap();
                let closing_start = node.closing_loc().start_offset();
                let body = node.body().map(|b| self.visit(b, Some(closing_start)));
                let body = self.wrap_as_statements(body, closing_start);
                let parens = fmt::Parens::new(body);
                fmt::Node::new(fmt::Kind::Parens(parens))
            }

            prism::Node::DefNode { .. } => {
                let node = node.as_def_node().unwrap();
                let (leading, def) = self.visit_def(node);
                fmt::Node::with_leading_trivia(leading, fmt::Kind::Def(def))
            }
            prism::Node::NoKeywordsParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::ForwardingParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::RequiredParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::RequiredKeywordParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::RestParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::KeywordRestParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::BlockParameterNode { .. } => self.parse_as_atom(node),
            prism::Node::OptionalParameterNode { .. } => {
                let node = node.as_optional_parameter_node().unwrap();
                let assign =
                    self.visit_variable_assign(node.name_loc(), node.operator_loc(), node.value());
                fmt::Node::new(fmt::Kind::Assign(assign))
            }
            prism::Node::OptionalKeywordParameterNode { .. } => {
                let node = node.as_optional_keyword_parameter_node().unwrap();
                let name = Self::source_lossy_at(&node.name_loc());
                let name = fmt::Node::new(fmt::Kind::Atom(fmt::Atom(name)));
                let value = node.value();
                let value = self.visit(value, None);
                let assoc = fmt::Assoc::new(name, None, value);
                fmt::Node::new(fmt::Kind::Assoc(assoc))
            }

            prism::Node::LambdaNode { .. } => {
                let node = node.as_lambda_node().unwrap();
                let lambda = self.visit_lambda(node);
                fmt::Node::new(fmt::Kind::Lambda(lambda))
            }

            prism::Node::UndefNode { .. } => {
                let node = node.as_undef_node().unwrap();
                let call_like = self.visit_undef(node);
                fmt::Node::new(fmt::Kind::CallLike(call_like))
            }
            prism::Node::DefinedNode { .. } => {
                let node = node.as_defined_node().unwrap();
                let call_like = self.visit_defined(node);
                fmt::Node::new(fmt::Kind::CallLike(call_like))
            }

            prism::Node::BeginNode { .. } => {
                let node = node.as_begin_node().unwrap();
                let end_loc = node.end_keyword_loc().expect("begin must have end");
                let keyword_next = node
                    .statements()
                    .map(|n| n.location().start_offset())
                    .or_else(|| node.rescue_clause().map(|n| n.location().start_offset()))
                    .or_else(|| node.else_clause().map(|n| n.location().start_offset()))
                    .or_else(|| node.ensure_clause().map(|n| n.location().start_offset()))
                    .unwrap_or(end_loc.start_offset());
                let keyword_trailing = self.take_trailing_comment(keyword_next);
                let body = self.visit_begin_body(node);
                let begin = fmt::Begin {
                    keyword_trailing,
                    body,
                };
                fmt::Node::new(fmt::Kind::Begin(begin))
            }

            prism::Node::ClassNode { .. } => {
                let node = node.as_class_node().unwrap();
                let (leading, class) = self.visit_class_like(
                    "class",
                    node.constant_path().location(),
                    node.superclass(),
                    node.body(),
                    node.end_keyword_loc(),
                );
                fmt::Node::with_leading_trivia(leading, fmt::Kind::ClassLike(class))
            }
            prism::Node::ModuleNode { .. } => {
                let node = node.as_module_node().unwrap();
                let (leading, module) = self.visit_class_like(
                    "module",
                    node.constant_path().location(),
                    None,
                    node.body(),
                    node.end_keyword_loc(),
                );
                fmt::Node::with_leading_trivia(leading, fmt::Kind::ClassLike(module))
            }
            prism::Node::SingletonClassNode { .. } => {
                let node = node.as_singleton_class_node().unwrap();
                let leading = self.take_leading_trivia(node.operator_loc().start_offset());
                let body = node.body();
                let end_loc = node.end_keyword_loc();
                let body_start = body.as_ref().and_then(|b| match b {
                    prism::Node::BeginNode { .. } => {
                        Self::start_of_begin_block_content(b.as_begin_node().unwrap())
                    }
                    _ => Some(b.location().start_offset()),
                });
                let expr_next = body_start.unwrap_or(end_loc.start_offset());
                let expr = self.visit(node.expression(), Some(expr_next));
                let body = self.parse_block_body(body, end_loc.start_offset());
                let class = fmt::SingletonClass {
                    expression: Box::new(expr),
                    body,
                };
                fmt::Node::with_leading_trivia(leading, fmt::Kind::SingletonClass(class))
            }

            prism::Node::RangeNode { .. } => {
                let node = node.as_range_node().unwrap();
                let op_loc = node.operator_loc();
                let op_start = op_loc.start_offset();
                let left = node.left().map(|n| self.visit(n, Some(op_start)));
                let operator = Self::source_lossy_at(&op_loc);
                let right = node.right().map(|n| self.visit(n, None));
                let range = fmt::RangeLike::new(left, operator, right);
                fmt::Node::new(fmt::Kind::RangeLike(range))
            }
            prism::Node::FlipFlopNode { .. } => {
                let node = node.as_flip_flop_node().unwrap();
                let op_loc = node.operator_loc();
                let op_start = op_loc.start_offset();
                let left = node.left().map(|n| self.visit(n, Some(op_start)));
                let operator = Self::source_lossy_at(&op_loc);
                let right = node.right().map(|n| self.visit(n, None));
                let flipflop = fmt::RangeLike::new(left, operator, right);
                fmt::Node::new(fmt::Kind::RangeLike(flipflop))
            }

            prism::Node::CaseMatchNode { .. } => {
                let node = node.as_case_match_node().unwrap();
                let case = self.visit_case_match(node);
                fmt::Node::new(fmt::Kind::CaseMatch(case))
            }
            prism::Node::MatchPredicateNode { .. } => {
                let node = node.as_match_predicate_node().unwrap();
                let match_assign =
                    self.visit_match_assign(node.value(), node.operator_loc(), node.pattern());
                fmt::Node::new(fmt::Kind::MatchAssign(match_assign))
            }
            prism::Node::MatchRequiredNode { .. } => {
                let node = node.as_match_required_node().unwrap();
                let match_assign =
                    self.visit_match_assign(node.value(), node.operator_loc(), node.pattern());
                fmt::Node::new(fmt::Kind::MatchAssign(match_assign))
            }

            prism::Node::ArrayPatternNode { .. } => {
                let node = node.as_array_pattern_node().unwrap();
                let array_pattern = self.visit_array_pattern(node);
                fmt::Node::new(fmt::Kind::ArrayPattern(array_pattern))
            }
            prism::Node::FindPatternNode { .. } => {
                let node = node.as_find_pattern_node().unwrap();
                let array_pattern = self.visit_find_pattern(node);
                fmt::Node::new(fmt::Kind::ArrayPattern(array_pattern))
            }
            prism::Node::HashPatternNode { .. } => {
                let node = node.as_hash_pattern_node().unwrap();
                let hash_pattern = self.visit_hash_pattern(node);
                fmt::Node::new(fmt::Kind::HashPattern(hash_pattern))
            }
            prism::Node::PinnedExpressionNode { .. } => {
                let node = node.as_pinned_expression_node().unwrap();
                let prefix = self.visit_pinned_expression(node);
                fmt::Node::new(fmt::Kind::Prefix(prefix))
            }
            prism::Node::PinnedVariableNode { .. } => {
                let node = node.as_pinned_variable_node().unwrap();
                let prefix = self.visit_pinned_variable(node);
                fmt::Node::new(fmt::Kind::Prefix(prefix))
            }
            prism::Node::CapturePatternNode { .. } => {
                let node = node.as_capture_pattern_node().unwrap();
                let assoc = self.visit_capture_pattern(node);
                fmt::Node::new(fmt::Kind::Assoc(assoc))
            }
            prism::Node::AlternationPatternNode { .. } => {
                let node = node.as_alternation_pattern_node().unwrap();
                let chain = self.visit_alternation_pattern(node);
                fmt::Node::new(fmt::Kind::AltPatternChain(chain))
            }

            prism::Node::PreExecutionNode { .. } => {
                let node = node.as_pre_execution_node().unwrap();
                let exec = self.visit_pre_post_exec(
                    node.keyword_loc(),
                    node.opening_loc(),
                    node.statements(),
                    node.closing_loc(),
                );
                fmt::Node::new(fmt::Kind::PrePostExec(exec))
            }
            prism::Node::PostExecutionNode { .. } => {
                let node = node.as_post_execution_node().unwrap();
                let exec = self.visit_pre_post_exec(
                    node.keyword_loc(),
                    node.opening_loc(),
                    node.statements(),
                    node.closing_loc(),
                );
                fmt::Node::new(fmt::Kind::PrePostExec(exec))
            }

            prism::Node::AliasMethodNode { .. } => {
                let node = node.as_alias_method_node().unwrap();
                let (leading, alias) = self.visit_alias(node.new_name(), node.old_name());
                println!("AA {:?}", leading);
                fmt::Node::with_leading_trivia(leading, fmt::Kind::Alias(alias))
            }
            prism::Node::AliasGlobalVariableNode { .. } => {
                let node = node.as_alias_global_variable_node().unwrap();
                let (leading, alias) = self.visit_alias(node.new_name(), node.old_name());
                fmt::Node::with_leading_trivia(leading, fmt::Kind::Alias(alias))
            }

            _ => todo!("parse {:?}", node),
        }
    }

    fn visit_constant_path(
        &mut self,
        parent: Option<prism::Node>,
        child: prism::Node,
    ) -> fmt::ConstantPath {
        let mut const_path = match parent {
            Some(parent) => {
                let parent = self.visit(parent, None);
                match parent.kind {
                    fmt::Kind::ConstantPath(const_path) => const_path,
                    _ => fmt::ConstantPath::new(Some(parent)),
                }
            }
            None => fmt::ConstantPath::new(None),
        };
        if !matches!(child, prism::Node::ConstantReadNode { .. }) {
            panic!("unexpected constant path child: {:?}", child);
        }
        let child_loc = child.location();
        let path_leading = self.take_leading_trivia(child_loc.start_offset());
        let path = Self::source_lossy_at(&child_loc);
        const_path.append_part(path_leading, path);
        const_path
    }

    fn visit_statements(
        &mut self,
        node: Option<prism::StatementsNode>,
        end: Option<usize>,
    ) -> fmt::Statements {
        let mut statements = fmt::Statements::new();
        if let Some(node) = node {
            Self::each_node_with_trailing_end(node.body().iter(), end, |node, trailing_end| {
                let fmt_node = self.visit(node, trailing_end);
                statements.append_node(fmt_node);
            });
        }
        let virtual_end = self.take_end_trivia_as_virtual_end(end);
        statements.set_virtual_end(virtual_end);
        statements
    }

    fn visit_else(&mut self, node: prism::ElseNode, else_end: usize) -> fmt::Else {
        let else_next_loc = node
            .statements()
            .as_ref()
            .map(|s| s.location().start_offset())
            .unwrap_or(else_end);
        let keyword_trailing = self.take_trailing_comment(else_next_loc);
        let body = self.visit_statements(node.statements(), Some(else_end));
        fmt::Else {
            keyword_trailing,
            body,
        }
    }

    fn visit_case_match(&mut self, node: prism::CaseMatchNode) -> fmt::CaseMatch {
        let conditions = node.conditions();
        let consequent = node.consequent();
        let end_loc = node.end_keyword_loc();
        let first_branch_start = conditions
            .iter()
            .next()
            .map(|n| n.location().start_offset());

        let pred_next = first_branch_start
            .or(consequent.as_ref().map(|c| c.location().start_offset()))
            .unwrap_or(end_loc.start_offset());
        let predicate = node.predicate().map(|n| self.visit(n, Some(pred_next)));
        let case_trailing = if predicate.is_some() {
            fmt::TrailingTrivia::none()
        } else {
            self.take_trailing_comment(pred_next)
        };

        let first_branch_leading = match first_branch_start {
            Some(first_branch_start) => self.take_leading_trivia(first_branch_start),
            None => fmt::LeadingTrivia::new(),
        };

        let mut branches = vec![];
        let conditions_next = consequent
            .as_ref()
            .map(|c| c.location().start_offset())
            .unwrap_or(end_loc.start_offset());
        Self::each_node_with_trailing_end(
            conditions.iter(),
            Some(conditions_next),
            |node, trailing_end| {
                let condition = match node {
                    prism::Node::InNode { .. } => {
                        let node = node.as_in_node().unwrap();
                        self.visit_case_in(node, trailing_end)
                    }
                    _ => panic!("unexpected case expression branch: {:?}", node),
                };
                branches.push(condition);
            },
        );

        let otherwise = consequent.map(|node| self.visit_else(node, end_loc.start_offset()));

        fmt::CaseMatch {
            case_trailing,
            predicate: predicate.map(Box::new),
            first_branch_leading,
            branches,
            otherwise,
        }
    }

    fn visit_case_in(&mut self, node: prism::InNode, body_end: Option<usize>) -> fmt::CaseIn {
        let loc = node.location();
        let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());

        let pattern_next = node
            .statements()
            .as_ref()
            .map(|n| n.location().start_offset());
        let pattern = self.visit(node.pattern(), pattern_next);

        let mut case_in = fmt::CaseIn::new(was_flat, pattern);
        let body = self.visit_statements(node.statements(), body_end);
        case_in.set_body(body);
        case_in
    }

    fn visit_match_assign(
        &mut self,
        expression: prism::Node,
        operator_loc: prism::Location,
        pattern: prism::Node,
    ) -> fmt::MatchAssign {
        let expression = self.visit(expression, Some(operator_loc.start_offset()));
        let pattern = self.visit(pattern, None);
        let operator = Self::source_lossy_at(&operator_loc);
        fmt::MatchAssign::new(expression, operator, pattern)
    }

    fn visit_arguments(
        &mut self,
        node: Option<prism::ArgumentsNode>,
        block_arg: Option<prism::BlockArgumentNode>,
        opening_loc: Option<prism::Location>,
        closing_loc: Option<prism::Location>,
    ) -> Option<fmt::Arguments> {
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let closing_start = closing_loc.as_ref().map(|l| l.start_offset());
        match node {
            None => {
                let block_arg =
                    block_arg.map(|block_arg| self.visit(block_arg.as_node(), closing_start));
                let virtual_end = closing_start.and_then(|closing_start| {
                    self.take_end_trivia_as_virtual_end(Some(closing_start))
                });
                match (block_arg, virtual_end, &opening) {
                    (None, None, None) => None,
                    (block_arg, virtual_end, _) => {
                        let mut args = fmt::Arguments::new(opening, closing);
                        if let Some(block_arg) = block_arg {
                            args.append_node(block_arg);
                            args.last_comma_allowed = false;
                        }
                        if virtual_end.is_some() {
                            args.set_virtual_end(virtual_end)
                        }
                        Some(args)
                    }
                }
            }
            Some(args_node) => {
                let mut args = fmt::Arguments::new(opening, closing);
                let mut nodes = args_node.arguments().iter().collect::<Vec<_>>();
                if let Some(block_arg) = block_arg {
                    nodes.push(block_arg.as_node());
                }
                let mut idx = 0;
                let last_idx = nodes.len() - 1;
                Self::each_node_with_trailing_end(
                    nodes.into_iter(),
                    closing_start,
                    |node, trailing_end| {
                        if idx == last_idx {
                            args.last_comma_allowed = !matches!(
                                node,
                                prism::Node::ForwardingArgumentsNode { .. }
                                    | prism::Node::BlockArgumentNode { .. }
                            );
                        }
                        match node {
                            prism::Node::KeywordHashNode { .. } => {
                                let node = node.as_keyword_hash_node().unwrap();
                                self.each_keyword_hash_element(node, trailing_end, |fmt_node| {
                                    args.append_node(fmt_node);
                                });
                            }
                            _ => {
                                let fmt_node = self.visit(node, trailing_end);
                                args.append_node(fmt_node);
                            }
                        }

                        idx += 1;
                    },
                );
                let virtual_end = self.take_end_trivia_as_virtual_end(closing_start);
                args.set_virtual_end(virtual_end);
                Some(args)
            }
        }
    }

    fn each_keyword_hash_element(
        &mut self,
        node: prism::KeywordHashNode,
        trailing_end: Option<usize>,
        mut f: impl FnMut(fmt::Node),
    ) {
        Self::each_node_with_trailing_end(
            node.elements().iter(),
            trailing_end,
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                f(element);
            },
        );
    }

    fn visit_block_arg(&mut self, node: prism::BlockArgumentNode) -> fmt::Prefix {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let expr = node.expression().map(|expr| self.visit(expr, None));
        fmt::Prefix::new(operator, expr)
    }

    fn visit_block(&mut self, node: prism::BlockNode) -> fmt::Block {
        let loc = node.location();
        let opening = Self::source_lossy_at(&node.opening_loc());
        let closing = Self::source_lossy_at(&node.closing_loc());
        let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());
        let mut method_block = fmt::Block::new(was_flat, opening, closing);

        let body = node.body();
        let body_start = body.as_ref().and_then(|b| match b {
            prism::Node::BeginNode { .. } => {
                Self::start_of_begin_block_content(b.as_begin_node().unwrap())
            }
            _ => Some(b.location().start_offset()),
        });
        let params = node.parameters();
        let params_start = params.as_ref().map(|p| p.location().start_offset());
        let closing_loc = node.closing_loc();

        let opening_next_loc = params_start
            .or(body_start)
            .unwrap_or(closing_loc.start_offset());
        let opening_trailing = self.take_trailing_comment(opening_next_loc);
        method_block.set_opening_trailing(opening_trailing);

        if let Some(params) = params {
            let params_next_loc = body_start.unwrap_or(closing_loc.start_offset());
            match params {
                prism::Node::BlockParametersNode { .. } => {
                    let node = params.as_block_parameters_node().unwrap();
                    let params = self.visit_block_parameters(node, params_next_loc);
                    method_block.set_parameters(params);
                }
                prism::Node::NumberedParametersNode { .. } => {}
                _ => panic!("unexpected node for call block params: {:?}", node),
            }
        }

        let body_end_loc = closing_loc.start_offset();
        let body = self.parse_block_body(body, body_end_loc);
        method_block.set_body(body);

        method_block
    }

    fn start_of_begin_block_content(begin: prism::BeginNode) -> Option<usize> {
        let loc = begin
            .statements()
            .map(|n| n.location())
            .or_else(|| begin.rescue_clause().map(|n| n.location()))
            .or_else(|| begin.else_clause().map(|n| n.location()))
            .or_else(|| begin.ensure_clause().map(|n| n.location()))
            .or(begin.end_keyword_loc());
        loc.map(|l| l.start_offset())
    }

    fn visit_block_parameters(
        &mut self,
        node: prism::BlockParametersNode,
        trailing_end: usize,
    ) -> fmt::BlockParameters {
        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();

        // In lambda literal, parentheses can be omitted (e.g. "-> a, b {}").
        let opening = opening_loc
            .as_ref()
            .map(Self::source_lossy_at)
            .unwrap_or_else(|| "(".to_string());
        let closing = closing_loc
            .as_ref()
            .map(Self::source_lossy_at)
            .unwrap_or_else(|| ")".to_string());
        let mut block_params = fmt::BlockParameters::new(opening, closing);

        let closing_start = closing_loc.map(|l| l.start_offset());

        let locals = node.locals();

        if let Some(params) = node.parameters() {
            let params_next = locals
                .iter()
                .next()
                .map(|n| n.location().start_offset())
                .or(closing_start);
            self.visit_parameter_nodes(params, params_next, |node| {
                block_params.append_param(node);
            });
        }

        if let Some(closing_start) = closing_start {
            Self::each_node_with_trailing_end(
                locals.iter(),
                Some(closing_start),
                |node, trailing_end| {
                    let fmt_node = self.visit(node, trailing_end);
                    block_params.append_local(fmt_node);
                },
            );
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
            block_params.set_virtual_end(virtual_end);
        }

        let trailing = self.take_trailing_comment(trailing_end);
        block_params.set_closing_trailing(trailing);
        block_params
    }

    fn visit_infix_op(
        &mut self,
        left: prism::Node,
        operator_loc: prism::Location,
        right: prism::Node,
    ) -> fmt::InfixChain {
        let left = self.visit(left, Some(operator_loc.start_offset()));
        let operator = Self::source_lossy_at(&operator_loc);
        let precedence = fmt::InfixPrecedence::from_operator(&operator);
        let mut chain = match left.kind {
            fmt::Kind::InfixChain(chain) if chain.precedence() == &precedence => chain,
            _ => fmt::InfixChain::new(left, precedence),
        };
        let right = self.visit(right, None);
        chain.append_right(operator, right);
        chain
    }

    fn visit_lambda(&mut self, node: prism::LambdaNode) -> fmt::Lambda {
        let params = node.parameters().map(|params| match params {
            prism::Node::BlockParametersNode { .. } => {
                let params = params.as_block_parameters_node().unwrap();
                let params_end = params.location().end_offset();
                self.visit_block_parameters(params, params_end)
            }
            _ => panic!("unexpected node for lambda params: {:?}", node),
        });

        let body_end = node.closing_loc().start_offset();
        let body_opening_trailing = self.take_trailing_comment(body_end);
        let body = self.parse_block_body(node.body(), body_end);

        let was_flat = !self.does_line_break_exist_in(
            node.opening_loc().end_offset(),
            node.closing_loc().start_offset(),
        );
        let opening = Self::source_lossy_at(&node.opening_loc());
        let closing = Self::source_lossy_at(&node.closing_loc());
        let mut block = fmt::Block::new(was_flat, opening, closing);
        block.set_opening_trailing(body_opening_trailing);
        block.set_body(body);

        fmt::Lambda::new(params, block)
    }

    fn parse_call_like(
        &mut self,
        name_loc: prism::Location,
        arguments: Option<prism::ArgumentsNode>,
    ) -> fmt::CallLike {
        let name = Self::source_lossy_at(&name_loc);
        let mut call_like = fmt::CallLike::new(name);
        let args = self.visit_arguments(arguments, None, None, None);
        if let Some(args) = args {
            call_like.set_arguments(args);
        }
        call_like
    }

    fn visit_yield(&mut self, node: prism::YieldNode) -> fmt::CallLike {
        let args =
            self.visit_arguments(node.arguments(), None, node.lparen_loc(), node.rparen_loc());
        let mut call_like = fmt::CallLike::new("yield".to_string());
        if let Some(mut args) = args {
            args.last_comma_allowed = false;
            call_like.set_arguments(args);
        }
        call_like
    }

    fn visit_undef(&mut self, undef: prism::UndefNode) -> fmt::CallLike {
        let mut args = fmt::Arguments::new(None, None);
        Self::each_node_with_trailing_end(undef.names().iter(), None, |node, trailing_end| {
            let node = self.visit(node, trailing_end);
            args.append_node(node);
        });
        let mut call_like = fmt::CallLike::new("undef".to_string());
        call_like.set_arguments(args);
        call_like
    }

    fn visit_defined(&mut self, defined: prism::DefinedNode) -> fmt::CallLike {
        let lparen_loc = defined.lparen_loc();
        let rparen_loc = defined.rparen_loc();

        let value = defined.value();
        let value_next = rparen_loc.as_ref().map(|l| l.start_offset());
        let value = self.visit(value, value_next);

        let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
        let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
        let mut args = fmt::Arguments::new(lparen, rparen);
        args.last_comma_allowed = false;
        args.append_node(value);

        let rparen_start = rparen_loc.as_ref().map(|l| l.start_offset());
        let virutla_end = self.take_end_trivia_as_virtual_end(rparen_start);
        args.set_virtual_end(virutla_end);

        let mut call_like = fmt::CallLike::new("defined?".to_string());
        call_like.set_arguments(args);
        call_like
    }

    fn visit_variable_assign(
        &mut self,
        name_loc: prism::Location,
        operator_loc: prism::Location,
        value: prism::Node,
    ) -> fmt::Assign {
        let name = Self::source_lossy_at(&name_loc);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, None);
        let target = fmt::Node::new(fmt::Kind::Atom(fmt::Atom(name)));
        fmt::Assign::new(target, operator, value)
    }

    fn visit_constant_path_assign(
        &mut self,
        const_path: prism::ConstantPathNode,
        operator_loc: prism::Location,
        value: prism::Node,
    ) -> fmt::Assign {
        let const_path = self.visit_constant_path(const_path.parent(), const_path.child());
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, None);
        let target = fmt::Node::new(fmt::Kind::ConstantPath(const_path));
        fmt::Assign::new(target, operator, value)
    }

    fn visit_call_assign(
        &mut self,
        call: &impl calls::CallRoot,
        operator_loc: prism::Location,
        value: prism::Node,
    ) -> fmt::Assign {
        let chain = self.parse_call_root(call);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, None);
        let target = fmt::Node::new(fmt::Kind::MethodChain(chain));
        fmt::Assign::new(target, operator, value)
    }

    fn visit_multi_assign(&mut self, node: prism::MultiWriteNode) -> fmt::Assign {
        let target = self.visit_multi_assign_target(
            node.lefts(),
            node.rest(),
            node.rights(),
            node.lparen_loc(),
            node.rparen_loc(),
        );
        let operator = Self::source_lossy_at(&node.operator_loc());
        let value = self.visit(node.value(), None);

        let target = fmt::Node::new(fmt::Kind::MultiAssignTarget(target));
        fmt::Assign::new(target, operator, value)
    }

    fn visit_multi_assign_target(
        &mut self,
        lefts: prism::NodeList,
        rest: Option<prism::Node>,
        rights: prism::NodeList,
        lparen_loc: Option<prism::Location>,
        rparen_loc: Option<prism::Location>,
    ) -> fmt::MultiAssignTarget {
        let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
        let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
        let mut multi = fmt::MultiAssignTarget::new(lparen, rparen);

        let implicit_rest = rest
            .as_ref()
            .map_or(false, |r| matches!(r, prism::Node::ImplicitRestNode { .. }));
        multi.set_implicit_rest(implicit_rest);

        let rest_start = if implicit_rest {
            None
        } else {
            rest.as_ref().map(|r| r.location().start_offset())
        };
        let rights_first_start = rights.iter().next().map(|n| n.location().start_offset());
        let rparen_start = rparen_loc.as_ref().map(|l| l.start_offset());

        let left_trailing_end = rest_start.or(rights_first_start).or(rparen_start);
        Self::each_node_with_trailing_end(lefts.iter(), left_trailing_end, |node, trailing_end| {
            let target = self.visit(node, trailing_end);
            multi.append_target(target);
        });

        if !implicit_rest {
            if let Some(rest) = rest {
                let rest_trailing_end = rights_first_start.or(rparen_start);
                let target = self.visit(rest, rest_trailing_end);
                multi.append_target(target);
            }
        }

        Self::each_node_with_trailing_end(rights.iter(), rparen_start, |node, trailing_end| {
            let target = self.visit(node, trailing_end);
            multi.append_target(target);
        });

        if let Some(rparen_loc) = rparen_loc {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(rparen_loc.start_offset()));
            multi.set_virtual_end(virtual_end);
        }

        multi
    }

    fn visit_def(&mut self, node: prism::DefNode) -> (fmt::LeadingTrivia, fmt::Def) {
        let receiver = node.receiver();
        let name_loc = node.name_loc();

        // Take leading trivia of receiver or method name.
        let leading_end = receiver
            .as_ref()
            .map(|r| r.location().start_offset())
            .unwrap_or_else(|| name_loc.start_offset());
        let leading = self.take_leading_trivia(leading_end);

        let name_end = name_loc.end_offset();
        let receiver = receiver.map(|r| self.visit(r, Some(name_end)));
        let name = Self::source_lossy_at(&node.name_loc());
        let mut def = fmt::Def::new(receiver, name);

        let lparen_loc = node.lparen_loc();
        let rparen_loc = node.rparen_loc();
        if let Some(params) = node.parameters() {
            let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
            let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
            let mut parameters = fmt::MethodParameters::new(lparen, rparen);
            let params_next = rparen_loc.as_ref().map(|l| l.start_offset());
            self.visit_parameter_nodes(params, params_next, |node| {
                parameters.append_param(node);
            });
            let virtual_end = self.take_end_trivia_as_virtual_end(params_next);
            parameters.set_virtual_end(virtual_end);
            def.set_parameters(parameters);
        } else if let (Some(lparen_loc), Some(rparen_loc)) = (&lparen_loc, &rparen_loc) {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(rparen_loc.start_offset()));
            if virtual_end.is_some() {
                let lparen = Self::source_lossy_at(lparen_loc);
                let rparen = Self::source_lossy_at(rparen_loc);
                let mut parameters = fmt::MethodParameters::new(Some(lparen), Some(rparen));
                parameters.set_virtual_end(virtual_end);
                def.set_parameters(parameters);
            }
        }

        if node.equal_loc().is_some() {
            let body = node.body().expect("shorthand def body must exist");
            let body = self.visit(body, None);
            def.set_body(fmt::DefBody::Short {
                body: Box::new(body),
            });
        } else {
            let end_loc = node.end_keyword_loc().expect("block def must have end");
            let body = node.body();
            let body_start = body.as_ref().and_then(|b| match b {
                prism::Node::BeginNode { .. } => {
                    Self::start_of_begin_block_content(b.as_begin_node().unwrap())
                }
                _ => Some(b.location().start_offset()),
            });
            let head_next = body_start.unwrap_or(end_loc.start_offset());
            let head_trailing = self.take_trailing_comment(head_next);
            let block_body = self.parse_block_body(body, end_loc.start_offset());
            def.set_body(fmt::DefBody::Block {
                head_trailing,
                body: block_body,
            });
        }

        (leading, def)
    }

    fn parse_block_body(
        &mut self,
        body: Option<prism::Node>,
        trailing_end: usize,
    ) -> fmt::BlockBody {
        match body {
            Some(body) => match body {
                prism::Node::StatementsNode { .. } => {
                    let stmts = body.as_statements_node().unwrap();
                    let statements = self.visit_statements(Some(stmts), Some(trailing_end));
                    fmt::BlockBody::new(statements)
                }
                prism::Node::BeginNode { .. } => {
                    let node = body.as_begin_node().unwrap();
                    self.visit_begin_body(node)
                }
                _ => panic!("unexpected def body: {:?}", body),
            },
            None => {
                let statements = self.wrap_as_statements(None, trailing_end);
                fmt::BlockBody::new(statements)
            }
        }
    }

    fn visit_begin_body(&mut self, node: prism::BeginNode) -> fmt::BlockBody {
        let rescue_start = node
            .rescue_clause()
            .as_ref()
            .map(|r| r.location().start_offset());
        let else_start = node
            .else_clause()
            .as_ref()
            .map(|e| e.location().start_offset());
        let ensure_start = node
            .ensure_clause()
            .as_ref()
            .map(|e| e.location().start_offset());
        // XXX: I cannot find the case where the begin block does not have end.
        let end_loc = node.end_keyword_loc().expect("begin must have end");

        let statements_next = rescue_start
            .or(else_start)
            .or(ensure_start)
            .unwrap_or(end_loc.start_offset());
        let statements = self.visit_statements(node.statements(), Some(statements_next));
        let mut body = fmt::BlockBody::new(statements);

        if let Some(rescue_node) = node.rescue_clause() {
            let rescues_next = else_start
                .or(ensure_start)
                .unwrap_or(end_loc.start_offset());
            let mut rescues = vec![];
            self.visit_rescue_chain(rescue_node, &mut rescues, rescues_next);
            body.set_rescues(rescues);
        }

        if let Some(else_node) = node.else_clause() {
            let statements = else_node.statements();
            let keyword_next = statements
                .as_ref()
                .map(|s| s.location().start_offset())
                .or(ensure_start)
                .unwrap_or(end_loc.start_offset());
            let else_trailing = self.take_trailing_comment(keyword_next);
            let else_next = ensure_start.unwrap_or(end_loc.start_offset());
            let else_statements = self.visit_statements(statements, Some(else_next));
            body.set_rescue_else(fmt::Else {
                keyword_trailing: else_trailing,
                body: else_statements,
            });
        }

        if let Some(ensure_node) = node.ensure_clause() {
            let statements = ensure_node.statements();
            let keyword_next = statements
                .as_ref()
                .map(|s| s.location().start_offset())
                .unwrap_or(end_loc.start_offset());
            let ensure_trailing = self.take_trailing_comment(keyword_next);
            let ensure_statements = self.visit_statements(statements, Some(end_loc.start_offset()));
            body.set_ensure(fmt::Else {
                keyword_trailing: ensure_trailing,
                body: ensure_statements,
            });
        }

        body
    }

    fn visit_rescue_chain(
        &mut self,
        node: prism::RescueNode,
        rescues: &mut Vec<fmt::Rescue>,
        final_next: usize,
    ) {
        let reference = node.reference();
        let reference_start = reference.as_ref().map(|c| c.location().start_offset());

        let statements = node.statements();
        let statements_start = statements.as_ref().map(|c| c.location().start_offset());

        let consequent = node.consequent();
        let consequent_start = consequent.as_ref().map(|c| c.location().start_offset());

        let mut rescue = fmt::Rescue::new();

        let head_next = reference_start
            .or(statements_start)
            .or(consequent_start)
            .unwrap_or(final_next);
        Self::each_node_with_trailing_end(
            node.exceptions().iter(),
            Some(head_next),
            |node, trailing_end| {
                let fmt_node = self.visit(node, trailing_end);
                rescue.append_exception(fmt_node);
            },
        );

        if let Some(reference) = reference {
            let reference_next = statements_start.or(consequent_start).unwrap_or(final_next);
            let reference = self.visit(reference, Some(reference_next));
            rescue.set_reference(reference);
        }

        let head_next = statements_start.or(consequent_start).unwrap_or(final_next);
        let head_trailing = self.take_trailing_comment(head_next);
        rescue.set_head_trailing(head_trailing);

        let statements_next = consequent_start.unwrap_or(final_next);
        let statements = self.visit_statements(statements, Some(statements_next));
        rescue.set_statements(statements);
        rescues.push(rescue);

        if let Some(consequent) = consequent {
            self.visit_rescue_chain(consequent, rescues, final_next);
        }
    }

    fn visit_parameter_nodes(
        &mut self,
        params: prism::ParametersNode,
        trailing_end: Option<usize>,
        mut f: impl FnMut(fmt::Node),
    ) {
        let mut nodes = vec![];
        for n in params.requireds().iter() {
            nodes.push(n);
        }
        for n in params.optionals().iter() {
            nodes.push(n);
        }
        if let Some(rest) = params.rest() {
            nodes.push(rest);
        }
        for n in params.posts().iter() {
            nodes.push(n);
        }
        for n in params.keywords().iter() {
            nodes.push(n);
        }
        if let Some(rest) = params.keyword_rest() {
            nodes.push(rest);
        }
        if let Some(block) = params.block() {
            nodes.push(block.as_node());
        }
        Self::each_node_with_trailing_end(nodes.into_iter(), trailing_end, |node, trailing_end| {
            let fmt_node = self.visit(node, trailing_end);
            f(fmt_node);
        });
    }

    fn visit_class_like(
        &mut self,
        keyword: &str,
        name_loc: prism::Location,
        superclass: Option<prism::Node>,
        body: Option<prism::Node>,
        end_loc: prism::Location,
    ) -> (fmt::LeadingTrivia, fmt::ClassLike) {
        let leading = self.take_leading_trivia(name_loc.start_offset());
        let name = Self::source_lossy_at(&name_loc);

        let body_start = body.as_ref().and_then(|b| match b {
            prism::Node::BeginNode { .. } => {
                Self::start_of_begin_block_content(b.as_begin_node().unwrap())
            }
            _ => Some(b.location().start_offset()),
        });
        let head_next = body_start.unwrap_or(end_loc.start_offset());
        let (superclass, head_trailing) = if let Some(superclass) = superclass {
            let fmt_node = self.visit(superclass, Some(head_next));
            (Some(fmt_node), fmt::TrailingTrivia::none())
        } else {
            let head_trailing = self.take_trailing_comment(head_next);
            (None, head_trailing)
        };

        let body = self.parse_block_body(body, end_loc.start_offset());
        let class = fmt::ClassLike {
            keyword: keyword.to_string(),
            name,
            superclass: superclass.map(Box::new),
            head_trailing,
            body,
        };
        (leading, class)
    }

    fn visit_array_pattern(&mut self, node: prism::ArrayPatternNode) -> fmt::ArrayPattern {
        let constant = node.constant().map(|c| self.visit(c, None));
        let opening = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = node.closing_loc().as_ref().map(Self::source_lossy_at);
        let mut array = fmt::ArrayPattern::new(constant, opening, closing);

        let rest = node.rest();
        let posts = node.posts();
        let posts_head = posts.iter().next();

        let closing_start = node.closing_loc().as_ref().map(|c| c.start_offset());
        let requireds_next = rest
            .as_ref()
            .map(|r| r.location().start_offset())
            .or_else(|| posts_head.as_ref().map(|p| p.location().start_offset()))
            .or(closing_start);
        Self::each_node_with_trailing_end(
            node.requireds().iter(),
            requireds_next,
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                array.append_element(element);
            },
        );

        if let Some(rest) = node.rest() {
            let rest_next = posts_head
                .as_ref()
                .map(|p| p.location().start_offset())
                .or(closing_start);
            let element = self.visit(rest, rest_next);
            array.append_element(element);
            array.last_comma_allowed = false;
        }

        if posts_head.is_some() {
            Self::each_node_with_trailing_end(
                node.posts().iter(),
                closing_start,
                |node, trailing_end| {
                    let element = self.visit(node, trailing_end);
                    array.append_element(element);
                },
            );
            array.last_comma_allowed = false;
        }

        let end = self.take_end_trivia_as_virtual_end(closing_start);
        array.set_virtual_end(end);

        array
    }

    fn visit_find_pattern(&mut self, node: prism::FindPatternNode) -> fmt::ArrayPattern {
        let constant = node.constant().map(|c| self.visit(c, None));
        let opening = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = node.closing_loc().as_ref().map(Self::source_lossy_at);
        let mut array = fmt::ArrayPattern::new(constant, opening, closing);
        array.last_comma_allowed = false;

        let requireds = node.requireds();
        let right = node.right();

        let left_next = requireds
            .iter()
            .next()
            .map(|n| n.location().start_offset())
            .unwrap_or(right.location().start_offset());
        let left = self.visit(node.left(), Some(left_next));
        array.append_element(left);

        Self::each_node_with_trailing_end(
            node.requireds().iter(),
            Some(right.location().start_offset()),
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                array.append_element(element);
            },
        );

        let closing_start = node.closing_loc().as_ref().map(|l| l.start_offset());

        let right = self.visit(right, closing_start);
        array.append_element(right);

        let end = self.take_end_trivia_as_virtual_end(closing_start);
        array.set_virtual_end(end);

        array
    }

    fn visit_hash_pattern(&mut self, node: prism::HashPatternNode) -> fmt::HashPattern {
        let constant = node.constant().map(|c| self.visit(c, None));
        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let should_be_inline = match (opening_loc.as_ref(), node.elements().iter().next()) {
            (Some(opening_loc), Some(first_element)) => !self.does_line_break_exist_in(
                opening_loc.start_offset(),
                first_element.location().start_offset(),
            ),
            _ => true,
        };
        let mut hash = fmt::HashPattern::new(constant, opening, closing, should_be_inline);

        let rest = node.rest();
        let closing_start = closing_loc.as_ref().map(|c| c.start_offset());

        let elements_next = rest
            .as_ref()
            .map(|r| r.location().start_offset())
            .or(closing_start);
        Self::each_node_with_trailing_end(
            node.elements().iter(),
            elements_next,
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                hash.append_element(element);
            },
        );

        if let Some(rest) = node.rest() {
            let rest = self.visit(rest, closing_start);
            hash.append_element(rest);
            hash.last_comma_allowed = false;
        }

        let end = self.take_end_trivia_as_virtual_end(closing_start);
        hash.set_virtual_end(end);

        hash
    }

    fn visit_pinned_expression(&mut self, node: prism::PinnedExpressionNode) -> fmt::Prefix {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let rparen_start = node.rparen_loc().start_offset();
        let expression = self.visit(node.expression(), Some(rparen_start));

        let mut stmts = fmt::Statements::new();
        stmts.append_node(expression);
        stmts.set_virtual_end(self.take_end_trivia_as_virtual_end(Some(rparen_start)));
        let mut parens = fmt::Parens::new(stmts);
        parens.closing_break_allowed = false;

        let node = fmt::Node::new(fmt::Kind::Parens(parens));
        fmt::Prefix::new(operator, Some(node))
    }

    fn visit_pinned_variable(&mut self, node: prism::PinnedVariableNode) -> fmt::Prefix {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let variable = self.visit(node.variable(), None);
        fmt::Prefix::new(operator, Some(variable))
    }

    fn visit_capture_pattern(&mut self, node: prism::CapturePatternNode) -> fmt::Assoc {
        let value = self.visit(node.value(), Some(node.operator_loc().start_offset()));
        let operator = Self::source_lossy_at(&node.operator_loc());
        let target = self.visit(node.target(), None);
        fmt::Assoc::new(value, Some(operator), target)
    }

    fn visit_alternation_pattern(
        &mut self,
        node: prism::AlternationPatternNode,
    ) -> fmt::AltPatternChain {
        let operator_loc = node.operator_loc();
        let left = self.visit(node.left(), Some(operator_loc.start_offset()));
        let mut chain = match left.kind {
            fmt::Kind::AltPatternChain(chain) => chain,
            _ => fmt::AltPatternChain::new(left),
        };
        let right = node.right();
        let right = self.visit(right, None);
        chain.append_right(right);
        chain
    }

    fn visit_pre_post_exec(
        &mut self,
        keyword_loc: prism::Location,
        opening_loc: prism::Location,
        statements: Option<prism::StatementsNode>,
        closing_loc: prism::Location,
    ) -> fmt::PrePostExec {
        let keyword = Self::source_lossy_at(&keyword_loc);
        let closing_start = closing_loc.start_offset();
        let was_flat = !self.does_line_break_exist_in(opening_loc.end_offset(), closing_start);
        let statements = self.visit_statements(statements, Some(closing_start));
        fmt::PrePostExec::new(keyword, statements, was_flat)
    }

    fn visit_alias(
        &mut self,
        new_name: prism::Node,
        old_name: prism::Node,
    ) -> (fmt::LeadingTrivia, fmt::Alias) {
        let additional_leading = self.take_leading_trivia(new_name.location().start_offset());
        let old_loc = old_name.location();
        let new_name = self.visit(new_name, Some(old_loc.start_offset()));
        let old_name = self.visit(old_name, None);
        let alias = fmt::Alias::new(new_name, old_name);
        (additional_leading, alias)
    }

    fn wrap_as_statements(&mut self, node: Option<fmt::Node>, end: usize) -> fmt::Statements {
        let (mut statements, should_take_end_trivia) = match node {
            None => (fmt::Statements::new(), true),
            Some(node) => match node.kind {
                fmt::Kind::Statements(statements) => (statements, false),
                _ => {
                    let mut statements = fmt::Statements::new();
                    statements.append_node(node);
                    (statements, true)
                }
            },
        };
        if should_take_end_trivia {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(end));
            statements.set_virtual_end(virtual_end);
        }
        statements
    }

    fn take_end_trivia_as_virtual_end(&mut self, end: Option<usize>) -> Option<fmt::VirtualEnd> {
        if let Some(end) = end {
            let trivia = self.take_leading_trivia(end);
            if !trivia.is_empty() {
                return Some(fmt::VirtualEnd::new(trivia));
            }
        }
        None
    }

    fn take_leading_trivia(&mut self, loc_start: usize) -> fmt::LeadingTrivia {
        let mut trivia = fmt::LeadingTrivia::new();

        while let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if !(self.last_loc_end..=loc_start).contains(&loc.start_offset()) {
                break;
            };
            let mut value = Self::source_lossy_at(&loc);
            if value.starts_with("=begin") {
                value = value.trim_end().to_string();
            }
            let fmt_comment = fmt::Comment { value };
            self.take_empty_lines_until(loc.start_offset(), &mut trivia);
            trivia.append_line(fmt::LineTrivia::Comment(fmt_comment));
            self.last_loc_end = loc.end_offset() - 1;
            self.comments.next();
        }

        self.take_empty_lines_until(loc_start, &mut trivia);
        trivia
    }

    fn take_empty_lines_until(&mut self, end: usize, trivia: &mut fmt::LeadingTrivia) {
        let range = self.last_empty_line_range_within(self.last_loc_end, end);
        if let Some(range) = range {
            trivia.append_line(fmt::LineTrivia::EmptyLine);
            self.last_loc_end = range.end;
        }
    }

    fn take_trailing_comment(&mut self, end: usize) -> fmt::TrailingTrivia {
        if let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if (self.last_loc_end..=end).contains(&loc.start_offset())
                && !self.is_at_line_start(loc.start_offset())
            {
                self.last_loc_end = loc.end_offset() - 1;
                self.comments.next();
                let value = Self::source_lossy_at(&loc);
                let comment = Some(fmt::Comment { value });
                return fmt::TrailingTrivia::new(comment);
            }
        };
        fmt::TrailingTrivia::none()
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
