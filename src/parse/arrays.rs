use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_array(&mut self, node: prism::ArrayNode) -> fmt::Node {
        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let mut array = fmt::Array::new(opening, closing);
        let closing_start = closing_loc.map(|l| l.start_offset());
        Self::each_node_with_trailing_end(
            node.elements().iter(),
            closing_start,
            |node, trailing_end| match node {
                prism::Node::KeywordHashNode { .. } => {
                    let node = node.as_keyword_hash_node().unwrap();
                    self.each_keyword_hash_element(node, trailing_end, |element| {
                        array.append_element(element);
                    });
                }
                _ => {
                    let element = self.parse(node, trailing_end);
                    array.append_element(element);
                }
            },
        );
        if let Some(closing_start) = closing_start {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
            array.set_virtual_end(virtual_end);
        }
        fmt::Node::new(fmt::Kind::Array(array))
    }

    pub(super) fn parse_splat(&mut self, node: prism::SplatNode) -> fmt::Node {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let expr = node.expression().map(|expr| self.parse(expr, None));
        let splat = fmt::Prefix::new(operator, expr, false);
        fmt::Node::new(fmt::Kind::Prefix(splat))
    }
}
