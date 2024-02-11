use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct Alias {
    pub shape: Shape,
    pub new_name: Box<Node>,
    pub old_name: Box<Node>,
}

impl Alias {
    pub(crate) fn new(new_name: Node, old_name: Node) -> Self {
        let shape = new_name.shape.add(&old_name.shape);
        Self {
            shape,
            new_name: Box::new(new_name),
            old_name: Box::new(old_name),
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("alias ");
        o.format(&self.new_name, ctx);
        o.push(' ');
        o.format(&self.old_name, ctx);
    }
}
