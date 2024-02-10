use crate::fmt::shape::Shape;

use super::Node;

#[derive(Debug)]
pub(crate) struct Assoc {
    pub shape: Shape,
    pub key: Box<Node>,
    pub value: Box<Node>,
    pub operator: Option<String>,
}

impl Assoc {
    pub(crate) fn new(key: Node, operator: Option<String>, value: Node) -> Self {
        let mut shape = key.shape.add(&value.shape);
        shape.append(&Shape::inline(1)); // space
        if let Some(op) = &operator {
            shape.append(&Shape::inline(op.len()));
            shape.append(&Shape::inline(1)); // space
        }
        Self {
            shape,
            key: Box::new(key),
            value: Box::new(value),
            operator,
        }
    }
}
