use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

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

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str(&self.operator);
        if let Some(expr) = &self.expression {
            o.format(expr, ctx);
        }
    }
}
