use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    TrailingTrivia,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct Ternary {
    pub shape: Shape,
    pub predicate: Box<Node>,
    pub predicate_trailing: TrailingTrivia,
    pub then: Box<Node>,
    pub otherwise: Box<Node>,
}

impl Ternary {
    pub(crate) fn new(
        predicate: Node,
        predicate_trailing: TrailingTrivia,
        then: Node,
        otherwise: Node,
    ) -> Self {
        let shape = predicate
            .shape
            .add(&Shape::inline(" ? ".len()))
            .add(predicate_trailing.shape())
            .add(&then.shape)
            .add(&Shape::inline(" : ".len()))
            .add(&otherwise.shape);
        Self {
            shape,
            predicate: Box::new(predicate),
            predicate_trailing,
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        // Format `predicate`.
        o.format(&self.predicate, ctx);
        o.push_str(" ?");

        // Format `then`.
        if self.predicate_trailing.is_none() && self.then.shape.fits_in_one_line(o.remaining_width)
        {
            o.push(' ');
            o.format(&self.then, ctx);
            self.then.trailing_trivia.format(o);
        } else {
            self.predicate_trailing.format(o);
            o.indent();
            o.break_line(ctx);
            self.then
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            o.format(&self.then, ctx);
            self.then.trailing_trivia.format(o);
            o.dedent();
        }

        // Format `otherwise`.
        if self.predicate_trailing.is_none()
            && self.then.shape.is_inline()
            && self.otherwise.shape.fits_in_one_line(o.remaining_width)
        {
            o.push_str(" : ");
            o.format(&self.otherwise, ctx);
            self.otherwise.trailing_trivia.format(o);
        } else {
            o.break_line(ctx);
            o.push(':');
            if self.otherwise.shape.fits_in_one_line(o.remaining_width)
                || self.otherwise.is_diagonal()
            {
                o.push(' ');
                o.format(&self.otherwise, ctx);
                self.otherwise.trailing_trivia.format(o);
            } else {
                o.indent();
                o.break_line(ctx);
                self.otherwise
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::trim());
                o.format(&self.otherwise, ctx);
                self.otherwise.trailing_trivia.format(o);
                o.dedent();
            }
        }
    }
}
