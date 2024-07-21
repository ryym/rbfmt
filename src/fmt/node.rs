mod alias;
mod alt_pattern_chain;
mod arguments;
mod array;
mod array_pattern;
mod assign;
mod assoc;
mod atom;
mod begin;
mod block;
mod call_like;
mod case;
mod case_match;
mod class_like;
mod constant_path;
mod def;
mod dyn_string_like;
mod fors;
mod hash;
mod hash_pattern;
mod heredoc;
mod ifs;
mod infix_chain;
mod lambda;
mod match_assign;
mod method_chain;
mod multi_assign_target;
mod parens;
mod postmodifier;
mod pre_post_exec;
mod prefix;
mod range_like;
mod singleton_class;
mod statements;
mod string_like;
mod ternary;
mod virtual_end;
mod whiles;

use std::mem;

pub(crate) use self::{
    alias::*, alt_pattern_chain::*, arguments::*, array::*, array_pattern::*, assign::*, assoc::*,
    atom::*, begin::*, block::*, call_like::*, case::*, case_match::*, class_like::*,
    constant_path::*, def::*, dyn_string_like::*, fors::*, hash::*, hash_pattern::*, heredoc::*,
    ifs::*, infix_chain::*, lambda::*, match_assign::*, method_chain::*, multi_assign_target::*,
    parens::*, postmodifier::*, pre_post_exec::*, prefix::*, range_like::*, singleton_class::*,
    statements::*, string_like::*, ternary::*, virtual_end::*, whiles::*,
};

use super::{
    output::{FormatContext, Output},
    shape::{ConcatStyle, Shape},
    LeadingTrivia, TrailingTrivia,
};

#[derive(Debug)]
pub(crate) struct Node {
    pub leading_trivia: LeadingTrivia,
    pub trailing_trivia: TrailingTrivia,
    pub kind: Kind,
    pub shape: Shape,
}

impl Node {
    pub(crate) fn new(kind: Kind) -> Self {
        Self::with_leading_trivia(LeadingTrivia::new(), kind)
    }

    pub(crate) fn with_leading_trivia(leading_trivia: LeadingTrivia, kind: Kind) -> Self {
        let shape = leading_trivia.shape().add(&kind.shape());
        Self {
            leading_trivia,
            trailing_trivia: TrailingTrivia::none(),
            kind,
            shape,
        }
    }

    pub(crate) fn prepend_leading_trivia(&mut self, leading_trivia: LeadingTrivia) {
        self.shape.insert(leading_trivia.shape());
        if self.leading_trivia.is_empty() {
            self.leading_trivia = leading_trivia;
        } else {
            let current = mem::replace(&mut self.leading_trivia, leading_trivia);
            self.leading_trivia.merge(current);
        }
    }

    pub(crate) fn set_trailing_trivia(&mut self, trailing_trivia: TrailingTrivia) {
        self.shape.append(trailing_trivia.shape());
        self.trailing_trivia = trailing_trivia;
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.kind.format(o, ctx);
    }

    pub(crate) fn can_continue_line(&self) -> bool {
        self.leading_trivia.shape().is_empty() && self.kind.can_continue_line()
    }

    pub(crate) fn concat_style(&self) -> ConcatStyle {
        if self.leading_trivia.is_empty() {
            self.kind.concat_style()
        } else {
            ConcatStyle::Vertical
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
    CaseMatch(CaseMatch),
    MatchAssign(MatchAssign),
    ArrayPattern(ArrayPattern),
    HashPattern(HashPattern),
    AltPatternChain(AltPatternChain),
    PrePostExec(PrePostExec),
    Alias(Alias),
}

impl Kind {
    pub(super) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        match self {
            Kind::Atom(atom) => atom.format(o),
            Kind::StringLike(str) => str.format(o),
            Kind::DynStringLike(dstr) => dstr.format(o, ctx),
            Kind::HeredocOpening(opening) => opening.format(o),
            Kind::ConstantPath(const_path) => const_path.format(o, ctx),
            Kind::Statements(statements) => statements.format(o, ctx, false),
            Kind::Parens(parens) => parens.format(o, ctx),
            Kind::If(ifexpr) => ifexpr.format(o, ctx),
            Kind::Ternary(ternary) => ternary.format(o, ctx),
            Kind::Case(case) => case.format(o, ctx),
            Kind::While(whle) => whle.format(o, ctx),
            Kind::For(expr) => expr.format(o, ctx),
            Kind::Postmodifier(modifier) => modifier.format(o, ctx),
            Kind::MethodChain(chain) => chain.format(o, ctx),
            Kind::Lambda(lambda) => lambda.format(o, ctx),
            Kind::CallLike(call) => call.format(o, ctx),
            Kind::InfixChain(chain) => chain.format(o, ctx),
            Kind::Assign(assign) => assign.format(o, ctx),
            Kind::MultiAssignTarget(multi) => multi.format(o, ctx),
            Kind::Prefix(prefix) => prefix.format(o, ctx),
            Kind::Array(array) => array.format(o, ctx),
            Kind::Hash(hash) => hash.format(o, ctx),
            Kind::Assoc(assoc) => assoc.format(o, ctx),
            Kind::Begin(begin) => begin.format(o, ctx),
            Kind::Def(def) => def.format(o, ctx),
            Kind::ClassLike(class) => class.format(o, ctx),
            Kind::SingletonClass(class) => class.format(o, ctx),
            Kind::RangeLike(range) => range.format(o, ctx),
            Kind::CaseMatch(case) => case.format(o, ctx),
            Kind::MatchAssign(assign) => assign.format(o, ctx),
            Kind::ArrayPattern(array) => array.format(o, ctx),
            Kind::HashPattern(hash) => hash.format(o, ctx),
            Kind::AltPatternChain(chain) => chain.format(o, ctx),
            Kind::PrePostExec(exec) => exec.format(o, ctx),
            Kind::Alias(alias) => alias.format(o, ctx),
        }
    }

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
            Self::CaseMatch(_) => CaseMatch::shape(),
            Self::MatchAssign(assign) => *assign.shape(),
            Self::ArrayPattern(array) => array.shape(),
            Self::HashPattern(hash) => hash.shape(),
            Self::AltPatternChain(chain) => chain.shape(),
            Self::PrePostExec(exec) => exec.shape,
            Self::Alias(alias) => alias.shape,
        }
    }

    pub(crate) fn can_continue_line(&self) -> bool {
        match self {
            Self::Statements(statements) => {
                if statements.nodes.len() > 1 {
                    return false;
                }
                match statements.nodes.get(0) {
                    Some(node) => node.can_continue_line(),
                    None => statements.virtual_end.is_none(),
                }
            }
            _ => true,
        }
    }

    pub(crate) fn concat_style(&self) -> ConcatStyle {
        match self {
            Self::Atom(atom) => ConcatStyle::Horizontal {
                min_first_line_len: atom.0.len(),
            },
            Self::StringLike(str) => str.shape.concat_style(),
            Self::HeredocOpening(opening) => opening.shape.concat_style(),
            Self::Parens(_) => ConcatStyle::Horizontal {
                min_first_line_len: "(".len(),
            },
            Self::Lambda(lambda) => {
                let min_len = if lambda.parameters.is_some() {
                    "->(".len()
                } else {
                    "-> {".len()
                };
                ConcatStyle::Horizontal {
                    min_first_line_len: min_len,
                }
            }
            Self::Prefix(prefix) => {
                let expr_style = prefix.expression.as_ref().map_or(
                    ConcatStyle::Horizontal {
                        min_first_line_len: 0,
                    },
                    |e| e.concat_style(),
                );
                ConcatStyle::Horizontal {
                    min_first_line_len: prefix.operator.len(),
                }
                .add(expr_style)
            }
            Self::Array(array) => match &array.opening {
                Some(opening) => ConcatStyle::Horizontal {
                    min_first_line_len: opening.len(),
                },
                None => array
                    .elements
                    .first()
                    .expect("non-brackets array must have elements")
                    .concat_style(),
            },
            Self::Hash(hash) => ConcatStyle::Horizontal {
                min_first_line_len: hash.opening.len(),
            },
            Self::Assoc(assoc) => match assoc.value.concat_style() {
                ConcatStyle::Vertical => ConcatStyle::Vertical,
                ConcatStyle::Horizontal {
                    min_first_line_len: value_len,
                } => assoc.key.concat_style().add(ConcatStyle::Horizontal {
                    min_first_line_len: ": ".len() + value_len,
                }),
            },
            _ => ConcatStyle::Vertical,
        }
    }
}
