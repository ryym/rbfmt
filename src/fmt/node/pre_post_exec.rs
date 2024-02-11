use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::Statements;

#[derive(Debug)]
pub(crate) struct PrePostExec {
    pub shape: Shape,
    pub keyword: String,
    pub statements: Statements,
}

impl PrePostExec {
    pub(crate) fn new(keyword: String, statements: Statements, was_flat: bool) -> Self {
        let shape = if was_flat {
            Shape::inline(keyword.len()).add(&statements.shape())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            keyword,
            statements,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.shape.fits_in_one_line(o.remaining_width) {
            o.push_str(&self.keyword);
            o.push_str(" {");
            if !self.statements.shape.is_empty() {
                o.push(' ');
                self.statements.format(o, ctx, false);
                o.push(' ');
            }
            o.push('}');
        } else {
            o.push_str(&self.keyword);
            o.push_str(" {");
            if !self.statements.shape.is_empty() {
                o.indent();
                o.break_line(ctx);
                self.statements.format(o, ctx, true);
                o.dedent();
            }
            o.break_line(ctx);
            o.push('}');
        }
    }
}
