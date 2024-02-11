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
        if let Some(superclass) = &self.superclass {
            o.push_str(" <");
            if superclass.shape.fits_in_one_line(o.remaining_width) || superclass.is_diagonal() {
                o.push(' ');
                superclass.format(o, ctx);
                superclass.trailing_trivia.format(o);
            } else {
                o.indent();
                o.break_line(ctx);
                superclass
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::trim());
                superclass.format(o, ctx);
                superclass.trailing_trivia.format(o);
                o.dedent();
            }
        } else {
            self.head_trailing.format(o);
        }
        self.body.format(o, ctx, true);
        o.break_line(ctx);
        o.push_str("end");
    }
}
