mod arrays;
mod assigns;
mod atoms;
mod begins;
mod blocks;
mod cases;
mod classes;
mod consts;
mod elses;
mod hashes;
mod ifs;
mod loops;
mod method_calls;
mod method_defs;
mod miscs;
mod pattern_matches;
mod postmodifiers;
mod ranges;
mod regexps;
mod statements;
mod strings;
mod symbols;
mod trivia;

use crate::fmt;
use std::{collections::HashMap, iter::Peekable};

#[derive(Debug)]
pub struct ParseError {
    messages: Vec<String>,
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "parse error:")?;
        for message in self.messages.iter() {
            writeln!(f, "{message}")?;
        }
        Ok(())
    }
}
impl std::error::Error for ParseError {}

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Result<ParserResult, ParseError> {
    let result = prism::parse(&source);

    let messages = result
        .errors()
        .map(|e| {
            let loc = e.location();
            format!(
                "[{},{}] {}",
                loc.start_offset(),
                loc.end_offset(),
                e.message()
            )
        })
        .collect::<Vec<_>>();
    if !messages.is_empty() {
        return Err(ParseError { messages });
    }

    let comments = result.comments().peekable();
    let mut parser = Parser::new(&source, comments);
    let fmt_node = parser.parse_from_prism_node(result.node());
    // dbg!(&fmt_node);
    // dbg!(&parser.heredoc_map);
    Ok(ParserResult {
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
    fn new<'src>(src: &'src [u8], comments: Peekable<prism::Comments<'src>>) -> Parser<'src> {
        Parser {
            src,
            comments,
            heredoc_map: HashMap::new(),
            position_gen: 0,
            last_loc_end: 0,
        }
    }

    fn parse_from_prism_node(&mut self, node: prism::Node) -> fmt::Node {
        self.parse(node, Some(self.src.len()))
    }

    fn parse(&mut self, node: prism::Node, trailing_end: Option<usize>) -> fmt::Node {
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
                self.parse_statements(node.statements(), trailing_end)
            }
            prism::Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                self.parse_statements(node, trailing_end)
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
                self.parse_constant_path(node.parent(), node.child())
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
                self.parse(node.call().as_node(), None)
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
                self.parse_call_root(&node)
            }
            prism::Node::SuperNode { .. } => {
                let node = node.as_super_node().unwrap();
                self.parse_call_root(&node)
            }
            prism::Node::YieldNode { .. } => {
                let node = node.as_yield_node().unwrap();
                self.parse_yield(node)
            }

            prism::Node::BreakNode { .. } => {
                let node = node.as_break_node().unwrap();
                self.parse_call_like(node.keyword_loc(), node.arguments())
            }
            prism::Node::NextNode { .. } => {
                let node = node.as_next_node().unwrap();
                self.parse_call_like(node.keyword_loc(), node.arguments())
            }
            prism::Node::ReturnNode { .. } => {
                let node = node.as_return_node().unwrap();
                self.parse_call_like(node.keyword_loc(), node.arguments())
            }

            prism::Node::AndNode { .. } => {
                let node = node.as_and_node().unwrap();
                self.parse_infix_operation(node.left(), node.operator_loc(), node.right())
            }
            prism::Node::OrNode { .. } => {
                let node = node.as_or_node().unwrap();
                self.parse_infix_operation(node.left(), node.operator_loc(), node.right())
            }

            prism::Node::LocalVariableWriteNode { .. } => {
                let node = node.as_local_variable_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::LocalVariableAndWriteNode { .. } => {
                let node = node.as_local_variable_and_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::LocalVariableOrWriteNode { .. } => {
                let node = node.as_local_variable_or_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::LocalVariableOperatorWriteNode { .. } => {
                let node = node.as_local_variable_operator_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }

            prism::Node::InstanceVariableWriteNode { .. } => {
                let node = node.as_instance_variable_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::InstanceVariableAndWriteNode { .. } => {
                let node = node.as_instance_variable_and_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::InstanceVariableOrWriteNode { .. } => {
                let node = node.as_instance_variable_or_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::InstanceVariableOperatorWriteNode { .. } => {
                let node = node.as_instance_variable_operator_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }

            prism::Node::ClassVariableWriteNode { .. } => {
                let node = node.as_class_variable_write_node().unwrap();
                self.parse_variable_assign(
                    node.name_loc(),
                    // XXX: When does the operator becomes None?
                    node.operator_loc().expect("must have operator"),
                    node.value(),
                )
            }
            prism::Node::ClassVariableAndWriteNode { .. } => {
                let node = node.as_class_variable_and_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::ClassVariableOrWriteNode { .. } => {
                let node = node.as_class_variable_or_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::ClassVariableOperatorWriteNode { .. } => {
                let node = node.as_class_variable_operator_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }

            prism::Node::GlobalVariableWriteNode { .. } => {
                let node = node.as_global_variable_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::GlobalVariableAndWriteNode { .. } => {
                let node = node.as_global_variable_and_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::GlobalVariableOrWriteNode { .. } => {
                let node = node.as_global_variable_or_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::GlobalVariableOperatorWriteNode { .. } => {
                let node = node.as_global_variable_operator_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }

            prism::Node::ConstantWriteNode { .. } => {
                let node = node.as_constant_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::ConstantAndWriteNode { .. } => {
                let node = node.as_constant_and_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::ConstantOrWriteNode { .. } => {
                let node = node.as_constant_or_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::ConstantOperatorWriteNode { .. } => {
                let node = node.as_constant_operator_write_node().unwrap();
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }

            prism::Node::ConstantPathWriteNode { .. } => {
                let node = node.as_constant_path_write_node().unwrap();
                self.parse_constant_path_assign(node.target(), node.operator_loc(), node.value())
            }
            prism::Node::ConstantPathAndWriteNode { .. } => {
                let node = node.as_constant_path_and_write_node().unwrap();
                self.parse_constant_path_assign(node.target(), node.operator_loc(), node.value())
            }
            prism::Node::ConstantPathOrWriteNode { .. } => {
                let node = node.as_constant_path_or_write_node().unwrap();
                self.parse_constant_path_assign(node.target(), node.operator_loc(), node.value())
            }
            prism::Node::ConstantPathOperatorWriteNode { .. } => {
                let node = node.as_constant_path_operator_write_node().unwrap();
                self.parse_constant_path_assign(node.target(), node.operator_loc(), node.value())
            }

            prism::Node::CallAndWriteNode { .. } => {
                let node = node.as_call_and_write_node().unwrap();
                self.parse_call_assign(&node, node.operator_loc(), node.value())
            }
            prism::Node::CallOrWriteNode { .. } => {
                let node = node.as_call_or_write_node().unwrap();
                self.parse_call_assign(&node, node.operator_loc(), node.value())
            }
            prism::Node::CallOperatorWriteNode { .. } => {
                let node = node.as_call_operator_write_node().unwrap();
                self.parse_call_assign(&node, node.operator_loc(), node.value())
            }

            prism::Node::IndexAndWriteNode { .. } => {
                let node = node.as_index_and_write_node().unwrap();
                self.parse_call_assign(&node, node.operator_loc(), node.value())
            }
            prism::Node::IndexOrWriteNode { .. } => {
                let node = node.as_index_or_write_node().unwrap();
                self.parse_call_assign(&node, node.operator_loc(), node.value())
            }
            prism::Node::IndexOperatorWriteNode { .. } => {
                let node = node.as_index_operator_write_node().unwrap();
                self.parse_call_assign(&node, node.operator_loc(), node.value())
            }

            prism::Node::LocalVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::InstanceVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::ClassVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::GlobalVariableTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::ConstantTargetNode { .. } => self.parse_as_atom(node),
            prism::Node::ConstantPathTargetNode { .. } => {
                let node = node.as_constant_path_target_node().unwrap();
                self.parse_constant_path(node.parent(), node.child())
            }
            prism::Node::CallTargetNode { .. } => {
                let node = node.as_call_target_node().unwrap();
                self.parse_call_root(&node)
            }
            prism::Node::IndexTargetNode { .. } => {
                let node = node.as_index_target_node().unwrap();
                self.parse_call_root(&node)
            }

            prism::Node::MultiWriteNode { .. } => {
                let node = node.as_multi_write_node().unwrap();
                self.parse_multi_assign(node)
            }
            prism::Node::MultiTargetNode { .. } => {
                let node = node.as_multi_target_node().unwrap();
                self.parse_multi_assign_target(
                    node.lefts(),
                    node.rest(),
                    node.rights(),
                    node.lparen_loc(),
                    node.rparen_loc(),
                )
            }
            prism::Node::ImplicitRestNode { .. } => self.parse_implicit(),

            prism::Node::SplatNode { .. } => {
                let node = node.as_splat_node().unwrap();
                self.parse_splat(node)
            }
            prism::Node::AssocSplatNode { .. } => {
                let node = node.as_assoc_splat_node().unwrap();
                self.parse_assoc_splat(node)
            }
            prism::Node::BlockArgumentNode { .. } => {
                let node = node.as_block_argument_node().unwrap();
                self.parse_block_arg(node)
            }

            prism::Node::ArrayNode { .. } => {
                let node = node.as_array_node().unwrap();
                self.parse_array(node)
            }

            prism::Node::HashNode { .. } => {
                let node = node.as_hash_node().unwrap();
                self.parse_hash(node)
            }
            prism::Node::AssocNode { .. } => {
                let node = node.as_assoc_node().unwrap();
                self.parse_assoc(node)
            }
            prism::Node::ImplicitNode { .. } => self.parse_implicit(),

            prism::Node::ParenthesesNode { .. } => {
                let node = node.as_parentheses_node().unwrap();
                self.parse_parentheses(node)
            }

            prism::Node::DefNode { .. } => {
                let node = node.as_def_node().unwrap();
                self.parse_def(node)
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
                self.parse_variable_assign(node.name_loc(), node.operator_loc(), node.value())
            }
            prism::Node::OptionalKeywordParameterNode { .. } => {
                let node = node.as_optional_keyword_parameter_node().unwrap();
                self.parse_optional_keyword_argument(node)
            }

            prism::Node::LambdaNode { .. } => {
                let node = node.as_lambda_node().unwrap();
                self.parse_lambda(node)
            }

            prism::Node::UndefNode { .. } => {
                let node = node.as_undef_node().unwrap();
                self.parse_undef(node)
            }
            prism::Node::DefinedNode { .. } => {
                let node = node.as_defined_node().unwrap();
                self.parse_defined(node)
            }

            prism::Node::BeginNode { .. } => {
                let node = node.as_begin_node().unwrap();
                self.parse_begin(node)
            }

            prism::Node::ClassNode { .. } => {
                let node = node.as_class_node().unwrap();
                self.parse_class_like(
                    "class",
                    node.constant_path().location(),
                    node.superclass(),
                    node.body(),
                    node.end_keyword_loc(),
                )
            }
            prism::Node::ModuleNode { .. } => {
                let node = node.as_module_node().unwrap();
                self.parse_class_like(
                    "module",
                    node.constant_path().location(),
                    None,
                    node.body(),
                    node.end_keyword_loc(),
                )
            }
            prism::Node::SingletonClassNode { .. } => {
                let node = node.as_singleton_class_node().unwrap();
                self.parse_singleton_class(node)
            }

            prism::Node::RangeNode { .. } => {
                let node = node.as_range_node().unwrap();
                self.parse_range_like(node.operator_loc(), node.left(), node.right())
            }
            prism::Node::FlipFlopNode { .. } => {
                let node = node.as_flip_flop_node().unwrap();
                self.parse_range_like(node.operator_loc(), node.left(), node.right())
            }

            prism::Node::CaseMatchNode { .. } => {
                let node = node.as_case_match_node().unwrap();
                self.parse_case_match(node)
            }
            prism::Node::MatchPredicateNode { .. } => {
                let node = node.as_match_predicate_node().unwrap();

                self.parse_match_assign(node.value(), node.operator_loc(), node.pattern())
            }
            prism::Node::MatchRequiredNode { .. } => {
                let node = node.as_match_required_node().unwrap();
                self.parse_match_assign(node.value(), node.operator_loc(), node.pattern())
            }

            prism::Node::ArrayPatternNode { .. } => {
                let node = node.as_array_pattern_node().unwrap();
                self.parse_array_pattern(node)
            }
            prism::Node::FindPatternNode { .. } => {
                let node = node.as_find_pattern_node().unwrap();
                self.parse_find_pattern(node)
            }
            prism::Node::HashPatternNode { .. } => {
                let node = node.as_hash_pattern_node().unwrap();
                self.parse_hash_pattern(node)
            }
            prism::Node::PinnedExpressionNode { .. } => {
                let node = node.as_pinned_expression_node().unwrap();
                self.parse_pinned_expression(node)
            }
            prism::Node::PinnedVariableNode { .. } => {
                let node = node.as_pinned_variable_node().unwrap();
                self.parse_pinned_variable(node)
            }
            prism::Node::CapturePatternNode { .. } => {
                let node = node.as_capture_pattern_node().unwrap();
                self.parse_capture_pattern(node)
            }
            prism::Node::AlternationPatternNode { .. } => {
                let node = node.as_alternation_pattern_node().unwrap();
                self.parse_alternation_pattern(node)
            }

            prism::Node::PreExecutionNode { .. } => {
                let node = node.as_pre_execution_node().unwrap();
                self.parse_pre_post_exec(
                    node.keyword_loc(),
                    node.opening_loc(),
                    node.statements(),
                    node.closing_loc(),
                )
            }
            prism::Node::PostExecutionNode { .. } => {
                let node = node.as_post_execution_node().unwrap();
                self.parse_pre_post_exec(
                    node.keyword_loc(),
                    node.opening_loc(),
                    node.statements(),
                    node.closing_loc(),
                )
            }

            prism::Node::AliasMethodNode { .. } => {
                let node = node.as_alias_method_node().unwrap();
                self.parse_alias(node.new_name(), node.old_name())
            }
            prism::Node::AliasGlobalVariableNode { .. } => {
                let node = node.as_alias_global_variable_node().unwrap();
                self.parse_alias(node.new_name(), node.old_name())
            }

            _ => unreachable!("prism node not handled: {:?}", node),
        }
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
                let element = self.parse(node, trailing_end);
                f(element);
            },
        );
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
