use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_class_like(
        &mut self,
        keyword: &str,
        name_loc: prism::Location,
        superclass: Option<prism::Node>,
        body: Option<prism::Node>,
        end_loc: prism::Location,
    ) -> fmt::Node {
        let leading = self.take_leading_trivia(name_loc.start_offset());
        let name = Self::source_lossy_at(&name_loc);

        let body_start = body.as_ref().and_then(|b| match b {
            prism::Node::BeginNode { .. } => {
                Self::start_of_begin_block_content(b.as_begin_node().unwrap())
            }
            _ => Some(b.location().start_offset()),
        });
        let head_next = body_start.unwrap_or(end_loc.start_offset());
        let (superclass, head_trailing) = if let Some(superclass) = superclass {
            let fmt_node = self.visit(superclass, Some(head_next));
            (Some(fmt_node), fmt::TrailingTrivia::none())
        } else {
            let head_trailing = self.take_trailing_comment(head_next);
            (None, head_trailing)
        };

        let body = self.parse_block_body(body, end_loc.start_offset());
        let class = fmt::ClassLike {
            keyword: keyword.to_string(),
            name,
            superclass: superclass.map(Box::new),
            head_trailing,
            body,
        };
        fmt::Node::with_leading_trivia(leading, fmt::Kind::ClassLike(class))
    }

    pub(super) fn parse_singleton_class(&mut self, node: prism::SingletonClassNode) -> fmt::Node {
        let leading = self.take_leading_trivia(node.operator_loc().start_offset());
        let body = node.body();
        let end_loc = node.end_keyword_loc();
        let body_start = body.as_ref().and_then(|b| match b {
            prism::Node::BeginNode { .. } => {
                Self::start_of_begin_block_content(b.as_begin_node().unwrap())
            }
            _ => Some(b.location().start_offset()),
        });
        let expr_next = body_start.unwrap_or(end_loc.start_offset());
        let expr = self.visit(node.expression(), Some(expr_next));
        let body = self.parse_block_body(body, end_loc.start_offset());
        let class = fmt::SingletonClass {
            expression: Box::new(expr),
            body,
        };
        fmt::Node::with_leading_trivia(leading, fmt::Kind::SingletonClass(class))
    }
}
