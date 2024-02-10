use crate::fmt::shape::Shape;

use super::{Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Hash {
    pub shape: Shape,
    pub opening: String,
    pub closing: String,
    pub elements: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl Hash {
    pub(crate) fn new(opening: String, closing: String, should_be_inline: bool) -> Self {
        let shape = if should_be_inline {
            Shape::inline(opening.len() + closing.len())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            opening,
            closing,
            elements: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if self.elements.is_empty() {
            self.shape.insert(&Shape::inline("  ".len()));
        } else {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.shape.insert(&element.shape);
        self.elements.push(element);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.insert(&end.shape);
        }
        self.virtual_end = end;
    }
}
