use std::collections::HashMap;

use crate::fmt;
use ruby_prism as prism;

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<ParserResult> {
    let result = prism::parse(&source);

    let decor_store = fmt::DecorStore::new();
    let heredoc_map = HashMap::new();

    let mut builder = FmtNodeBuilder {
        // src: &source,
        decor_store,
        heredoc_map,
        position_gen: 0,
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

#[derive(Debug)]
struct FmtNodeBuilder {
    // src: &'pr [u8],
    // comments: Vec<Comment>,
    // token_set: TokenSet,
    decor_store: fmt::DecorStore,
    heredoc_map: fmt::HeredocMap,
    position_gen: usize,
    // last_pos: fmt::Pos,
    // last_loc_end: usize,
}

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: prism::Node) -> fmt::Node {
        self.visit(node)
    }

    fn next_pos(&mut self) -> fmt::Pos {
        self.position_gen += 1;
        fmt::Pos(self.position_gen)
    }

    fn visit(&mut self, node: prism::Node) -> fmt::Node {
        use prism::Node;

        let node = match node {
            Node::ProgramNode { .. } => {
                let node = node.as_program_node().unwrap();
                let pos = self.next_pos();
                let mut nodes = vec![];
                for n in node.statements().body().iter() {
                    nodes.push(self.visit(n));
                }
                fmt::Node::new(pos, fmt::Kind::Exprs(fmt::Exprs(nodes)))
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
        node
    }

    fn parse_atom(&mut self, node: prism::Node) -> fmt::Node {
        let pos = self.next_pos();
        let loc = node.location();
        let value = String::from_utf8_lossy(loc.as_slice()).to_string();
        fmt::Node::new(pos, fmt::Kind::Atom(value))
    }
}
