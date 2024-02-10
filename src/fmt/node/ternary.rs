use crate::fmt::{shape::Shape, TrailingTrivia};

use super::Node;

#[derive(Debug)]
pub(crate) struct Ternary {
    pub shape: Shape,
    pub predicate: Box<Node>,
    pub predicate_trailing: TrailingTrivia,
    pub then: Box<Node>,
    pub otherwise: Box<Node>,
}

impl Ternary {
    pub(crate) fn new(
        predicate: Node,
        predicate_trailing: TrailingTrivia,
        then: Node,
        otherwise: Node,
    ) -> Self {
        let shape = predicate
            .shape
            .add(&Shape::inline(" ? ".len()))
            .add(predicate_trailing.shape())
            .add(&then.shape)
            .add(&Shape::inline(" : ".len()))
            .add(&otherwise.shape);
        Self {
            shape,
            predicate: Box::new(predicate),
            predicate_trailing,
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }
}
