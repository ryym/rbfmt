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
    pub value: Option<Box<Node>>,
    pub operator: Option<String>,
}

impl Assoc {
    pub(crate) fn new(key: Node, operator: Option<String>, value: Option<Node>) -> Self {
        let mut shape = key.shape;
        if let Some(value) = &value {
            shape.append(&value.shape);
        }
        shape.append(&Shape::inline(1)); // space
        if let Some(op) = &operator {
            shape.append(&Shape::inline(op.len()));
            shape.append(&Shape::inline(1)); // space
        }
        Self {
            shape,
            key: Box::new(key),
            value: value.map(Box::new),
            operator,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.key.format(o, ctx);
        if let Some(value) = &self.value {
            if value.shape.fits_in_inline(o.remaining_width) || value.can_continue_line() {
                if let Some(op) = &self.operator {
                    o.push(' ');
                    o.push_str(op);
                }
                if !value.shape.is_empty() {
                    o.push(' ');
                    value.format(o, ctx);
                }
            } else {
                if let Some(op) = &self.operator {
                    o.push(' ');
                    o.push_str(op);
                }
                o.break_line(ctx);
                o.indent();
                value
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::trim());
                value.format(o, ctx);
                o.dedent();
            }
        }
    }
}
