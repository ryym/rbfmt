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
        if self.index.shape.fits_in_inline(o.remaining_width) || self.index.is_diagonal() {
            o.push(' ');
            o.format(&self.index, ctx);
        } else {
            o.indent();
            o.break_line(ctx);
            o.write_leading_trivia(
                &self.index.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            o.format(&self.index, ctx);
            o.dedent();
        }
        o.push_str(" in");
        let collection = &self.collection;
        if collection.shape.fits_in_inline(o.remaining_width) || collection.is_diagonal() {
            o.push(' ');
            o.format(collection, ctx);
            o.write_trailing_comment(&collection.trailing_trivia);
        } else {
            o.indent();
            o.break_line(ctx);
            o.write_leading_trivia(
                &collection.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            o.format(collection, ctx);
            o.write_trailing_comment(&collection.trailing_trivia);
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
