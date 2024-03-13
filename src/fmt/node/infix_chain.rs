use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct InfixChain {
    pub shape: Shape,
    pub left: Box<Node>,
    pub precedence: InfixPrecedence,
    pub rights_shape: Shape,
    pub rights: Vec<InfixRight>,
}

impl InfixChain {
    pub(crate) fn new(left: Node, precedence: InfixPrecedence) -> Self {
        Self {
            shape: left.shape,
            left: Box::new(left),
            precedence,
            rights_shape: Shape::inline(0),
            rights: vec![],
        }
    }

    pub(crate) fn precedence(&self) -> &InfixPrecedence {
        &self.precedence
    }

    pub(crate) fn append_right(&mut self, op: String, right: Node) {
        let right = InfixRight::new(op, right);
        self.shape.append(&right.shape);
        self.rights_shape.append(&right.shape);
        self.rights.push(right);
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.left.format(o, ctx);
        if self.rights_shape.fits_in_one_line(o.remaining_width) {
            for right in &self.rights {
                o.push(' ');
                o.push_str(&right.operator);
                o.push(' ');
                right.value.format(o, ctx);
            }
        } else {
            for right in &self.rights {
                o.push(' ');
                o.push_str(&right.operator);
                o.indent();
                o.break_line(ctx);
                right
                    .value
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::none());
                o.put_indent_if_needed();
                right.value.format(o, ctx);
                o.dedent();
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct InfixRight {
    pub shape: Shape,
    pub operator: String,
    pub value: Node,
}

impl InfixRight {
    fn new(operator: String, value: Node) -> Self {
        let shape = Shape::inline(operator.len() + "  ".len()).add(&value.shape);
        Self {
            shape,
            operator,
            value,
        }
    }
}

// https://ruby-doc.org/core-2.6.2/doc/syntax/precedence_rdoc.html#label-Precedence
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InfixPrecedence {
    WordAndOr, // and, or
    // Assign,    // =, etc
    Range,     // .., ...
    Or,        // ||
    And,       // &&
    Eq,        // <=>, ==, ===, !=, =~, !~
    Comp,      // >, >=, <, <=
    Union,     // |, ^
    Intersect, // &
    Shift,     // <<, >>
    Add,       // +, -
    Mult,      // *, /, %
    Power,     // **
}

impl InfixPrecedence {
    pub(crate) fn from_operator(op: &str) -> Self {
        match op {
            "**" => Self::Power,
            "*" | "/" | "%" => Self::Mult,
            "+" | "-" => Self::Add,
            "<<" | ">>" => Self::Shift,
            "&" => Self::Intersect,
            "|" | "^" => Self::Union,
            ">" | ">=" | "<" | "<=" => Self::Comp,
            "<=>" | "==" | "===" | "!=" | "=~" | "!~" => Self::Eq,
            "&&" => Self::And,
            "||" => Self::Or,
            ".." | "..." => Self::Range,
            // "=" => Self::Assign,
            "and" | "or" => Self::WordAndOr,
            _ => panic!("unexpected infix operator: {}", op),
        }
    }
}
