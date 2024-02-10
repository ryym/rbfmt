use crate::fmt::shape::Shape;

use super::Node;

#[derive(Debug)]
pub(crate) struct Assign {
    pub shape: Shape,
    pub target: Box<Node>,
    pub operator: String,
    pub value: Box<Node>,
}

impl Assign {
    pub(crate) fn new(target: Node, operator: String, value: Node) -> Self {
        let shape = target
            .shape
            .add(&value.shape)
            .add(&Shape::inline(operator.len() + "  ".len()));
        Self {
            shape,
            target: Box::new(target),
            operator,
            value: Box::new(value),
        }
    }
}
