mod atom;
mod dyn_string_like;
mod string_like;

pub(crate) use self::{atom::Atom, dyn_string_like::*, string_like::*};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos(pub usize);

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
pub(crate) struct Heredoc {
    pub id: String,
    pub indent_mode: HeredocIndentMode,
    pub parts: Vec<HeredocPart>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HeredocIndentMode {
    None,
    EndIndented,
    AllIndented,
}

impl HeredocIndentMode {
    pub(crate) fn parse_mode_and_id(opening: &[u8]) -> (Self, &[u8]) {
        let (indent_mode, id_start) = match opening[2] {
            b'~' => (Self::AllIndented, 3),
            b'-' => (Self::EndIndented, 3),
            _ => (Self::None, 2),
        };
        (indent_mode, &opening[id_start..])
    }

    pub(crate) fn prefix_symbols(&self) -> &'static str {
        match self {
            Self::None => "<<",
            Self::EndIndented => "<<-",
            Self::AllIndented => "<<~",
        }
    }
}

#[derive(Debug)]
pub(crate) enum HeredocPart {
    Str(StringLike),
    Statements(EmbeddedStatements),
    Variable(EmbeddedVariable),
}

#[derive(Debug)]
pub(crate) struct ConstantPath {
    pub shape: Shape,
    pub root: Option<Box<Node>>,
    pub parts: Vec<(LeadingTrivia, String)>,
}

impl ConstantPath {
    pub(crate) fn new(root: Option<Node>) -> Self {
        let shape = root.as_ref().map_or(Shape::inline(0), |r| r.shape);
        Self {
            shape,
            root: root.map(Box::new),
            parts: vec![],
        }
    }

    pub(crate) fn append_part(&mut self, leading: LeadingTrivia, path: String) {
        self.shape.append(leading.shape());
        self.shape.append(&Shape::inline("::".len() + path.len()));
        self.parts.push((leading, path));
    }
}

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

#[derive(Debug)]
pub(crate) struct Statements {
    pub shape: Shape,
    pub nodes: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl Statements {
    pub(crate) fn new() -> Self {
        Self {
            shape: Shape::inline(0),
            nodes: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        if self.nodes.is_empty() && !matches!(node.kind, Kind::HeredocOpening(_)) {
            self.shape = node.shape;
        } else {
            self.shape = Shape::Multilines;
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn shape(&self) -> Shape {
        self.shape
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.virtual_end.is_none()
    }
}

#[derive(Debug)]
pub(crate) struct Parens {
    pub shape: Shape,
    pub body: Statements,
}

impl Parens {
    pub(crate) fn new(body: Statements) -> Self {
        let mut shape = Shape::inline("()".len());
        shape.insert(&body.shape);
        Self { shape, body }
    }
}

#[derive(Debug)]
pub(crate) struct HeredocOpening {
    pub pos: Pos,
    pub shape: Shape,
    pub id: String,
    pub indent_mode: HeredocIndentMode,
}

impl HeredocOpening {
    pub(crate) fn new(pos: Pos, id: String, indent_mode: HeredocIndentMode) -> Self {
        let shape = Shape::inline(id.len() + indent_mode.prefix_symbols().len());
        Self {
            pos,
            shape,
            id,
            indent_mode,
        }
    }

    pub(crate) fn shape(&self) -> &Shape {
        &self.shape
    }
}

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

#[derive(Debug)]
pub(crate) struct While {
    pub is_while: bool,
    pub content: Conditional,
}

impl While {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct For {
    pub index: Box<Node>,
    pub collection: Box<Node>,
    pub body: Statements,
}

impl For {
    pub(crate) fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct Postmodifier {
    pub shape: Shape,
    pub keyword: String,
    pub conditional: Conditional,
}

impl Postmodifier {
    pub(crate) fn new(keyword: String, conditional: Conditional) -> Self {
        let kwd_shape = Shape::inline(keyword.len() + 2); // keyword and spaces around it.
        let shape = conditional.shape.add(&kwd_shape);
        Self {
            shape,
            keyword,
            conditional,
        }
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
pub(crate) struct MessageCall {
    shape: Shape,
    leading_trivia: LeadingTrivia,
    operator: Option<String>,
    name: String,
    arguments: Option<Arguments>,
    block: Option<Block>,
}

impl MessageCall {
    pub(crate) fn new(
        leading_trivia: LeadingTrivia,
        operator: Option<String>,
        name: String,
        arguments: Option<Arguments>,
        block: Option<Block>,
    ) -> Self {
        let operator_len = operator.as_ref().map_or(0, |s| s.len());
        let msg_shape = Shape::inline(name.len() + operator_len);
        let mut shape = leading_trivia.shape().add(&msg_shape);
        if let Some(args) = &arguments {
            shape.append(&args.shape);
        }
        if let Some(block) = &block {
            shape.append(&block.shape);
        }
        Self {
            shape,
            leading_trivia,
            operator,
            name,
            arguments,
            block,
        }
    }
}

#[derive(Debug)]
pub(crate) struct IndexCall {
    pub shape: Shape,
    pub arguments: Arguments,
    pub block: Option<Block>,
}

impl IndexCall {
    pub(crate) fn new(arguments: Arguments, block: Option<Block>) -> Self {
        let mut shape = arguments.shape;
        if let Some(block) = &block {
            shape.append(&block.shape);
        }
        Self {
            shape,
            arguments,
            block,
        }
    }

    pub(crate) fn min_first_line_len(&self) -> usize {
        self.arguments.opening.as_ref().map_or(0, |op| op.len())
    }
}

#[derive(Debug)]
pub(crate) struct CallUnit {
    pub shape: Shape,
    pub leading_trivia: LeadingTrivia,
    pub trailing_trivia: TrailingTrivia,
    pub operator: Option<String>,
    pub name: String,
    pub arguments: Option<Arguments>,
    pub block: Option<Block>,
    pub index_calls: Vec<IndexCall>,
}

impl CallUnit {
    fn from_message(call: MessageCall) -> Self {
        Self {
            shape: call.shape,
            leading_trivia: call.leading_trivia,
            trailing_trivia: TrailingTrivia::none(),
            operator: call.operator,
            name: call.name,
            arguments: call.arguments,
            block: call.block,
            index_calls: vec![],
        }
    }

    fn append_index_call(&mut self, idx_call: IndexCall) {
        self.shape.append(&idx_call.shape);
        self.index_calls.push(idx_call);
    }

    pub(crate) fn min_first_line_len(&self) -> Option<usize> {
        if self.leading_trivia.is_empty() {
            let mut len = self.operator.as_ref().map_or(0, |op| op.len());
            len += self.name.len();
            if let Some(args) = &self.arguments {
                len += args.opening.as_ref().map_or(0, |op| op.len());
            } else if let Some(block) = &self.block {
                len += block.min_first_line_len();
            } else if let Some(index) = self.index_calls.first() {
                len += index.min_first_line_len();
            };
            Some(len)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct Block {
    pub shape: Shape,
    pub opening: String,
    pub closing: String,
    pub opening_trailing: TrailingTrivia,
    pub parameters: Option<BlockParameters>,
    pub body: BlockBody,
}

impl Block {
    pub(crate) fn new(was_flat: bool, opening: String, closing: String) -> Self {
        let shape = if was_flat {
            Shape::inline(opening.len() + closing.len())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            opening,
            closing,
            opening_trailing: TrailingTrivia::none(),
            parameters: None,
            body: BlockBody::new(Statements::new()),
        }
    }

    pub(crate) fn set_opening_trailing(&mut self, trailing: TrailingTrivia) {
        self.shape.insert(trailing.shape());
        self.opening_trailing = trailing;
    }

    pub(crate) fn set_parameters(&mut self, parameters: BlockParameters) {
        self.shape.insert(&Shape::inline(" ".len()));
        self.shape.insert(&parameters.shape);
        self.parameters = Some(parameters);
    }

    pub(crate) fn set_body(&mut self, body: BlockBody) {
        self.shape.insert(&Shape::inline("  ".len()));
        self.shape.insert(&body.shape);
        self.body = body;
    }

    pub(crate) fn is_empty(&self) -> bool {
        !matches!(self.shape, Shape::Multilines) && self.body.is_empty()
    }

    fn min_first_line_len(&self) -> usize {
        let params_opening_len = self.parameters.as_ref().map_or(0, |_| 2); // " |"
        self.opening.len() + params_opening_len
    }
}

#[derive(Debug)]
pub(crate) struct MethodChain {
    pub shape: Shape,
    pub head: MethodChainHead,
    pub calls: Vec<CallUnit>,
    pub calls_shape: Shape,
}

impl MethodChain {
    pub(crate) fn with_receiver(receiver: Node) -> Self {
        Self {
            shape: receiver.shape,
            head: MethodChainHead::Receiver(Receiver::new(receiver)),
            calls: vec![],
            calls_shape: Shape::inline(0),
        }
    }

    pub(crate) fn without_receiver(call: MessageCall) -> Self {
        Self {
            shape: call.shape,
            head: MethodChainHead::FirstCall(CallUnit::from_message(call)),
            calls: vec![],
            calls_shape: Shape::inline(0),
        }
    }

    pub(crate) fn append_message_call(
        &mut self,
        last_call_trailing: TrailingTrivia,
        msg_call: MessageCall,
    ) {
        self.shape.append(last_call_trailing.shape());
        self.shape.append(&msg_call.shape);
        self.calls_shape.append(last_call_trailing.shape());
        self.calls_shape.append(&msg_call.shape);

        if !last_call_trailing.is_none() {
            let last_call = self
                .calls
                .last_mut()
                .or(match &mut self.head {
                    MethodChainHead::FirstCall(call) => Some(call),
                    _ => None,
                })
                .expect("call must exist when last trailing exist");
            last_call.shape.append(last_call_trailing.shape());
            last_call.trailing_trivia = last_call_trailing;
        }

        let call = CallUnit::from_message(msg_call);
        self.calls.push(call);
    }

    pub(crate) fn append_index_call(&mut self, idx_call: IndexCall) {
        self.shape.append(&idx_call.shape);
        self.calls_shape.append(&idx_call.shape);

        if let Some(prev) = self.calls.last_mut() {
            prev.append_index_call(idx_call);
        } else {
            self.head.append_index_call(idx_call);
        }
    }
}

#[derive(Debug)]
pub(crate) enum MethodChainHead {
    Receiver(Receiver),
    FirstCall(CallUnit),
}

impl MethodChainHead {
    pub(crate) fn has_trailing_trivia(&self) -> bool {
        match self {
            Self::Receiver(receiver) => !receiver.node.trailing_trivia.is_none(),
            Self::FirstCall(call) => !call.trailing_trivia.is_none(),
        }
    }

    fn append_index_call(&mut self, idx_call: IndexCall) {
        match self {
            Self::Receiver(receiver) => receiver.append_index_call(idx_call),
            Self::FirstCall(call) => call.append_index_call(idx_call),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Receiver {
    pub shape: Shape,
    pub node: Box<Node>,
    pub index_calls: Vec<IndexCall>,
}

impl Receiver {
    fn new(node: Node) -> Self {
        Self {
            shape: node.shape,
            node: Box::new(node),
            index_calls: vec![],
        }
    }

    fn append_index_call(&mut self, idx_call: IndexCall) {
        self.shape.append(&idx_call.shape);
        self.index_calls.push(idx_call);
    }
}

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
}

#[derive(Debug)]
pub(crate) struct InfixRight {
    pub shape: Shape,
    pub operator: String,
    pub value: Box<Node>,
}

impl InfixRight {
    fn new(operator: String, value: Node) -> Self {
        let shape = Shape::inline(operator.len() + "  ".len()).add(&value.shape);
        Self {
            shape,
            operator,
            value: Box::new(value),
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

#[derive(Debug)]
pub(crate) struct Lambda {
    pub shape: Shape,
    pub parameters: Option<BlockParameters>,
    pub block: Block,
}

impl Lambda {
    pub(crate) fn new(params: Option<BlockParameters>, block: Block) -> Self {
        let mut shape = Shape::inline("->".len());
        if let Some(params) = &params {
            shape.append(&params.shape);
        }
        shape.append(&Shape::inline(1));
        shape.append(&block.shape);
        Self {
            shape,
            parameters: params,
            block,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CallLike {
    pub shape: Shape,
    pub name: String,
    pub arguments: Option<Arguments>,
}

impl CallLike {
    pub(crate) fn new(name: String) -> Self {
        Self {
            shape: Shape::inline(name.len()),
            name,
            arguments: None,
        }
    }

    pub(crate) fn set_arguments(&mut self, args: Arguments) {
        self.shape.append(&args.shape);
        self.arguments = Some(args);
    }
}

#[derive(Debug)]
pub(crate) struct Assign {
    pub shape: Shape,
    pub target: Box<Node>,
    pub operator: String,
    pub value: Box<Node>,
}

impl Assign {
    pub(crate) fn new(target: Node, operator: String, value: Node) -> Self {
        let shape = target
            .shape
            .add(&value.shape)
            .add(&Shape::inline(operator.len() + "  ".len()));
        Self {
            shape,
            target: Box::new(target),
            operator,
            value: Box::new(value),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MultiAssignTarget {
    pub shape: Shape,
    pub lparen: Option<String>,
    pub rparen: Option<String>,
    pub targets: Vec<Node>,
    pub with_implicit_rest: bool,
    pub virtual_end: Option<VirtualEnd>,
}

impl MultiAssignTarget {
    pub(crate) fn new(lparen: Option<String>, rparen: Option<String>) -> Self {
        let parens_len = match (&lparen, &rparen) {
            (Some(lp), Some(rp)) => lp.len() + rp.len(),
            _ => 0,
        };
        Self {
            shape: Shape::inline(parens_len),
            lparen,
            rparen,
            targets: vec![],
            with_implicit_rest: false,
            virtual_end: None,
        }
    }

    pub(crate) fn append_target(&mut self, target: Node) {
        self.shape.insert(&target.shape);
        self.targets.push(target);
    }

    pub(crate) fn set_implicit_rest(&mut self, yes: bool) {
        if yes {
            self.shape.insert(&Shape::inline(",".len()));
        }
        self.with_implicit_rest = yes;
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }
}

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

#[derive(Debug)]
pub(crate) struct Prefix {
    pub shape: Shape,
    pub operator: String,
    pub expression: Option<Box<Node>>,
}

impl Prefix {
    pub(crate) fn new(operator: String, expression: Option<Node>) -> Self {
        let mut shape = Shape::inline(operator.len());
        if let Some(expr) = &expression {
            shape.append(&expr.shape);
        }
        Self {
            shape,
            operator,
            expression: expression.map(Box::new),
        }
    }
}

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

#[derive(Debug)]
pub(crate) struct Assoc {
    pub shape: Shape,
    pub key: Box<Node>,
    pub value: Box<Node>,
    pub operator: Option<String>,
}

impl Assoc {
    pub(crate) fn new(key: Node, operator: Option<String>, value: Node) -> Self {
        let mut shape = key.shape.add(&value.shape);
        shape.append(&Shape::inline(1)); // space
        if let Some(op) = &operator {
            shape.append(&Shape::inline(op.len()));
            shape.append(&Shape::inline(1)); // space
        }
        Self {
            shape,
            key: Box::new(key),
            value: Box::new(value),
            operator,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Begin {
    pub keyword_trailing: TrailingTrivia,
    pub body: BlockBody,
}

impl Begin {
    fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct Def {
    pub shape: Shape,
    pub receiver: Option<Box<Node>>,
    pub name: String,
    pub parameters: Option<MethodParameters>,
    pub body: DefBody,
}

impl Def {
    pub(crate) fn new(receiver: Option<Node>, name: String) -> Self {
        let mut shape = Shape::inline("def ".len() + name.len());
        if let Some(receiver) = &receiver {
            shape.insert(&receiver.shape);
        }
        Self {
            shape,
            receiver: receiver.map(Box::new),
            name,
            parameters: None,
            body: DefBody::Block {
                head_trailing: TrailingTrivia::none(),
                body: BlockBody::new(Statements::new()),
            },
        }
    }

    pub(crate) fn set_parameters(&mut self, parameters: MethodParameters) {
        self.shape.append(&parameters.shape);
        self.parameters = Some(parameters);
    }

    pub(crate) fn set_body(&mut self, body: DefBody) {
        self.shape.append(&body.shape());
        self.body = body;
    }
}

#[derive(Debug)]
pub(crate) enum DefBody {
    Short {
        body: Box<Node>,
    },
    Block {
        head_trailing: TrailingTrivia,
        body: BlockBody,
    },
}

impl DefBody {
    pub(crate) fn shape(&self) -> Shape {
        match self {
            Self::Short { body } => body.shape,
            Self::Block { .. } => Shape::Multilines,
        }
    }
}

#[derive(Debug)]
pub(crate) struct BlockBody {
    pub shape: Shape,
    pub statements: Statements,
    pub rescues: Vec<Rescue>,
    pub rescue_else: Option<Else>,
    pub ensure: Option<Else>,
}

impl BlockBody {
    pub(crate) fn new(statements: Statements) -> Self {
        Self {
            shape: statements.shape(),
            statements,
            rescues: vec![],
            rescue_else: None,
            ensure: None,
        }
    }

    pub(crate) fn set_rescues(&mut self, rescues: Vec<Rescue>) {
        if !rescues.is_empty() {
            self.shape = Shape::Multilines;
        }
        self.rescues = rescues;
    }

    pub(crate) fn set_rescue_else(&mut self, rescue_else: Else) {
        self.shape = Shape::Multilines;
        self.rescue_else = Some(rescue_else);
    }

    pub(crate) fn set_ensure(&mut self, ensure: Else) {
        self.shape = Shape::Multilines;
        self.ensure = Some(ensure);
    }

    fn is_empty(&self) -> bool {
        self.statements.is_empty() && self.rescues.is_empty() && self.ensure.is_none()
    }
}

#[derive(Debug)]
pub(crate) struct Rescue {
    pub exceptions: Vec<Node>,
    pub exceptions_shape: Shape,
    pub reference: Option<Box<Node>>,
    pub head_trailing: TrailingTrivia,
    pub statements: Statements,
}

impl Rescue {
    pub(crate) fn new() -> Self {
        Self {
            exceptions: vec![],
            exceptions_shape: Shape::inline(0),
            reference: None,
            head_trailing: TrailingTrivia::none(),
            statements: Statements::new(),
        }
    }

    pub(crate) fn append_exception(&mut self, exception: Node) {
        self.exceptions_shape.append(&exception.shape);
        self.exceptions.push(exception);
    }

    pub(crate) fn set_reference(&mut self, reference: Node) {
        self.reference = Some(Box::new(reference))
    }

    pub(crate) fn set_head_trailing(&mut self, trailing: TrailingTrivia) {
        self.head_trailing = trailing;
    }

    pub(crate) fn set_statements(&mut self, statements: Statements) {
        self.statements = statements;
    }
}

#[derive(Debug)]
pub(crate) struct MethodParameters {
    pub shape: Shape,
    pub opening: Option<String>,
    pub closing: Option<String>,
    pub params: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl MethodParameters {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |o| o.len());
        let closing_len = closing.as_ref().map_or(0, |c| c.len());
        let shape = Shape::inline(opening_len + closing_len);
        Self {
            shape,
            opening,
            closing,
            params: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_param(&mut self, node: Node) {
        self.shape.insert(&node.shape);
        if !self.params.is_empty() {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.params.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }
}

#[derive(Debug)]
pub(crate) struct BlockParameters {
    pub shape: Shape,
    pub opening: String,
    pub closing: String,
    pub params: Vec<Node>,
    pub locals: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
    pub closing_trailing: TrailingTrivia,
}

impl BlockParameters {
    pub(crate) fn new(opening: String, closing: String) -> Self {
        let shape = Shape::inline(opening.len() + closing.len());
        Self {
            shape,
            opening,
            closing,
            params: vec![],
            locals: vec![],
            closing_trailing: TrailingTrivia::none(),
            virtual_end: None,
        }
    }

    pub(crate) fn append_param(&mut self, node: Node) {
        if !self.params.is_empty() {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.shape.insert(&node.shape);
        self.params.push(node);
    }

    pub(crate) fn append_local(&mut self, node: Node) {
        if self.locals.is_empty() {
            self.shape.insert(&Shape::inline("; ".len()));
        }
        self.shape.insert(&node.shape);
        self.locals.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn set_closing_trailing(&mut self, trailing: TrailingTrivia) {
        self.shape.append(trailing.shape());
        self.closing_trailing = trailing;
    }
}

#[derive(Debug)]
pub(crate) struct ClassLike {
    pub keyword: String,
    pub name: String,
    pub superclass: Option<Box<Node>>,
    pub head_trailing: TrailingTrivia,
    pub body: BlockBody,
}

impl ClassLike {
    fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct SingletonClass {
    pub expression: Box<Node>,
    pub body: BlockBody,
}

impl SingletonClass {
    fn shape() -> Shape {
        Shape::Multilines
    }
}

#[derive(Debug)]
pub(crate) struct RangeLike {
    pub shape: Shape,
    pub left: Option<Box<Node>>,
    pub operator: String,
    pub right: Option<Box<Node>>,
}

impl RangeLike {
    pub(crate) fn new(left: Option<Node>, operator: String, right: Option<Node>) -> Self {
        let mut shape = Shape::inline(0);
        if let Some(left) = &left {
            shape.append(&left.shape);
        }
        shape.append(&Shape::inline(operator.len()));
        if let Some(right) = &right {
            shape.append(&right.shape);
        }
        Self {
            shape,
            left: left.map(Box::new),
            operator,
            right: right.map(Box::new),
        }
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
