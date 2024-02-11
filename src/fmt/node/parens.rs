use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::Statements;

#[derive(Debug)]
pub(crate) struct Parens {
    pub shape: Shape,
    pub body: Statements,
}

impl Parens {
    pub(crate) fn new(body: Statements) -> Self {
        let mut shape = Shape::inline("()".len());
        shape.insert(&body.shape);
        Self { shape, body }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.body.shape().is_empty() {
            o.push_str("()");
        } else {
            o.push('(');
            if self.body.shape.fits_in_inline(o.remaining_width) {
                self.body.format(o, ctx, false);
            } else {
                o.indent();
                o.break_line(ctx);
                self.body.format(o, ctx, true);
                o.dedent();
                o.break_line(ctx);
            }
            o.push(')');
        }
    }
}
