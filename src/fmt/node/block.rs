use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
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
                o.format_block_parameters(params, ctx);
            }
            if !self.body.shape.is_empty() {
                o.push(' ');
                o.format_block_body(&self.body, ctx, false);
                o.push(' ');
            }
            if &self.closing == "end" {
                o.push(' ');
            }
            o.push_str(&self.closing);
        } else {
            o.push(' ');
            o.push_str(&self.opening);
            o.write_trailing_comment(&self.opening_trailing);
            if let Some(params) = &self.parameters {
                if self.opening_trailing.is_none() {
                    o.push(' ');
                    o.format_block_parameters(params, ctx);
                } else {
                    o.indent();
                    o.break_line(ctx);
                    o.format_block_parameters(params, ctx);
                    o.dedent();
                }
            }
            if !self.body.shape.is_empty() {
                o.format_block_body(&self.body, ctx, true);
            }
            o.break_line(ctx);
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
}
