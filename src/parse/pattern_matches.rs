use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_case_match(&mut self, node: prism::CaseMatchNode) -> fmt::Node {
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
                    prism::Node::InNode { .. } => {
                        let node = node.as_in_node().unwrap();
                        self.parse_case_in(node, trailing_end)
                    }
                    _ => panic!("unexpected case expression branch: {:?}", node),
                };
                branches.push(condition);
            },
        );

        let otherwise = consequent.map(|node| self.visit_else(node, end_loc.start_offset()));

        let case_match = fmt::CaseMatch {
            case_trailing,
            predicate: predicate.map(Box::new),
            first_branch_leading,
            branches,
            otherwise,
        };
        fmt::Node::new(fmt::Kind::CaseMatch(case_match))
    }

    fn parse_case_in(&mut self, node: prism::InNode, body_end: Option<usize>) -> fmt::CaseIn {
        let loc = node.location();
        let was_flat = !self.does_line_break_exist_in(loc.start_offset(), loc.end_offset());

        let pattern_next = node
            .statements()
            .as_ref()
            .map(|n| n.location().start_offset());
        let pattern = self.visit(node.pattern(), pattern_next);

        let mut case_in = fmt::CaseIn::new(was_flat, pattern);
        let body = self.visit_statements(node.statements(), body_end);
        case_in.set_body(body);
        case_in
    }

    pub(super) fn parse_match_assign(
        &mut self,
        expression: prism::Node,
        operator_loc: prism::Location,
        pattern: prism::Node,
    ) -> fmt::Node {
        let expression = self.visit(expression, Some(operator_loc.start_offset()));
        let pattern = self.visit(pattern, None);
        let operator = Self::source_lossy_at(&operator_loc);
        let match_assign = fmt::MatchAssign::new(expression, operator, pattern);
        fmt::Node::new(fmt::Kind::MatchAssign(match_assign))
    }

    pub(super) fn parse_array_pattern(&mut self, node: prism::ArrayPatternNode) -> fmt::Node {
        let constant = node.constant().map(|c| self.visit(c, None));
        let opening = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = node.closing_loc().as_ref().map(Self::source_lossy_at);
        let mut array = fmt::ArrayPattern::new(constant, opening, closing);

        let rest = node.rest();
        let posts = node.posts();
        let posts_head = posts.iter().next();

        let closing_start = node.closing_loc().as_ref().map(|c| c.start_offset());
        let requireds_next = rest
            .as_ref()
            .map(|r| r.location().start_offset())
            .or_else(|| posts_head.as_ref().map(|p| p.location().start_offset()))
            .or(closing_start);
        Self::each_node_with_trailing_end(
            node.requireds().iter(),
            requireds_next,
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                array.append_element(element);
            },
        );

        if let Some(rest) = node.rest() {
            let rest_next = posts_head
                .as_ref()
                .map(|p| p.location().start_offset())
                .or(closing_start);
            let element = self.visit(rest, rest_next);
            array.append_element(element);
            array.last_comma_allowed = false;
        }

        if posts_head.is_some() {
            Self::each_node_with_trailing_end(
                node.posts().iter(),
                closing_start,
                |node, trailing_end| {
                    let element = self.visit(node, trailing_end);
                    array.append_element(element);
                },
            );
            array.last_comma_allowed = false;
        }

        let end = self.take_end_trivia_as_virtual_end(closing_start);
        array.set_virtual_end(end);

        fmt::Node::new(fmt::Kind::ArrayPattern(array))
    }

    pub(super) fn parse_find_pattern(&mut self, node: prism::FindPatternNode) -> fmt::Node {
        let constant = node.constant().map(|c| self.visit(c, None));
        let opening = node.opening_loc().as_ref().map(Self::source_lossy_at);
        let closing = node.closing_loc().as_ref().map(Self::source_lossy_at);
        let mut array = fmt::ArrayPattern::new(constant, opening, closing);
        array.last_comma_allowed = false;

        let requireds = node.requireds();
        let right = node.right();

        let left_next = requireds
            .iter()
            .next()
            .map(|n| n.location().start_offset())
            .unwrap_or(right.location().start_offset());
        let left = self.visit(node.left(), Some(left_next));
        array.append_element(left);

        Self::each_node_with_trailing_end(
            node.requireds().iter(),
            Some(right.location().start_offset()),
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                array.append_element(element);
            },
        );

        let closing_start = node.closing_loc().as_ref().map(|l| l.start_offset());

        let right = self.visit(right, closing_start);
        array.append_element(right);

        let end = self.take_end_trivia_as_virtual_end(closing_start);
        array.set_virtual_end(end);

        fmt::Node::new(fmt::Kind::ArrayPattern(array))
    }

    pub(super) fn parse_hash_pattern(&mut self, node: prism::HashPatternNode) -> fmt::Node {
        let constant = node.constant().map(|c| self.visit(c, None));
        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let should_be_inline = match (opening_loc.as_ref(), node.elements().iter().next()) {
            (Some(opening_loc), Some(first_element)) => !self.does_line_break_exist_in(
                opening_loc.start_offset(),
                first_element.location().start_offset(),
            ),
            _ => true,
        };
        let mut hash = fmt::HashPattern::new(constant, opening, closing, should_be_inline);

        let rest = node.rest();
        let closing_start = closing_loc.as_ref().map(|c| c.start_offset());

        let elements_next = rest
            .as_ref()
            .map(|r| r.location().start_offset())
            .or(closing_start);
        Self::each_node_with_trailing_end(
            node.elements().iter(),
            elements_next,
            |node, trailing_end| {
                let element = self.visit(node, trailing_end);
                hash.append_element(element);
            },
        );

        if let Some(rest) = node.rest() {
            let rest = self.visit(rest, closing_start);
            hash.append_element(rest);
            hash.last_comma_allowed = false;
        }

        let end = self.take_end_trivia_as_virtual_end(closing_start);
        hash.set_virtual_end(end);

        fmt::Node::new(fmt::Kind::HashPattern(hash))
    }

    pub(super) fn parse_pinned_expression(
        &mut self,
        node: prism::PinnedExpressionNode,
    ) -> fmt::Node {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let rparen_start = node.rparen_loc().start_offset();
        let expression = self.visit(node.expression(), Some(rparen_start));

        let mut stmts = fmt::Statements::new();
        stmts.append_node(expression);
        stmts.set_virtual_end(self.take_end_trivia_as_virtual_end(Some(rparen_start)));
        let mut parens = fmt::Parens::new(stmts);
        parens.closing_break_allowed = false;

        let node = fmt::Node::new(fmt::Kind::Parens(parens));
        let prefix = fmt::Prefix::new(operator, Some(node));
        fmt::Node::new(fmt::Kind::Prefix(prefix))
    }

    pub(super) fn parse_pinned_variable(&mut self, node: prism::PinnedVariableNode) -> fmt::Node {
        let operator = Self::source_lossy_at(&node.operator_loc());
        let variable = self.visit(node.variable(), None);
        let prefix = fmt::Prefix::new(operator, Some(variable));
        fmt::Node::new(fmt::Kind::Prefix(prefix))
    }

    pub(super) fn parse_capture_pattern(&mut self, node: prism::CapturePatternNode) -> fmt::Node {
        let value = self.visit(node.value(), Some(node.operator_loc().start_offset()));
        let operator = Self::source_lossy_at(&node.operator_loc());
        let target = self.visit(node.target(), None);
        let assoc = fmt::Assoc::new(value, Some(operator), target);
        fmt::Node::new(fmt::Kind::Assoc(assoc))
    }

    pub(super) fn parse_alternation_pattern(
        &mut self,
        node: prism::AlternationPatternNode,
    ) -> fmt::Node {
        let operator_loc = node.operator_loc();
        let left = self.visit(node.left(), Some(operator_loc.start_offset()));
        let mut chain = match left.kind {
            fmt::Kind::AltPatternChain(chain) => chain,
            _ => fmt::AltPatternChain::new(left),
        };
        let right = node.right();
        let right = self.visit(right, None);
        chain.append_right(right);
        fmt::Node::new(fmt::Kind::AltPatternChain(chain))
    }
}
