use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{Kind, Node};

#[derive(Debug)]
pub(crate) struct RangeLike {
    pub shape: Shape,
    pub left: Option<Box<Node>>,
    pub operator: String,
    pub right: Option<Box<Node>>,
}

impl RangeLike {
    pub(crate) fn new(left: Option<Node>, operator: String, right: Option<Node>) -> Self {
        let mut shape = Shape::inline(0);
        if let Some(left) = &left {
            shape.append(&left.shape);
        }
        shape.append(&Shape::inline(operator.len()));
        if let Some(right) = &right {
            shape.append(&right.shape);
        }
        Self {
            shape,
            left: left.map(Box::new),
            operator,
            right: right.map(Box::new),
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if let Some(left) = &self.left {
            left.format(o, ctx);
        }
        o.push_str(&self.operator);
        if let Some(right) = &self.right {
            if right.shape.fits_in_one_line(o.remaining_width) || right.is_diagonal() {
                let need_space = match &right.kind {
                    Kind::RangeLike(range) => range.left.is_none(),
                    _ => false,
                };
                if need_space {
                    o.push(' ');
                }
                right.format(o, ctx);
            } else {
                o.indent();
                o.break_line(ctx);
                right
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::trim());
                right.format(o, ctx);
                o.dedent();
            }
        }
    }
}
