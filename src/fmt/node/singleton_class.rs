use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{BlockBody, Node};

#[derive(Debug)]
pub(crate) struct SingletonClass {
    pub expression: Box<Node>,
    pub body: BlockBody,
}

impl SingletonClass {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("class <<");
        if self.expression.shape.fits_in_one_line(o.remaining_width)
            || self.expression.is_diagonal()
        {
            o.push(' ');
            o.format(&self.expression, ctx);
            o.write_trailing_comment(&self.expression.trailing_trivia);
        } else {
            o.indent();
            o.indent();
            o.break_line(ctx);
            self.expression
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::none());
            o.format(&self.expression, ctx);
            o.write_trailing_comment(&self.expression.trailing_trivia);
            o.dedent();
            o.dedent();
        }
        self.body.format(o, ctx, true);
        o.break_line(ctx);
        o.push_str("end");
    }
}
