use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    LeadingTrivia, TrailingTrivia,
};

use super::{Else, Node, Statements};

#[derive(Debug)]
pub(crate) struct Case {
    pub predicate: Option<Box<Node>>,
    pub case_trailing: TrailingTrivia,
    pub first_branch_leading: LeadingTrivia,
    pub branches: Vec<CaseWhen>,
    pub otherwise: Option<Else>,
}

impl Case {
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
pub(crate) struct CaseWhen {
    pub shape: Shape,
    pub conditions: Vec<Node>,
    pub conditions_shape: Shape,
    pub body: Statements,
}

impl CaseWhen {
    pub(crate) fn new(was_flat: bool) -> Self {
        let shape = if was_flat {
            Shape::inline(0)
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            conditions: vec![],
            conditions_shape: Shape::inline(0),
            body: Statements::new(),
        }
    }

    pub(crate) fn append_condition(&mut self, cond: Node) {
        self.shape.append(&cond.shape);
        self.conditions_shape.append(&cond.shape);
        self.conditions.push(cond);
    }

    pub(crate) fn set_body(&mut self, body: Statements) {
        self.shape.append(&body.shape);
        self.body = body;
    }

    fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("when");
        if self.shape.fits_in_one_line(o.remaining_width) {
            o.push(' ');
            for (i, cond) in self.conditions.iter().enumerate() {
                if i > 0 {
                    o.push_str(", ");
                }
                cond.format(o, ctx);
                cond.trailing_trivia.format(o);
            }
            if !self.body.shape.is_empty() {
                o.push_str(" then ");
                self.body.format(o, ctx, false);
            }
        } else {
            if self.conditions_shape.fits_in_one_line(o.remaining_width) {
                for (i, cond) in self.conditions.iter().enumerate() {
                    if i == 0 {
                        o.push(' ');
                    } else {
                        o.push_str(", ");
                    }
                    cond.format(o, ctx);
                    cond.trailing_trivia.format(o);
                }
            } else {
                if self.conditions[0].can_continue_line() {
                    o.push(' ');
                    o.indent();
                    self.conditions[0].format(o, ctx);
                    o.dedent();
                } else {
                    o.indent();
                    o.indent();
                    o.break_line(ctx);
                    self.conditions[0]
                        .leading_trivia
                        .format(o, ctx, EmptyLineHandling::trim());
                    self.conditions[0].format(o, ctx);
                    o.dedent();
                    o.dedent();
                }
                if self.conditions.len() > 1 {
                    o.push(',');
                }
                self.conditions[0].trailing_trivia.format(o);
                if self.conditions.len() > 1 {
                    o.indent();
                    o.indent();
                    let last_idx = self.conditions.len() - 1;
                    for (i, cond) in self.conditions.iter().enumerate().skip(1) {
                        o.break_line(ctx);
                        cond.leading_trivia
                            .format(o, ctx, EmptyLineHandling::none());
                        cond.format(o, ctx);
                        if i < last_idx {
                            o.push(',');
                        }
                        cond.trailing_trivia.format(o);
                    }
                    o.dedent();
                    o.dedent();
                }
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
