use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_undef(&mut self, undef: prism::UndefNode) -> fmt::Node {
        let mut args = fmt::Arguments::new(None, None);
        Self::each_node_with_trailing_end(undef.names().iter(), None, |node, trailing_end| {
            let node = self.parse(node, trailing_end);
            args.append_node(node);
        });
        let mut call_like = fmt::CallLike::new("undef".to_string());
        call_like.set_arguments(args);
        fmt::Node::new(fmt::Kind::CallLike(call_like))
    }

    pub(super) fn parse_defined(&mut self, defined: prism::DefinedNode) -> fmt::Node {
        let lparen_loc = defined.lparen_loc();
        let rparen_loc = defined.rparen_loc();

        let value = defined.value();
        let value_next = rparen_loc.as_ref().map(|l| l.start_offset());
        let value = self.parse(value, value_next);

        let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
        let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
        let mut args = fmt::Arguments::new(lparen, rparen);
        args.last_comma_allowed = false;
        args.append_node(value);

        let rparen_start = rparen_loc.as_ref().map(|l| l.start_offset());
        let virutla_end = self.take_end_trivia_as_virtual_end(rparen_start);
        args.set_virtual_end(virutla_end);

        let mut call_like = fmt::CallLike::new("defined?".to_string());
        call_like.set_arguments(args);
        fmt::Node::new(fmt::Kind::CallLike(call_like))
    }

    pub(super) fn parse_pre_post_exec(
        &mut self,
        keyword_loc: prism::Location,
        opening_loc: prism::Location,
        statements: Option<prism::StatementsNode>,
        closing_loc: prism::Location,
    ) -> fmt::Node {
        let keyword = Self::source_lossy_at(&keyword_loc);
        let closing_start = closing_loc.start_offset();
        let was_flat = !self.does_line_break_exist_in(opening_loc.end_offset(), closing_start);
        let statements = self.parse_statements_body(statements, Some(closing_start));
        let exec = fmt::PrePostExec::new(keyword, statements, was_flat);
        fmt::Node::new(fmt::Kind::PrePostExec(exec))
    }

    pub(super) fn parse_alias(
        &mut self,
        new_name: prism::Node,
        old_name: prism::Node,
    ) -> fmt::Node {
        let additional_leading = self.take_leading_trivia(new_name.location().start_offset());
        let old_loc = old_name.location();
        let new_name = self.parse(new_name, Some(old_loc.start_offset()));
        let old_name = self.parse(old_name, None);
        let alias = fmt::Alias::new(new_name, old_name);
        fmt::Node::with_leading_trivia(additional_leading, fmt::Kind::Alias(alias))
    }
}
