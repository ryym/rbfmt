use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_hash(&mut self, node: prism::HashNode) -> fmt::Node {
        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();
        let opening = Self::source_lossy_at(&opening_loc);
        let closing = Self::source_lossy_at(&closing_loc);
        let should_be_inline = if let Some(first_element) = node.elements().iter().next() {
            !self.does_line_break_exist_in(
                opening_loc.start_offset(),
                first_element.location().start_offset(),
            )
        } else {
            true
        };
        let mut hash = fmt::Hash::new(opening, closing, should_be_inline);
        let closing_start = closing_loc.start_offset();
        Self::each_node_with_trailing_end(
            node.elements().iter(),
            Some(closing_start),
            |node, trailing_end| {
                let element = self.parse(node, trailing_end);
                hash.append_element(element);
            },
        );
        let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
        hash.set_virtual_end(virtual_end);
        fmt::Node::new(fmt::Kind::Hash(hash))
    }

    pub(super) fn parse_assoc(&mut self, node: prism::AssocNode) -> fmt::Node {
        let key = node.key();
        let key = self.parse(key, None);
        let operator = node.operator_loc().map(|l| Self::source_lossy_at(&l));
        let value = self.parse(node.value(), None);
        let assoc = fmt::Assoc::new(key, operator, value);
        fmt::Node::new(fmt::Kind::Assoc(assoc))
    }

    pub(super) fn parse_assoc_splat(&mut self, node: prism::AssocSplatNode) -> fmt::Node {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let value = node.value().map(|v| self.parse(v, None));
        let splat = fmt::Prefix::new(operator, value);
        fmt::Node::new(fmt::Kind::Prefix(splat))
    }
}
