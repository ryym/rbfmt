use crate::fmt::shape::Shape;

#[derive(Debug)]
pub(crate) struct StringLike {
    pub shape: Shape,
    pub opening: Option<String>,
    pub value: Vec<u8>,
    pub closing: Option<String>,
}

impl StringLike {
    pub(crate) fn new(opening: Option<String>, value: Vec<u8>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |s| s.len());
        let closing_len = closing.as_ref().map_or(0, |s| s.len());
        let len = value.len() + opening_len + closing_len;
        let shape = if value.iter().any(|b| *b == b'\n') {
            Shape::Multilines
        } else {
            Shape::inline(len)
        };
        Self {
            shape,
            opening,
            value,
            closing,
        }
    }
}
