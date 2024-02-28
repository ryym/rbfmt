use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_as_atom(&self, node: prism::Node) -> fmt::Node {
        let value = super::Parser::source_lossy_at(&node.location());
        fmt::Node::new(fmt::Kind::Atom(fmt::Atom(value)))
    }

    pub(super) fn parse_implicit(&self) -> fmt::Node {
        let atom = fmt::Atom("".to_string());
        fmt::Node::new(fmt::Kind::Atom(atom))
    }
}
