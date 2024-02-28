use crate::fmt;

use super::postmodifiers;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_while(&mut self, node: prism::WhileNode) -> fmt::Node {
        if let Some(closing_loc) = node.closing_loc() {
            let whle =
                self.parse_while_or_until(true, node.predicate(), node.statements(), closing_loc);
            fmt::Node::new(fmt::Kind::While(whle))
        } else {
            self.parse_postmodifier(postmodifiers::Postmodifier {
                keyword: "while".to_string(),
                keyword_loc: node.keyword_loc(),
                predicate: node.predicate(),
                statements: node.statements(),
            })
        }
    }

    pub(super) fn parse_until(&mut self, node: prism::UntilNode) -> fmt::Node {
        if let Some(closing_loc) = node.closing_loc() {
            let whle =
                self.parse_while_or_until(false, node.predicate(), node.statements(), closing_loc);
            fmt::Node::new(fmt::Kind::While(whle))
        } else {
            self.parse_postmodifier(postmodifiers::Postmodifier {
                keyword: "until".to_string(),
                keyword_loc: node.keyword_loc(),
                predicate: node.predicate(),
                statements: node.statements(),
            })
        }
    }

    fn parse_while_or_until(
        &mut self,
        is_while: bool,
        predicate: prism::Node,
        body: Option<prism::StatementsNode>,
        closing_loc: prism::Location,
    ) -> fmt::While {
        let predicate_next = body
            .as_ref()
            .map(|b| b.location().start_offset())
            .unwrap_or(closing_loc.start_offset());
        let predicate = self.visit(predicate, Some(predicate_next));
        let body = self.parse_statements_body(body, Some(closing_loc.start_offset()));
        let content = fmt::Conditional::new(predicate, body);
        fmt::While { is_while, content }
    }

    pub(super) fn parse_for(&mut self, node: prism::ForNode) -> fmt::Node {
        let body = node.statements();
        let end_loc = node.end_keyword_loc();

        let index = self.visit(node.index(), Some(node.in_keyword_loc().start_offset()));
        let collection_next = body
            .as_ref()
            .map(|b| b.location().start_offset())
            .unwrap_or(end_loc.start_offset());
        let collection = self.visit(node.collection(), Some(collection_next));
        let body = self.parse_statements_body(body, Some(end_loc.start_offset()));

        let for_kind = fmt::For {
            index: Box::new(index),
            collection: Box::new(collection),
            body,
        };
        fmt::Node::new(fmt::Kind::For(for_kind))
    }
}
