use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_case(&mut self, node: prism::CaseNode) -> fmt::Node {
        let conditions = node.conditions();
        let consequent = node.consequent();
        let end_loc = node.end_keyword_loc();
        let first_branch_start = conditions
            .iter()
            .next()
            .map(|n| n.location().start_offset());

        let pred_next = first_branch_start
            .or(consequent.as_ref().map(|c| c.location().start_offset()))
            .unwrap_or(end_loc.start_offset());
        let predicate = node.predicate().map(|n| self.visit(n, Some(pred_next)));
        let case_trailing = if predicate.is_some() {
            fmt::TrailingTrivia::none()
        } else {
            self.take_trailing_comment(pred_next)
        };

        let first_branch_leading = match first_branch_start {
            Some(first_branch_start) => self.take_leading_trivia(first_branch_start),
            None => fmt::LeadingTrivia::new(),
        };

        let mut branches = vec![];
        let conditions_next = consequent
            .as_ref()
            .map(|c| c.location().start_offset())
            .unwrap_or(end_loc.start_offset());
        Self::each_node_with_trailing_end(
            conditions.iter(),
            Some(conditions_next),
            |node, trailing_end| {
                let condition = match node {
                    prism::Node::WhenNode { .. } => {
                        let node = node.as_when_node().unwrap();
                        self.parse_case_when(node, trailing_end)
                    }
                    _ => panic!("unexpected case expression branch: {:?}", node),
                };
                branches.push(condition);
            },
        );

        let otherwise = consequent.map(|node| self.visit_else(node, end_loc.start_offset()));

        let case = fmt::Case {
            case_trailing,
            predicate: predicate.map(Box::new),
            first_branch_leading,
            branches,
            otherwise,
        };
        fmt::Node::new(fmt::Kind::Case(case))
    }

    fn parse_case_when(&mut self, node: prism::WhenNode, body_end: Option<usize>) -> fmt::CaseWhen {
        let loc = node.location();
        let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());
        let mut when = fmt::CaseWhen::new(was_flat);

        let conditions_next = node
            .statements()
            .as_ref()
            .map(|n| n.location().start_offset())
            .or(body_end);
        Self::each_node_with_trailing_end(
            node.conditions().iter(),
            conditions_next,
            |node, trailing_end| {
                let cond = self.visit(node, trailing_end);
                when.append_condition(cond);
            },
        );

        let body = self.visit_statements(node.statements(), body_end);
        when.set_body(body);
        when
    }
}
