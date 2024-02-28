use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_symbol(&mut self, node: prism::SymbolNode) -> fmt::Node {
        // XXX: I cannot find the case where the value_loc is None.
        let value_loc = node.value_loc().expect("symbol value must exist");
        let str = self.parse_string(node.opening_loc(), value_loc, node.closing_loc());
        fmt::Node::new(fmt::Kind::StringLike(str))
    }

    pub(super) fn parse_interpolated_symbol(
        &mut self,
        node: prism::InterpolatedSymbolNode,
    ) -> fmt::Node {
        let str =
            self.parse_interpolated_string(node.opening_loc(), node.parts(), node.closing_loc());
        let kind = fmt::Kind::DynStringLike(str);
        fmt::Node::new(kind)
    }
}
