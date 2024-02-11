use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    TrailingTrivia,
};

use super::BlockBody;

#[derive(Debug)]
pub(crate) struct Begin {
    pub keyword_trailing: TrailingTrivia,
    pub body: BlockBody,
}

impl Begin {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("begin");
        o.write_trailing_comment(&self.keyword_trailing);
        self.body.format(o, ctx, true);
        o.break_line(ctx);
        o.push_str("end");
    }
}
