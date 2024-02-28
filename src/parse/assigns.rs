use crate::fmt;

use super::method_calls;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_variable_assign(
        &mut self,
        name_loc: prism::Location,
        operator_loc: prism::Location,
        value: prism::Node,
    ) -> fmt::Node {
        let name = Self::source_lossy_at(&name_loc);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.parse(value, None);
        let target = fmt::Node::new(fmt::Kind::Atom(fmt::Atom(name)));
        let assign = fmt::Assign::new(target, operator, value);
        fmt::Node::new(fmt::Kind::Assign(assign))
    }

    pub(super) fn parse_constant_path_assign(
        &mut self,
        const_path: prism::ConstantPathNode,
        operator_loc: prism::Location,
        value: prism::Node,
    ) -> fmt::Node {
        let target = self.parse_constant_path(const_path.parent(), const_path.child());
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.parse(value, None);
        let assign = fmt::Assign::new(target, operator, value);
        fmt::Node::new(fmt::Kind::Assign(assign))
    }

    pub(super) fn parse_call_assign(
        &mut self,
        call: &impl method_calls::CallRoot,
        operator_loc: prism::Location,
        value: prism::Node,
    ) -> fmt::Node {
        let target = self.parse_call_root(call);
        let operator = Self::source_lossy_at(&operator_loc);
        let value = self.parse(value, None);
        let assign = fmt::Assign::new(target, operator, value);
        fmt::Node::new(fmt::Kind::Assign(assign))
    }

    pub(super) fn parse_multi_assign(&mut self, node: prism::MultiWriteNode) -> fmt::Node {
        let target = self.parse_multi_assign_target(
            node.lefts(),
            node.rest(),
            node.rights(),
            node.lparen_loc(),
            node.rparen_loc(),
        );
        let operator = Self::source_lossy_at(&node.operator_loc());
        let value = self.parse(node.value(), None);
        let assign = fmt::Assign::new(target, operator, value);
        fmt::Node::new(fmt::Kind::Assign(assign))
    }

    pub(super) fn parse_multi_assign_target(
        &mut self,
        lefts: prism::NodeList,
        rest: Option<prism::Node>,
        rights: prism::NodeList,
        lparen_loc: Option<prism::Location>,
        rparen_loc: Option<prism::Location>,
    ) -> fmt::Node {
        let lparen = lparen_loc.as_ref().map(Self::source_lossy_at);
        let rparen = rparen_loc.as_ref().map(Self::source_lossy_at);
        let mut multi = fmt::MultiAssignTarget::new(lparen, rparen);

        let implicit_rest = rest
            .as_ref()
            .map_or(false, |r| matches!(r, prism::Node::ImplicitRestNode { .. }));
        multi.set_implicit_rest(implicit_rest);

        let rest_start = if implicit_rest {
            None
        } else {
            rest.as_ref().map(|r| r.location().start_offset())
        };
        let rights_first_start = rights.iter().next().map(|n| n.location().start_offset());
        let rparen_start = rparen_loc.as_ref().map(|l| l.start_offset());

        let left_trailing_end = rest_start.or(rights_first_start).or(rparen_start);
        Self::each_node_with_trailing_end(lefts.iter(), left_trailing_end, |node, trailing_end| {
            let target = self.parse(node, trailing_end);
            multi.append_target(target);
        });

        if !implicit_rest {
            if let Some(rest) = rest {
                let rest_trailing_end = rights_first_start.or(rparen_start);
                let target = self.parse(rest, rest_trailing_end);
                multi.append_target(target);
            }
        }

        Self::each_node_with_trailing_end(rights.iter(), rparen_start, |node, trailing_end| {
            let target = self.parse(node, trailing_end);
            multi.append_target(target);
        });

        if let Some(rparen_loc) = rparen_loc {
            let virtual_end = self.take_end_trivia_as_virtual_end(Some(rparen_loc.start_offset()));
            multi.set_virtual_end(virtual_end);
        }

        fmt::Node::new(fmt::Kind::MultiAssignTarget(multi))
    }
}
