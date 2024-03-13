use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::Statements;

#[derive(Debug)]
pub(crate) struct Parens {
    pub shape: Shape,
    pub body: Statements,
    pub closing_break_allowed: bool,
}

impl Parens {
    pub(crate) fn new(body: Statements) -> Self {
        let mut shape = Shape::inline("()".len());
        shape.insert(&body.shape);
        Self {
            shape,
            body,
            closing_break_allowed: true,
        }
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
                if self.closing_break_allowed {
                    o.break_line(ctx);
                    o.put_indent_if_needed();
                }
            }
            o.push(')');
        }
    }
}
