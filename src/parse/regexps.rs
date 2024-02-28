use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_regexp(&mut self, node: prism::RegularExpressionNode) -> fmt::Node {
        let str = self.parse_string(
            Some(node.opening_loc()),
            node.content_loc(),
            Some(node.closing_loc()),
        );
        fmt::Node::new(fmt::Kind::StringLike(str))
    }

    pub(super) fn parse_interpolated_regexp(
        &mut self,
        node: prism::InterpolatedRegularExpressionNode,
    ) -> fmt::Node {
        let str = self.parse_interpolated_string(
            Some(node.opening_loc()),
            node.parts(),
            Some(node.closing_loc()),
        );
        let kind = fmt::Kind::DynStringLike(str);
        fmt::Node::new(kind)
    }

    pub(super) fn parse_match_last_line(&mut self, node: prism::MatchLastLineNode) -> fmt::Node {
        let str = self.parse_string(
            Some(node.opening_loc()),
            node.content_loc(),
            Some(node.closing_loc()),
        );
        fmt::Node::new(fmt::Kind::StringLike(str))
    }

    pub(super) fn parse_interpolated_match_last_line(
        &mut self,
        node: prism::InterpolatedMatchLastLineNode,
    ) -> fmt::Node {
        let str = self.parse_interpolated_string(
            Some(node.opening_loc()),
            node.parts(),
            Some(node.closing_loc()),
        );
        let kind = fmt::Kind::DynStringLike(str);
        fmt::Node::new(kind)
    }
}
