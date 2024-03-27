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
    need_separation: bool,
}

impl Prefix {
    pub(crate) fn new(operator: String, expression: Option<Node>, need_separation: bool) -> Self {
        let mut shape = Shape::inline(operator.len());
        if let Some(expr) = &expression {
            if need_separation {
                shape.append(&Shape::inline(1));
            }
            shape.append(&expr.shape);
        }
        Self {
            shape,
            operator,
            expression: expression.map(Box::new),
            need_separation,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str(&self.operator);
        if let Some(expr) = &self.expression {
            if self.need_separation {
                o.push(' ');
            }
            expr.format(o, ctx);
        }
    }
}
