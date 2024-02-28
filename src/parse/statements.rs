use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_statements(
        &mut self,
        node: prism::StatementsNode,
        trailing_end: Option<usize>,
    ) -> fmt::Node {
        let statements = self.parse_statements_body(Some(node), trailing_end);
        let kind = fmt::Kind::Statements(statements);
        fmt::Node::new(kind)
    }

    pub(super) fn parse_statements_body(
        &mut self,
        node: Option<prism::StatementsNode>,
        end: Option<usize>,
    ) -> fmt::Statements {
        let mut statements = fmt::Statements::new();
        if let Some(node) = node {
            Self::each_node_with_trailing_end(node.body().iter(), end, |node, trailing_end| {
                let fmt_node = self.visit(node, trailing_end);
                statements.append_node(fmt_node);
            });
        }
        let virtual_end = self.take_end_trivia_as_virtual_end(end);
        statements.set_virtual_end(virtual_end);
        statements
    }

    pub(super) fn parse_parentheses(&mut self, node: prism::ParenthesesNode) -> fmt::Node {
        let closing_start = node.closing_loc().start_offset();
        let body = node.body().map(|b| self.visit(b, Some(closing_start)));
        let body = self.wrap_as_statements(body, closing_start);
        let parens = fmt::Parens::new(body);
        fmt::Node::new(fmt::Kind::Parens(parens))
    }

    pub(super) fn wrap_as_statements(
        &mut self,
        node: Option<fmt::Node>,
        end: usize,
    ) -> fmt::Statements {
        let (mut statements, should_take_end_trivia) = match node {
            None => (fmt::Statements::new(), true),
            Some(node) => match node.kind {
                fmt::Kind::Statements(statements) => (statements, false),
                _ => {
                    let mut statements = fmt::Statements::new();
                    statements.append_node(node);
                    (statements, true)
                }
            },
        };
        if should_take_end_trivia {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(end));
            statements.set_virtual_end(virtual_end);
        }
        statements
    }
}
