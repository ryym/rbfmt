use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_begin(&mut self, node: prism::BeginNode) -> fmt::Node {
        let end_loc = node.end_keyword_loc().expect("begin must have end");
        let keyword_next = node
            .statements()
            .map(|n| n.location().start_offset())
            .or_else(|| node.rescue_clause().map(|n| n.location().start_offset()))
            .or_else(|| node.else_clause().map(|n| n.location().start_offset()))
            .or_else(|| node.ensure_clause().map(|n| n.location().start_offset()))
            .unwrap_or(end_loc.start_offset());
        let keyword_trailing = self.take_trailing_comment(keyword_next);
        let body = self.parse_begin_body(node);
        let begin = fmt::Begin {
            keyword_trailing,
            body,
        };
        fmt::Node::new(fmt::Kind::Begin(begin))
    }

    pub(super) fn parse_begin_body(&mut self, node: prism::BeginNode) -> fmt::BlockBody {
        let rescue_start = node
            .rescue_clause()
            .as_ref()
            .map(|r| r.location().start_offset());
        let else_start = node
            .else_clause()
            .as_ref()
            .map(|e| e.location().start_offset());
        let ensure_start = node
            .ensure_clause()
            .as_ref()
            .map(|e| e.location().start_offset());
        // XXX: I cannot find the case where the begin block does not have end.
        let end_loc = node.end_keyword_loc().expect("begin must have end");

        let statements_next = rescue_start
            .or(else_start)
            .or(ensure_start)
            .unwrap_or(end_loc.start_offset());
        let statements = self.visit_statements(node.statements(), Some(statements_next));
        let mut body = fmt::BlockBody::new(statements);

        if let Some(rescue_node) = node.rescue_clause() {
            let rescues_next = else_start
                .or(ensure_start)
                .unwrap_or(end_loc.start_offset());
            let mut rescues = vec![];
            self.parse_rescue_chain(rescue_node, &mut rescues, rescues_next);
            body.set_rescues(rescues);
        }

        if let Some(else_node) = node.else_clause() {
            let statements = else_node.statements();
            let keyword_next = statements
                .as_ref()
                .map(|s| s.location().start_offset())
                .or(ensure_start)
                .unwrap_or(end_loc.start_offset());
            let else_trailing = self.take_trailing_comment(keyword_next);
            let else_next = ensure_start.unwrap_or(end_loc.start_offset());
            let else_statements = self.visit_statements(statements, Some(else_next));
            body.set_rescue_else(fmt::Else {
                keyword_trailing: else_trailing,
                body: else_statements,
            });
        }

        if let Some(ensure_node) = node.ensure_clause() {
            let statements = ensure_node.statements();
            let keyword_next = statements
                .as_ref()
                .map(|s| s.location().start_offset())
                .unwrap_or(end_loc.start_offset());
            let ensure_trailing = self.take_trailing_comment(keyword_next);
            let ensure_statements = self.visit_statements(statements, Some(end_loc.start_offset()));
            body.set_ensure(fmt::Else {
                keyword_trailing: ensure_trailing,
                body: ensure_statements,
            });
        }

        body
    }

    fn parse_rescue_chain(
        &mut self,
        node: prism::RescueNode,
        rescues: &mut Vec<fmt::Rescue>,
        final_next: usize,
    ) {
        let reference = node.reference();
        let reference_start = reference.as_ref().map(|c| c.location().start_offset());

        let statements = node.statements();
        let statements_start = statements.as_ref().map(|c| c.location().start_offset());

        let consequent = node.consequent();
        let consequent_start = consequent.as_ref().map(|c| c.location().start_offset());

        let mut rescue = fmt::Rescue::new();

        let head_next = reference_start
            .or(statements_start)
            .or(consequent_start)
            .unwrap_or(final_next);
        Self::each_node_with_trailing_end(
            node.exceptions().iter(),
            Some(head_next),
            |node, trailing_end| {
                let fmt_node = self.visit(node, trailing_end);
                rescue.append_exception(fmt_node);
            },
        );

        if let Some(reference) = reference {
            let reference_next = statements_start.or(consequent_start).unwrap_or(final_next);
            let reference = self.visit(reference, Some(reference_next));
            rescue.set_reference(reference);
        }

        let head_next = statements_start.or(consequent_start).unwrap_or(final_next);
        let head_trailing = self.take_trailing_comment(head_next);
        rescue.set_head_trailing(head_trailing);

        let statements_next = consequent_start.unwrap_or(final_next);
        let statements = self.visit_statements(statements, Some(statements_next));
        rescue.set_statements(statements);
        rescues.push(rescue);

        if let Some(consequent) = consequent {
            self.parse_rescue_chain(consequent, rescues, final_next);
        }
    }
}

pub(super) fn start_of_begin_block_content(begin: prism::BeginNode) -> Option<usize> {
    let loc = begin
        .statements()
        .map(|n| n.location())
        .or_else(|| begin.rescue_clause().map(|n| n.location()))
        .or_else(|| begin.else_clause().map(|n| n.location()))
        .or_else(|| begin.ensure_clause().map(|n| n.location()))
        .or(begin.end_keyword_loc());
    loc.map(|l| l.start_offset())
}
