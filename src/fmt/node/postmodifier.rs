use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::Conditional;

#[derive(Debug)]
pub(crate) struct Postmodifier {
    pub shape: Shape,
    pub keyword: String,
    pub conditional: Conditional,
}

impl Postmodifier {
    pub(crate) fn new(keyword: String, conditional: Conditional) -> Self {
        let kwd_shape = Shape::inline(keyword.len() + 2); // keyword and spaces around it.
        let shape = conditional.shape.add(&kwd_shape);
        Self {
            shape,
            keyword,
            conditional,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.conditional.body.format(o, ctx, false);
        o.push(' ');
        o.push_str(&self.keyword);
        let cond = &self.conditional;
        if cond.predicate.is_diagonal() {
            o.push(' ');
            cond.predicate.format(o, ctx);
            cond.predicate.trailing_trivia.format(o);
        } else {
            o.indent();
            o.break_line(ctx);
            cond.predicate
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            cond.predicate.format(o, ctx);
            cond.predicate.trailing_trivia.format(o);
            o.dedent();
        }
    }
}
