use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    TrailingTrivia,
};

use super::{BlockBody, Node, Statements, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Def {
    pub shape: Shape,
    pub receiver: Option<Box<Node>>,
    pub name: String,
    pub parameters: Option<MethodParameters>,
    pub body: DefBody,
}

impl Def {
    pub(crate) fn new(receiver: Option<Node>, name: String) -> Self {
        let mut shape = Shape::inline("def ".len() + name.len());
        if let Some(receiver) = &receiver {
            shape.insert(&receiver.shape);
        }
        Self {
            shape,
            receiver: receiver.map(Box::new),
            name,
            parameters: None,
            body: DefBody::Block {
                head_trailing: TrailingTrivia::none(),
                body: BlockBody::new(Statements::new()),
            },
        }
    }

    pub(crate) fn set_parameters(&mut self, parameters: MethodParameters) {
        self.shape.append(&parameters.shape);
        self.parameters = Some(parameters);
    }

    pub(crate) fn set_body(&mut self, body: DefBody) {
        self.shape.append(&body.shape());
        self.body = body;
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("def");
        if let Some(receiver) = &self.receiver {
            if receiver.shape.fits_in_one_line(o.remaining_width) || receiver.can_continue_line() {
                o.push(' ');
                receiver.format(o, ctx);
            } else {
                o.indent();
                o.break_line(ctx);
                // no leading trivia here.
                receiver.format(o, ctx);
            }
            o.push('.');
            if receiver.trailing_trivia.is_none() {
                o.push_str(&self.name);
                if let Some(params) = &self.parameters {
                    params.format(o, ctx);
                }
            } else {
                receiver.trailing_trivia.format(o);
                o.indent();
                o.break_line(ctx);
                o.push_str(&self.name);
                if let Some(params) = &self.parameters {
                    params.format(o, ctx);
                }
                o.dedent();
            }
        } else {
            o.push(' ');
            o.push_str(&self.name);
            if let Some(params) = &self.parameters {
                params.format(o, ctx);
            }
        }
        match &self.body {
            // self foo = body
            DefBody::Short { body } => {
                o.push_str(" =");
                if body.shape.fits_in_one_line(o.remaining_width) || body.can_continue_line() {
                    o.push(' ');
                    body.format(o, ctx);
                    body.trailing_trivia.format(o);
                } else {
                    o.indent();
                    o.break_line(ctx);
                    body.leading_trivia
                        .format(o, ctx, EmptyLineHandling::trim());
                    body.format(o, ctx);
                    body.trailing_trivia.format(o);
                    o.dedent();
                }
            }
            // self foo\n body\n end
            DefBody::Block {
                head_trailing,
                body,
            } => {
                head_trailing.format(o);
                body.format(o, ctx, true);
                o.break_line(ctx);
                o.push_str("end");
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum DefBody {
    Short {
        body: Box<Node>,
    },
    Block {
        head_trailing: TrailingTrivia,
        body: BlockBody,
    },
}

impl DefBody {
    pub(crate) fn shape(&self) -> Shape {
        match self {
            Self::Short { body } => body.shape,
            Self::Block { .. } => Shape::Multilines,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MethodParameters {
    pub shape: Shape,
    pub opening: Option<String>,
    pub closing: Option<String>,
    pub params: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl MethodParameters {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |o| o.len());
        let closing_len = closing.as_ref().map_or(0, |c| c.len());
        let shape = Shape::inline(opening_len + closing_len);
        Self {
            shape,
            opening,
            closing,
            params: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_param(&mut self, node: Node) {
        self.shape.insert(&node.shape);
        if !self.params.is_empty() {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.params.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(super) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.shape.fits_in_one_line(o.remaining_width) {
            let opening = self.opening.as_deref().unwrap_or(" ");
            o.push_str(opening);
            for (i, n) in self.params.iter().enumerate() {
                if i > 0 {
                    o.push_str(", ");
                }
                n.format(o, ctx);
            }
            if let Some(closing) = &self.closing {
                o.push_str(closing);
            }
        } else {
            o.push('(');
            o.indent();
            if !self.params.is_empty() {
                let last_idx = self.params.len() - 1;
                for (i, n) in self.params.iter().enumerate() {
                    o.break_line(ctx);
                    n.leading_trivia.format(
                        o,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
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
            o.push(')');
        }
    }
}
