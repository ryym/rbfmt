use crate::fmt::{shape::Shape, LeadingTrivia};

#[derive(Debug)]
pub(crate) struct VirtualEnd {
    pub shape: Shape,
    pub leading_trivia: LeadingTrivia,
}

impl VirtualEnd {
    pub(crate) fn new(leading_trivia: LeadingTrivia) -> Self {
        Self {
            shape: *leading_trivia.shape(),
            leading_trivia,
        }
    }
}
