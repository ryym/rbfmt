use crate::fmt::shape::Shape;

use super::Statements;

#[derive(Debug)]
pub(crate) struct Parens {
    pub shape: Shape,
    pub body: Statements,
}

impl Parens {
    pub(crate) fn new(body: Statements) -> Self {
        let mut shape = Shape::inline("()".len());
        shape.insert(&body.shape);
        Self { shape, body }
    }
}
