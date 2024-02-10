use crate::fmt::shape::Shape;

use super::{Kind, Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Statements {
    pub shape: Shape,
    pub nodes: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl Statements {
    pub(crate) fn new() -> Self {
        Self {
            shape: Shape::inline(0),
            nodes: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        if self.nodes.is_empty() && !matches!(node.kind, Kind::HeredocOpening(_)) {
            self.shape = node.shape;
        } else {
            self.shape = Shape::Multilines;
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn shape(&self) -> Shape {
        self.shape
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.virtual_end.is_none()
    }
}
