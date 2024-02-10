use crate::fmt::shape::Shape;

use super::{BlockBody, Node};

#[derive(Debug)]
pub(crate) struct SingletonClass {
    pub expression: Box<Node>,
    pub body: BlockBody,
}

impl SingletonClass {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}
