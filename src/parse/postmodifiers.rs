use crate::fmt;

pub(super) struct Postmodifier<'src> {
    pub keyword: String,
    pub keyword_loc: prism::Location<'src>,
    pub predicate: prism::Node<'src>,
    pub statements: Option<prism::StatementsNode<'src>>,
}

impl<'src> super::Parser<'src> {
    pub(super) fn parse_postmodifier(&mut self, postmod: Postmodifier) -> fmt::Node {
        let kwd_loc = postmod.keyword_loc;
        let statements =
            self.parse_statements_body(postmod.statements, Some(kwd_loc.start_offset()));

        let predicate = self.parse(postmod.predicate, None);

        let postmod = fmt::Postmodifier::new(
            postmod.keyword,
            fmt::Conditional::new(predicate, statements),
        );

        fmt::Node::new(fmt::Kind::Postmodifier(postmod))
    }

    pub(super) fn parse_rescue_modifier(&mut self, node: prism::RescueModifierNode) -> fmt::Node {
        let kwd_loc = node.keyword_loc();
        let expr = self.parse(node.expression(), Some(kwd_loc.start_offset()));
        let statements = self.wrap_as_statements(Some(expr), kwd_loc.start_offset());

        let rescue_expr = node.rescue_expression();
        let rescue_expr = self.parse(rescue_expr, None);

        let postmod = fmt::Postmodifier::new(
            "rescue".to_string(),
            fmt::Conditional::new(rescue_expr, statements),
        );
        fmt::Node::new(fmt::Kind::Postmodifier(postmod))
    }
}
