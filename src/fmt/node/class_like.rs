use crate::fmt::{shape::Shape, TrailingTrivia};

use super::{BlockBody, Node};

#[derive(Debug)]
pub(crate) struct ClassLike {
    pub keyword: String,
    pub name: String,
    pub superclass: Option<Box<Node>>,
    pub head_trailing: TrailingTrivia,
    pub body: BlockBody,
}

impl ClassLike {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}
