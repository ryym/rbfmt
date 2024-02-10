use crate::fmt::shape::Shape;

use super::Conditional;

#[derive(Debug)]
pub(crate) struct While {
    pub is_while: bool,
    pub content: Conditional,
}

impl While {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}
