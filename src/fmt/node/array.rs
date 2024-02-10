use crate::fmt::shape::Shape;

use super::{Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Array {
    pub shape: Shape,
    pub opening: Option<String>,
    pub closing: Option<String>,
    pub elements: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl Array {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |s| s.len());
        let closing_len = closing.as_ref().map_or(0, |s| s.len());
        let shape = Shape::inline(opening_len + closing_len);
        Self {
            shape,
            opening,
            closing,
            elements: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn separator(&self) -> &str {
        if let Some(opening) = &self.opening {
            if opening.as_bytes()[0] == b'%' {
                return "";
            }
        }
        ","
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if !self.elements.is_empty() {
            let sep_len = self.separator().len() + 1; // space
            self.shape.insert(&Shape::inline(sep_len));
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
