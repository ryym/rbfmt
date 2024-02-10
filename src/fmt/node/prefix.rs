use crate::fmt::shape::Shape;

use super::Node;

#[derive(Debug)]
pub(crate) struct Prefix {
    pub shape: Shape,
    pub operator: String,
    pub expression: Option<Box<Node>>,
}

impl Prefix {
    pub(crate) fn new(operator: String, expression: Option<Node>) -> Self {
        let mut shape = Shape::inline(operator.len());
        if let Some(expr) = &expression {
            shape.append(&expr.shape);
        }
        Self {
            shape,
            operator,
            expression: expression.map(Box::new),
        }
    }
}
