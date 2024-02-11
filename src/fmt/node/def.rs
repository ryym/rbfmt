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
            if receiver.shape.fits_in_one_line(o.remaining_width) || receiver.is_diagonal() {
                o.push(' ');
                o.format(receiver, ctx);
            } else {
                o.indent();
                o.break_line(ctx);
                // no leading trivia here.
                o.format(receiver, ctx);
            }
            o.push('.');
            if receiver.trailing_trivia.is_none() {
                o.push_str(&self.name);
                if let Some(params) = &self.parameters {
                    params.format(o, ctx);
                }
            } else {
                o.write_trailing_comment(&receiver.trailing_trivia);
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
                if body.shape.fits_in_one_line(o.remaining_width) || body.is_diagonal() {
                    o.push(' ');
                    o.format(body, ctx);
                    o.write_trailing_comment(&body.trailing_trivia);
                } else {
                    o.indent();
                    o.break_line(ctx);
                    o.write_leading_trivia(&body.leading_trivia, ctx, EmptyLineHandling::trim());
                    o.format(body, ctx);
                    o.write_trailing_comment(&body.trailing_trivia);
                    o.dedent();
                }
            }
            // self foo\n body\n end
            DefBody::Block {
                head_trailing,
                body,
            } => {
                o.write_trailing_comment(head_trailing);
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
                o.format(n, ctx);
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
                    o.write_leading_trivia(
                        &n.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    o.format(n, ctx);
                    if i < last_idx {
                        o.push(',');
                    }
                    o.write_trailing_comment(&n.trailing_trivia);
                }
            }
            o.write_trivia_at_virtual_end(ctx, &self.virtual_end, true, self.params.is_empty());
            o.dedent();
            o.break_line(ctx);
            o.push(')');
        }
    }
}
