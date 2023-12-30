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

struct Postmodifier<'src> {
    keyword: String,
    loc: prism::Location<'src>,
    keyword_loc: prism::Location<'src>,
    predicate: prism::Node<'src>,
    statements: Option<prism::StatementsNode<'src>>,
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

    fn each_node_with_next_start(
        mut nodes: prism::NodeListIter,
        next_loc_start: usize,
        mut f: impl FnMut(prism::Node, usize),
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
                let exprs = self.visit_statements(Some(node.statements()), next_loc_start);
                fmt::Node {
                    pos: fmt::Pos::none(),
                    trivia: fmt::Trivia::new(),
                    width: exprs.width(),
                    kind: fmt::Kind::Exprs(exprs),
                }
            }
            prism::Node::StatementsNode { .. } => {
                let node = node.as_statements_node().unwrap();
                let exprs = self.visit_statements(Some(node), next_loc_start);
                fmt::Node {
                    pos: fmt::Pos::none(),
                    trivia: fmt::Trivia::new(),
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
            prism::Node::LocalVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::InstanceVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ClassVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::GlobalVariableReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ConstantReadNode { .. } => self.parse_atom(node, next_loc_start),
            prism::Node::ConstantPathNode { .. } => {
                let pos = self.next_pos();
                let (path, trivia) = self.visit_constant_path(node, next_loc_start);
                fmt::Node::new(pos, trivia, fmt::Kind::Atom(path))
            }

            prism::Node::StringNode { .. } => {
                let node = node.as_string_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let mut trivia = self.take_leading_trivia(loc.start_offset());
                let kind = if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let heredoc_opening = self.visit_simple_heredoc(pos, node);
                    fmt::Kind::HeredocOpening(heredoc_opening)
                } else {
                    let str = self.visit_string(node);
                    fmt::Kind::Str(str)
                };
                trivia.set_trailing(self.take_trailing_comment(next_loc_start));
                fmt::Node::new(pos, trivia, kind)
            }
            prism::Node::InterpolatedStringNode { .. } => {
                let node = node.as_interpolated_string_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let mut trivia = self.take_leading_trivia(loc.start_offset());
                let kind = if Self::is_heredoc(node.opening_loc().as_ref()) {
                    let heredoc_opening = self.visit_complex_heredoc(pos, node);
                    fmt::Kind::HeredocOpening(heredoc_opening)
                } else {
                    let str = self.visit_interpolated_string(node);
                    fmt::Kind::DynStr(str)
                };
                trivia.set_trailing(self.take_trailing_comment(next_loc_start));
                fmt::Node::new(pos, trivia, kind)
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

            prism::Node::CallNode { .. } => {
                let node = node.as_call_node().unwrap();
                let pos = self.next_pos();
                let loc = node.location();
                let mut trivia = self.take_leading_trivia(loc.start_offset());
                let chain = self.visit_call_root(&node, next_loc_start, None);
                trivia.set_trailing(self.take_trailing_comment(next_loc_start));
                fmt::Node::new(pos, trivia, fmt::Kind::MethodChain(chain))
            }

            prism::Node::LocalVariableWriteNode { .. } => {
                let node = node.as_local_variable_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::LocalVariableAndWriteNode { .. } => {
                let node = node.as_local_variable_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::LocalVariableOrWriteNode { .. } => {
                let node = node.as_local_variable_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::LocalVariableOperatorWriteNode { .. } => {
                let node = node.as_local_variable_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::InstanceVariableWriteNode { .. } => {
                let node = node.as_instance_variable_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::InstanceVariableAndWriteNode { .. } => {
                let node = node.as_instance_variable_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::InstanceVariableOrWriteNode { .. } => {
                let node = node.as_instance_variable_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::InstanceVariableOperatorWriteNode { .. } => {
                let node = node.as_instance_variable_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::ClassVariableWriteNode { .. } => {
                let node = node.as_class_variable_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    // XXX: When does the operator becomes None?
                    node.operator_loc().expect("must have operator"),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ClassVariableAndWriteNode { .. } => {
                let node = node.as_class_variable_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ClassVariableOrWriteNode { .. } => {
                let node = node.as_class_variable_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ClassVariableOperatorWriteNode { .. } => {
                let node = node.as_class_variable_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::GlobalVariableWriteNode { .. } => {
                let node = node.as_global_variable_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::GlobalVariableAndWriteNode { .. } => {
                let node = node.as_global_variable_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::GlobalVariableOrWriteNode { .. } => {
                let node = node.as_global_variable_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::GlobalVariableOperatorWriteNode { .. } => {
                let node = node.as_global_variable_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::ConstantWriteNode { .. } => {
                let node = node.as_constant_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantAndWriteNode { .. } => {
                let node = node.as_constant_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantOrWriteNode { .. } => {
                let node = node.as_constant_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantOperatorWriteNode { .. } => {
                let node = node.as_constant_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_variable_assign(
                    node.location(),
                    node.name_loc(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::ConstantPathWriteNode { .. } => {
                let node = node.as_constant_path_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantPathAndWriteNode { .. } => {
                let node = node.as_constant_path_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantPathOrWriteNode { .. } => {
                let node = node.as_constant_path_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::ConstantPathOperatorWriteNode { .. } => {
                let node = node.as_constant_path_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_constant_path_assign(
                    node.target(),
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::CallAndWriteNode { .. } => {
                let node = node.as_call_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::CallOrWriteNode { .. } => {
                let node = node.as_call_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::CallOperatorWriteNode { .. } => {
                let node = node.as_call_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            prism::Node::IndexAndWriteNode { .. } => {
                let node = node.as_index_and_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::IndexOrWriteNode { .. } => {
                let node = node.as_index_or_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }
            prism::Node::IndexOperatorWriteNode { .. } => {
                let node = node.as_index_operator_write_node().unwrap();
                let pos = self.next_pos();
                let (assign, trivia) = self.visit_call_assign(
                    &node,
                    node.operator_loc(),
                    node.value(),
                    next_loc_start,
                );
                fmt::Node::new(pos, trivia, fmt::Kind::Assign(assign))
            }

            _ => todo!("parse {:?}", node),
        };

        // XXX: We should take trailing comment after setting the last location end.
        self.last_loc_end = loc_end;
        node
    }

    fn parse_atom(&mut self, node: prism::Node, next_loc_start: usize) -> fmt::Node {
        let pos = self.next_pos();
        let loc = node.location();
        let mut trivia = self.take_leading_trivia(loc.start_offset());
        let value = Self::source_lossy_at(&loc);
        trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        fmt::Node::new(pos, trivia, fmt::Kind::Atom(value))
    }

    fn visit_constant_path(
        &mut self,
        node: prism::Node,
        next_loc_start: usize,
    ) -> (String, fmt::Trivia) {
        let loc = node.location();

        // Use `end_offset` to treat any trivia inside the path as leading trivia for simplicity.
        // e.g. "Foo::\n#comment\nBar" -> "#comment\nFoo::Bar"
        let mut trivia = self.take_leading_trivia(loc.end_offset());

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
        trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        (path, trivia)
    }

    fn visit_string(&mut self, node: prism::StringNode) -> fmt::Str {
        let value = Self::source_lossy_at(&node.content_loc());
        let opening = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = node.closing_loc().as_ref().map(Self::source_lossy_at);
        fmt::Str::new(opening, value.into(), closing)
    }

    fn visit_interpolated_string(&mut self, itp_str: prism::InterpolatedStringNode) -> fmt::DynStr {
        let opening = itp_str.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = itp_str.closing_loc().as_ref().map(Self::source_lossy_at);
        let mut dstr = fmt::DynStr::new(opening, closing);
        for part in itp_str.parts().iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.visit_string(node);
                    dstr.append_part(fmt::DynStrPart::Str(str));
                    self.last_loc_end = node_end;
                }
                prism::Node::InterpolatedStringNode { .. } => {
                    let node = part.as_interpolated_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.visit_interpolated_string(node);
                    dstr.append_part(fmt::DynStrPart::DynStr(str));
                    self.last_loc_end = node_end;
                }
                prism::Node::EmbeddedStatementsNode { .. } => {
                    let node = part.as_embedded_statements_node().unwrap();
                    let loc = node.location();
                    self.last_loc_end = node.opening_loc().end_offset();

                    let exprs = self.visit_statements(node.statements(), loc.end_offset());
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    let embedded_exprs = fmt::EmbeddedExprs::new(opening, exprs, closing);
                    dstr.append_part(fmt::DynStrPart::Exprs(embedded_exprs));
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
        pos: fmt::Pos,
        node: prism::StringNode,
    ) -> fmt::HeredocOpening {
        let open = node.opening_loc().unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let opening_id = String::from_utf8_lossy(id).to_string();
        let str = self.visit_string(node);
        let heredoc = fmt::Heredoc {
            id: opening_id.clone(),
            indent_mode,
            parts: vec![fmt::HeredocPart::Str(str)],
        };
        self.heredoc_map.insert(pos, heredoc);
        fmt::HeredocOpening::new(opening_id, indent_mode)
    }

    fn visit_complex_heredoc(
        &mut self,
        pos: fmt::Pos,
        node: prism::InterpolatedStringNode,
    ) -> fmt::HeredocOpening {
        let open = node.opening_loc().unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let opening_id = String::from_utf8_lossy(id).to_string();
        let mut parts = vec![];
        let mut last_str_end: Option<usize> = None;
        for part in node.parts().iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.visit_string(node);
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
                            let str = fmt::Str::new(None, value, None);
                            parts.push(fmt::HeredocPart::Str(str))
                        }
                    }

                    let exprs = self.visit_statements(node.statements(), loc.end_offset());
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    let embedded = fmt::EmbeddedExprs::new(opening, exprs, closing);
                    parts.push(fmt::HeredocPart::Exprs(embedded));
                }
                _ => panic!("unexpected heredoc part: {:?}", part),
            }
        }
        let heredoc = fmt::Heredoc {
            id: opening_id.clone(),
            indent_mode,
            parts,
        };
        self.heredoc_map.insert(pos, heredoc);
        fmt::HeredocOpening::new(opening_id, indent_mode)
    }

    fn visit_statements(&mut self, node: Option<prism::StatementsNode>, end: usize) -> fmt::Exprs {
        let mut exprs = fmt::Exprs::new();
        if let Some(node) = node {
            Self::each_node_with_next_start(node.body().iter(), end, |prev, next_start| {
                let fmt_node = self.visit(prev, next_start);
                exprs.append_node(fmt_node);
            });
        }
        let virtual_end = self.take_end_trivia_as_virtual_end(Some(end));
        exprs.set_virtual_end(virtual_end);
        exprs
    }

    fn visit_if_or_unless(&mut self, node: IfOrUnless, next_loc_start: usize) -> fmt::Node {
        let pos = self.next_pos();
        let mut if_trivia = self.take_leading_trivia(node.loc.start_offset());

        let end_loc = node.end_loc.expect("if/unless expression must have end");
        let end_start = end_loc.start_offset();

        let if_next_loc = node.predicate.location().start_offset();
        let mut trivia = fmt::Trivia::new();
        trivia.set_trailing(self.take_trailing_comment(if_next_loc));

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
                let if_first = fmt::Conditional::new(trivia, predicate, body);
                let mut ifexpr = fmt::IfExpr::new(node.is_if, if_first);
                self.visit_ifelse(conseq, &mut ifexpr);
                ifexpr
            }
            // if...end
            None => {
                let body = self.visit_statements(node.statements, end_start);
                let if_first = fmt::Conditional::new(trivia, predicate, body);
                fmt::IfExpr::new(node.is_if, if_first)
            }
        };

        if_trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        fmt::Node::new(pos, if_trivia, fmt::Kind::IfExpr(ifexpr))
    }

    fn visit_ifelse(&mut self, node: prism::Node, ifexpr: &mut fmt::IfExpr) {
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
                let mut trivia = fmt::Trivia::new();
                trivia.set_trailing(self.take_trailing_comment(elsif_next_loc));

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

                let conditional = fmt::Conditional::new(trivia, predicate, body);
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
                let mut trivia = fmt::Trivia::new();
                trivia.set_trailing(self.take_trailing_comment(else_next_loc));

                let body = self.visit_statements(node.statements(), end_loc.start_offset());
                ifexpr.if_last = Some(fmt::Else { trivia, body });
            }
            _ => {
                panic!("unexpected node in IfNode: {:?}", node);
            }
        }
    }

    fn visit_postmodifier(&mut self, postmod: Postmodifier, next_loc_start: usize) -> fmt::Node {
        let pos = self.next_pos();
        let mut trivia = self.take_leading_trivia(postmod.loc.start_offset());

        let kwd_loc = postmod.keyword_loc;
        let exprs = self.visit_statements(postmod.statements, kwd_loc.start_offset());

        let pred_loc = postmod.predicate.location();
        let mut kwd_trivia = fmt::Trivia::new();
        kwd_trivia.set_trailing(self.take_trailing_comment(pred_loc.start_offset()));

        let predicate = self.visit(postmod.predicate, next_loc_start);

        let postmod = fmt::Postmodifier::new(
            postmod.keyword,
            fmt::Conditional::new(kwd_trivia, predicate, exprs),
        );

        trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        fmt::Node::new(pos, trivia, fmt::Kind::Postmodifier(postmod))
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

        let mut trivia = if let Some(msg_loc) = call.message_loc() {
            self.take_leading_trivia(msg_loc.start_offset())
        } else {
            fmt::Trivia::new()
            // foo.\n#hoge\n(2)
        };

        let call_op = call.call_operator_loc().map(|l| Self::source_lossy_at(&l));
        let name = String::from_utf8_lossy(call.name()).to_string();
        let mut method_call = fmt::MethodCall::new(call_op, name);

        let args = match call.arguments() {
            None => {
                if let Some(closing_loc) = call.closing_loc().map(|l| l.start_offset()) {
                    let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_loc));
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
                let closing_start = call.closing_loc().map(|l| l.start_offset());
                let next_loc_start = closing_start
                    .or_else(|| call.block().map(|a| a.location().start_offset()))
                    .or(next_msg_start)
                    .unwrap_or(next_loc_start);
                Self::each_node_with_next_start(
                    args_node.arguments().iter(),
                    next_loc_start,
                    |prev, next_start| {
                        let fmt_node = self.visit(prev, next_start);
                        args.append_node(fmt_node);
                    },
                );

                let virtual_end = self.take_end_trivia_as_virtual_end(closing_start);
                args.set_virtual_end(virtual_end);

                Some(args)
            }
        };
        if let Some(args) = args {
            method_call.set_args(args);
        }

        let block = call.block().map(|node| match node {
            prism::Node::BlockNode { .. } => {
                let node = node.as_block_node().unwrap();

                let block_next_loc = node
                    .body()
                    .map(|b| b.location())
                    .unwrap_or(node.closing_loc())
                    .start_offset();
                let mut trivia = fmt::Trivia::new();
                trivia.set_trailing(self.take_trailing_comment(block_next_loc));

                let body_end_loc = node.closing_loc().start_offset();
                let body = node.body().map(|n| self.visit(n, body_end_loc));
                // XXX: Is this necessary? I cannot find the case where the body is not a StatementNode.
                let body = self.wrap_as_exprs(body, Some(body_end_loc));

                let loc = node.location();
                let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());

                let width = if was_flat {
                    body.width().add(&fmt::Width::Flat(" {  }".len()))
                } else {
                    fmt::Width::NotFlat
                };

                fmt::MethodBlock {
                    trivia,
                    width,
                    body,
                    was_flat,
                }
            }
            _ => panic!("unexpected node for call block: {:?}", node),
        });
        if let Some(block) = block {
            method_call.set_block(block);
        }

        if let Some(next_msg_start) = next_msg_start {
            trivia.set_trailing(self.take_trailing_comment(next_msg_start));
        }

        method_call.set_trivia(trivia);
        chain.append_call(method_call);

        self.last_loc_end = call.location().end_offset();
        chain
    }

    fn visit_variable_assign(
        &mut self,
        node_loc: prism::Location,
        name_loc: prism::Location,
        operator_loc: prism::Location,
        value: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::Assign, fmt::Trivia) {
        let mut trivia = self.take_leading_trivia(node_loc.start_offset());
        let name = Self::source_lossy_at(&name_loc);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, next_loc_start);
        trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        let target = fmt::AssignTarget::Atom(name);
        (fmt::Assign::new(target, operator, value), trivia)
    }

    fn visit_constant_path_assign(
        &mut self,
        const_path: prism::ConstantPathNode,
        operator_loc: prism::Location,
        value: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::Assign, fmt::Trivia) {
        let (path, mut trivia) =
            self.visit_constant_path(const_path.as_node(), operator_loc.start_offset());
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, next_loc_start);
        trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        let target = fmt::AssignTarget::Atom(path);
        (fmt::Assign::new(target, operator, value), trivia)
    }

    fn visit_call_assign(
        &mut self,
        call: &impl CallRoot,
        operator_loc: prism::Location,
        value: prism::Node,
        next_loc_start: usize,
    ) -> (fmt::Assign, fmt::Trivia) {
        let mut trivia = self.take_leading_trivia(call.location().start_offset());
        let chain = self.visit_call_root(call, next_loc_start, None);
        let target = fmt::AssignTarget::Call(chain);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.visit(value, next_loc_start);
        trivia.set_trailing(self.take_trailing_comment(next_loc_start));
        (fmt::Assign::new(target, operator, value), trivia)
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
        let virtual_end = self.take_end_trivia_as_virtual_end(end);
        exprs.set_virtual_end(virtual_end);
        exprs
    }

    fn take_end_trivia_as_virtual_end(&mut self, end: Option<usize>) -> Option<fmt::VirtualEnd> {
        if let Some(end) = end {
            let trivia = self.take_leading_trivia(end);
            if !trivia.leading.is_empty() {
                let width = trivia.width;
                return Some(fmt::VirtualEnd { trivia, width });
            }
        }
        None
    }

    fn take_leading_trivia(&mut self, loc_start: usize) -> fmt::Trivia {
        let mut trivia = fmt::Trivia::new();

        while let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if !(self.last_loc_end..=loc_start).contains(&loc.start_offset()) {
                break;
            };
            // We treat the found comment as line comment always.
            let value = Self::source_lossy_at(&loc);
            let fmt_comment = fmt::Comment { value };
            self.take_empty_lines_until(loc.start_offset(), &mut trivia);
            trivia.append_leading(fmt::LineTrivia::Comment(fmt_comment));
            self.last_loc_end = loc.end_offset() - 1;
            self.comments.next();
        }

        self.take_empty_lines_until(loc_start, &mut trivia);
        trivia
    }

    fn take_empty_lines_until(&mut self, end: usize, trivia: &mut fmt::Trivia) {
        let range = self.last_empty_line_range_within(self.last_loc_end, end);
        if let Some(range) = range {
            trivia.append_leading(fmt::LineTrivia::EmptyLine);
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
