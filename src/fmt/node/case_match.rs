use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    LeadingTrivia, TrailingTrivia,
};

use super::{Else, Node, Statements};

#[derive(Debug)]
pub(crate) struct CaseMatch {
    pub predicate: Option<Box<Node>>,
    pub case_trailing: TrailingTrivia,
    pub first_branch_leading: LeadingTrivia,
    pub branches: Vec<CaseIn>,
    pub otherwise: Option<Else>,
}

impl CaseMatch {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("case");
        match &self.predicate {
            Some(pred) => {
                if pred.shape.fits_in_one_line(o.remaining_width) || pred.can_continue_line() {
                    o.push(' ');
                    pred.format(o, ctx);
                    pred.trailing_trivia.format(o);
                } else {
                    o.indent();
                    o.break_line(ctx);
                    pred.leading_trivia
                        .format(o, ctx, EmptyLineHandling::trim());
                    pred.format(o, ctx);
                    pred.trailing_trivia.format(o);
                    o.dedent();
                }
            }
            None => {
                self.case_trailing.format(o);
            }
        }
        if self.first_branch_leading.is_empty() {
            o.break_line(ctx);
        } else {
            o.indent();
            o.break_line(ctx);
            self.first_branch_leading
                .format(o, ctx, EmptyLineHandling::trim());
            o.dedent();
        }
        for (i, branch) in self.branches.iter().enumerate() {
            if i > 0 {
                o.break_line(ctx);
            }
            branch.format(o, ctx);
        }
        if let Some(otherwise) = &self.otherwise {
            o.break_line(ctx);
            o.push_str("else");
            otherwise.keyword_trailing.format(o);
            if !otherwise.body.shape.is_empty() {
                o.indent();
                o.break_line(ctx);
                otherwise.body.format(o, ctx, true);
                o.dedent();
            }
        }
        o.break_line(ctx);
        o.push_str("end");
    }
}

#[derive(Debug)]
pub(crate) struct CaseIn {
    pub shape: Shape,
    pub pattern: Box<Node>,
    pub body: Statements,
}

impl CaseIn {
    pub(crate) fn new(was_flat: bool, pattern: Node) -> Self {
        let shape = if was_flat {
            Shape::inline("in  then ".len()).add(&pattern.shape)
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            pattern: Box::new(pattern),
            body: Statements::new(),
        }
    }

    pub(crate) fn set_body(&mut self, body: Statements) {
        self.shape.append(&body.shape);
        self.body = body;
    }

    fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("in");
        if self.shape.fits_in_one_line(o.remaining_width) {
            o.push(' ');
            self.pattern.format(o, ctx);
            self.pattern.trailing_trivia.format(o);
            if !self.body.shape.is_empty() {
                o.push_str(" then ");
                self.body.format(o, ctx, false);
            }
        } else {
            if self.pattern.can_continue_line() {
                o.push(' ');
                o.indent();
                self.pattern.format(o, ctx);
                self.pattern.trailing_trivia.format(o);
                o.dedent();
            } else {
                o.indent();
                o.indent();
                o.break_line(ctx);
                self.pattern
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::trim());
                self.pattern.format(o, ctx);
                self.pattern.trailing_trivia.format(o);
                o.dedent();
                o.dedent();
            }
            if !self.body.shape.is_empty() {
                o.indent();
                o.break_line(ctx);
                self.body.format(o, ctx, true);
                o.dedent();
            }
        }
    }
}
