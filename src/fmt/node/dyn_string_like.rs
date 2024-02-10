use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::{Statements, StringLike};

#[derive(Debug)]
pub(crate) struct DynStringLike {
    pub shape: Shape,
    pub opening: Option<String>,
    pub parts: Vec<DynStrPart>,
    pub closing: Option<String>,
}

impl DynStringLike {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |s| s.len());
        let closing_len = closing.as_ref().map_or(0, |s| s.len());
        Self {
            shape: Shape::inline(opening_len + closing_len),
            opening,
            parts: vec![],
            closing,
        }
    }

    pub(crate) fn append_part(&mut self, part: DynStrPart) {
        self.shape.insert(part.shape());
        self.parts.push(part);
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if let Some(opening) = &self.opening {
            o.push_str(opening);
        }
        let mut divided = false;
        for part in &self.parts {
            if divided {
                o.push(' ');
            }
            match part {
                DynStrPart::Str(str) => {
                    divided = str.opening.is_some();
                    str.format(o);
                }
                DynStrPart::DynStr(dstr) => {
                    divided = true;
                    dstr.format(o, ctx);
                }
                DynStrPart::Statements(embedded) => {
                    embedded.format(o, ctx);
                }
                DynStrPart::Variable(var) => {
                    var.format(o);
                }
            }
        }
        if let Some(closing) = &self.closing {
            o.push_str(closing);
        }
    }
}

#[derive(Debug)]
pub(crate) enum DynStrPart {
    Str(StringLike),
    DynStr(DynStringLike),
    Statements(EmbeddedStatements),
    Variable(EmbeddedVariable),
}

impl DynStrPart {
    pub(crate) fn shape(&self) -> &Shape {
        match self {
            Self::Str(s) => &s.shape,
            Self::DynStr(s) => &s.shape,
            Self::Statements(e) => &e.shape,
            Self::Variable(s) => &s.shape,
        }
    }
}

#[derive(Debug)]
pub(crate) struct EmbeddedStatements {
    pub shape: Shape,
    pub opening: String,
    pub statements: Statements,
    pub closing: String,
}

impl EmbeddedStatements {
    pub(crate) fn new(opening: String, statements: Statements, closing: String) -> Self {
        let shape = Shape::inline(opening.len() + closing.len()).add(&statements.shape);
        Self {
            shape,
            opening,
            statements,
            closing,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str(&self.opening);

        if self.shape.is_inline() {
            let remaining = o.remaining_width;
            o.remaining_width = usize::MAX;
            o.format_statements(&self.statements, ctx, false);
            o.remaining_width = remaining;
        } else {
            o.indent();
            o.break_line(ctx);
            o.format_statements(&self.statements, ctx, true);
            o.break_line(ctx);
            o.dedent();
        }

        o.push_str(&self.closing);
    }
}

#[derive(Debug)]
pub(crate) struct EmbeddedVariable {
    pub shape: Shape,
    pub operator: String,
    pub variable: String,
}

impl EmbeddedVariable {
    pub(crate) fn new(operator: String, variable: String) -> Self {
        let shape = Shape::inline(operator.len() + variable.len());
        Self {
            shape,
            operator,
            variable,
        }
    }

    pub(crate) fn format(&self, o: &mut Output) {
        o.push_str(&self.operator);
        o.push_str(&self.variable);
    }
}
