use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct MatchAssign {
    shape: Shape,
    expression: Box<Node>,
    operator: String,
    pattern: Box<Node>,
}

impl MatchAssign {
    pub(crate) fn new(expression: Node, operator: String, pattern: Node) -> Self {
        let shape = expression
            .shape
            .add(&Shape::inline(operator.len() + "  ".len()))
            .add(&pattern.shape);
        Self {
            shape,
            expression: Box::new(expression),
            operator,
            pattern: Box::new(pattern),
        }
    }

    pub(crate) fn shape(&self) -> &Shape {
        &self.shape
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.expression.format(o, ctx);
        o.push(' ');
        o.push_str(&self.operator);
        self.format_right_hand_side(o, ctx);
    }

    fn format_right_hand_side(&self, o: &mut Output, ctx: &FormatContext) {
        if self.pattern.shape.fits_in_one_line(o.remaining_width)
            || self.pattern.can_continue_line()
        {
            o.push(' ');
            self.pattern.format(o, ctx);
        } else {
            o.break_line(ctx);
            o.indent();
            self.pattern
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            o.put_indent_if_needed();
            self.pattern.format(o, ctx);
            o.dedent();
        }
    }
}
