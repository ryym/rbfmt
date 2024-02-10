use crate::fmt::{shape::Shape, LeadingTrivia, TrailingTrivia};

use super::{Else, Node, Statements};

#[derive(Debug)]
pub(crate) struct Case {
    pub predicate: Option<Box<Node>>,
    pub case_trailing: TrailingTrivia,
    pub first_branch_leading: LeadingTrivia,
    pub branches: Vec<CaseWhen>,
    pub otherwise: Option<Else>,
}

impl Case {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct CaseWhen {
    pub shape: Shape,
    pub conditions: Vec<Node>,
    pub conditions_shape: Shape,
    pub body: Statements,
}

impl CaseWhen {
    pub(crate) fn new(was_flat: bool) -> Self {
        let shape = if was_flat {
            Shape::inline(0)
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            conditions: vec![],
            conditions_shape: Shape::inline(0),
            body: Statements::new(),
        }
    }

    pub(crate) fn append_condition(&mut self, cond: Node) {
        self.shape.append(&cond.shape);
        self.conditions_shape.append(&cond.shape);
        self.conditions.push(cond);
    }

    pub(crate) fn set_body(&mut self, body: Statements) {
        self.shape.append(&body.shape);
        self.body = body;
    }
}
