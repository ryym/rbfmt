use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    TrailingTrivia,
};

use super::{Node, Statements};

#[derive(Debug)]
pub(crate) struct If {
    pub is_if: bool,
    pub if_first: Conditional,
    pub elsifs: Vec<Conditional>,
    pub if_last: Option<Else>,
}

impl If {
    pub(crate) fn new(is_if: bool, if_first: Conditional) -> Self {
        Self {
            is_if,
            if_first,
            elsifs: vec![],
            if_last: None,
        }
    }

    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.is_if {
            o.push_str("if");
        } else {
            o.push_str("unless");
        }

        self.if_first.format(o, ctx);
        if !self.if_first.body.shape.is_empty() {
            o.indent();
            o.break_line(ctx);
            self.if_first.body.format(o, ctx, true);
            o.dedent();
        }

        for elsif in &self.elsifs {
            o.break_line(ctx);
            o.push_str("elsif");
            elsif.format(o, ctx);
            if !elsif.body.shape.is_empty() {
                o.indent();
                o.break_line(ctx);
                elsif.body.format(o, ctx, true);
                o.dedent();
            }
        }

        if let Some(if_last) = &self.if_last {
            o.break_line(ctx);
            o.push_str("else");
            if_last.keyword_trailing.format(o);
            if !if_last.body.shape.is_empty() {
                o.indent();
                o.break_line(ctx);
                if_last.body.format(o, ctx, true);
                o.dedent();
            }
        }

        o.break_line(ctx);
        o.push_str("end");
    }
}

#[derive(Debug)]
pub(crate) struct Conditional {
    pub shape: Shape,
    pub predicate: Box<Node>,
    pub body: Statements,
}

impl Conditional {
    pub(crate) fn new(predicate: Node, body: Statements) -> Self {
        let shape = predicate.shape.add(&body.shape);
        Self {
            shape,
            predicate: Box::new(predicate),
            body,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.predicate.is_diagonal() {
            o.push(' ');
            o.indent();
            self.predicate.format(o, ctx);
            self.predicate.trailing_trivia.format(o);
            o.dedent();
        } else {
            o.indent();
            o.indent();
            o.break_line(ctx);
            self.predicate
                .leading_trivia
                .format(o, ctx, EmptyLineHandling::trim());
            self.predicate.format(o, ctx);
            self.predicate.trailing_trivia.format(o);
            o.dedent();
            o.dedent();
        }
    }
}

#[derive(Debug)]
pub(crate) struct Else {
    pub keyword_trailing: TrailingTrivia,
    pub body: Statements,
}
