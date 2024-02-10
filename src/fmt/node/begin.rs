use crate::fmt::{shape::Shape, TrailingTrivia};

use super::BlockBody;

#[derive(Debug)]
pub(crate) struct Begin {
    pub keyword_trailing: TrailingTrivia,
    pub body: BlockBody,
}

impl Begin {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}
