use crate::fmt;

use super::begins;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_def(&mut self, node: prism::DefNode) -> fmt::Node {
        let receiver = node.receiver();
        let name_loc = node.name_loc();

        // Take leading trivia of receiver or method name.
        let leading_end = receiver
            .as_ref()
            .map(|r| r.location().start_offset())
            .unwrap_or_else(|| name_loc.start_offset());
        let leading = self.take_leading_trivia(leading_end);

        let name_end = name_loc.end_offset();
        let receiver = receiver.map(|r| self.visit(r, Some(name_end)));
        let name = Self::source_lossy_at(&node.name_loc());
        let mut def = fmt::Def::new(receiver, name);

        let lparen_loc = node.lparen_loc();
        let rparen_loc = node.rparen_loc();
        if let Some(params) = node.parameters() {
            let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
            let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
            let mut parameters = fmt::MethodParameters::new(lparen, rparen);
            let params_next = rparen_loc.as_ref().map(|l| l.start_offset());
            self.parse_parameter_nodes(params, params_next, |node| {
                parameters.append_param(node);
            });
            let virtual_end = self.take_end_trivia_as_virtual_end(params_next);
            parameters.set_virtual_end(virtual_end);
            def.set_parameters(parameters);
        } else if let (Some(lparen_loc), Some(rparen_loc)) = (&lparen_loc, &rparen_loc) {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(rparen_loc.start_offset()));
            if virtual_end.is_some() {
                let lparen = Self::source_lossy_at(lparen_loc);
                let rparen = Self::source_lossy_at(rparen_loc);
                let mut parameters = fmt::MethodParameters::new(Some(lparen), Some(rparen));
                parameters.set_virtual_end(virtual_end);
                def.set_parameters(parameters);
            }
        }

        if node.equal_loc().is_some() {
            let body = node.body().expect("shorthand def body must exist");
            let body = self.visit(body, None);
            def.set_body(fmt::DefBody::Short {
                body: Box::new(body),
            });
        } else {
            let end_loc = node.end_keyword_loc().expect("block def must have end");
            let body = node.body();
            let body_start = body.as_ref().and_then(|b| match b {
                prism::Node::BeginNode { .. } => {
                    begins::start_of_begin_block_content(b.as_begin_node().unwrap())
                }
                _ => Some(b.location().start_offset()),
            });
            let head_next = body_start.unwrap_or(end_loc.start_offset());
            let head_trailing = self.take_trailing_comment(head_next);
            let block_body = self.parse_block_body(body, end_loc.start_offset());
            def.set_body(fmt::DefBody::Block {
                head_trailing,
                body: block_body,
            });
        }

        fmt::Node::with_leading_trivia(leading, fmt::Kind::Def(def))
    }

    pub(super) fn parse_lambda(&mut self, node: prism::LambdaNode) -> fmt::Node {
        let params = node.parameters().map(|params| match params {
            prism::Node::BlockParametersNode { .. } => {
                let params = params.as_block_parameters_node().unwrap();
                let params_end = params.location().end_offset();
                self.parse_block_parameters(params, params_end)
            }
            _ => panic!("unexpected node for lambda params: {:?}", node),
        });

        let body_end = node.closing_loc().start_offset();
        let body_opening_trailing = self.take_trailing_comment(body_end);
        let body = self.parse_block_body(node.body(), body_end);

        let was_flat = !self.does_line_break_exist_in(
            node.opening_loc().end_offset(),
            node.closing_loc().start_offset(),
        );
        let opening = Self::source_lossy_at(&node.opening_loc());
        let closing = Self::source_lossy_at(&node.closing_loc());
        let mut block = fmt::Block::new(was_flat, opening, closing);
        block.set_opening_trailing(body_opening_trailing);
        block.set_body(body);

        let lambda = fmt::Lambda::new(params, block);
        fmt::Node::new(fmt::Kind::Lambda(lambda))
    }

    pub(super) fn parse_optional_keyword_argument(
        &mut self,
        node: prism::OptionalKeywordParameterNode,
    ) -> fmt::Node {
        let name = Self::source_lossy_at(&node.name_loc());
        let name = fmt::Node::new(fmt::Kind::Atom(fmt::Atom(name)));
        let value = node.value();
        let value = self.visit(value, None);
        let assoc = fmt::Assoc::new(name, None, value);
        fmt::Node::new(fmt::Kind::Assoc(assoc))
    }

    pub(super) fn parse_block_arg(&mut self, node: prism::BlockArgumentNode) -> fmt::Node {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let expr = node.expression().map(|expr| self.visit(expr, None));
        let prefix = fmt::Prefix::new(operator, expr);
        fmt::Node::new(fmt::Kind::Prefix(prefix))
    }

    pub(super) fn parse_block_parameters(
        &mut self,
        node: prism::BlockParametersNode,
        trailing_end: usize,
    ) -> fmt::BlockParameters {
        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();

        // In lambda literal, parentheses can be omitted (e.g. "-> a, b {}").
        let opening = opening_loc
            .as_ref()
            .map(Self::source_lossy_at)
            .unwrap_or_else(|| "(".to_string());
        let closing = closing_loc
            .as_ref()
            .map(Self::source_lossy_at)
            .unwrap_or_else(|| ")".to_string());
        let mut block_params = fmt::BlockParameters::new(opening, closing);

        let closing_start = closing_loc.map(|l| l.start_offset());

        let locals = node.locals();

        if let Some(params) = node.parameters() {
            let params_next = locals
                .iter()
                .next()
                .map(|n| n.location().start_offset())
                .or(closing_start);
            self.parse_parameter_nodes(params, params_next, |node| {
                block_params.append_param(node);
            });
        }

        if let Some(closing_start) = closing_start {
            Self::each_node_with_trailing_end(
                locals.iter(),
                Some(closing_start),
                |node, trailing_end| {
                    let fmt_node = self.visit(node, trailing_end);
                    block_params.append_local(fmt_node);
                },
            );
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
            block_params.set_virtual_end(virtual_end);
        }

        let trailing = self.take_trailing_comment(trailing_end);
        block_params.set_closing_trailing(trailing);
        block_params
    }

    pub(super) fn parse_parameter_nodes(
        &mut self,
        params: prism::ParametersNode,
        trailing_end: Option<usize>,
        mut f: impl FnMut(fmt::Node),
    ) {
        let mut nodes = vec![];
        for n in params.requireds().iter() {
            nodes.push(n);
        }
        for n in params.optionals().iter() {
            nodes.push(n);
        }
        if let Some(rest) = params.rest() {
            nodes.push(rest);
        }
        for n in params.posts().iter() {
            nodes.push(n);
        }
        for n in params.keywords().iter() {
            nodes.push(n);
        }
        if let Some(rest) = params.keyword_rest() {
            nodes.push(rest);
        }
        if let Some(block) = params.block() {
            nodes.push(block.as_node());
        }
        Self::each_node_with_trailing_end(nodes.into_iter(), trailing_end, |node, trailing_end| {
            let fmt_node = self.visit(node, trailing_end);
            f(fmt_node);
        });
    }
}
