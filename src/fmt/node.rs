mod array;
mod assign;
mod assoc;
mod atom;
mod begin;
mod block;
mod call_like;
mod case;
mod class_like;
mod constant_path;
mod def;
mod dyn_string_like;
mod fors;
mod hash;
mod heredoc;
mod ifs;
mod infix_chain;
mod lambda;
mod method_chain;
mod multi_assign_target;
mod parens;
mod postmodifier;
mod prefix;
mod range_like;
mod singleton_class;
mod statements;
mod string_like;
mod ternary;
mod virtual_end;
mod whiles;

pub(crate) use self::{
    array::*, assign::*, assoc::*, atom::*, begin::*, block::*, call_like::*, case::*,
    class_like::*, constant_path::*, def::*, dyn_string_like::*, fors::*, hash::*, heredoc::*,
    ifs::*, infix_chain::*, lambda::*, method_chain::*, multi_assign_target::*, parens::*,
    postmodifier::*, prefix::*, range_like::*, singleton_class::*, statements::*, string_like::*,
    ternary::*, virtual_end::*, whiles::*,
};

use super::{
    shape::{ArgumentStyle, Shape},
    LeadingTrivia, TrailingTrivia,
};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Node {
    pub leading_trivia: LeadingTrivia,
    pub trailing_trivia: TrailingTrivia,
    pub kind: Kind,
    pub shape: Shape,
}

impl Node {
    pub(crate) fn new(
        leading_trivia: LeadingTrivia,
        kind: Kind,
        trailing_trivia: TrailingTrivia,
    ) -> Self {
        let shape = leading_trivia
            .shape()
            .add(&kind.shape())
            .add(trailing_trivia.shape());
        Self {
            leading_trivia,
            trailing_trivia,
            kind,
            shape,
        }
    }

    pub(crate) fn without_trivia(kind: Kind) -> Self {
        Self::new(LeadingTrivia::new(), kind, TrailingTrivia::none())
    }

    pub(crate) fn is_diagonal(&self) -> bool {
        if self.leading_trivia.shape().is_empty() {
            self.kind.is_diagonal()
        } else {
            false
        }
    }

    pub(crate) fn argument_style(&self) -> ArgumentStyle {
        if self.leading_trivia.is_empty() {
            self.kind.argument_style()
        } else {
            ArgumentStyle::Vertical
        }
    }
}

#[derive(Debug)]
pub(crate) enum Kind {
    Atom(Atom),
    StringLike(StringLike),
    DynStringLike(DynStringLike),
    HeredocOpening(HeredocOpening),
    ConstantPath(ConstantPath),
    Statements(Statements),
    Parens(Parens),
    If(If),
    Ternary(Ternary),
    Case(Case),
    While(While),
    For(For),
    Postmodifier(Postmodifier),
    MethodChain(MethodChain),
    InfixChain(InfixChain),
    Lambda(Lambda),
    CallLike(CallLike),
    Assign(Assign),
    MultiAssignTarget(MultiAssignTarget),
    Prefix(Prefix),
    Array(Array),
    Hash(Hash),
    Assoc(Assoc),
    Begin(Begin),
    Def(Def),
    ClassLike(ClassLike),
    SingletonClass(SingletonClass),
    RangeLike(RangeLike),
    PrePostExec(PrePostExec),
    Alias(Alias),
}

impl Kind {
    pub(crate) fn shape(&self) -> Shape {
        match self {
            Self::Atom(atom) => Shape::inline(atom.0.len()),
            Self::StringLike(s) => s.shape,
            Self::DynStringLike(s) => s.shape,
            Self::HeredocOpening(opening) => *opening.shape(),
            Self::ConstantPath(c) => c.shape,
            Self::Statements(statements) => statements.shape,
            Self::Parens(parens) => parens.shape,
            Self::If(_) => If::shape(),
            Self::Ternary(ternary) => ternary.shape,
            Self::Case(_) => Case::shape(),
            Self::While(_) => While::shape(),
            Self::For(_) => For::shape(),
            Self::Postmodifier(pmod) => pmod.shape,
            Self::MethodChain(chain) => chain.shape,
            Self::InfixChain(chain) => chain.shape,
            Self::Lambda(lambda) => lambda.shape,
            Self::CallLike(call) => call.shape,
            Self::Assign(assign) => assign.shape,
            Self::MultiAssignTarget(multi) => multi.shape,
            Self::Prefix(prefix) => prefix.shape,
            Self::Array(array) => array.shape,
            Self::Hash(hash) => hash.shape,
            Self::Assoc(assoc) => assoc.shape,
            Self::Begin(_) => Begin::shape(),
            Self::Def(def) => def.shape,
            Self::ClassLike(_) => ClassLike::shape(),
            Self::SingletonClass(_) => SingletonClass::shape(),
            Self::RangeLike(range) => range.shape,
            Self::PrePostExec(exec) => exec.shape,
            Self::Alias(alias) => alias.shape,
        }
    }

    pub(crate) fn is_diagonal(&self) -> bool {
        match self {
            Self::Statements(statements) => {
                if statements.nodes.len() > 1 {
                    return false;
                }
                match statements.nodes.get(0) {
                    Some(node) => node.is_diagonal(),
                    None => statements.virtual_end.is_none(),
                }
            }
            Self::Atom(_) => true,
            Self::StringLike(_) => true,
            Self::DynStringLike(_) => true,
            Self::HeredocOpening(_) => false,
            Self::ConstantPath(_) => false,
            Self::Parens(_) => true,
            Self::If(_) => false,
            Self::Ternary(_) => true,
            Self::Case(_) => false,
            Self::While(_) => false,
            Self::For(_) => false,
            Self::Postmodifier(_) => true,
            Self::MethodChain(_) => true,
            Self::InfixChain(_) => true,
            Self::Lambda(_) => true,
            Self::CallLike(_) => true,
            Self::Assign(_) => true,
            Self::MultiAssignTarget(_) => true,
            Self::Prefix(_) => true,
            Self::Array(_) => true,
            Self::Hash(_) => true,
            Self::Assoc(_) => true,
            Self::Begin(_) => false,
            Self::Def(_) => false,
            Self::ClassLike(_) => false,
            Self::SingletonClass(_) => false,
            Self::RangeLike(_) => true,
            Self::PrePostExec(_) => true,
            Self::Alias(_) => true,
        }
    }

    pub(crate) fn argument_style(&self) -> ArgumentStyle {
        match self {
            Self::Atom(atom) => ArgumentStyle::Horizontal {
                min_first_line_len: atom.0.len(),
            },
            Self::StringLike(str) => str.shape.argument_style(),
            Self::HeredocOpening(opening) => opening.shape.argument_style(),
            Self::Parens(_) => ArgumentStyle::Horizontal {
                min_first_line_len: "(".len(),
            },
            Self::Lambda(lambda) => {
                let min_len = if lambda.parameters.is_some() {
                    "->(".len()
                } else {
                    "-> {".len()
                };
                ArgumentStyle::Horizontal {
                    min_first_line_len: min_len,
                }
            }
            Self::Prefix(prefix) => {
                let expr_style = prefix.expression.as_ref().map_or(
                    ArgumentStyle::Horizontal {
                        min_first_line_len: 0,
                    },
                    |e| e.argument_style(),
                );
                ArgumentStyle::Horizontal {
                    min_first_line_len: prefix.operator.len(),
                }
                .add(expr_style)
            }
            Self::Array(array) => match &array.opening {
                Some(opening) => ArgumentStyle::Horizontal {
                    min_first_line_len: opening.len(),
                },
                None => array
                    .elements
                    .first()
                    .expect("non-brackets array must have elements")
                    .argument_style(),
            },
            Self::Hash(hash) => ArgumentStyle::Horizontal {
                min_first_line_len: hash.opening.len(),
            },
            Self::Assoc(assoc) => match assoc.value.argument_style() {
                ArgumentStyle::Vertical => ArgumentStyle::Vertical,
                ArgumentStyle::Horizontal {
                    min_first_line_len: value_len,
                } => assoc.key.argument_style().add(ArgumentStyle::Horizontal {
                    min_first_line_len: ": ".len() + value_len,
                }),
            },
            _ => ArgumentStyle::Vertical,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Arguments {
    pub opening: Option<String>,
    pub closing: Option<String>,
    pub shape: Shape,
    pub nodes: Vec<Node>,
    pub last_comma_allowed: bool,
    pub virtual_end: Option<VirtualEnd>,
}

impl Arguments {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |o| o.len());
        let closing_len = closing.as_ref().map_or(0, |o| o.len());
        Self {
            opening,
            closing,
            shape: Shape::inline(opening_len + closing_len),
            nodes: vec![],
            last_comma_allowed: true,
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        self.shape.insert(&node.shape);
        if !self.nodes.is_empty() {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.insert(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.virtual_end.is_none()
    }
}

#[derive(Debug)]
pub(crate) struct PrePostExec {
    pub shape: Shape,
    pub keyword: String,
    pub statements: Statements,
}

impl PrePostExec {
    pub(crate) fn new(keyword: String, statements: Statements, was_flat: bool) -> Self {
        let shape = if was_flat {
            Shape::inline(keyword.len()).add(&statements.shape())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            keyword,
            statements,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Alias {
    pub shape: Shape,
    pub new_name: Box<Node>,
    pub old_name: Box<Node>,
}

impl Alias {
    pub(crate) fn new(new_name: Node, old_name: Node) -> Self {
        let shape = new_name.shape.add(&old_name.shape);
        Self {
            shape,
            new_name: Box::new(new_name),
            old_name: Box::new(old_name),
        }
    }
}

pub(crate) type HeredocMap = HashMap<Pos, Heredoc>;
