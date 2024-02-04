use std::{
    collections::{HashMap, VecDeque},
    mem,
};

pub(crate) fn format(node: Node, heredoc_map: HeredocMap) -> String {
    let config = FormatConfig {
        line_width: 100,
        indent_size: 2,
    };
    let ctx = FormatContext { heredoc_map };
    let mut formatter = Formatter {
        remaining_width: config.line_width,
        line_count: 0,
        config,
        buffer: String::new(),
        indent: 0,
        heredoc_queue: VecDeque::new(),
        drafts: vec![],
    };
    formatter.format(&node, &ctx);
    if !formatter.buffer.is_empty() {
        formatter.break_line(&ctx);
    }
    formatter.buffer
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos(pub usize);

#[derive(Debug, Clone, Copy)]
pub(crate) enum Shape {
    Inline { len: usize },
    LineEnd { len: usize },
    Multilines,
}

impl Shape {
    pub(crate) fn inline(len: usize) -> Self {
        Self::Inline { len }
    }

    pub(crate) fn is_inline(&self) -> bool {
        matches!(self, Self::Inline { .. })
    }

    pub(crate) fn fits_in_inline(&self, width: usize) -> bool {
        match self {
            Self::Inline { len } => *len <= width,
            _ => false,
        }
    }

    pub(crate) fn fits_in_one_line(&self, width: usize) -> bool {
        match self {
            Self::Inline { len } | Self::LineEnd { len } => *len <= width,
            Self::Multilines => false,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Self::Inline { len } | Self::LineEnd { len } => *len == 0,
            Self::Multilines => false,
        }
    }

    pub(crate) fn append(&mut self, other: &Self) {
        let shape = self.add(other);
        let _ = mem::replace(self, shape);
    }

    pub(crate) fn add(self, other: &Self) -> Self {
        match self {
            Self::Inline { len: len1 } => match other {
                Self::Inline { len: len2 } => Self::Inline { len: len1 + len2 },
                Self::LineEnd { len: len2 } => Self::LineEnd { len: len1 + len2 },
                Self::Multilines => Self::Multilines,
            },
            Self::LineEnd { .. } | Self::Multilines => Self::Multilines,
        }
    }

    pub(crate) fn insert(&mut self, other: &Self) {
        let shape = match self {
            Self::Inline { len: len1 } => match other {
                Self::Inline { len: len2 } => Self::Inline { len: *len1 + *len2 },
                Self::LineEnd { len: len2 } => {
                    if *len1 == 0 {
                        Self::LineEnd { len: *len2 }
                    } else {
                        Self::Multilines
                    }
                }
                Self::Multilines => Self::Multilines,
            },
            Self::LineEnd { .. } | Self::Multilines => Self::Multilines,
        };
        let _ = mem::replace(self, shape);
    }
}

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
            .shape
            .add(&kind.shape())
            .add(&trailing_trivia.shape);
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
        if !self.leading_trivia.shape.is_empty() {
            false
        } else {
            self.kind.is_diagonal()
        }
    }
}

#[derive(Debug)]
pub(crate) enum Kind {
    Atom(String),
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
            Self::Atom(s) => Shape::inline(s.len()),
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
}

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
        Self {
            shape: Shape::inline(len),
            opening,
            value,
            closing,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DynStringLike {
    pub shape: Shape,
    pub opening: Option<String>,
    pub parts: Vec<DynStrPart>,
    pub closing: Option<String>,
}

impl DynStringLike {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |s| s.len());
        let closing_len = closing.as_ref().map_or(0, |s| s.len());
        Self {
            shape: Shape::inline(opening_len + closing_len),
            opening,
            parts: vec![],
            closing,
        }
    }

    pub(crate) fn append_part(&mut self, part: DynStrPart) {
        self.shape.insert(part.shape());
        self.parts.push(part);
    }
}

#[derive(Debug)]
pub(crate) enum DynStrPart {
    Str(StringLike),
    DynStr(DynStringLike),
    Statements(EmbeddedStatements),
    Variable(EmbeddedVariable),
}

impl DynStrPart {
    pub(crate) fn shape(&self) -> &Shape {
        match self {
            Self::Str(s) => &s.shape,
            Self::DynStr(s) => &s.shape,
            Self::Statements(e) => &e.shape,
            Self::Variable(s) => &s.shape,
        }
    }
}

#[derive(Debug)]
pub(crate) struct EmbeddedStatements {
    pub shape: Shape,
    pub opening: String,
    pub statements: Statements,
    pub closing: String,
}

impl EmbeddedStatements {
    pub(crate) fn new(opening: String, statements: Statements, closing: String) -> Self {
        let shape = Shape::inline(opening.len() + closing.len()).add(&statements.shape);
        Self {
            shape,
            opening,
            statements,
            closing,
        }
    }
}

#[derive(Debug)]
pub(crate) struct EmbeddedVariable {
    shape: Shape,
    operator: String,
    variable: String,
}

impl EmbeddedVariable {
    pub(crate) fn new(operator: String, variable: String) -> Self {
        let shape = Shape::inline(operator.len() + variable.len());
        Self {
            shape,
            operator,
            variable,
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

    fn prefix_symbols(&self) -> &'static str {
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
    shape: Shape,
    root: Option<Box<Node>>,
    parts: Vec<(LeadingTrivia, String)>,
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
        self.shape.append(&leading.shape);
        self.shape.append(&Shape::inline("::".len() + path.len()));
        self.parts.push((leading, path));
    }
}

#[derive(Debug)]
pub(crate) struct VirtualEnd {
    shape: Shape,
    leading_trivia: LeadingTrivia,
}

impl VirtualEnd {
    pub(crate) fn new(leading_trivia: LeadingTrivia) -> Self {
        Self {
            shape: leading_trivia.shape,
            leading_trivia,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Statements {
    shape: Shape,
    nodes: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
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
}

#[derive(Debug)]
pub(crate) struct Parens {
    shape: Shape,
    body: Statements,
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
    pos: Pos,
    shape: Shape,
    id: String,
    indent_mode: HeredocIndentMode,
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
    shape: Shape,
    predicate: Box<Node>,
    predicate_trailing: TrailingTrivia,
    then: Box<Node>,
    otherwise: Box<Node>,
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
            .add(&predicate_trailing.shape)
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
    shape: Shape,
    conditions: Vec<Node>,
    conditions_shape: Shape,
    body: Statements,
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
    shape: Shape,
    predicate: Box<Node>,
    body: Statements,
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
    opening: Option<String>,
    closing: Option<String>,
    shape: Shape,
    nodes: Vec<Node>,
    pub last_comma_allowed: bool,
    virtual_end: Option<VirtualEnd>,
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
        let mut shape = leading_trivia.shape.add(&msg_shape);
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
    shape: Shape,
    arguments: Arguments,
    block: Option<Block>,
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
struct CallUnit {
    shape: Shape,
    leading_trivia: LeadingTrivia,
    trailing_trivia: TrailingTrivia,
    operator: Option<String>,
    name: String,
    arguments: Option<Arguments>,
    block: Option<Block>,
    index_calls: Vec<IndexCall>,
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

    fn min_first_line_len(&self) -> Option<usize> {
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
    shape: Shape,
    opening: String,
    closing: String,
    opening_trailing: TrailingTrivia,
    parameters: Option<BlockParameters>,
    body: BlockBody,
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
        self.shape.insert(&trailing.shape);
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

    fn min_first_line_len(&self) -> usize {
        let params_opening_len = self.parameters.as_ref().map_or(0, |_| 2); // " |"
        self.opening.len() + params_opening_len
    }
}

#[derive(Debug)]
pub(crate) struct MethodChain {
    shape: Shape,
    head: MethodChainHead,
    calls: Vec<CallUnit>,
    calls_shape: Shape,
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
        self.shape.append(&last_call_trailing.shape);
        self.shape.append(&msg_call.shape);
        self.calls_shape.append(&last_call_trailing.shape);
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
            last_call.shape.append(&last_call_trailing.shape);
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
enum MethodChainHead {
    Receiver(Receiver),
    FirstCall(CallUnit),
}

impl MethodChainHead {
    fn has_trailing_trivia(&self) -> bool {
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
    shape: Shape,
    node: Box<Node>,
    index_calls: Vec<IndexCall>,
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
    shape: Shape,
    left: Box<Node>,
    precedence: InfixPrecedence,
    rights_shape: Shape,
    rights: Vec<InfixRight>,
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
struct InfixRight {
    shape: Shape,
    operator: String,
    value: Box<Node>,
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
    shape: Shape,
    parameters: Option<BlockParameters>,
    block: Block,
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
    shape: Shape,
    name: String,
    arguments: Option<Arguments>,
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
    shape: Shape,
    target: Box<Node>,
    operator: String,
    value: Box<Node>,
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
    shape: Shape,
    lparen: Option<String>,
    rparen: Option<String>,
    targets: Vec<Node>,
    with_implicit_rest: bool,
    virtual_end: Option<VirtualEnd>,
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
    shape: Shape,
    opening: Option<String>,
    closing: Option<String>,
    elements: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
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
    shape: Shape,
    operator: String,
    expression: Option<Box<Node>>,
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
    shape: Shape,
    opening: String,
    closing: String,
    elements: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
}

impl Hash {
    pub(crate) fn new(opening: String, closing: String) -> Self {
        let shape = Shape::inline(opening.len() + closing.len());
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
    shape: Shape,
    key: Box<Node>,
    value: Box<Node>,
    operator: Option<String>,
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
    shape: Shape,
    receiver: Option<Box<Node>>,
    name: String,
    parameters: Option<MethodParameters>,
    body: DefBody,
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
    shape: Shape,
    statements: Statements,
    rescues: Vec<Rescue>,
    rescue_else: Option<Else>,
    ensure: Option<Else>,
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
}

#[derive(Debug)]
pub(crate) struct Rescue {
    exceptions: Vec<Node>,
    exceptions_shape: Shape,
    reference: Option<Box<Node>>,
    head_trailing: TrailingTrivia,
    statements: Statements,
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
    shape: Shape,
    opening: Option<String>,
    closing: Option<String>,
    params: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
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
    shape: Shape,
    opening: String,
    closing: String,
    params: Vec<Node>,
    locals: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
    closing_trailing: TrailingTrivia,
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
        self.shape.append(&trailing.shape);
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
    shape: Shape,
    left: Option<Box<Node>>,
    operator: String,
    right: Option<Box<Node>>,
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
    shape: Shape,
    keyword: String,
    statements: Statements,
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
    shape: Shape,
    new_name: Box<Node>,
    old_name: Box<Node>,
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

#[derive(Debug)]
pub(crate) struct LeadingTrivia {
    lines: Vec<LineTrivia>,
    shape: Shape,
}

impl LeadingTrivia {
    pub(crate) fn new() -> Self {
        Self {
            lines: vec![],
            shape: Shape::inline(0),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub(crate) fn append_line(&mut self, trivia: LineTrivia) {
        if matches!(trivia, LineTrivia::Comment(_)) {
            self.shape = Shape::Multilines;
        }
        self.lines.push(trivia);
    }
}

#[derive(Debug)]
pub(crate) struct TrailingTrivia {
    comment: Option<Comment>,
    shape: Shape,
}

impl TrailingTrivia {
    pub(crate) fn new(comment: Option<Comment>) -> Self {
        let shape = if comment.is_some() {
            Shape::LineEnd {
                // Do not take into account the length of trailing comment.
                len: 0,
            }
        } else {
            Shape::inline(0)
        };
        Self { comment, shape }
    }

    pub(crate) fn none() -> Self {
        Self::new(None)
    }

    pub(crate) fn is_none(&self) -> bool {
        self.comment.is_none()
    }
}

#[derive(Debug)]
pub(crate) struct Comment {
    pub value: String,
}

#[derive(Debug)]
pub(crate) enum LineTrivia {
    EmptyLine,
    Comment(Comment),
}

#[derive(Debug)]
enum EmptyLineHandling {
    Trim { start: bool, end: bool },
    Skip,
}

pub(crate) type HeredocMap = HashMap<Pos, Heredoc>;

#[derive(Debug)]
struct FormatConfig {
    line_width: usize,
    indent_size: usize,
}

#[derive(Debug)]
struct FormatContext {
    heredoc_map: HeredocMap,
}

#[derive(Debug)]
struct FormatDraft {
    index: usize,
    snapshot: FormatStateSnapshot,
}

#[derive(Debug)]
struct FormatStateSnapshot {
    buffer_len: usize,
    remaining_width: usize,
    line_count: usize,
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
}

#[derive(Debug)]
enum DraftResult {
    Commit,
    Rollback,
}

#[derive(Debug)]
struct Formatter {
    config: FormatConfig,
    remaining_width: usize,
    line_count: usize,
    buffer: String,
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
    drafts: Vec<FormatDraft>,
}

impl Formatter {
    fn draft(&mut self, mut f: impl FnMut(&mut Self) -> DraftResult) -> DraftResult {
        let index = self.drafts.len();
        let draft = FormatDraft {
            index,
            snapshot: FormatStateSnapshot {
                buffer_len: self.buffer.len(),
                remaining_width: self.remaining_width,
                line_count: self.line_count,
                indent: self.indent,
                heredoc_queue: self.heredoc_queue.clone(),
            },
        };
        self.drafts.push(draft);
        let result = f(self);
        let draft = self.drafts.pop();
        match draft {
            Some(draft) if draft.index == index => match result {
                DraftResult::Commit => {}
                DraftResult::Rollback => {
                    self.buffer.truncate(draft.snapshot.buffer_len);
                    self.remaining_width = draft.snapshot.remaining_width;
                    self.line_count = draft.snapshot.line_count;
                    self.indent = draft.snapshot.indent;
                    self.heredoc_queue = draft.snapshot.heredoc_queue;
                }
            },
            _ => panic!("invalid draft state: {:?} finished in {}", draft, index),
        };
        result
    }

    fn format(&mut self, node: &Node, ctx: &FormatContext) {
        match &node.kind {
            Kind::Atom(value) => self.format_atom(value),
            Kind::StringLike(str) => self.format_string_like(str),
            Kind::DynStringLike(dstr) => self.format_dyn_string_like(dstr, ctx),
            Kind::HeredocOpening(opening) => self.format_heredoc_opening(opening),
            Kind::ConstantPath(const_path) => self.format_constant_path(const_path, ctx),
            Kind::Statements(statements) => self.format_statements(statements, ctx, false),
            Kind::Parens(parens) => self.format_parens(parens, ctx),
            Kind::If(ifexpr) => self.format_if(ifexpr, ctx),
            Kind::Ternary(ternary) => self.format_ternary(ternary, ctx),
            Kind::Case(case) => self.format_case(case, ctx),
            Kind::While(whle) => self.format_while(whle, ctx),
            Kind::For(expr) => self.format_for(expr, ctx),
            Kind::Postmodifier(modifier) => self.format_postmodifier(modifier, ctx),
            Kind::MethodChain(chain) => self.format_method_chain(chain, ctx),
            Kind::Lambda(lambda) => self.format_lambda(lambda, ctx),
            Kind::CallLike(call) => self.format_call_like(call, ctx),
            Kind::InfixChain(chain) => self.format_infix_chain(chain, ctx),
            Kind::Assign(assign) => self.format_assign(assign, ctx),
            Kind::MultiAssignTarget(multi) => self.format_multi_assign_target(multi, ctx),
            Kind::Prefix(prefix) => self.format_prefix(prefix, ctx),
            Kind::Array(array) => self.format_array(array, ctx),
            Kind::Hash(hash) => self.format_hash(hash, ctx),
            Kind::Assoc(assoc) => self.format_assoc(assoc, ctx),
            Kind::Begin(begin) => self.format_begin(begin, ctx),
            Kind::Def(def) => self.format_def(def, ctx),
            Kind::ClassLike(class) => self.format_class_like(class, ctx),
            Kind::SingletonClass(class) => self.format_singleton_class(class, ctx),
            Kind::RangeLike(range) => self.format_range_like(range, ctx),
            Kind::PrePostExec(exec) => self.format_pre_post_exec(exec, ctx),
            Kind::Alias(alias) => self.format_alias(alias, ctx),
        }
    }

    fn format_atom(&mut self, value: &str) {
        self.push_str(value);
    }

    fn format_string_like(&mut self, str: &StringLike) {
        // Ignore non-UTF8 source code for now.
        let value = String::from_utf8_lossy(&str.value);
        if let Some(opening) = &str.opening {
            self.push_str(opening);
        }
        self.push_str(&value);
        if let Some(closing) = &str.closing {
            self.push_str(closing);
        }
    }

    fn format_dyn_string_like(&mut self, dstr: &DynStringLike, ctx: &FormatContext) {
        if let Some(opening) = &dstr.opening {
            self.push_str(opening);
        }
        let mut divided = false;
        for part in &dstr.parts {
            if divided {
                self.push(' ');
            }
            match part {
                DynStrPart::Str(str) => {
                    divided = str.opening.is_some();
                    self.format_string_like(str);
                }
                DynStrPart::DynStr(dstr) => {
                    divided = true;
                    self.format_dyn_string_like(dstr, ctx);
                }
                DynStrPart::Statements(embedded) => {
                    self.format_embedded_statements(embedded, ctx);
                }
                DynStrPart::Variable(var) => {
                    self.format_embedded_variable(var);
                }
            }
        }
        if let Some(closing) = &dstr.closing {
            self.push_str(closing);
        }
    }

    fn format_embedded_statements(&mut self, embedded: &EmbeddedStatements, ctx: &FormatContext) {
        self.push_str(&embedded.opening);

        if embedded.shape.is_inline() {
            let remaining = self.remaining_width;
            self.remaining_width = usize::MAX;
            self.format_statements(&embedded.statements, ctx, false);
            self.remaining_width = remaining;
        } else {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&embedded.statements, ctx, true);
            self.break_line(ctx);
            self.dedent();
        }

        self.push_str(&embedded.closing);
    }

    fn format_embedded_variable(&mut self, var: &EmbeddedVariable) {
        self.push_str(&var.operator);
        self.format_atom(&var.variable);
    }

    fn format_heredoc_opening(&mut self, opening: &HeredocOpening) {
        self.push_str(opening.indent_mode.prefix_symbols());
        self.push_str(&opening.id);
        self.heredoc_queue.push_back(opening.pos);
    }

    fn format_constant_path(&mut self, const_path: &ConstantPath, ctx: &FormatContext) {
        if let Some(root) = &const_path.root {
            self.format(root, ctx);
        }
        self.push_str("::");
        let last_idx = const_path.parts.len() - 1;
        for (i, (leading, path)) in const_path.parts.iter().enumerate() {
            if leading.is_empty() {
                self.push_str(path);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    leading,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.push_str(path);
                self.dedent();
            }
            if i < last_idx {
                self.push_str("::");
            }
        }
    }

    fn format_statements(
        &mut self,
        statements: &Statements,
        ctx: &FormatContext,
        block_always: bool,
    ) {
        if statements.shape.is_inline() && !block_always {
            if let Some(node) = statements.nodes.get(0) {
                self.format(node, ctx);
            }
            return;
        }
        for (i, n) in statements.nodes.iter().enumerate() {
            if i > 0 {
                self.break_line(ctx);
            }
            self.write_leading_trivia(
                &n.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: i == 0,
                    end: false,
                },
            );
            self.format(n, ctx);
            self.write_trailing_comment(&n.trailing_trivia);
        }
        self.write_trivia_at_virtual_end(
            ctx,
            &statements.virtual_end,
            !statements.nodes.is_empty(),
            statements.nodes.is_empty(),
        );
    }

    fn format_parens(&mut self, parens: &Parens, ctx: &FormatContext) {
        if parens.body.shape().is_empty() {
            self.push_str("()");
        } else {
            self.push('(');
            if parens.body.shape.fits_in_inline(self.remaining_width) {
                self.format_statements(&parens.body, ctx, false);
            } else {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&parens.body, ctx, true);
                self.dedent();
                self.break_line(ctx);
            }
            self.push(')');
        }
    }

    fn write_trivia_at_virtual_end(
        &mut self,
        ctx: &FormatContext,
        end: &Option<VirtualEnd>,
        break_first: bool,
        trim_start: bool,
    ) {
        if let Some(end) = end {
            let mut trailing_empty_lines = 0;
            let leading_lines = &end.leading_trivia.lines;
            for trivia in leading_lines.iter().rev() {
                match trivia {
                    LineTrivia::EmptyLine => {
                        trailing_empty_lines += 1;
                    }
                    LineTrivia::Comment(_) => {
                        break;
                    }
                }
            }
            if trailing_empty_lines == leading_lines.len() {
                return;
            }

            if break_first {
                self.break_line(ctx);
            }
            let target_len = leading_lines.len() - trailing_empty_lines;
            let last_idx = target_len - 1;
            for (i, trivia) in leading_lines.iter().take(target_len).enumerate() {
                match trivia {
                    LineTrivia::EmptyLine => {
                        if !(trim_start && i == 0) || i == last_idx {
                            self.break_line(ctx);
                        }
                    }
                    LineTrivia::Comment(comment) => {
                        self.push_str(&comment.value);
                        if i < last_idx {
                            self.break_line(ctx);
                        }
                    }
                }
            }
        }
    }

    fn format_if(&mut self, expr: &If, ctx: &FormatContext) {
        if expr.is_if {
            self.push_str("if");
        } else {
            self.push_str("unless");
        }

        self.format_conditional(&expr.if_first, ctx);
        if !expr.if_first.body.shape.is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&expr.if_first.body, ctx, true);
            self.dedent();
        }

        for elsif in &expr.elsifs {
            self.break_line(ctx);
            self.push_str("elsif");
            self.format_conditional(elsif, ctx);
            if !elsif.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&elsif.body, ctx, true);
                self.dedent();
            }
        }

        if let Some(if_last) = &expr.if_last {
            self.break_line(ctx);
            self.push_str("else");
            self.write_trailing_comment(&if_last.keyword_trailing);
            if !if_last.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&if_last.body, ctx, true);
                self.dedent();
            }
        }

        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_case(&mut self, case: &Case, ctx: &FormatContext) {
        self.push_str("case");
        match &case.predicate {
            Some(pred) => {
                if pred.shape.fits_in_one_line(self.remaining_width) || pred.is_diagonal() {
                    self.push(' ');
                    self.format(pred, ctx);
                    self.write_trailing_comment(&pred.trailing_trivia);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &pred.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: true,
                            end: true,
                        },
                    );
                    self.format(pred, ctx);
                    self.write_trailing_comment(&pred.trailing_trivia);
                    self.dedent();
                }
            }
            None => {
                self.write_trailing_comment(&case.case_trailing);
            }
        }
        if case.first_branch_leading.is_empty() {
            self.break_line(ctx);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &case.first_branch_leading,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.dedent();
        }
        for (i, branch) in case.branches.iter().enumerate() {
            if i > 0 {
                self.break_line(ctx);
            }
            self.format_case_when(branch, ctx);
        }
        if let Some(otherwise) = &case.otherwise {
            self.break_line(ctx);
            self.push_str("else");
            self.write_trailing_comment(&otherwise.keyword_trailing);
            if !otherwise.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&otherwise.body, ctx, true);
                self.dedent();
            }
        }
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_case_when(&mut self, when: &CaseWhen, ctx: &FormatContext) {
        self.push_str("when");
        if when.shape.fits_in_one_line(self.remaining_width) {
            self.push(' ');
            for (i, cond) in when.conditions.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(cond, ctx);
                self.write_trailing_comment(&cond.trailing_trivia);
            }
            if !when.body.shape.is_empty() {
                self.push_str(" then ");
                self.format_statements(&when.body, ctx, false);
            }
        } else {
            if when.conditions_shape.fits_in_one_line(self.remaining_width) {
                for (i, cond) in when.conditions.iter().enumerate() {
                    if i == 0 {
                        self.push(' ');
                    } else {
                        self.push_str(", ");
                    }
                    self.format(cond, ctx);
                    self.write_trailing_comment(&cond.trailing_trivia);
                }
            } else {
                if when.conditions[0].is_diagonal() {
                    self.push(' ');
                    self.format(&when.conditions[0], ctx);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &when.conditions[0].leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: true,
                            end: true,
                        },
                    );
                    self.format(&when.conditions[0], ctx);
                    self.dedent();
                }
                if when.conditions.len() > 1 {
                    self.push(',');
                }
                self.write_trailing_comment(&when.conditions[0].trailing_trivia);
                if when.conditions.len() > 1 {
                    self.indent();
                    let last_idx = when.conditions.len() - 1;
                    for (i, cond) in when.conditions.iter().enumerate().skip(1) {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &cond.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: false,
                                end: false,
                            },
                        );
                        self.format(cond, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&cond.trailing_trivia);
                    }
                    self.dedent();
                }
            }
            if !when.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&when.body, ctx, true);
                self.dedent();
            }
        }
    }

    fn format_ternary(&mut self, tern: &Ternary, ctx: &FormatContext) {
        // Format `predicate`.
        self.format(&tern.predicate, ctx);
        self.push_str(" ?");

        // Format `then`.
        if tern.predicate_trailing.is_none()
            && tern.then.shape.fits_in_one_line(self.remaining_width)
        {
            self.push(' ');
            self.format(&tern.then, ctx);
            self.write_trailing_comment(&tern.then.trailing_trivia);
        } else {
            self.write_trailing_comment(&tern.predicate_trailing);
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &tern.then.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&tern.then, ctx);
            self.write_trailing_comment(&tern.then.trailing_trivia);
            self.dedent();
        }

        // Format `otherwise`.
        if tern.predicate_trailing.is_none()
            && tern.then.shape.is_inline()
            && tern.otherwise.shape.fits_in_one_line(self.remaining_width)
        {
            self.push_str(" : ");
            self.format(&tern.otherwise, ctx);
            self.write_trailing_comment(&tern.otherwise.trailing_trivia);
        } else {
            self.break_line(ctx);
            self.push(':');
            if tern.otherwise.shape.fits_in_one_line(self.remaining_width)
                || tern.otherwise.is_diagonal()
            {
                self.push(' ');
                self.format(&tern.otherwise, ctx);
                self.write_trailing_comment(&tern.otherwise.trailing_trivia);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &tern.otherwise.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.format(&tern.otherwise, ctx);
                self.write_trailing_comment(&tern.otherwise.trailing_trivia);
                self.dedent();
            }
        }
    }

    fn format_while(&mut self, expr: &While, ctx: &FormatContext) {
        if expr.is_while {
            self.push_str("while");
        } else {
            self.push_str("until");
        }
        self.format_conditional(&expr.content, ctx);
        if !expr.content.body.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&expr.content.body, ctx, true);
            self.dedent();
        }
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_for(&mut self, expr: &For, ctx: &FormatContext) {
        self.push_str("for");
        if expr.index.shape.fits_in_inline(self.remaining_width) || expr.index.is_diagonal() {
            self.push(' ');
            self.format(&expr.index, ctx);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &expr.index.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&expr.index, ctx);
            self.dedent();
        }
        self.push_str(" in");
        let collection = &expr.collection;
        if collection.shape.fits_in_inline(self.remaining_width) || collection.is_diagonal() {
            self.push(' ');
            self.format(collection, ctx);
            self.write_trailing_comment(&collection.trailing_trivia);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &collection.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(collection, ctx);
            self.write_trailing_comment(&collection.trailing_trivia);
            self.dedent();
        }
        if !expr.body.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&expr.body, ctx, true);
            self.dedent();
        }
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_postmodifier(&mut self, modifier: &Postmodifier, ctx: &FormatContext) {
        self.format_statements(&modifier.conditional.body, ctx, false);
        self.push(' ');
        self.push_str(&modifier.keyword);
        let cond = &modifier.conditional;
        if cond.predicate.is_diagonal() {
            self.push(' ');
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &cond.predicate.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
            self.dedent();
        }
    }

    fn format_conditional(&mut self, cond: &Conditional, ctx: &FormatContext) {
        if cond.predicate.is_diagonal() {
            self.push(' ');
            self.indent();
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
            self.dedent();
        } else {
            self.indent();
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &cond.predicate.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
            self.dedent();
            self.dedent();
        }
    }

    fn format_method_chain(&mut self, chain: &MethodChain, ctx: &FormatContext) {
        match &chain.head {
            MethodChainHead::Receiver(receiver) => {
                self.format(&receiver.node, ctx);
                self.write_trailing_comment(&receiver.node.trailing_trivia);
                for idx_call in &receiver.index_calls {
                    self.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        self.format_block(block, ctx);
                    }
                }
            }
            MethodChainHead::FirstCall(call) => {
                self.push_str(&call.name);
                if let Some(args) = &call.arguments {
                    self.format_arguments(args, ctx);
                }
                if let Some(block) = &call.block {
                    self.format_block(block, ctx);
                }
                for idx_call in &call.index_calls {
                    self.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        self.format_block(block, ctx);
                    }
                }
                self.write_trailing_comment(&call.trailing_trivia);
            }
        }

        // horizontal format
        //   foo.bar.baz
        //   foo.bar.baz do
        //     ...
        //   end
        // vertical format
        //   foo
        //     .bar
        //     .baz

        let can_be_horizontal = !chain.head.has_trailing_trivia()
            && chain
                .calls
                .iter()
                .all(|call| call.leading_trivia.is_empty() && call.trailing_trivia.is_none());
        let committed = if can_be_horizontal {
            let result = self.draft(|d| {
                let mut multilines_call_count = 0;
                for call in &chain.calls {
                    if let Some(min_first_line_len) = call.min_first_line_len() {
                        if min_first_line_len > d.remaining_width {
                            return DraftResult::Rollback;
                        }
                    }
                    let prev_line_count = d.line_count;
                    if let Some(call_op) = &call.operator {
                        d.push_str(call_op);
                    }
                    d.push_str(&call.name);
                    if let Some(args) = &call.arguments {
                        d.format_arguments(args, ctx);
                    }
                    if let Some(block) = &call.block {
                        d.format_block(block, ctx);
                    }
                    for idx_call in &call.index_calls {
                        d.format_arguments(&idx_call.arguments, ctx);
                        if let Some(block) = &idx_call.block {
                            d.format_block(block, ctx);
                        }
                    }
                    if prev_line_count < d.line_count {
                        multilines_call_count += 1;
                        if multilines_call_count > 1 {
                            return DraftResult::Rollback;
                        }
                    }
                }
                DraftResult::Commit
            });
            matches!(result, DraftResult::Commit)
        } else {
            false
        };

        if !committed {
            self.indent();
            for call in chain.calls.iter() {
                if let Some(call_op) = &call.operator {
                    self.break_line(ctx);
                    self.write_leading_trivia(&call.leading_trivia, ctx, EmptyLineHandling::Skip);
                    self.push_str(call_op);
                }
                self.push_str(&call.name);
                if let Some(args) = &call.arguments {
                    self.format_arguments(args, ctx);
                }
                if let Some(block) = &call.block {
                    self.format_block(block, ctx);
                }
                for idx_call in &call.index_calls {
                    self.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        self.format_block(block, ctx);
                    }
                }
                self.write_trailing_comment(&call.trailing_trivia);
            }
            self.dedent();
        }
    }

    fn format_call_like(&mut self, call: &CallLike, ctx: &FormatContext) {
        self.push_str(&call.name);
        if let Some(args) = &call.arguments {
            self.format_arguments(args, ctx);
        }
    }

    fn format_arguments(&mut self, args: &Arguments, ctx: &FormatContext) {
        if args.shape.fits_in_inline(self.remaining_width) {
            if let Some(opening) = &args.opening {
                self.push_str(opening);
            } else {
                self.push(' ');
            }
            for (i, arg) in args.nodes.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(arg, ctx);
            }
            if let Some(closing) = &args.closing {
                self.push_str(closing);
            }
        } else if let Some(opening) = &args.opening {
            self.push_str(opening);
            self.indent();
            if !args.nodes.is_empty() {
                let last_idx = args.nodes.len() - 1;
                for (i, arg) in args.nodes.iter().enumerate() {
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &arg.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    self.format(arg, ctx);
                    if i < last_idx || args.last_comma_allowed {
                        self.push(',');
                    }
                    self.write_trailing_comment(&arg.trailing_trivia);
                }
            }
            self.write_trivia_at_virtual_end(ctx, &args.virtual_end, true, args.nodes.is_empty());
            self.dedent();
            self.break_line(ctx);
            if let Some(closing) = &args.closing {
                self.push_str(closing);
            }
        } else if !args.nodes.is_empty() {
            self.push(' ');
            self.format(&args.nodes[0], ctx);
            if args.nodes.len() > 1 {
                self.push(',');
            }
            self.write_trailing_comment(&args.nodes[0].trailing_trivia);
            if args.nodes.len() > 1 {
                self.indent();
                let last_idx = args.nodes.len() - 1;
                for (i, arg) in args.nodes.iter().enumerate().skip(1) {
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &arg.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    self.format(arg, ctx);
                    if i < last_idx {
                        self.push(',');
                    }
                    self.write_trailing_comment(&arg.trailing_trivia);
                }
                self.dedent();
            }
        }
    }

    fn format_block(&mut self, block: &Block, ctx: &FormatContext) {
        if block.shape.fits_in_one_line(self.remaining_width) {
            self.push(' ');
            self.push_str(&block.opening);
            if let Some(params) = &block.parameters {
                self.push(' ');
                self.format_block_parameters(params, ctx);
            }
            if !block.body.shape.is_empty() {
                self.push(' ');
                self.format_block_body(&block.body, ctx, false);
                self.push(' ');
            }
            if &block.closing == "end" {
                self.push(' ');
            }
            self.push_str(&block.closing);
        } else {
            self.push(' ');
            self.push_str(&block.opening);
            self.write_trailing_comment(&block.opening_trailing);
            if let Some(params) = &block.parameters {
                if block.opening_trailing.is_none() {
                    self.push(' ');
                    self.format_block_parameters(params, ctx);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.format_block_parameters(params, ctx);
                    self.dedent();
                }
            }
            if !block.body.shape.is_empty() {
                self.format_block_body(&block.body, ctx, true);
            }
            self.break_line(ctx);
            self.push_str(&block.closing);
        }
    }

    fn format_lambda(&mut self, lambda: &Lambda, ctx: &FormatContext) {
        self.push_str("->");
        if let Some(params) = &lambda.parameters {
            self.format_block_parameters(params, ctx);
        }
        self.format_block(&lambda.block, ctx);
    }

    fn format_infix_chain(&mut self, chain: &InfixChain, ctx: &FormatContext) {
        self.format(&chain.left, ctx);
        if chain.rights_shape.fits_in_one_line(self.remaining_width) {
            for right in &chain.rights {
                self.push(' ');
                self.push_str(&right.operator);
                self.push(' ');
                self.format(&right.value, ctx);
            }
        } else {
            for right in &chain.rights {
                self.push(' ');
                self.push_str(&right.operator);
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &right.value.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: false,
                        end: false,
                    },
                );
                self.format(&right.value, ctx);
                self.dedent();
            }
        }
    }

    fn format_assign(&mut self, assign: &Assign, ctx: &FormatContext) {
        self.format(&assign.target, ctx);
        self.push(' ');
        self.push_str(&assign.operator);
        self.format_assign_right(&assign.value, ctx);
    }

    fn format_assign_right(&mut self, value: &Node, ctx: &FormatContext) {
        if value.shape.fits_in_one_line(self.remaining_width) || value.is_diagonal() {
            self.push(' ');
            self.format(value, ctx);
        } else {
            self.break_line(ctx);
            self.indent();
            self.write_leading_trivia(
                &value.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(value, ctx);
            self.dedent();
        }
    }

    fn format_multi_assign_target(&mut self, multi: &MultiAssignTarget, ctx: &FormatContext) {
        if multi.shape.fits_in_inline(self.remaining_width) {
            if let Some(lparen) = &multi.lparen {
                self.push_str(lparen);
            }
            for (i, target) in multi.targets.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(target, ctx);
            }
            if multi.with_implicit_rest {
                self.push(',');
            }
            if let Some(rparen) = &multi.rparen {
                self.push_str(rparen);
            }
        } else {
            self.push('(');
            self.indent();
            let last_idx = multi.targets.len() - 1;
            for (i, target) in multi.targets.iter().enumerate() {
                self.break_line(ctx);
                self.write_leading_trivia(
                    &target.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.format(target, ctx);
                if i < last_idx || multi.with_implicit_rest {
                    self.push(',');
                }
                self.write_trailing_comment(&target.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(ctx, &multi.virtual_end, true, false);
            self.dedent();
            self.break_line(ctx);
            self.push(')');
        }
    }

    fn format_prefix(&mut self, prefix: &Prefix, ctx: &FormatContext) {
        self.push_str(&prefix.operator);
        if let Some(expr) = &prefix.expression {
            self.format(expr, ctx);
        }
    }

    fn format_array(&mut self, array: &Array, ctx: &FormatContext) {
        if array.shape.fits_in_one_line(self.remaining_width) {
            if let Some(opening) = &array.opening {
                self.push_str(opening);
            }
            for (i, n) in array.elements.iter().enumerate() {
                if i > 0 {
                    self.push_str(array.separator());
                    self.push(' ');
                }
                self.format(n, ctx);
            }
            if let Some(closing) = &array.closing {
                self.push_str(closing);
            }
        } else {
            self.push_str(array.opening.as_deref().unwrap_or("["));
            self.indent();
            for (i, element) in array.elements.iter().enumerate() {
                self.break_line(ctx);
                self.write_leading_trivia(
                    &element.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.format(element, ctx);
                self.push_str(array.separator());
                self.write_trailing_comment(&element.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(
                ctx,
                &array.virtual_end,
                true,
                array.elements.is_empty(),
            );
            self.dedent();
            self.break_line(ctx);
            self.push_str(array.closing.as_deref().unwrap_or("]"));
        }
    }

    fn format_hash(&mut self, hash: &Hash, ctx: &FormatContext) {
        if hash.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&hash.opening);
            if !hash.elements.is_empty() {
                self.push(' ');
                for (i, n) in hash.elements.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
                self.push(' ');
            }
            self.push_str(&hash.closing);
        } else {
            self.push_str(&hash.opening);
            self.indent();
            for (i, element) in hash.elements.iter().enumerate() {
                self.break_line(ctx);
                self.write_leading_trivia(
                    &element.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.format(element, ctx);
                self.push(',');
                self.write_trailing_comment(&element.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(
                ctx,
                &hash.virtual_end,
                true,
                hash.elements.is_empty(),
            );
            self.dedent();
            self.break_line(ctx);
            self.push_str(&hash.closing);
        }
    }

    fn format_assoc(&mut self, assoc: &Assoc, ctx: &FormatContext) {
        self.format(&assoc.key, ctx);
        if assoc.value.shape.fits_in_inline(self.remaining_width) || assoc.value.is_diagonal() {
            if let Some(op) = &assoc.operator {
                self.push(' ');
                self.push_str(op);
            }
            if !assoc.value.shape.is_empty() {
                self.push(' ');
                self.format(&assoc.value, ctx);
            }
        } else {
            if let Some(op) = &assoc.operator {
                self.push(' ');
                self.push_str(op);
            }
            self.break_line(ctx);
            self.indent();
            self.write_leading_trivia(
                &assoc.value.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&assoc.value, ctx);
            self.dedent();
        }
    }

    fn format_begin(&mut self, begin: &Begin, ctx: &FormatContext) {
        self.push_str("begin");
        self.write_trailing_comment(&begin.keyword_trailing);
        self.format_block_body(&begin.body, ctx, true);
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_def(&mut self, def: &Def, ctx: &FormatContext) {
        self.push_str("def");
        if let Some(receiver) = &def.receiver {
            if receiver.shape.fits_in_one_line(self.remaining_width) || receiver.is_diagonal() {
                self.push(' ');
                self.format(receiver, ctx);
            } else {
                self.indent();
                self.break_line(ctx);
                // no leading trivia here.
                self.format(receiver, ctx);
            }
            self.push('.');
            if receiver.trailing_trivia.is_none() {
                self.push_str(&def.name);
                self.format_method_parameters(&def.parameters, ctx);
            } else {
                self.write_trailing_comment(&receiver.trailing_trivia);
                self.indent();
                self.break_line(ctx);
                self.push_str(&def.name);
                self.format_method_parameters(&def.parameters, ctx);
                self.dedent();
            }
        } else {
            self.push(' ');
            self.push_str(&def.name);
            self.format_method_parameters(&def.parameters, ctx);
        }
        match &def.body {
            // def foo = body
            DefBody::Short { body } => {
                self.push_str(" =");
                if body.shape.fits_in_one_line(self.remaining_width) || body.is_diagonal() {
                    self.push(' ');
                    self.format(body, ctx);
                    self.write_trailing_comment(&body.trailing_trivia);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &body.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: true,
                            end: true,
                        },
                    );
                    self.format(body, ctx);
                    self.write_trailing_comment(&body.trailing_trivia);
                    self.dedent();
                }
            }
            // def foo\n body\n end
            DefBody::Block {
                head_trailing,
                body,
            } => {
                self.write_trailing_comment(head_trailing);
                self.format_block_body(body, ctx, true);
                self.break_line(ctx);
                self.push_str("end");
            }
        }
    }

    fn format_block_body(&mut self, body: &BlockBody, ctx: &FormatContext, block_always: bool) {
        if body.shape.fits_in_inline(self.remaining_width) && !block_always {
            self.format_statements(&body.statements, ctx, block_always);
            return;
        }

        if !body.statements.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&body.statements, ctx, true);
            self.dedent();
        }
        for rescue in &body.rescues {
            self.break_line(ctx);
            self.format_rescue(rescue, ctx);
        }
        if let Some(rescue_else) = &body.rescue_else {
            self.break_line(ctx);
            self.push_str("else");
            self.write_trailing_comment(&rescue_else.keyword_trailing);
            if !rescue_else.body.shape().is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&rescue_else.body, ctx, true);
                self.dedent();
            }
        }
        if let Some(ensure) = &body.ensure {
            self.break_line(ctx);
            self.push_str("ensure");
            self.write_trailing_comment(&ensure.keyword_trailing);
            if !ensure.body.shape().is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&ensure.body, ctx, true);
                self.dedent();
            }
        }
    }

    fn format_rescue(&mut self, rescue: &Rescue, ctx: &FormatContext) {
        self.push_str("rescue");
        if !rescue.exceptions.is_empty() {
            if rescue
                .exceptions_shape
                .fits_in_one_line(self.remaining_width)
            {
                self.push(' ');
                for (i, exception) in rescue.exceptions.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(exception, ctx);
                    self.write_trailing_comment(&exception.trailing_trivia);
                }
            } else {
                self.push(' ');
                self.format(&rescue.exceptions[0], ctx);
                if rescue.exceptions.len() > 1 {
                    self.push(',');
                }
                self.write_trailing_comment(&rescue.exceptions[0].trailing_trivia);
                if rescue.exceptions.len() > 1 {
                    self.indent();
                    let last_idx = rescue.exceptions.len() - 1;
                    for (i, exception) in rescue.exceptions.iter().enumerate().skip(1) {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &exception.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: false,
                                end: false,
                            },
                        );
                        self.format(exception, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&exception.trailing_trivia);
                    }
                    self.dedent();
                }
            }
        }
        if let Some(reference) = &rescue.reference {
            self.push_str(" =>");
            if reference.shape.fits_in_one_line(self.remaining_width) || reference.is_diagonal() {
                self.push(' ');
                self.format(reference, ctx);
                self.write_trailing_comment(&reference.trailing_trivia);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &reference.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: false,
                    },
                );
                self.format(reference, ctx);
                self.write_trailing_comment(&reference.trailing_trivia);
                self.dedent();
            }
        }
        self.write_trailing_comment(&rescue.head_trailing);
        if !rescue.statements.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&rescue.statements, ctx, true);
            self.dedent();
        }
    }

    fn format_method_parameters(&mut self, params: &Option<MethodParameters>, ctx: &FormatContext) {
        if let Some(params) = params {
            if params.shape.fits_in_one_line(self.remaining_width) {
                let opening = params.opening.as_deref().unwrap_or(" ");
                self.push_str(opening);
                for (i, n) in params.params.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
                if let Some(closing) = &params.closing {
                    self.push_str(closing);
                }
            } else {
                self.push('(');
                self.indent();
                if !params.params.is_empty() {
                    let last_idx = params.params.len() - 1;
                    for (i, n) in params.params.iter().enumerate() {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &n.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: i == 0,
                                end: false,
                            },
                        );
                        self.format(n, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&n.trailing_trivia);
                    }
                }
                self.write_trivia_at_virtual_end(
                    ctx,
                    &params.virtual_end,
                    true,
                    params.params.is_empty(),
                );
                self.dedent();
                self.break_line(ctx);
                self.push(')');
            }
        }
    }

    fn format_block_parameters(&mut self, params: &BlockParameters, ctx: &FormatContext) {
        if params.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&params.opening);
            for (i, n) in params.params.iter().enumerate() {
                if n.shape.is_empty() {
                    self.push(',');
                } else {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
            }
            if !params.locals.is_empty() {
                self.push_str("; ");
                for (i, n) in params.locals.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
            }
            self.push_str(&params.closing);
            self.write_trailing_comment(&params.closing_trailing);
        } else {
            self.push_str(&params.opening);
            self.indent();
            if !params.params.is_empty() {
                let last_idx = params.params.len() - 1;
                for (i, n) in params.params.iter().enumerate() {
                    if n.shape.is_empty() {
                        self.write_trailing_comment(&n.trailing_trivia);
                        continue;
                    }
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &n.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    self.format(n, ctx);
                    if i < last_idx {
                        self.push(',');
                    }
                    self.write_trailing_comment(&n.trailing_trivia);
                }
            }
            if !params.locals.is_empty() {
                self.break_line(ctx);
                self.push(';');
                let last_idx = params.locals.len() - 1;
                for (i, n) in params.locals.iter().enumerate() {
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &n.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: false,
                            end: false,
                        },
                    );
                    self.format(n, ctx);
                    if i < last_idx {
                        self.push(',');
                    }
                    self.write_trailing_comment(&n.trailing_trivia);
                }
            }
            self.write_trivia_at_virtual_end(
                ctx,
                &params.virtual_end,
                true,
                params.params.is_empty(),
            );
            self.dedent();
            self.break_line(ctx);
            self.push_str(&params.closing);
            self.write_trailing_comment(&params.closing_trailing);
        }
    }

    fn format_class_like(&mut self, class: &ClassLike, ctx: &FormatContext) {
        self.push_str(&class.keyword);
        self.push(' ');
        self.push_str(&class.name);
        if let Some(superclass) = &class.superclass {
            self.push_str(" <");
            if superclass.shape.fits_in_one_line(self.remaining_width) || superclass.is_diagonal() {
                self.push(' ');
                self.format(superclass, ctx);
                self.write_trailing_comment(&superclass.trailing_trivia);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &superclass.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.format(superclass, ctx);
                self.write_trailing_comment(&superclass.trailing_trivia);
                self.dedent();
            }
        } else {
            self.write_trailing_comment(&class.head_trailing);
        }
        self.format_block_body(&class.body, ctx, true);
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_singleton_class(&mut self, class: &SingletonClass, ctx: &FormatContext) {
        self.push_str("class <<");
        if class
            .expression
            .shape
            .fits_in_one_line(self.remaining_width)
            || class.expression.is_diagonal()
        {
            self.push(' ');
            self.format(&class.expression, ctx);
            self.write_trailing_comment(&class.expression.trailing_trivia);
        } else {
            self.indent();
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &class.expression.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: false,
                    end: false,
                },
            );
            self.format(&class.expression, ctx);
            self.write_trailing_comment(&class.expression.trailing_trivia);
            self.dedent();
            self.dedent();
        }
        self.format_block_body(&class.body, ctx, true);
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_range_like(&mut self, range: &RangeLike, ctx: &FormatContext) {
        if let Some(left) = &range.left {
            self.format(left, ctx);
        }
        self.push_str(&range.operator);
        if let Some(right) = &range.right {
            if right.shape.fits_in_one_line(self.remaining_width) || right.is_diagonal() {
                let need_space = match &right.kind {
                    Kind::RangeLike(range) => range.left.is_none(),
                    _ => false,
                };
                if need_space {
                    self.push(' ');
                }
                self.format(right, ctx);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &right.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.format(right, ctx);
                self.dedent();
            }
        }
    }

    fn format_pre_post_exec(&mut self, exec: &PrePostExec, ctx: &FormatContext) {
        if exec.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&exec.keyword);
            self.push_str(" {");
            if !exec.statements.shape.is_empty() {
                self.push(' ');
                self.format_statements(&exec.statements, ctx, false);
                self.push(' ');
            }
            self.push('}');
        } else {
            self.push_str(&exec.keyword);
            self.push_str(" {");
            if !exec.statements.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&exec.statements, ctx, true);
                self.dedent();
            }
            self.break_line(ctx);
            self.push('}');
        }
    }

    fn format_alias(&mut self, alias: &Alias, ctx: &FormatContext) {
        self.push_str("alias ");
        self.format(&alias.new_name, ctx);
        self.push(' ');
        self.format(&alias.old_name, ctx);
    }

    fn write_leading_trivia(
        &mut self,
        trivia: &LeadingTrivia,
        ctx: &FormatContext,
        emp_line_handling: EmptyLineHandling,
    ) {
        if trivia.is_empty() {
            return;
        }
        let last_idx = trivia.lines.len() - 1;
        for (i, trivia) in trivia.lines.iter().enumerate() {
            match trivia {
                LineTrivia::EmptyLine => {
                    let should_skip = match emp_line_handling {
                        EmptyLineHandling::Skip => true,
                        EmptyLineHandling::Trim { start, end } => {
                            (start && i == 0) || (end && i == last_idx)
                        }
                    };
                    if !should_skip {
                        self.break_line(ctx);
                    }
                }
                LineTrivia::Comment(comment) => {
                    self.push_str(&comment.value);
                    self.break_line(ctx);
                }
            }
        }
    }

    fn write_trailing_comment(&mut self, trivia: &TrailingTrivia) {
        if let Some(comment) = &trivia.comment {
            self.push(' ');
            self.buffer.push_str(&comment.value);
        }
    }

    fn push(&mut self, c: char) {
        if self.remaining_width == self.config.line_width {
            self.put_indent();
        }
        self.buffer.push(c);
        self.remaining_width = self.remaining_width.saturating_sub(1);
    }

    fn push_str(&mut self, str: &str) {
        if self.remaining_width == self.config.line_width {
            self.put_indent();
        }
        self.push_str_without_indent(str);
    }

    fn push_str_without_indent(&mut self, str: &str) {
        self.buffer.push_str(str);
        self.remaining_width = self.remaining_width.saturating_sub(str.len());
    }

    fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
        self.remaining_width = self.remaining_width.saturating_sub(spaces.len());
    }

    fn indent(&mut self) {
        self.indent += self.config.indent_size;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(self.config.indent_size);
    }

    fn break_line(&mut self, ctx: &FormatContext) {
        self.buffer.push('\n');
        self.remaining_width = self.config.line_width;
        self.line_count += 1;
        let mut queue = mem::take(&mut self.heredoc_queue);
        while let Some(pos) = queue.pop_front() {
            self.write_heredoc_body(&pos, ctx);
        }
    }

    fn write_heredoc_body(&mut self, pos: &Pos, ctx: &FormatContext) {
        let heredoc = ctx.heredoc_map.get(pos).expect("heredoc must exist");
        match heredoc.indent_mode {
            HeredocIndentMode::None | HeredocIndentMode::EndIndented => {
                for part in &heredoc.parts {
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            self.push_str_without_indent(&value);
                        }
                        HeredocPart::Statements(embedded) => {
                            self.format_embedded_statements(embedded, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            self.format_embedded_variable(var);
                        }
                    }
                }
                if matches!(heredoc.indent_mode, HeredocIndentMode::EndIndented) {
                    self.put_indent();
                }
                self.push_str(&heredoc.id);
            }
            HeredocIndentMode::AllIndented => {
                for part in &heredoc.parts {
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            self.push_str_without_indent(&value);
                        }
                        HeredocPart::Statements(embedded) => {
                            self.format_embedded_statements(embedded, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            self.format_embedded_variable(var);
                        }
                    }
                }
                self.put_indent();
                self.push_str(&heredoc.id);
            }
        }
        self.buffer.push('\n');
    }
}
