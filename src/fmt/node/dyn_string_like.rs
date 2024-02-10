use crate::fmt::shape::Shape;

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
}
