use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct Assoc {
    pub shape: Shape,
    pub key: Box<Node>,
    pub value: Box<Node>,
    pub operator: Option<String>,
}

impl Assoc {
    pub(crate) fn new(key: Node, operator: Option<String>, value: Node) -> Self {
        let mut shape = key.shape.add(&value.shape);
        shape.append(&Shape::inline(1)); // space
        if let Some(op) = &operator {
            shape.append(&Shape::inline(op.len()));
            shape.append(&Shape::inline(1)); // space
        }
        Self {
            shape,
            key: Box::new(key),
            value: Box::new(value),
            operator,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.key.format(o, ctx);
        if self.value.shape.fits_in_inline(o.remaining_width) || self.value.can_continue_line() {
            if let Some(op) = &self.operator {
                o.push(' ');
                o.push_str(op);
            }
            if !self.value.shape.is_empty() {
                o.push(' ');
                self.value.format(o, ctx);
            }
        } else {
            if let Some(op) = &self.operator {
                o.push(' ');
                o.push_str(op);
            }
            o.break_line(ctx);
            o.indent();
            self.value
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            o.put_indent_if_needed();
            self.value.format(o, ctx);
            o.dedent();
        }
    }
}
