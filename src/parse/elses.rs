use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_else(&mut self, node: prism::ElseNode, else_end: usize) -> fmt::Else {
        let else_next_loc = node
            .statements()
            .as_ref()
            .map(|s| s.location().start_offset())
            .unwrap_or(else_end);
        let keyword_trailing = self.take_trailing_comment(else_next_loc);
        let body = self.parse_statements_body(node.statements(), Some(else_end));
        fmt::Else {
            keyword_trailing,
            body,
        }
    }
}
