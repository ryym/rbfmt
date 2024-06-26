use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    TrailingTrivia,
};

use super::{Else, Node, Statements, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Block {
    pub shape: Shape,
    pub opening: String,
    pub closing: String,
    pub opening_trailing: TrailingTrivia,
    pub parameters: Option<BlockParameters>,
    pub body: BlockBody,
}

impl Block {
    pub(crate) fn new(was_flat: bool, opening: String, closing: String) -> Self {
        let shape = if was_flat {
            Shape::inline(opening.len() + closing.len())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            opening,
            closing,
            opening_trailing: TrailingTrivia::none(),
            parameters: None,
            body: BlockBody::new(Statements::new()),
        }
    }

    pub(crate) fn set_opening_trailing(&mut self, trailing: TrailingTrivia) {
        self.shape.insert(trailing.shape());
        self.opening_trailing = trailing;
    }

    pub(crate) fn set_parameters(&mut self, parameters: BlockParameters) {
        self.shape.insert(&Shape::inline(" ".len()));
        self.shape.insert(&parameters.shape);
        self.parameters = Some(parameters);
    }

    pub(crate) fn set_body(&mut self, body: BlockBody) {
        self.shape.insert(&Shape::inline("  ".len()));
        self.shape.insert(&body.shape);
        self.body = body;
    }

    pub(crate) fn is_empty(&self) -> bool {
        !matches!(self.shape, Shape::Multilines) && self.body.is_empty()
    }

    pub(crate) fn min_first_line_len(&self) -> usize {
        let params_opening_len = self.parameters.as_ref().map_or(0, |_| 2); // " |"
        self.opening.len() + params_opening_len
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.shape.fits_in_one_line(o.remaining_width) {
            o.push(' ');
            o.push_str(&self.opening);
            if let Some(params) = &self.parameters {
                o.push(' ');
                params.format(o, ctx);
            }
            if !self.body.shape.is_empty() {
                o.push(' ');
                self.body.format(o, ctx, false);
                o.push(' ');
            }
            if &self.closing == "end" {
                o.push(' ');
            }
            o.push_str(&self.closing);
        } else {
            o.push(' ');
            o.push_str(&self.opening);
            self.opening_trailing.format(o);
            if let Some(params) = &self.parameters {
                if self.opening_trailing.is_none() {
                    o.push(' ');
                    params.format(o, ctx);
                } else {
                    o.indent();
                    o.break_line(ctx);
                    o.put_indent_if_needed();
                    params.format(o, ctx);
                    o.dedent();
                }
            }
            if !self.body.shape.is_empty() {
                self.body.format(o, ctx, true);
            }
            o.break_line(ctx);
            o.put_indent_if_needed();
            o.push_str(&self.closing);
        }
    }
}

#[derive(Debug)]
pub(crate) struct BlockParameters {
    pub shape: Shape,
    pub opening: String,
    pub closing: String,
    pub params: Vec<Node>,
    pub locals: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
    pub closing_trailing: TrailingTrivia,
}

impl BlockParameters {
    pub(crate) fn new(opening: String, closing: String) -> Self {
        let shape = Shape::inline(opening.len() + closing.len());
        Self {
            shape,
            opening,
            closing,
            params: vec![],
            locals: vec![],
            closing_trailing: TrailingTrivia::none(),
            virtual_end: None,
        }
    }

    pub(crate) fn append_param(&mut self, node: Node) {
        if !self.params.is_empty() {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.shape.insert(&node.shape);
        self.params.push(node);
    }

    pub(crate) fn append_local(&mut self, node: Node) {
        if self.locals.is_empty() {
            self.shape.insert(&Shape::inline("; ".len()));
        }
        self.shape.insert(&node.shape);
        self.locals.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn set_closing_trailing(&mut self, trailing: TrailingTrivia) {
        self.shape.append(trailing.shape());
        self.closing_trailing = trailing;
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.shape.fits_in_one_line(o.remaining_width) {
            o.push_str(&self.opening);
            for (i, n) in self.params.iter().enumerate() {
                if n.shape.is_empty() {
                    o.push(',');
                } else {
                    if i > 0 {
                        o.push_str(", ");
                    }
                    n.format(o, ctx);
                }
            }
            if !self.locals.is_empty() {
                o.push_str("; ");
                for (i, n) in self.locals.iter().enumerate() {
                    if i > 0 {
                        o.push_str(", ");
                    }
                    n.format(o, ctx);
                }
            }
            o.push_str(&self.closing);
            self.closing_trailing.format(o);
        } else {
            o.push_str(&self.opening);
            o.indent();
            if !self.params.is_empty() {
                let last_idx = self.params.len() - 1;
                for (i, n) in self.params.iter().enumerate() {
                    if n.shape.is_empty() {
                        n.trailing_trivia.format(o);
                        continue;
                    }
                    o.break_line(ctx);
                    n.leading_trivia.format(
                        o,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    o.put_indent_if_needed();
                    n.format(o, ctx);
                    if i < last_idx {
                        o.push(',');
                    }
                    n.trailing_trivia.format(o);
                }
            }
            if !self.locals.is_empty() {
                o.break_line(ctx);
                o.put_indent_if_needed();
                o.push(';');
                let last_idx = self.locals.len() - 1;
                for (i, n) in self.locals.iter().enumerate() {
                    o.break_line(ctx);
                    n.leading_trivia.format(o, ctx, EmptyLineHandling::trim());
                    o.put_indent_if_needed();
                    n.format(o, ctx);
                    if i < last_idx {
                        o.push(',');
                    }
                    n.trailing_trivia.format(o);
                }
            }
            o.write_trivia_at_virtual_end(ctx, &self.virtual_end, true, self.params.is_empty());
            o.dedent();
            o.break_line(ctx);
            o.put_indent_if_needed();
            o.push_str(&self.closing);
            self.closing_trailing.format(o);
        }
    }
}

#[derive(Debug)]
pub(crate) struct BlockBody {
    pub shape: Shape,
    pub statements: Statements,
    pub rescues: Vec<Rescue>,
    pub rescue_else: Option<Else>,
    pub ensure: Option<Else>,
}

impl BlockBody {
    pub(crate) fn new(statements: Statements) -> Self {
        Self {
            shape: statements.shape(),
            statements,
            rescues: vec![],
            rescue_else: None,
            ensure: None,
        }
    }

    pub(crate) fn set_rescues(&mut self, rescues: Vec<Rescue>) {
        if !rescues.is_empty() {
            self.shape = Shape::Multilines;
        }
        self.rescues = rescues;
    }

    pub(crate) fn set_rescue_else(&mut self, rescue_else: Else) {
        self.shape = Shape::Multilines;
        self.rescue_else = Some(rescue_else);
    }

    pub(crate) fn set_ensure(&mut self, ensure: Else) {
        self.shape = Shape::Multilines;
        self.ensure = Some(ensure);
    }

    fn is_empty(&self) -> bool {
        self.statements.is_empty() && self.rescues.is_empty() && self.ensure.is_none()
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext, block_always: bool) {
        if self.shape.fits_in_inline(o.remaining_width) && !block_always {
            self.statements.format(o, ctx, block_always);
            return;
        }

        if !self.statements.shape().is_empty() {
            o.indent();
            o.break_line(ctx);
            self.statements.format(o, ctx, true);
            o.dedent();
        }
        for rescue in &self.rescues {
            o.break_line(ctx);
            o.put_indent_if_needed();
            rescue.format(o, ctx);
        }
        if let Some(rescue_else) = &self.rescue_else {
            o.break_line(ctx);
            o.put_indent_if_needed();
            o.push_str("else");
            rescue_else.keyword_trailing.format(o);
            if !rescue_else.body.shape().is_empty() {
                o.indent();
                o.break_line(ctx);
                rescue_else.body.format(o, ctx, true);
                o.dedent();
            }
        }
        if let Some(ensure) = &self.ensure {
            o.break_line(ctx);
            o.put_indent_if_needed();
            o.push_str("ensure");
            ensure.keyword_trailing.format(o);
            if !ensure.body.shape().is_empty() {
                o.indent();
                o.break_line(ctx);
                ensure.body.format(o, ctx, true);
                o.dedent();
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Rescue {
    pub exceptions: Vec<Node>,
    pub exceptions_shape: Shape,
    pub reference: Option<Box<Node>>,
    pub head_trailing: TrailingTrivia,
    pub statements: Statements,
}

impl Rescue {
    pub(crate) fn new() -> Self {
        Self {
            exceptions: vec![],
            exceptions_shape: Shape::inline(0),
            reference: None,
            head_trailing: TrailingTrivia::none(),
            statements: Statements::new(),
        }
    }

    pub(crate) fn append_exception(&mut self, exception: Node) {
        self.exceptions_shape.append(&exception.shape);
        self.exceptions.push(exception);
    }

    pub(crate) fn set_reference(&mut self, reference: Node) {
        self.reference = Some(Box::new(reference))
    }

    pub(crate) fn set_head_trailing(&mut self, trailing: TrailingTrivia) {
        self.head_trailing = trailing;
    }

    pub(crate) fn set_statements(&mut self, statements: Statements) {
        self.statements = statements;
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("rescue");
        if !self.exceptions.is_empty() {
            if self.exceptions_shape.fits_in_one_line(o.remaining_width) {
                o.push(' ');
                for (i, exception) in self.exceptions.iter().enumerate() {
                    if i > 0 {
                        o.push_str(", ");
                    }
                    exception.format(o, ctx);
                    exception.trailing_trivia.format(o);
                }
            } else {
                o.push(' ');
                self.exceptions[0].format(o, ctx);
                if self.exceptions.len() > 1 {
                    o.push(',');
                }
                self.exceptions[0].trailing_trivia.format(o);
                if self.exceptions.len() > 1 {
                    o.indent();
                    let last_idx = self.exceptions.len() - 1;
                    for (i, exception) in self.exceptions.iter().enumerate().skip(1) {
                        o.break_line(ctx);
                        exception
                            .leading_trivia
                            .format(o, ctx, EmptyLineHandling::none());
                        o.put_indent_if_needed();
                        exception.format(o, ctx);
                        if i < last_idx {
                            o.push(',');
                        }
                        exception.trailing_trivia.format(o);
                    }
                    o.dedent();
                }
            }
        }
        if let Some(reference) = &self.reference {
            o.push_str(" =>");
            if reference.shape.fits_in_one_line(o.remaining_width) || reference.can_continue_line()
            {
                o.push(' ');
                reference.format(o, ctx);
                reference.trailing_trivia.format(o);
            } else {
                o.indent();
                o.break_line(ctx);
                reference
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::trim());
                o.put_indent_if_needed();
                reference.format(o, ctx);
                reference.trailing_trivia.format(o);
                o.dedent();
            }
        }
        self.head_trailing.format(o);
        if !self.statements.shape().is_empty() {
            o.indent();
            o.break_line(ctx);
            self.statements.format(o, ctx, true);
            o.dedent();
        }
    }
}
