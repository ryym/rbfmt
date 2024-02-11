use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::Conditional;

#[derive(Debug)]
pub(crate) struct While {
    pub is_while: bool,
    pub content: Conditional,
}

impl While {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.is_while {
            o.push_str("while");
        } else {
            o.push_str("until");
        }
        self.content.format(o, ctx);
        if !self.content.body.shape().is_empty() {
            o.indent();
            o.break_line(ctx);
            self.content.body.format(o, ctx, true);
            o.dedent();
        }
        o.break_line(ctx);
        o.push_str("end");
    }
}
