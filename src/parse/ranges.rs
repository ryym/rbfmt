use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_range_like(
        &mut self,
        operator_loc: prism::Location,
        left: Option<prism::Node>,
        right: Option<prism::Node>,
    ) -> fmt::Node {
        let op_start = operator_loc.start_offset();
        let left = left.map(|n| self.visit(n, Some(op_start)));
        let operator = Self::source_lossy_at(&operator_loc);
        let right = right.map(|n| self.visit(n, None));
        let range = fmt::RangeLike::new(left, operator, right);
        fmt::Node::new(fmt::Kind::RangeLike(range))
    }
}
