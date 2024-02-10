use crate::fmt::{shape::Shape, TrailingTrivia};

use super::{Node, Statements};

#[derive(Debug)]
pub(crate) struct If {
    pub is_if: bool,
    pub if_first: Conditional,
    pub elsifs: Vec<Conditional>,
    pub if_last: Option<Else>,
}

impl If {
    pub(crate) fn new(is_if: bool, if_first: Conditional) -> Self {
        Self {
            is_if,
            if_first,
            elsifs: vec![],
            if_last: None,
        }
    }

    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct Conditional {
    pub shape: Shape,
    pub predicate: Box<Node>,
    pub body: Statements,
}

impl Conditional {
    pub(crate) fn new(predicate: Node, body: Statements) -> Self {
        let shape = predicate.shape.add(&body.shape);
        Self {
            shape,
            predicate: Box::new(predicate),
            body,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Else {
    pub keyword_trailing: TrailingTrivia,
    pub body: Statements,
}
