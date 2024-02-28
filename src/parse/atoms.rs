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

    pub(super) fn parse_constant_path(
        &mut self,
        parent: Option<prism::Node>,
        child: prism::Node,
    ) -> fmt::Node {
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
        fmt::Node::new(fmt::Kind::ConstantPath(const_path))
    }
}
