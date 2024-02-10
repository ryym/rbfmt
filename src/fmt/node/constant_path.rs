use crate::fmt::{shape::Shape, LeadingTrivia};

use super::Node;

#[derive(Debug)]
pub(crate) struct ConstantPath {
    pub shape: Shape,
    pub root: Option<Box<Node>>,
    pub parts: Vec<(LeadingTrivia, String)>,
}

impl ConstantPath {
    pub(crate) fn new(root: Option<Node>) -> Self {
        let shape = root.as_ref().map_or(Shape::inline(0), |r| r.shape);
        Self {
            shape,
            root: root.map(Box::new),
            parts: vec![],
        }
    }

    pub(crate) fn append_part(&mut self, leading: LeadingTrivia, path: String) {
        self.shape.append(leading.shape());
        self.shape.append(&Shape::inline("::".len() + path.len()));
        self.parts.push((leading, path));
    }
}
