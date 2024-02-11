use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct Assign {
    pub shape: Shape,
    pub target: Box<Node>,
    pub operator: String,
    pub value: Box<Node>,
}

impl Assign {
    pub(crate) fn new(target: Node, operator: String, value: Node) -> Self {
        let shape = target
            .shape
            .add(&value.shape)
            .add(&Shape::inline(operator.len() + "  ".len()));
        Self {
            shape,
            target: Box::new(target),
            operator,
            value: Box::new(value),
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.format(&self.target, ctx);
        o.push(' ');
        o.push_str(&self.operator);
        self.format_assign_right(o, ctx);
    }

    fn format_assign_right(&self, o: &mut Output, ctx: &FormatContext) {
        if self.value.shape.fits_in_one_line(o.remaining_width) || self.value.is_diagonal() {
            o.push(' ');
            o.format(&self.value, ctx);
        } else {
            o.break_line(ctx);
            o.indent();
            o.write_leading_trivia(&self.value.leading_trivia, ctx, EmptyLineHandling::trim());
            o.format(&self.value, ctx);
            o.dedent();
        }
    }
}
