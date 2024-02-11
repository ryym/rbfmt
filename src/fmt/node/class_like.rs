use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    TrailingTrivia,
};

use super::{BlockBody, Node};

#[derive(Debug)]
pub(crate) struct ClassLike {
    pub keyword: String,
    pub name: String,
    pub superclass: Option<Box<Node>>,
    pub head_trailing: TrailingTrivia,
    pub body: BlockBody,
}

impl ClassLike {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str(&self.keyword);
        o.push(' ');
        o.push_str(&self.name);
        if let Some(superself) = &self.superclass {
            o.push_str(" <");
            if superself.shape.fits_in_one_line(o.remaining_width) || superself.is_diagonal() {
                o.push(' ');
                o.format(superself, ctx);
                o.write_trailing_comment(&superself.trailing_trivia);
            } else {
                o.indent();
                o.break_line(ctx);
                o.write_leading_trivia(&superself.leading_trivia, ctx, EmptyLineHandling::trim());
                o.format(superself, ctx);
                o.write_trailing_comment(&superself.trailing_trivia);
                o.dedent();
            }
        } else {
            o.write_trailing_comment(&self.head_trailing);
        }
        self.body.format(o, ctx, true);
        o.break_line(ctx);
        o.push_str("end");
    }
}
