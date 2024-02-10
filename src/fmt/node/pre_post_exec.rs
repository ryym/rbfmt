use crate::fmt::shape::Shape;

use super::Statements;

#[derive(Debug)]
pub(crate) struct PrePostExec {
    pub shape: Shape,
    pub keyword: String,
    pub statements: Statements,
}

impl PrePostExec {
    pub(crate) fn new(keyword: String, statements: Statements, was_flat: bool) -> Self {
        let shape = if was_flat {
            Shape::inline(keyword.len()).add(&statements.shape())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            keyword,
            statements,
        }
    }
}
