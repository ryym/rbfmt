use crate::fmt;
use ruby_prism as prism;
use std::{collections::HashMap, iter::Peekable, ops::Range};

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<ParserResult> {
    let result = prism::parse(&source);

    let comments = result.comments().peekable();
    let heredoc_map = HashMap::new();

    let mut builder = FmtNodeBuilder {
        src: &source,
        comments,
        heredoc_map,
        position_gen: 0,
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(result.node());
    // dbg!(&fmt_node);
    // dbg!(&builder.heredoc_map);
    Some(ParserResult {
        node: fmt_node,
        heredoc_map: builder.heredoc_map,
    })
}

#[derive(Debug)]
pub(crate) struct ParserResult {
    pub node: fmt::Node,
    pub heredoc_map: fmt::HeredocMap,
}

struct IfOrUnless<'src> {
    is_if: bool,
    loc: prism::Location<'src>,
    predicate: prism::Node<'src>,
    statements: Option<prism::StatementsNode<'src>>,
    consequent: Option<prism::Node<'src>>,
    end_loc: Option<prism::Location<'src>>,
}

trait CallRoot {
    fn location(&self) -> prism::Location;
    fn receiver(&self) -> Option<prism::Node>;
    fn message_loc(&self) -> Option<prism::Location>;
    fn call_operator_loc(&self) -> Option<prism::Location>;
    fn name(&self) -> &[u8];
    fn arguments(&self) -> Option<prism::ArgumentsNode>;
    fn opening_loc(&self) -> Option<prism::Location>;
    fn closing_loc(&self) -> Option<prism::Location>;
    fn block(&self) -> Option<prism::Node>;
}

impl<'src> CallRoot for prism::CallNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        self.opening_loc()
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        self.closing_loc()
    }
    fn block(&self) -> Option<prism::Node> {
        self.block()
    }
}

impl<'src> CallRoot for prism::CallAndWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.read_name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}
impl<'src> CallRoot for prism::CallOrWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.read_name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}
impl<'src> CallRoot for prism::CallOperatorWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.read_name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}
impl<'src> CallRoot for prism::CallTargetNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        Some(self.receiver())
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.message_loc())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        Some(self.call_operator_loc())
    }
    fn name(&self) -> &[u8] {
        // We cannot use `name()` because it automatically has `=` suffix.
        self.message_loc().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::IndexAndWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}
impl<'src> CallRoot for prism::IndexOrWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}
impl<'src> CallRoot for prism::IndexOperatorWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}
impl<'src> CallRoot for prism::IndexTargetNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        Some(self.receiver())
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        None
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::ForwardingSuperNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        None
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        None
    }
    fn name(&self) -> &[u8] {
        b"super"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        self.block().map(|b| b.as_node())
    }
}
impl<'src> CallRoot for prism::SuperNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        None
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        None
    }
    fn name(&self) -> &[u8] {
        b"super"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        self.lparen_loc()
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        self.rparen_loc()
    }
    fn block(&self) -> Option<prism::Node> {
        self.block()
    }
}

struct Postmodifier<'src> {
    keyword: String,
    loc: prism::Location<'src>,
    keyword_loc: prism::Location<'src>,
    predicate: prism::Node<'src>,
    statements: Option<prism::StatementsNode<'src>>,
}

enum MethodType {
    Normal,      // foo(a)
    Unary,       // -a
    Binary,      // a - b
    Assign,      // a = b
    IndexAssign, // a[b] = c
}

struct FmtNodeBuilder<'src> {
    src: &'src [u8],
    comments: Peekable<prism::Comments<'src>>,
    heredoc_map: fmt::HeredocMap,
    position_gen: usize,
    last_loc_end: usize,
}

impl FmtNodeBuilder<'_> {
    fn build_fmt_node(&mut self, node: prism::Node) -> fmt::Node {
        self.visit(node, self.src.len())
    }

    fn next_pos(&mut self) -> fmt::Pos {
        self.position_gen += 1;
        fmt::Pos(self.position_gen)
    }

    fn source_lossy_at(loc: &prism::Location) -> String {
        String::from_utf8_lossy(loc.as_slice()).to_string()
    }

    fn each_node_with_next_start<'a>(
        mut nodes: impl Iterator<Item = prism::Node<'a>>,
        next_loc_start: usize,
        mut f: impl FnMut(prism::Node<'a>, usize),
    ) {
        if let Some(node) = nodes.next() {
            let mut prev = node;
            for next in nodes {
                f(prev, next.location().start_offset());
                prev = next;
            }
            f(prev, next_loc_start);
        }
    }

    fn visit(&mut self, node: prism::Node, next_loc_start: usize) -> fmt::Node {
        let loc_end = node.location().end_offset();
        let node = match node {
            prism::Node::ProgramNode { .. } => {
                let node = node.as_program_node().unwrap();
                let statements = self.visit_statements(Some(node.statements()), next_loc_start);
                let kind = fmt::Kind::Statements(statements);
                fmt::Node::without_trivia(kind)
            }
            prism::Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                let statements = self.visit_statements(Some(node), next_loc_start);
                let kind = fmt::Kind::Statements(statements);
                fmt::Node::without_trivia(kind)
            }

            prism::Node::SelfNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::NilNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::TrueNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::FalseNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::IntegerNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::FloatNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::RationalNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ImaginaryNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::LocalVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::InstanceVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ClassVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::GlobalVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::BackReferenceReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::NumberedReferenceReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ConstantReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ConstantPathNode { .. } => {
                let (leading, path, trailing) = self.visit_constant_path(node, next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Atom(path), trailing)
            }
            prism::Node::BlockLocalVariableNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ForwardingArgumentsNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::SourceFileNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::SourceLineNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::SourceEncodingNode { .. } => self.parse_atom(node, next_loc_start),

            prism::Node::StringNode { .. } => {
                let node = node.as_string_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let kind = if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let heredoc_opening = self.visit_simple_heredoc(
                        node.opening_loc(),
                        node.content_loc(),
                        node.closing_loc(),
                    );
                    fmt::Kind::HeredocOpening(heredoc_opening)
                } else {
                    let str = self.visit_string_like(
                        node.opening_loc(),
                        node.content_loc(),
                        node.closing_loc(),
                    );
                    fmt::Kind::StringLike(str)
                };
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, kind, trailing)
            }
            prism::Node::InterpolatedStringNode { .. } => {
                let node = node.as_interpolated_string_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let kind = if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let heredoc_opening = self.visit_complex_heredoc(
                        node.opening_loc(),
                        node.parts(),
                        node.closing_loc(),
                    );
                    fmt::Kind::HeredocOpening(heredoc_opening)
                } else {
                    let str = self.visit_interpolated(
                        node.opening_loc(),
                        node.parts(),
                        node.closing_loc(),
                    );
                    fmt::Kind::DynStringLike(str)
                };
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, kind, trailing)
            }

            prism::Node::XStringNode { .. } => {
                let node = node.as_x_string_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let kind = if Self::is_heredoc(Some(&node.opening_loc())) {
                    let heredoc_opening = self.visit_simple_heredoc(
                        Some(node.opening_loc()),
                        node.content_loc(),
                        Some(node.closing_loc()),
                    );
                    fmt::Kind::HeredocOpening(heredoc_opening)
                } else {
                    let str = self.visit_string_like(
                        Some(node.opening_loc()),
                        node.content_loc(),
                        Some(node.closing_loc()),
                    );
                    fmt::Kind::StringLike(str)
                };
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, kind, trailing)
            }
            prism::Node::InterpolatedXStringNode { .. } => {
                let node = node.as_interpolated_x_string_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let kind = if Self::is_heredoc(Some(&node.opening_loc())) {
                    let heredoc_opening = self.visit_complex_heredoc(
                        Some(node.opening_loc()),
                        node.parts(),
                        Some(node.closing_loc()),
                    );
                    fmt::Kind::HeredocOpening(heredoc_opening)
                } else {
                    let str = self.visit_interpolated(
                        Some(node.opening_loc()),
                        node.parts(),
                        Some(node.closing_loc()),
                    );
                    fmt::Kind::DynStringLike(str)
                };
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, kind, trailing)
            }

            prism::Node::SymbolNode { .. } => {
                let node = node.as_symbol_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                // XXX: I cannot find the case where the value_loc is None.
                let value_loc = node.value_loc().expect("symbol value must exist");
                let str = self.visit_string_like(node.opening_loc(), value_loc, node.closing_loc());
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::StringLike(str), trailing)
            }
            prism::Node::InterpolatedSymbolNode { .. } => {
                let node = node.as_interpolated_symbol_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let str =
                    self.visit_interpolated(node.opening_loc(), node.parts(), node.closing_loc());
                let kind = fmt::Kind::DynStringLike(str);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, kind, trailing)
            }

            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();
                if node.end_keyword_loc().is_some() {
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
                } else if node.then_keyword_loc().map(|l| l.as_slice()) == Some(b":") {
                    todo!("ternery if: {:?}", node);
                } else {
                    self.visit_postmodifier(
                        Postmodifier {
                            keyword: "if".to_string(),
                            loc: node.location(),
                            keyword_loc: node.if_keyword_loc().expect("if modifier must have if"),
                            predicate: node.predicate(),
                            statements: node.statements(),
                        },
                        next_loc_start,
                    )
                }
            }
            prism::Node::UnlessNode { .. } => {
                let node = node.as_unless_node().unwrap();
                if node.end_keyword_loc().is_some() {
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
                } else if node.then_keyword_loc().map(|l| l.as_slice()) == Some(b":") {
                    todo!("ternery if: {:?}", node);
                } else {
                    self.visit_postmodifier(
                        Postmodifier {
                            keyword: "unless".to_string(),
                            loc: node.location(),
                            keyword_loc: node.keyword_loc(),
                            predicate: node.predicate(),
                            statements: node.statements(),
                        },
                        next_loc_start,
                    )
                }
            }

            prism::Node::RescueModifierNode { .. } => {
                let node = node.as_rescue_modifier_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let postmod = self.visit_rescue_modifier(node);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Postmodifier(postmod), trailing)
            }

            prism::Node::CallNode { .. } => {
                let node = node.as_call_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let kind = match Self::detect_method_type(&node) {
                    MethodType::Normal => {
                        let chain = self.visit_call_root(&node, next_loc_start, None);
                        fmt::Kind::MethodChain(chain)
                    }
                    MethodType::Unary => {
                        let prefix = self.visit_prefix_call(node);
                        fmt::Kind::Prefix(prefix)
                    }
                    MethodType::Binary => {
                        let chain = self.visit_infix_call(node);
                        fmt::Kind::InfixChain(chain)
                    }
                    MethodType::Assign => {
                        let assign = self.visit_write_call(node);
                        fmt::Kind::Assign(assign)
                    }
                    MethodType::IndexAssign => {
                        let assign = self.visit_index_write_call(node);
                        fmt::Kind::Assign(assign)
                    }
                };
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, kind, trailing)
            }
            prism::Node::ForwardingSuperNode { .. } => {
                let node = node.as_forwarding_super_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let chain = self.visit_call_root(&node, next_loc_start, None);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::MethodChain(chain), trailing)
            }
            prism::Node::SuperNode { .. } => {
                let node = node.as_super_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let chain = self.visit_call_root(&node, next_loc_start, None);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::MethodChain(chain), trailing)
            }

            prism::Node::ReturnNode { .. } => {
                let node = node.as_return_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let call_like =
                    self.parse_call_like(node.keyword_loc(), node.arguments(), next_loc_start);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::CallLike(call_like), trailing)
            }

            prism::Node::AndNode { .. } => {
                let node = node.as_and_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let chain = self.visit_infix_op(node.left(), node.operator_loc(), node.right());
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::InfixChain(chain), trailing)
            }
            prism::Node::OrNode { .. } => {
                let node = node.as_or_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let chain = self.visit_infix_op(node.left(), node.operator_loc(), node.right());
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::InfixChain(chain), trailing)
            }

            prism::Node::LocalVariableWriteNode { .. } => {
                let node = node.as_local_variable_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::LocalVariableAndWriteNode { .. } => {
                let node = node.as_local_variable_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::LocalVariableOrWriteNode { .. } => {
                let node = node.as_local_variable_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::LocalVariableOperatorWriteNode { .. } => {
                let node = node.as_local_variable_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::InstanceVariableWriteNode { .. } => {
                let node = node.as_instance_variable_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::InstanceVariableAndWriteNode { .. } => {
                let node = node.as_instance_variable_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::InstanceVariableOrWriteNode { .. } => {
                let node = node.as_instance_variable_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::InstanceVariableOperatorWriteNode { .. } => {
                let node = node.as_instance_variable_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::ClassVariableWriteNode { .. } => {
                let node = node.as_class_variable_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    // XXX: When does the operator becomes None?
                    node.operator_loc().expect("must have operator"),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ClassVariableAndWriteNode { .. } => {
                let node = node.as_class_variable_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ClassVariableOrWriteNode { .. } => {
                let node = node.as_class_variable_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ClassVariableOperatorWriteNode { .. } => {
                let node = node.as_class_variable_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::GlobalVariableWriteNode { .. } => {
                let node = node.as_global_variable_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::GlobalVariableAndWriteNode { .. } => {
                let node = node.as_global_variable_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::GlobalVariableOrWriteNode { .. } => {
                let node = node.as_global_variable_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::GlobalVariableOperatorWriteNode { .. } => {
                let node = node.as_global_variable_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::ConstantWriteNode { .. } => {
                let node = node.as_constant_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ConstantAndWriteNode { .. } => {
                let node = node.as_constant_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ConstantOrWriteNode { .. } => {
                let node = node.as_constant_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ConstantOperatorWriteNode { .. } => {
                let node = node.as_constant_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::ConstantPathWriteNode { .. } => {
                let node = node.as_constant_path_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ConstantPathAndWriteNode { .. } => {
                let node = node.as_constant_path_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ConstantPathOrWriteNode { .. } => {
                let node = node.as_constant_path_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::ConstantPathOperatorWriteNode { .. } => {
                let node = node.as_constant_path_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::CallAndWriteNode { .. } => {
                let node = node.as_call_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::CallOrWriteNode { .. } => {
                let node = node.as_call_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::CallOperatorWriteNode { .. } => {
                let node = node.as_call_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::IndexAndWriteNode { .. } => {
                let node = node.as_index_and_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::IndexOrWriteNode { .. } => {
                let node = node.as_index_or_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::IndexOperatorWriteNode { .. } => {
                let node = node.as_index_operator_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }

            prism::Node::LocalVariableTargetNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::InstanceVariableTargetNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ClassVariableTargetNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::GlobalVariableTargetNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ConstantTargetNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ConstantPathTargetNode { .. } => {
                let (leading, path, trailing) = self.visit_constant_path(node, next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Atom(path), trailing)
            }
            prism::Node::CallTargetNode { .. } => {
                let node = node.as_call_target_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let chain = self.visit_call_root(&node, next_loc_start, None);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::MethodChain(chain), trailing)
            }
            prism::Node::IndexTargetNode { .. } => {
                let node = node.as_index_target_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let chain = self.visit_call_root(&node, next_loc_start, None);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::MethodChain(chain), trailing)
            }

            prism::Node::MultiWriteNode { .. } => {
                let node = node.as_multi_write_node().unwrap();
                let (leading, assign, trailing) = self.visit_multi_assign(node, next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::MultiTargetNode { .. } => {
                let node = node.as_multi_target_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let target = self.visit_multi_assign_target(
                    node.lefts(),
                    node.rest(),
                    node.rights(),
                    node.lparen_loc(),
                    node.rparen_loc(),
                    next_loc_start,
                );
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::MultiAssignTarget(target), trailing)
            }
            prism::Node::ImplicitRestNode { .. } => {
                let leading = fmt::LeadingTrivia::new();
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Atom("".to_string()), trailing)
            }

            prism::Node::SplatNode { .. } => {
                let node = node.as_splat_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let operator = Self::source_lossy_at(&node.operator_loc());
                // XXX: I cannot find the case where the expression is None.
                let expr = node.expression().expect("SplatNode must have expression");
                let expr = self.visit(expr, loc.end_offset());
                let splat = fmt::Prefix::new(operator, Some(expr));
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Prefix(splat), trailing)
            }
            prism::Node::AssocSplatNode { .. } => {
                let node = node.as_assoc_splat_node().unwrap();
                let loc = node.location();
                let leading = self.take_leading_trivia(loc.start_offset());
                let operator = Self::source_lossy_at(&node.operator_loc());
                // XXX: I cannot find the case where the value is None.
                let value = node.value().expect("AssocSplatNode must have value");
                let value = self.visit(value, loc.end_offset());
                let splat = fmt::Prefix::new(operator, Some(value));
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Prefix(splat), trailing)
            }
            prism::Node::BlockArgumentNode { .. } => {
                let node = node.as_block_argument_node().unwrap();
                let (leading, prefix, trailing) = self.visit_block_arg(node, next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Prefix(prefix), trailing)
            }

            prism::Node::ArrayNode { .. } => {
                let node = node.as_array_node().unwrap();
                let opening_loc = node.opening_loc();
                let closing_loc = node.closing_loc();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let opening = opening_loc.as_ref().map(Self::source_lossy_at);
                let closing = closing_loc.as_ref().map(Self::source_lossy_at);
                let mut array = fmt::Array::new(opening, closing);
                let closing_start = closing_loc.map(|l| l.start_offset());
                Self::each_node_with_next_start(
                    node.elements().iter(),
                    closing_start.unwrap_or(next_loc_start),
                    |node, next_start| {
                        let element = self.visit(node, next_start);
                        array.append_element(element);
                    },
                );
                if let Some(closing_start) = closing_start {
                    let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
                    array.set_virtual_end(virtual_end);
                }
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Array(array), trailing)
            }

            prism::Node::HashNode { .. } => {
                let node = node.as_hash_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let opening_loc = node.opening_loc();
                let closing_loc = node.closing_loc();
                let opening = Self::source_lossy_at(&opening_loc);
                let closing = Self::source_lossy_at(&closing_loc);
                let mut hash = fmt::Hash::new(opening, closing);
                let closing_start = closing_loc.start_offset();
                Self::each_node_with_next_start(
                    node.elements().iter(),
                    closing_start,
                    |node, next_start| {
                        let element = self.visit(node, next_start);
                        hash.append_element(element);
                    },
                );
                let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
                hash.set_virtual_end(virtual_end);
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Hash(hash), trailing)
            }
            prism::Node::KeywordHashNode { .. } => {
                let node = node.as_keyword_hash_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let mut khash = fmt::KeywordHash::new();
                Self::each_node_with_next_start(
                    node.elements().iter(),
                    // Do not consume trailing comment of the last element.
                    // Since the keyword hash places at the end of an array or arguments,
                    // the last trailing comment should be handled by its container array or arguments.
                    // If the keyword hash itself handles the last trailing comment,
                    // its container accidentally put the comma after the trailing comment.
                    0,
                    |node, next_start| {
                        let element = self.visit(node, next_start);
                        khash.append_element(element);
                    },
                );
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::KeywordHash(khash), trailing)
            }
            prism::Node::AssocNode { .. } => {
                let node = node.as_assoc_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let key = node.key();
                let key_loc = key.location();
                let key = self.visit(key, key_loc.end_offset());
                let operator = node.operator_loc().map(|l| Self::source_lossy_at(&l));
                // XXX: I cannot find the case where the value is None.
                let value = node.value().expect("assoc node must have value");
                let value_loc = value.location();
                let value = self.visit(value, value_loc.end_offset());
                let trailing = self.take_trailing_comment(next_loc_start);
                let assoc = fmt::Assoc::new(key, operator, value);
                fmt::Node::new(leading, fmt::Kind::Assoc(assoc), trailing)
            }
            prism::Node::ImplicitNode { .. } => {
                fmt::Node::without_trivia(fmt::Kind::Atom("".to_string()))
            }

            prism::Node::ParenthesesNode { .. } => {
                let node = node.as_parentheses_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let closing_start = node.closing_loc().start_offset();
                let body = node.body().map(|b| self.visit(b, closing_start));
                let body = self.wrap_as_statements(body, closing_start);
                let trailing = self.take_trailing_comment(next_loc_start);
                let parens = fmt::Parens::new(body);
                fmt::Node::new(leading, fmt::Kind::Parens(parens), trailing)
            }

            prism::Node::DefNode { .. } => {
                let node = node.as_def_node().unwrap();
                let (leading, def, trailing) = self.visit_def(node, next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Def(def), trailing)
            }
            prism::Node::NoKeywordsParameterNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ForwardingParameterNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::RequiredParameterNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::RequiredKeywordParameterNode { .. } => {
                self.parse_atom(node, next_loc_start)
            }
            prism::Node::RestParameterNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::KeywordRestParameterNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::BlockParameterNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::OptionalParameterNode { .. } => {
                let node = node.as_optional_parameter_node().unwrap();
                let (leading, assign, trailing) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::Assign(assign), trailing)
            }
            prism::Node::OptionalKeywordParameterNode { .. } => {
                let node = node.as_optional_keyword_parameter_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
                let name = Self::source_lossy_at(&node.name_loc());
                let name = fmt::Node::without_trivia(fmt::Kind::Atom(name));
                let value = node.value();
                let value_loc = value.location();
                let value = self.visit(value, value_loc.end_offset());
                let trailing = self.take_trailing_comment(next_loc_start);
                let assoc = fmt::Assoc::new(name, None, value);
                fmt::Node::new(leading, fmt::Kind::Assoc(assoc), trailing)
            }

            prism::Node::BeginNode { .. } => {
                let node = node.as_begin_node().unwrap();
                let leading = self.take_leading_trivia(node.location().start_offset());
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
                let trailing = self.take_trailing_comment(next_loc_start);
                fmt::Node::new(leading, fmt::Kind::Begin(begin), trailing)
            }

            prism::Node::ClassNode { .. } => {
                let node = node.as_class_node().unwrap();
                let (leading, class, trailing) = self.visit_class_like(
                    "class",
                    node.constant_path().location(),
                    node.superclass(),
                    node.body(),
                    node.end_keyword_loc(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::ClassLike(class), trailing)
            }
            prism::Node::ModuleNode { .. } => {
                let node = node.as_module_node().unwrap();
                let (leading, module, trailing) = self.visit_class_like(
                    "module",
                    node.constant_path().location(),
                    None,
                    node.body(),
                    node.end_keyword_loc(),
                    next_loc_start,
                );
                fmt::Node::new(leading, fmt::Kind::ClassLike(module), trailing)
            }
            prism::Node::SingletonClassNode { .. } => {
                let node = node.as_singleton_class_node().unwrap();
                let leading = self.take_leading_trivia(node.operator_loc().start_offset());
                let body = node.body();
                let end_loc = node.end_keyword_loc();
                let expr_next = body
                    .as_ref()
                    .map(|b| b.location().start_offset())
                    .unwrap_or(end_loc.start_offset());
                let expr = self.visit(node.expression(), expr_next);
                let body = self.parse_block_body(body, end_loc.start_offset());
                let trailing = self.take_trailing_comment(next_loc_start);
                let class = fmt::SingletonClass {
                    expression: Box::new(expr),
                    body,
                };
                fmt::Node::new(leading, fmt::Kind::SingletonClass(class), trailing)
            }

            _ => todo!("parse {:?}", node),
        };

        // XXX: We should take trailing comment after setting the last location end.
        self.last_loc_end = loc_end;
        node
    }

    fn parse_atom(&mut self, node: prism::Node, next_loc_start: usize) -> fmt::Node {
        let loc = node.location();
        let leading = self.take_leading_trivia(loc.start_offset());
        let value = Self::source_lossy_at(&loc);
        let trailing = self.take_trailing_comment(next_loc_start);
        fmt::Node::new(leading, fmt::Kind::Atom(value), trailing)
    }

    fn visit_constant_path(
        &mut self,
        node: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, String, fmt::TrailingTrivia) {
        let loc = node.location();

        // Use `end_offset` to treat any trivia inside the path as leading trivia for simplicity.
        // e.g. "Foo::\n#comment\nBar" -> "#comment\nFoo::Bar"
        let leading_trivia = self.take_leading_trivia(loc.end_offset());

        fn rec(node: Option<prism::Node>, parts: &mut Vec<u8>) {
            if let Some(node) = node {
                match node {
                    prism::Node::ConstantReadNode { .. } => {
                        let node = node.as_constant_read_node().unwrap();
                        parts.extend(node.location().as_slice());
                    }
                    prism::Node::ConstantPathNode { .. } => {
                        let node = node.as_constant_path_node().unwrap();
                        rec(node.parent(), parts);
                        parts.extend_from_slice(node.delimiter_loc().as_slice());
                        parts.extend_from_slice(node.child().location().as_slice());
                    }
                    _ => panic!("unexpected constant path part: {:?}", node),
                }
            }
        }
        let mut parts = vec![];
        rec(Some(node), &mut parts);

        let path = String::from_utf8_lossy(&parts).to_string();
        let trailing_trivia = self.take_trailing_comment(next_loc_start);
        (leading_trivia, path, trailing_trivia)
    }

    fn visit_string_like(
        &mut self,
        opening_loc: Option<prism::Location>,
        value_loc: prism::Location,
        closing_loc: Option<prism::Location>,
    ) -> fmt::StringLike {
        let value = Self::source_lossy_at(&value_loc);
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        fmt::StringLike::new(opening, value.into(), closing)
    }

    fn visit_interpolated(
        &mut self,
        opening_loc: Option<prism::Location>,
        parts: prism::NodeList,
        closing_loc: Option<prism::Location>,
    ) -> fmt::DynStringLike {
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let mut dstr = fmt::DynStringLike::new(opening, closing);
        for part in parts.iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.visit_string_like(
                        node.opening_loc(),
                        node.content_loc(),
                        node.closing_loc(),
                    );
                    dstr.append_part(fmt::DynStrPart::Str(str));
                    self.last_loc_end = node_end;
                }
                prism::Node::InterpolatedStringNode { .. } => {
                    let node = part.as_interpolated_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.visit_interpolated(
                        node.opening_loc(),
                        node.parts(),
                        node.closing_loc(),
                    );
                    dstr.append_part(fmt::DynStrPart::DynStr(str));
                    self.last_loc_end = node_end;
                }
                prism::Node::EmbeddedStatementsNode { .. } => {
                    let node = part.as_embedded_statements_node().unwrap();
                    let loc = node.location();
                    self.last_loc_end = node.opening_loc().end_offset();
                    let statements = self.visit_statements(node.statements(), loc.end_offset());
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    let embedded_stmts = fmt::EmbeddedStatements::new(opening, statements, closing);
                    dstr.append_part(fmt::DynStrPart::Statements(embedded_stmts));
                }
                prism::Node::EmbeddedVariableNode { .. } => {
                    let node = part.as_embedded_variable_node().unwrap();
                    let operator = Self::source_lossy_at(&node.operator_loc());
                    let variable = Self::source_lossy_at(&node.variable().location());
                    let embedded_var = fmt::EmbeddedVariable::new(operator, variable);
                    dstr.append_part(fmt::DynStrPart::Variable(embedded_var));
                }
                _ => panic!("unexpected string interpolation node: {:?}", part),
            }
        }
        dstr
    }

    fn is_heredoc(str_opening_loc: Option<&prism::Location>) -> bool {
        if let Some(loc) = str_opening_loc {
            let bytes = loc.as_slice();
            bytes.len() > 2 && bytes[0] == b'<' && bytes[1] == b'<'
        } else {
            false
        }
    }

    fn visit_simple_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        content_loc: prism::Location,
        closing_loc: Option<prism::Location>,
    ) -> fmt::HeredocOpening {
        let open = opening_loc.as_ref().unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let opening_id = String::from_utf8_lossy(id).to_string();
        let closing_loc = closing_loc.expect("heredoc must have closing");
        let closing_id = Self::source_lossy_at(&closing_loc)
            .trim_start()
            .trim_end_matches('\n')
            .to_string();
        let str = self.visit_string_like(None, content_loc, None);
        let heredoc = fmt::Heredoc {
            id: closing_id,
            indent_mode,
            parts: vec![fmt::HeredocPart::Str(str)],
        };
        let pos = self.next_pos();
        self.heredoc_map.insert(pos, heredoc);
        fmt::HeredocOpening::new(pos, opening_id, indent_mode)
    }

    fn visit_complex_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        content_parts: prism::NodeList,
        closing_loc: Option<prism::Location>,
    ) -> fmt::HeredocOpening {
        let open = opening_loc.unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let opening_id = String::from_utf8_lossy(id).to_string();
        let mut parts = vec![];
        let mut last_str_end: Option<usize> = None;
        for part in content_parts.iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.visit_string_like(
                        node.opening_loc(),
                        node.content_loc(),
                        node.closing_loc(),
                    );
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
                            let str = fmt::StringLike::new(None, value, None);
                            parts.push(fmt::HeredocPart::Str(str))
                        }
                    }
                    let statements = self.visit_statements(node.statements(), loc.end_offset());
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    let embedded = fmt::EmbeddedStatements::new(opening, statements, closing);
                    parts.push(fmt::HeredocPart::Statements(embedded));
                }
                prism::Node::EmbeddedVariableNode { .. } => {
                    let node = part.as_embedded_variable_node().unwrap();
                    let operator = Self::source_lossy_at(&node.operator_loc());
                    let variable = Self::source_lossy_at(&node.variable().location());
                    let embedded_var = fmt::EmbeddedVariable::new(operator, variable);
                    parts.push(fmt::HeredocPart::Variable(embedded_var));
                }
                _ => panic!("unexpected heredoc part: {:?}", part),
            }
        }
        let closing_loc = closing_loc.expect("heredoc must have closing");
        let closing_id = Self::source_lossy_at(&closing_loc)
            .trim_start()
            .trim_end_matches('\n')
            .to_string();
        let heredoc = fmt::Heredoc {
            id: closing_id,
            indent_mode,
            parts,
        };
        let pos = self.next_pos();
        self.heredoc_map.insert(pos, heredoc);
        fmt::HeredocOpening::new(pos, opening_id, indent_mode)
    }

    fn visit_statements(
        &mut self,
        node: Option<prism::StatementsNode>,
        end: usize,
    ) -> fmt::Statements {
        let mut statements = fmt::Statements::new();
        if let Some(node) = node {
            Self::each_node_with_next_start(node.body().iter(), end, |prev, next_start| {
                let fmt_node = self.visit(prev, next_start);
                statements.append_node(fmt_node);
            });
        }
        let virtual_end = self.take_end_trivia_as_virtual_end(Some(end));
        statements.set_virtual_end(virtual_end);
        statements
    }

    fn visit_if_or_unless(&mut self, node: IfOrUnless, next_loc_start: usize) -> fmt::Node {
        let if_leading = self.take_leading_trivia(node.loc.start_offset());

        let end_loc = node.end_loc.expect("if/unless expression must have end");
        let end_start = end_loc.start_offset();

        let if_next_loc = node.predicate.location().start_offset();
        let keyword_trailing = self.take_trailing_comment(if_next_loc);

        let conseq = node.consequent;
        let next_pred_loc_start = node
            .statements
            .as_ref()
            .map(|s| s.location())
            .or_else(|| conseq.as_ref().map(|c| c.location()))
            .map(|l| l.start_offset())
            .unwrap_or(end_loc.start_offset());
        let predicate = self.visit(node.predicate, next_pred_loc_start);

        let ifexpr = match conseq {
            // if...(elsif...|else...)+end
            Some(conseq) => {
                // take trailing of else/elsif
                let else_start = conseq.location().start_offset();
                let body = self.visit_statements(node.statements, else_start);
                let if_first = fmt::Conditional::new(keyword_trailing, predicate, body);
                let mut ifexpr = fmt::If::new(node.is_if, if_first);
                self.visit_ifelse(conseq, &mut ifexpr);
                ifexpr
            }
            // if...end
            None => {
                let body = self.visit_statements(node.statements, end_start);
                let if_first = fmt::Conditional::new(keyword_trailing, predicate, body);
                fmt::If::new(node.is_if, if_first)
            }
        };

        let if_trailing = self.take_trailing_comment(next_loc_start);
        fmt::Node::new(if_leading, fmt::Kind::If(ifexpr), if_trailing)
    }

    fn visit_ifelse(&mut self, node: prism::Node, ifexpr: &mut fmt::If) {
        match node {
            // elsif ("if" only, "unles...elsif" is syntax error)
            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();

                let end_loc = node
                    .end_keyword_loc()
                    .expect("if/unless expression must have end");

                let elsif_next_loc = node
                    .statements()
                    .as_ref()
                    .map(|s| s.location().start_offset())
                    .unwrap_or(end_loc.start_offset());
                let keyword_trailing = self.take_trailing_comment(elsif_next_loc);

                let conseq = node.consequent();
                let next_loc_start = node
                    .statements()
                    .map(|s| s.location())
                    .or_else(|| conseq.as_ref().map(|c| c.location()))
                    .map(|l| l.start_offset())
                    .unwrap_or(end_loc.start_offset());
                let predicate = self.visit(node.predicate(), next_loc_start);

                let body_end_loc = conseq
                    .as_ref()
                    .map(|n| n.location().start_offset())
                    .unwrap_or(end_loc.start_offset());
                let body = self.visit_statements(node.statements(), body_end_loc);

                let conditional = fmt::Conditional::new(keyword_trailing, predicate, body);
                ifexpr.elsifs.push(conditional);
                if let Some(conseq) = conseq {
                    self.visit_ifelse(conseq, ifexpr);
                }
            }
            // else
            prism::Node::ElseNode { .. } => {
                let node = node.as_else_node().unwrap();

                let end_loc = node
                    .end_keyword_loc()
                    .expect("if/unless expression must have end");

                let else_next_loc = node
                    .statements()
                    .as_ref()
                    .map(|s| s.location().start_offset())
                    .unwrap_or(end_loc.start_offset());
                let keyword_trailing = self.take_trailing_comment(else_next_loc);

                let body = self.visit_statements(node.statements(), end_loc.start_offset());
                ifexpr.if_last = Some(fmt::Else {
                    keyword_trailing,
                    body,
                });
            }
            _ => {
                panic!("unexpected node in IfNode: {:?}", node);
            }
        }
    }

    fn visit_postmodifier(&mut self, postmod: Postmodifier, next_loc_start: usize) -> fmt::Node {
        let leading = self.take_leading_trivia(postmod.loc.start_offset());

        let kwd_loc = postmod.keyword_loc;
        let statements = self.visit_statements(postmod.statements, kwd_loc.start_offset());

        let pred_loc = postmod.predicate.location();
        let keyword_trailing = self.take_trailing_comment(pred_loc.start_offset());

        let predicate = self.visit(postmod.predicate, next_loc_start);

        let postmod = fmt::Postmodifier::new(
            postmod.keyword,
            fmt::Conditional::new(keyword_trailing, predicate, statements),
        );

        let trailing = self.take_trailing_comment(next_loc_start);
        fmt::Node::new(leading, fmt::Kind::Postmodifier(postmod), trailing)
    }

    fn visit_rescue_modifier(&mut self, node: prism::RescueModifierNode) -> fmt::Postmodifier {
        let kwd_loc = node.keyword_loc();
        let expr = self.visit(node.expression(), kwd_loc.start_offset());
        let statements = self.wrap_as_statements(Some(expr), kwd_loc.start_offset());

        let rescue_expr = node.rescue_expression();
        let rescue_expr_loc = rescue_expr.location();
        let keyword_trailing = self.take_trailing_comment(rescue_expr_loc.start_offset());
        let rescue_expr = self.visit(rescue_expr, rescue_expr_loc.end_offset());

        fmt::Postmodifier::new(
            "rescue".to_string(),
            fmt::Conditional::new(keyword_trailing, rescue_expr, statements),
        )
    }

    fn visit_call_root<C: CallRoot>(
        &mut self,
        call: &C,
        next_loc_start: usize,
        next_msg_start: Option<usize>,
    ) -> fmt::MethodChain {
        let mut chain = match call.receiver() {
            Some(receiver) => match receiver {
                prism::Node::CallNode { .. } => {
                    let node = receiver.as_call_node().unwrap();
                    let msg_end = call.message_loc().as_ref().map(|l| l.start_offset());
                    self.visit_call_root(&node, next_loc_start, msg_end)
                }
                _ => {
                    let next_loc_start = call
                        .message_loc()
                        .or_else(|| call.opening_loc())
                        .or_else(|| call.arguments().map(|a| a.location()))
                        .or_else(|| call.block().map(|a| a.location()))
                        .map(|l| l.start_offset())
                        .or(next_msg_start)
                        .unwrap_or(next_loc_start);
                    let recv = self.visit(receiver, next_loc_start);
                    fmt::MethodChain::new(Some(recv))
                }
            },
            None => fmt::MethodChain::new(None),
        };

        let call_leading = if let Some(msg_loc) = call.message_loc() {
            self.take_leading_trivia(msg_loc.start_offset())
        } else {
            fmt::LeadingTrivia::new()
            // foo.\n#hoge\n(2)
        };

        let call_op = call.call_operator_loc().map(|l| Self::source_lossy_at(&l));
        let name = String::from_utf8_lossy(call.name()).to_string();
        let mut method_call = fmt::MethodCall::new(call_leading, call_op, name);

        let arguments = call.arguments();
        let block = call.block();
        let opening_loc = call.opening_loc();
        let closing_loc = call.closing_loc();
        let closing_next_start = block
            .as_ref()
            .map(|b| b.location().start_offset())
            .or(next_msg_start)
            .unwrap_or(next_loc_start);
        let (args, block) = match block {
            Some(node) => match node {
                // method call with block literal (e.g. "foo { a }", "foo(a) { b }")
                prism::Node::BlockNode { .. } => {
                    let args = self.visit_arguments(
                        arguments,
                        None,
                        opening_loc,
                        closing_loc,
                        closing_next_start,
                    );
                    let block = node.as_block_node().unwrap();
                    let block = self.visit_block(block);
                    (args, Some(block))
                }
                // method call with a block argument (e.g. "foo(&a)", "foo(a, &b)")
                prism::Node::BlockArgumentNode { .. } => {
                    let block_arg = node.as_block_argument_node().unwrap();
                    let args = self.visit_arguments(
                        arguments,
                        Some(block_arg),
                        opening_loc,
                        closing_loc,
                        closing_next_start,
                    );
                    (args, None)
                }
                _ => panic!("unexpected block node of call: {:?}", node),
            },
            // method call without block (e.g. "foo", "foo(a)")
            None => {
                let args = self.visit_arguments(
                    arguments,
                    None,
                    opening_loc,
                    closing_loc,
                    closing_next_start,
                );
                (args, None)
            }
        };
        if let Some(args) = args {
            method_call.set_args(args);
        }
        if let Some(block) = block {
            method_call.set_block(block);
        }

        if let Some(next_msg_start) = next_msg_start {
            let call_trailing = self.take_trailing_comment(next_msg_start);
            method_call.set_trailing_trivia(call_trailing);
        }

        chain.append_call(method_call);
        self.last_loc_end = call.location().end_offset();
        chain
    }

    fn visit_arguments(
        &mut self,
        node: Option<prism::ArgumentsNode>,
        block_arg: Option<prism::BlockArgumentNode>,
        opening_loc: Option<prism::Location>,
        closing_loc: Option<prism::Location>,
        closing_next_start: usize,
    ) -> Option<fmt::Arguments> {
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let closing_start = closing_loc.as_ref().map(|l| l.start_offset());
        match node {
            None => {
                let block_arg = block_arg.map(|block_arg| {
                    let next_start = closing_start.unwrap_or(closing_next_start);
                    let (leading, prefix, trailing) = self.visit_block_arg(block_arg, next_start);
                    fmt::Node::new(leading, fmt::Kind::Prefix(prefix), trailing)
                });
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
                let has_closing = closing.is_some();
                let mut args = fmt::Arguments::new(opening, closing);
                let next_loc_start = closing_start.unwrap_or(closing_next_start);
                let mut nodes = args_node.arguments().iter().collect::<Vec<_>>();
                if let Some(block_arg) = block_arg {
                    nodes.push(block_arg.as_node());
                }
                Self::each_node_with_next_start(
                    nodes.into_iter(),
                    next_loc_start,
                    |node, next_start| {
                        let is_last = next_start == next_loc_start;
                        if is_last {
                            args.last_comma_allowed = !matches!(
                                node,
                                prism::Node::ForwardingArgumentsNode { .. }
                                    | prism::Node::BlockArgumentNode { .. }
                            );
                        }
                        let next_start = if is_last && !has_closing {
                            node.location().end_offset()
                        } else {
                            next_start
                        };
                        let fmt_node = self.visit(node, next_start);
                        args.append_node(fmt_node);
                    },
                );
                let virtual_end = self.take_end_trivia_as_virtual_end(closing_start);
                args.set_virtual_end(virtual_end);
                Some(args)
            }
        }
    }

    fn visit_block_arg(
        &mut self,
        node: prism::BlockArgumentNode,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::Prefix, fmt::TrailingTrivia) {
        let leading = self.take_leading_trivia(node.location().start_offset());
        let operator = Self::source_lossy_at(&node.operator_loc());
        let expr = node.expression().map(|expr| {
            let expr_end = expr.location().end_offset();
            self.visit(expr, expr_end)
        });
        let trailing = self.take_trailing_comment(next_loc_start);
        let prefix = fmt::Prefix::new(operator, expr);
        (leading, prefix, trailing)
    }

    fn visit_block(&mut self, node: prism::BlockNode) -> fmt::MethodBlock {
        let loc = node.location();
        let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());
        let mut method_block = fmt::MethodBlock::new(was_flat);

        let body = node.body();
        let body_start = body.as_ref().map(|b| b.location().start_offset());
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
        let body = body.map(|n| self.visit(n, body_end_loc));
        // XXX: Is this necessary? I cannot find the case where the body is not a StatementNode.
        let body = self.wrap_as_statements(body, body_end_loc);
        method_block.set_body(body);

        method_block
    }

    fn detect_method_type(call: &prism::CallNode) -> MethodType {
        if call.receiver().is_some()
            && call.call_operator_loc().is_none()
            && call.opening_loc().is_none()
        {
            return if call.arguments().is_some() {
                MethodType::Binary
            } else {
                MethodType::Unary
            };
        }
        let method_name = call.name().as_slice();
        if method_name[method_name.len() - 1] == b'='
            && method_name != b"=="
            && method_name != b"==="
            && method_name != b"<="
            && method_name != b">="
        {
            return if method_name == b"[]=" {
                MethodType::IndexAssign
            } else {
                MethodType::Assign
            };
        }
        MethodType::Normal
    }

    fn visit_write_call(&mut self, call: prism::CallNode) -> fmt::Assign {
        let msg_loc = call.message_loc().expect("call write must have message");
        let receiver = call.receiver().expect("call write must have receiver");
        let receiver = self.visit(receiver, msg_loc.start_offset());
        let call_op = call.call_operator_loc().as_ref().map(Self::source_lossy_at);

        let call_leading = self.take_leading_trivia(msg_loc.start_offset());
        let mut name = String::from_utf8_lossy(call.name().as_slice()).to_string();
        name.truncate(name.len() - 1); // Remove '='
        let setter_call = fmt::MethodCall::new(call_leading, call_op, name);

        let arg = call
            .arguments()
            .and_then(|args| args.arguments().iter().next())
            .expect("call write must have argument");

        let mut chain = fmt::MethodChain::new(Some(receiver));
        chain.append_call(setter_call);
        let left = fmt::Node::without_trivia(fmt::Kind::MethodChain(chain));
        let arg_end = arg.location().end_offset();
        let right = self.visit(arg, arg_end);
        let operator = "=".to_string();
        fmt::Assign::new(left, operator, right)
    }

    fn visit_index_write_call(&mut self, call: prism::CallNode) -> fmt::Assign {
        let (opening_loc, closing_loc) = match (call.opening_loc(), call.closing_loc()) {
            (Some(op), Some(cl)) => (op, cl),
            _ => panic!("index write must have opening and closing"),
        };

        let receiver = call.receiver().expect("index write must have receiver");
        let receiver = self.visit(receiver, opening_loc.start_offset());

        let args = call.arguments().expect("index write must have arguments");
        let mut args_iter = args.arguments().iter();
        let (arg1, arg2) = match (args_iter.next(), args_iter.next(), args_iter.next()) {
            (Some(arg1), Some(arg2), None) => (arg1, arg2),
            _ => panic!("index write must have exactly two arguments"),
        };

        let index_call_leading = fmt::LeadingTrivia::new();
        let mut index_call = fmt::MethodCall::new(index_call_leading, None, "[]".to_string());
        let mut left_args = fmt::Arguments::new(Some("[".to_string()), Some("]".to_string()));
        let closing_start = closing_loc.start_offset();
        left_args.append_node(self.visit(arg1, closing_start));
        let left_args_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
        left_args.set_virtual_end(left_args_end);
        index_call.set_args(left_args);

        let mut chain = fmt::MethodChain::new(Some(receiver));
        chain.append_call(index_call);
        let left = fmt::Node::without_trivia(fmt::Kind::MethodChain(chain));
        let arg2_end = arg2.location().end_offset();
        let right = self.visit(arg2, arg2_end);
        let operator = "=".to_string();
        fmt::Assign::new(left, operator, right)
    }

    fn visit_block_parameters(
        &mut self,
        node: prism::BlockParametersNode,
        next_loc_start: usize,
    ) -> fmt::BlockParameters {
        // XXX: I cannot find the case where the block parameters bars don't exist.
        let (opening_loc, closing_loc) = match (node.opening_loc(), node.closing_loc()) {
            (Some(op), Some(cl)) => (op, cl),
            _ => panic!("block parameters must have opening and closing"),
        };
        let opening = Self::source_lossy_at(&opening_loc);
        let closing = Self::source_lossy_at(&closing_loc);
        let mut block_params = fmt::BlockParameters::new(opening, closing);

        let locals = node.locals();
        let params_next = locals
            .iter()
            .next()
            .map(|n| n.location().start_offset())
            .unwrap_or(closing_loc.start_offset());

        if let Some(params) = node.parameters() {
            self.visit_parameter_nodes(params, params_next, |node| {
                block_params.append_param(node);
            });
        }
        Self::each_node_with_next_start(
            locals.iter(),
            closing_loc.start_offset(),
            |node, next_start| {
                let fmt_node = self.visit(node, next_start);
                block_params.append_local(fmt_node);
            },
        );

        let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_loc.start_offset()));
        block_params.set_virtual_end(virtual_end);

        let trailing = self.take_trailing_comment(next_loc_start);
        block_params.set_closing_trailing(trailing);
        block_params
    }

    fn visit_prefix_call(&mut self, call: prism::CallNode) -> fmt::Prefix {
        let msg_loc = call
            .message_loc()
            .expect("prefix operation must have message");
        let operator = Self::source_lossy_at(&msg_loc);
        let receiver = call
            .receiver()
            .expect("prefix operation must have receiver");
        let receiver_end = receiver.location().end_offset();
        let receiver = self.visit(receiver, receiver_end);
        fmt::Prefix::new(operator, Some(receiver))
    }

    fn visit_infix_call(&mut self, call: prism::CallNode) -> fmt::InfixChain {
        let msg_loc = call
            .message_loc()
            .expect("infix operation must have message");
        let receiver = call.receiver().expect("infix operation must have receiver");
        let right = call
            .arguments()
            .and_then(|args| args.arguments().iter().next())
            .expect("infix operation must have argument");
        self.visit_infix_op(receiver, msg_loc, right)
    }

    fn visit_infix_op(
        &mut self,
        left: prism::Node,
        operator_loc: prism::Location,
        right: prism::Node,
    ) -> fmt::InfixChain {
        let left = self.visit(left, operator_loc.start_offset());
        let mut chain = match left.kind {
            fmt::Kind::InfixChain(chain) => chain,
            _ => fmt::InfixChain::new(left),
        };
        let operator = Self::source_lossy_at(&operator_loc);
        let right_end = right.location().end_offset();
        let right = self.visit(right, right_end);
        chain.append_right(operator, right);
        chain
    }

    fn parse_call_like(
        &mut self,
        name_loc: prism::Location,
        arguments: Option<prism::ArgumentsNode>,
        next_loc_start: usize,
    ) -> fmt::CallLike {
        let name = Self::source_lossy_at(&name_loc);
        let mut call_like = fmt::CallLike::new(name);
        let args = self.visit_arguments(arguments, None, None, None, next_loc_start);
        if let Some(args) = args {
            call_like.set_arguments(args);
        }
        call_like
    }

    fn visit_variable_assign(
        &mut self,
        node_loc: prism::Location,
        name_loc: prism::Location,
        operator_loc: prism::Location,
        value: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::Assign, fmt::TrailingTrivia) {
        let leading = self.take_leading_trivia(node_loc.start_offset());
        let name = Self::source_lossy_at(&name_loc);
        let operator = Self::source_lossy_at(&operator_loc);
        // Pass 0 to associate trailing trivia to the Assign kind, not the value.
        let value_end = value.location().end_offset();
        let value = self.visit(value, value_end);
        let trailing = self.take_trailing_comment(next_loc_start);
        let target = fmt::Node::without_trivia(fmt::Kind::Atom(name));
        (leading, fmt::Assign::new(target, operator, value), trailing)
    }

    fn visit_constant_path_assign(
        &mut self,
        const_path: prism::ConstantPathNode,
        operator_loc: prism::Location,
        value: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::Assign, fmt::TrailingTrivia) {
        let (leading, path, _) =
            self.visit_constant_path(const_path.as_node(), operator_loc.start_offset());
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, next_loc_start);
        let trailing = self.take_trailing_comment(next_loc_start);
        let target = fmt::Node::without_trivia(fmt::Kind::Atom(path));
        (leading, fmt::Assign::new(target, operator, value), trailing)
    }

    fn visit_call_assign(
        &mut self,
        call: &impl CallRoot,
        operator_loc: prism::Location,
        value: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::Assign, fmt::TrailingTrivia) {
        let leading = self.take_leading_trivia(call.location().start_offset());
        let chain = self.visit_call_root(call, next_loc_start, None);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, next_loc_start);
        let trailing = self.take_trailing_comment(next_loc_start);
        let target = fmt::Node::without_trivia(fmt::Kind::MethodChain(chain));
        (leading, fmt::Assign::new(target, operator, value), trailing)
    }

    fn visit_multi_assign(
        &mut self,
        node: prism::MultiWriteNode,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::Assign, fmt::TrailingTrivia) {
        let leading = self.take_leading_trivia(node.location().start_offset());
        let target = self.visit_multi_assign_target(
            node.lefts(),
            node.rest(),
            node.rights(),
            node.lparen_loc(),
            node.rparen_loc(),
            node.operator_loc().start_offset(),
        );
        let operator = Self::source_lossy_at(&node.operator_loc());
        let value = self.visit(node.value(), next_loc_start);
        let trailing = self.take_trailing_comment(next_loc_start);

        let target = fmt::Node::without_trivia(fmt::Kind::MultiAssignTarget(target));
        (leading, fmt::Assign::new(target, operator, value), trailing)
    }

    fn visit_multi_assign_target(
        &mut self,
        lefts: prism::NodeList,
        rest: Option<prism::Node>,
        rights: prism::NodeList,
        lparen_loc: Option<prism::Location>,
        rparen_loc: Option<prism::Location>,
        next_loc_start: usize,
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

        let left_next_start = rest_start
            .or(rights_first_start)
            .or(rparen_start)
            .unwrap_or(next_loc_start);
        Self::each_node_with_next_start(lefts.iter(), left_next_start, |node, next_start| {
            let target = self.visit(node, next_start);
            multi.append_target(target);
        });

        if !implicit_rest {
            if let Some(rest) = rest {
                let rest_next_start = rights_first_start
                    .or(rparen_start)
                    .unwrap_or(next_loc_start);
                let target = self.visit(rest, rest_next_start);
                multi.append_target(target);
            }
        }

        let right_next_start = rparen_start.unwrap_or(next_loc_start);
        Self::each_node_with_next_start(rights.iter(), right_next_start, |node, next_start| {
            let target = self.visit(node, next_start);
            multi.append_target(target);
        });

        if let Some(rparen_loc) = rparen_loc {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(rparen_loc.start_offset()));
            multi.set_virtual_end(virtual_end);
        }

        multi
    }

    fn visit_def(
        &mut self,
        node: prism::DefNode,
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::Def, fmt::TrailingTrivia) {
        let receiver = node.receiver();
        let name_loc = node.name_loc();

        // Take leading trivia of receiver or method name.
        let leading_end = receiver
            .as_ref()
            .map(|r| r.location().start_offset())
            .unwrap_or_else(|| name_loc.start_offset());
        let leading = self.take_leading_trivia(leading_end);

        let receiver = receiver.map(|r| self.visit(r, name_loc.end_offset()));
        let name = Self::source_lossy_at(&node.name_loc());
        let mut def = fmt::Def::new(receiver, name);

        let lparen_loc = node.lparen_loc();
        let rparen_loc = node.rparen_loc();
        if let Some(params) = node.parameters() {
            let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
            let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
            let mut parameters = fmt::MethodParameters::new(lparen, rparen);
            let params_next = rparen_loc.as_ref().map(|l| l.start_offset()).unwrap_or(0);
            self.visit_parameter_nodes(params, params_next, |node| {
                parameters.append_param(node);
            });
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(params_next));
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
            let body = self.visit(body, 0);
            def.set_body(fmt::DefBody::Short {
                body: Box::new(body),
            });
        } else {
            let end_loc = node.end_keyword_loc().expect("block def must have end");
            let body = node.body();
            let head_next = body
                .as_ref()
                .map(|b| b.location().start_offset())
                .unwrap_or(end_loc.start_offset());
            let head_trailing = self.take_trailing_comment(head_next);
            let block_body = self.parse_block_body(body, end_loc.start_offset());
            def.set_body(fmt::DefBody::Block {
                head_trailing,
                body: block_body,
            });
        }

        let trailing = self.take_trailing_comment(next_loc_start);

        (leading, def, trailing)
    }

    fn parse_block_body(
        &mut self,
        body: Option<prism::Node>,
        next_loc_start: usize,
    ) -> fmt::BlockBody {
        match body {
            Some(body) => match body {
                prism::Node::StatementsNode { .. } => {
                    let stmts = body.as_statements_node().unwrap();
                    let statements = self.visit_statements(Some(stmts), next_loc_start);
                    fmt::BlockBody::new(statements)
                }
                prism::Node::BeginNode { .. } => {
                    let node = body.as_begin_node().unwrap();
                    self.visit_begin_body(node)
                }
                _ => panic!("unexpected def body: {:?}", body),
            },
            None => {
                let statements = self.wrap_as_statements(None, next_loc_start);
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
        let statements = self.visit_statements(node.statements(), statements_next);
        let mut body = fmt::BlockBody::new(statements);

        if let Some(rescue_node) = node.rescue_clause() {
            let rescues_next = else_start
                .or(ensure_start)
                .unwrap_or(end_loc.start_offset());
            let mut rescues = vec![];
            self.visit_rescue_chain(rescue_node, &mut rescues, rescues_next);
            body.rescues = rescues;
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
            let else_statements = self.visit_statements(statements, else_next);
            body.rescue_else = Some(fmt::Else {
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
            let ensure_statements = self.visit_statements(statements, end_loc.start_offset());
            body.ensure = Some(fmt::Else {
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
        Self::each_node_with_next_start(node.exceptions().iter(), head_next, |node, next_start| {
            let fmt_node = self.visit(node, next_start);
            rescue.append_exception(fmt_node);
        });

        if let Some(reference) = node.reference() {
            let reference_next = statements_start.or(consequent_start).unwrap_or(final_next);
            let reference = self.visit(reference, reference_next);
            rescue.set_reference(reference);
        }

        let statements_next = consequent_start.unwrap_or(final_next);
        let statements = self.visit_statements(statements, statements_next);
        rescue.set_statements(statements);
        rescues.push(rescue);

        if let Some(consequent) = consequent {
            self.visit_rescue_chain(consequent, rescues, final_next);
        }
    }

    fn visit_parameter_nodes(
        &mut self,
        params: prism::ParametersNode,
        next_loc_start: usize,
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
        Self::each_node_with_next_start(nodes.into_iter(), next_loc_start, |node, next_start| {
            let fmt_node = self.visit(node, next_start);
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
        next_loc_start: usize,
    ) -> (fmt::LeadingTrivia, fmt::ClassLike, fmt::TrailingTrivia) {
        let leading = self.take_leading_trivia(name_loc.start_offset());
        let name = Self::source_lossy_at(&name_loc);

        let head_next = body
            .as_ref()
            .map(|b| b.location().start_offset())
            .unwrap_or(end_loc.start_offset());
        let (superclass, head_trailing) = if let Some(superclass) = superclass {
            let fmt_node = self.visit(superclass, head_next);
            (Some(fmt_node), fmt::TrailingTrivia::none())
        } else {
            let head_trailing = self.take_trailing_comment(head_next);
            (None, head_trailing)
        };

        let body = self.parse_block_body(body, end_loc.start_offset());
        let trailing = self.take_trailing_comment(next_loc_start);
        let class = fmt::ClassLike {
            keyword: keyword.to_string(),
            name,
            superclass: superclass.map(Box::new),
            head_trailing,
            body,
        };
        (leading, class, trailing)
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

    fn take_trailing_comment(&mut self, next_loc_start: usize) -> fmt::TrailingTrivia {
        if let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if (self.last_loc_end..=next_loc_start).contains(&loc.start_offset())
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
