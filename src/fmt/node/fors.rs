use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{Node, Statements};

#[derive(Debug)]
pub(crate) struct For {
    pub index: Box<Node>,
    pub collection: Box<Node>,
    pub body: Statements,
}

impl For {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("for");
        if self.index.shape.fits_in_inline(o.remaining_width) || self.index.can_continue_line() {
            o.push(' ');
            self.index.format(o, ctx);
        } else {
            o.indent();
            o.break_line(ctx);
            self.index
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            self.index.format(o, ctx);
            o.dedent();
        }
        o.push_str(" in");
        let collection = &self.collection;
        if collection.shape.fits_in_inline(o.remaining_width) || collection.can_continue_line() {
            o.push(' ');
            collection.format(o, ctx);
            collection.trailing_trivia.format(o);
        } else {
            o.indent();
            o.break_line(ctx);
            collection
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            collection.format(o, ctx);
            collection.trailing_trivia.format(o);
            o.dedent();
        }
        if !self.body.shape().is_empty() {
            o.indent();
            o.break_line(ctx);
            self.body.format(o, ctx, true);
            o.dedent();
        }
        o.break_line(ctx);
        o.push_str("end");
    }
}
