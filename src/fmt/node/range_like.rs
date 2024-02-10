use crate::fmt::shape::Shape;

use super::Node;

#[derive(Debug)]
pub(crate) struct RangeLike {
    pub shape: Shape,
    pub left: Option<Box<Node>>,
    pub operator: String,
    pub right: Option<Box<Node>>,
}

impl RangeLike {
    pub(crate) fn new(left: Option<Node>, operator: String, right: Option<Node>) -> Self {
        let mut shape = Shape::inline(0);
        if let Some(left) = &left {
            shape.append(&left.shape);
        }
        shape.append(&Shape::inline(operator.len()));
        if let Some(right) = &right {
            shape.append(&right.shape);
        }
        Self {
            shape,
            left: left.map(Box::new),
            operator,
            right: right.map(Box::new),
        }
    }
}
