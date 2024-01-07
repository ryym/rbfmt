use std::{
    collections::{HashMap, VecDeque},
    mem,
};

pub(crate) fn format(node: Node, heredoc_map: HeredocMap) -> String {
    let config = FormatConfig { line_width: 100 };
    let ctx = FormatContext { heredoc_map };
    let mut formatter = Formatter {
        remaining_width: config.line_width,
        config,
        buffer: String::new(),
        indent: 0,
        heredoc_queue: VecDeque::new(),
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
            Self::Inline { len } => *len < width,
            _ => false,
        }
    }

    pub(crate) fn fits_in_one_line(&self, width: usize) -> bool {
        match self {
            Self::Inline { len } | Self::LineEnd { len } => *len < width,
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
                Self::LineEnd { .. } | Self::Multilines => Self::Multilines,
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
    Exprs(Exprs),
    Parens(Parens),
    IfExpr(IfExpr),
    Postmodifier(Postmodifier),
    MethodChain(MethodChain),
    Assign(Assign),
    MultiAssignTarget(MultiAssignTarget),
    Splat(Splat),
    Array(Array),
    Hash(Hash),
    KeywordHash(KeywordHash),
    Assoc(Assoc),
    Def(Def),
}

impl Kind {
    pub(crate) fn shape(&self) -> Shape {
        match self {
            Self::Atom(s) => Shape::inline(s.len()),
            Self::StringLike(s) => s.shape,
            Self::DynStringLike(s) => s.shape,
            Self::HeredocOpening(opening) => *opening.shape(),
            Self::Exprs(exprs) => exprs.shape,
            Self::Parens(parens) => parens.shape,
            Self::IfExpr(_) => IfExpr::shape(),
            Self::Postmodifier(pmod) => pmod.shape,
            Self::MethodChain(chain) => chain.shape,
            Self::Assign(assign) => assign.shape,
            Self::MultiAssignTarget(multi) => multi.shape,
            Self::Splat(splat) => splat.shape,
            Self::Array(array) => array.shape,
            Self::Hash(hash) => hash.shape,
            Self::KeywordHash(khash) => khash.shape,
            Self::Assoc(assoc) => assoc.shape,
            Self::Def(def) => def.shape,
        }
    }

    pub(crate) fn is_diagonal(&self) -> bool {
        match self {
            Self::Atom(_) => false,
            Self::StringLike(_) => false,
            Self::DynStringLike(_) => false,
            Self::HeredocOpening(_) => false,
            Self::Exprs(_) => false,
            Self::Parens(_) => true,
            Self::IfExpr(_) => false,
            Self::Postmodifier(_) => true,
            Self::MethodChain(_) => true,
            Self::Assign(_) => true,
            Self::MultiAssignTarget(_) => true,
            Self::Splat(_) => false,
            Self::Array(_) => true,
            Self::Hash(_) => true,
            Self::KeywordHash(_) => true,
            Self::Assoc(_) => true,
            Self::Def(_) => false,
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
    Exprs(EmbeddedExprs),
}

impl DynStrPart {
    pub(crate) fn shape(&self) -> &Shape {
        match self {
            Self::Str(s) => &s.shape,
            Self::DynStr(s) => &s.shape,
            Self::Exprs(e) => &e.shape,
        }
    }
}

#[derive(Debug)]
pub(crate) struct EmbeddedExprs {
    pub shape: Shape,
    pub opening: String,
    pub exprs: Exprs,
    pub closing: String,
}

impl EmbeddedExprs {
    pub(crate) fn new(opening: String, exprs: Exprs, closing: String) -> Self {
        let shape = Shape::inline(opening.len() + closing.len()).add(&exprs.shape);
        Self {
            shape,
            opening,
            exprs,
            closing,
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
    Exprs(EmbeddedExprs),
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
pub(crate) struct Exprs {
    shape: Shape,
    nodes: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
}

impl Exprs {
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
    body: Exprs,
}

impl Parens {
    pub(crate) fn new(body: Exprs) -> Self {
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
pub(crate) struct IfExpr {
    pub is_if: bool,
    pub if_first: Conditional,
    pub elsifs: Vec<Conditional>,
    pub if_last: Option<Else>,
}

impl IfExpr {
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
    pub keyword_trailing: TrailingTrivia,
    pub cond: Box<Node>,
    pub body: Exprs,
}

impl Conditional {
    pub(crate) fn new(keyword_trailing: TrailingTrivia, cond: Node, body: Exprs) -> Self {
        let shape = keyword_trailing.shape.add(&cond.shape).add(&body.shape);
        Self {
            shape,
            keyword_trailing,
            cond: Box::new(cond),
            body,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Else {
    pub keyword_trailing: TrailingTrivia,
    pub body: Exprs,
}

#[derive(Debug)]
pub(crate) struct Arguments {
    nodes: Vec<Node>,
    shape: Shape,
    virtual_end: Option<VirtualEnd>,
}

impl Arguments {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![],
            shape: Shape::inline(0),
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        self.shape.append(&node.shape);
        if !self.nodes.is_empty() {
            self.shape.append(&Shape::inline(", ".len()));
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }
}

#[derive(Debug)]
pub(crate) struct MethodCall {
    pub shape: Shape,
    pub leading_trivia: LeadingTrivia,
    pub trailing_trivia: TrailingTrivia,
    pub call_op: Option<String>,
    pub name: String,
    pub args: Option<Arguments>,
    pub block: Option<MethodBlock>,
}

impl MethodCall {
    pub(crate) fn new(
        leading_trivia: LeadingTrivia,
        call_op: Option<String>,
        name: String,
    ) -> Self {
        let call_op_len = call_op.as_ref().map_or(0, |s| s.len());
        let msg_shape = Shape::inline(name.len() + call_op_len);
        let shape = leading_trivia.shape.add(&msg_shape);
        Self {
            shape,
            leading_trivia,
            trailing_trivia: TrailingTrivia::none(),
            call_op,
            name,
            args: None,
            block: None,
        }
    }

    pub(crate) fn set_args(&mut self, args: Arguments) {
        // For now surround the arguments by parentheses always.
        self.shape.append(&Shape::inline("(".len()));
        self.shape.append(&args.shape);
        self.shape.append(&Shape::inline(")".len()));
        self.args = Some(args);
    }

    pub(crate) fn set_block(&mut self, block: MethodBlock) {
        self.shape.append(&block.shape);
        self.block = Some(block);
    }

    pub(crate) fn set_trailing_trivia(&mut self, trivia: TrailingTrivia) {
        self.shape.append(&trivia.shape);
        self.trailing_trivia = trivia;
    }
}

#[derive(Debug)]
pub(crate) struct MethodBlock {
    pub shape: Shape,
    pub trailing_trivia: TrailingTrivia,
    // pub args
    pub body: Exprs,
    pub was_flat: bool,
}

#[derive(Debug)]
pub(crate) struct MethodChain {
    shape: Shape,
    receiver: Option<Box<Node>>,
    calls: Vec<MethodCall>,
}

impl MethodChain {
    pub(crate) fn new(receiver: Option<Node>) -> Self {
        Self {
            shape: receiver.as_ref().map_or(Shape::inline(0), |r| r.shape),
            receiver: receiver.map(Box::new),
            calls: vec![],
        }
    }

    pub(crate) fn append_call(&mut self, call: MethodCall) {
        self.shape.append(&call.shape);
        self.calls.push(call);
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
            .add(&Shape::inline(operator.len()));
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
    opening: String,
    closing: String,
    elements: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
}

impl Array {
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

    pub(crate) fn separator(&self) -> &str {
        if self.opening.as_bytes()[0] == b'%' {
            ""
        } else {
            ","
        }
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
pub(crate) struct Splat {
    shape: Shape,
    operator: String,
    expression: Box<Node>,
}

impl Splat {
    pub(crate) fn new(operator: String, expression: Node) -> Self {
        let shape = Shape::inline(operator.len()).add(&expression.shape);
        Self {
            shape,
            operator,
            expression: Box::new(expression),
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
        if !self.elements.is_empty() {
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
pub(crate) struct KeywordHash {
    shape: Shape,
    elements: Vec<Node>,
}

impl KeywordHash {
    pub(crate) fn new() -> Self {
        let shape = Shape::inline(0);
        Self {
            shape,
            elements: vec![],
        }
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if !self.elements.is_empty() {
            self.shape.append(&Shape::inline(", ".len()));
        }
        self.shape.append(&element.shape);
        self.elements.push(element);
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
pub(crate) struct Def {
    shape: Shape,
    receiver: Option<Box<Node>>,
    name: String,
    // parameters
    // name_trailing
    // body (statements | begin)
    // is_inline
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

impl EmptyLineHandling {
    fn trim_start() -> Self {
        Self::Trim {
            start: true,
            end: false,
        }
    }
}

pub(crate) type HeredocMap = HashMap<Pos, Heredoc>;

#[derive(Debug)]
struct FormatConfig {
    line_width: usize,
}

#[derive(Debug)]
struct FormatContext {
    heredoc_map: HeredocMap,
}

#[derive(Debug)]
struct Formatter {
    config: FormatConfig,
    remaining_width: usize,
    buffer: String,
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
}

impl Formatter {
    fn format(&mut self, node: &Node, ctx: &FormatContext) {
        match &node.kind {
            Kind::Atom(value) => self.format_atom(value),
            Kind::StringLike(str) => self.format_string_like(str),
            Kind::DynStringLike(dstr) => self.format_dyn_string_like(dstr, ctx),
            Kind::HeredocOpening(opening) => self.format_heredoc_opening(opening),
            Kind::Exprs(exprs) => self.format_exprs(exprs, ctx, false),
            Kind::Parens(parens) => self.format_parens(parens, ctx),
            Kind::IfExpr(expr) => self.format_if_expr(expr, ctx),
            Kind::Postmodifier(modifier) => self.format_postmodifier(modifier, ctx),
            Kind::MethodChain(chain) => self.format_method_chain(chain, ctx),
            Kind::Assign(assign) => self.format_assign(assign, ctx),
            Kind::MultiAssignTarget(multi) => self.format_multi_assign_target(multi, ctx),
            Kind::Splat(splat) => self.format_splat(splat, ctx),
            Kind::Array(array) => self.format_array(array, ctx),
            Kind::Hash(hash) => self.format_hash(hash, ctx),
            Kind::KeywordHash(khash) => self.format_keyword_hash(khash, ctx),
            Kind::Assoc(assoc) => self.format_assoc(assoc, ctx),
            Kind::Def(def) => self.format_def(def, ctx),
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
                DynStrPart::Exprs(embedded) => {
                    self.format_embedded_exprs(embedded, ctx);
                }
            }
        }
        if let Some(closing) = &dstr.closing {
            self.push_str(closing);
        }
    }

    fn format_embedded_exprs(&mut self, embedded: &EmbeddedExprs, ctx: &FormatContext) {
        self.push_str(&embedded.opening);

        if embedded.exprs.shape.fits_in_inline(self.remaining_width) {
            self.format_exprs(&embedded.exprs, ctx, false);
        } else {
            self.break_line(ctx);
            self.indent();
            self.format_exprs(&embedded.exprs, ctx, false);
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
        }

        self.push_str(&embedded.closing);
    }

    fn format_heredoc_opening(&mut self, opening: &HeredocOpening) {
        self.push_str(opening.indent_mode.prefix_symbols());
        self.push_str(&opening.id);
        self.heredoc_queue.push_back(opening.pos);
    }

    fn format_exprs(&mut self, exprs: &Exprs, ctx: &FormatContext, block_always: bool) {
        if exprs.shape.is_inline() && !block_always {
            if let Some(node) = exprs.nodes.get(0) {
                self.format(node, ctx);
            }
            return;
        }
        for (i, n) in exprs.nodes.iter().enumerate() {
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
            self.put_indent();
            self.format(n, ctx);
            self.write_trailing_comment(&n.trailing_trivia);
        }
        self.write_trivia_at_virtual_end(
            ctx,
            &exprs.virtual_end,
            !exprs.nodes.is_empty(),
            exprs.nodes.is_empty(),
        );
    }

    fn format_parens(&mut self, parens: &Parens, ctx: &FormatContext) {
        if parens.body.shape().is_empty() {
            self.push_str("()");
        } else {
            self.push('(');
            if parens.body.shape.fits_in_inline(self.remaining_width) {
                self.format_exprs(&parens.body, ctx, false);
            } else {
                self.indent();
                self.break_line(ctx);
                self.format_exprs(&parens.body, ctx, false);
                self.dedent();
                self.break_line(ctx);
                self.put_indent();
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
                        self.put_indent();
                        self.push_str(&comment.value);
                        if i < last_idx {
                            self.break_line(ctx);
                        }
                    }
                }
            }
        }
    }

    fn format_if_expr(&mut self, expr: &IfExpr, ctx: &FormatContext) {
        if expr.is_if {
            self.push_str("if");
        } else {
            self.push_str("unless");
        }

        self.format_node_after_keyword(ctx, &expr.if_first.keyword_trailing, &expr.if_first.cond);
        if !expr.if_first.body.shape.is_empty() {
            self.break_line(ctx);
            self.format_exprs(&expr.if_first.body, ctx, true);
        }

        for elsif in &expr.elsifs {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.push_str("elsif");
            self.format_node_after_keyword(ctx, &elsif.keyword_trailing, &elsif.cond);
            if !elsif.body.shape.is_empty() {
                self.break_line(ctx);
                self.format_exprs(&elsif.body, ctx, true);
            }
        }

        if let Some(if_last) = &expr.if_last {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.push_str("else");
            self.write_trailing_comment(&if_last.keyword_trailing);
            self.indent();
            if !if_last.body.shape.is_empty() {
                self.break_line(ctx);
                self.format_exprs(&if_last.body, ctx, true);
            }
        }

        self.break_line(ctx);
        self.dedent();
        self.put_indent();
        self.push_str("end");
    }

    fn format_postmodifier(&mut self, modifier: &Postmodifier, ctx: &FormatContext) {
        self.format_exprs(&modifier.conditional.body, ctx, false);
        self.push(' ');
        self.push_str(&modifier.keyword);
        self.format_node_after_keyword(
            ctx,
            &modifier.conditional.keyword_trailing,
            &modifier.conditional.cond,
        );
        self.dedent();
    }

    // Handle comments like "if # foo\n #bar\n predicate"
    fn format_node_after_keyword(
        &mut self,
        ctx: &FormatContext,
        keyword_trailing: &TrailingTrivia,
        node: &Node,
    ) {
        if keyword_trailing.is_none() && node.leading_trivia.is_empty() {
            self.push(' ');
            self.format(node, ctx);
            self.write_trailing_comment(&node.trailing_trivia);
            self.indent();
        } else {
            self.write_trailing_comment(keyword_trailing);
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(&node.leading_trivia, ctx, EmptyLineHandling::trim_start());
            self.put_indent();
            self.format(node, ctx);
            self.write_trailing_comment(&node.trailing_trivia);
        }
    }

    fn format_method_chain(&mut self, chain: &MethodChain, ctx: &FormatContext) {
        if let Some(recv) = &chain.receiver {
            self.format(recv, ctx);
            self.write_trailing_comment(&recv.trailing_trivia);
        }

        if chain.shape.fits_in_inline(self.remaining_width) {
            for call in chain.calls.iter() {
                if let Some(call_op) = &call.call_op {
                    self.push_str(call_op);
                }
                let args_parens = if call.name == "[]" {
                    ('[', ']')
                } else {
                    self.push_str(&call.name);
                    ('(', ')')
                };

                if let Some(args) = &call.args {
                    self.push(args_parens.0);
                    for (i, arg) in args.nodes.iter().enumerate() {
                        if i > 0 {
                            self.push_str(", ");
                        }
                        self.format(arg, ctx);
                    }
                    self.push(args_parens.1);
                }
                if let Some(block) = &call.block {
                    if block.body.shape.is_empty() {
                        self.push_str(" {}");
                    } else {
                        self.push_str(" { ");
                        self.format_exprs(&block.body, ctx, false);
                        self.push_str(" }");
                    }
                }
            }
        } else {
            let mut indented = false;
            for call in chain.calls.iter() {
                if call.call_op.is_some() && !indented {
                    self.indent();
                    indented = true;
                }
                if let Some(call_op) = &call.call_op {
                    self.break_line(ctx);
                    self.write_leading_trivia(&call.leading_trivia, ctx, EmptyLineHandling::Skip);
                    self.put_indent();
                    self.push_str(call_op);
                }
                let args_parens = if call.name == "[]" {
                    ('[', ']')
                } else {
                    self.push_str(&call.name);
                    ('(', ')')
                };

                if let Some(args) = &call.args {
                    self.push(args_parens.0);
                    let remaining = self.remaining_width.saturating_sub(1);
                    if args.shape.fits_in_inline(remaining) {
                        for (i, arg) in args.nodes.iter().enumerate() {
                            if i > 0 {
                                self.push_str(", ");
                            }
                            self.format(arg, ctx);
                        }
                    } else {
                        self.indent();
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
                            self.put_indent();
                            self.format(arg, ctx);
                            self.push(',');
                            self.write_trailing_comment(&arg.trailing_trivia);
                        }
                        self.write_trivia_at_virtual_end(
                            ctx,
                            &args.virtual_end,
                            true,
                            args.nodes.is_empty(),
                        );
                        self.dedent();
                        self.break_line(ctx);
                        self.put_indent();
                    }
                    self.push(args_parens.1);
                }

                if let Some(block) = &call.block {
                    if !block.trailing_trivia.is_none()
                        || !block.body.shape.fits_in_inline(self.remaining_width)
                        || !block.was_flat
                    {
                        self.push_str(" do");
                        self.write_trailing_comment(&block.trailing_trivia);
                        self.indent();
                        if !block.body.shape.is_empty() {
                            self.break_line(ctx);
                            self.format_exprs(&block.body, ctx, true);
                        }
                        self.dedent();
                        self.break_line(ctx);
                        self.put_indent();
                        self.push_str("end");
                    } else if block.body.shape.is_empty() {
                        self.push_str(" {}");
                    } else {
                        self.push_str(" { ");
                        self.format_exprs(&block.body, ctx, false);
                        self.push_str(" }");
                    }
                }
                self.write_trailing_comment(&call.trailing_trivia);
            }
            if indented {
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
            self.write_trailing_comment(&value.trailing_trivia);
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
            self.put_indent();
            self.format(value, ctx);
            self.write_trailing_comment(&value.trailing_trivia);
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
                self.put_indent();
                self.format(target, ctx);
                if i < last_idx || multi.with_implicit_rest {
                    self.push(',');
                }
                self.write_trailing_comment(&target.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(ctx, &multi.virtual_end, true, false);
            self.dedent();
            self.break_line(ctx);
            self.put_indent();
            self.push(')');
        }
    }

    fn format_splat(&mut self, splat: &Splat, ctx: &FormatContext) {
        self.push_str(&splat.operator);
        self.format(&splat.expression, ctx);
    }

    fn format_array(&mut self, array: &Array, ctx: &FormatContext) {
        if array.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&array.opening);
            for (i, n) in array.elements.iter().enumerate() {
                if i > 0 {
                    self.push_str(array.separator());
                    self.push(' ');
                }
                self.format(n, ctx);
            }
            self.push_str(&array.closing);
        } else {
            self.push_str(&array.opening);
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
                self.put_indent();
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
            self.put_indent();
            self.push_str(&array.closing);
        }
    }

    fn format_hash(&mut self, hash: &Hash, ctx: &FormatContext) {
        if hash.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&hash.opening);
            self.push(' ');
            for (i, n) in hash.elements.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(n, ctx);
            }
            self.push(' ');
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
                self.put_indent();
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
            self.put_indent();
            self.push_str(&hash.closing);
        }
    }

    fn format_keyword_hash(&mut self, khash: &KeywordHash, ctx: &FormatContext) {
        if khash.shape.fits_in_inline(self.remaining_width) {
            for (i, n) in khash.elements.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(n, ctx);
            }
        } else {
            let last_idx = khash.elements.len() - 1;
            for (i, element) in khash.elements.iter().enumerate() {
                if i > 0 {
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &element.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: false,
                            end: false,
                        },
                    );
                    self.put_indent();
                }
                self.format(element, ctx);
                if i < last_idx {
                    self.push(',');
                }
                self.write_trailing_comment(&element.trailing_trivia);
            }
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
            self.put_indent();
            self.format(&assoc.value, ctx);
            self.dedent();
        }
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
                self.put_indent();
                // no leading trivia here.
                self.format(receiver, ctx);
            }
            self.push('.');
            if receiver.trailing_trivia.is_none() {
                self.push_str(&def.name);
                self.break_line(ctx);
                self.put_indent();
            } else {
                self.write_trailing_comment(&receiver.trailing_trivia);
                self.indent();
                self.break_line(ctx);
                self.put_indent();
                self.push_str(&def.name);
                self.break_line(ctx);
                self.dedent();
            }
        } else {
            self.push(' ');
            self.push_str(&def.name);
            self.break_line(ctx);
            self.put_indent();
        }
        self.push_str("end");
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
                    self.put_indent();
                    self.push_str(&comment.value);
                    self.break_line(ctx);
                }
            }
        }
    }

    fn write_trailing_comment(&mut self, trivia: &TrailingTrivia) {
        if let Some(comment) = &trivia.comment {
            self.push(' ');
            self.push_str(&comment.value);
        }
    }

    fn push(&mut self, c: char) {
        self.buffer.push(c);
        self.remaining_width = self.remaining_width.saturating_sub(1);
    }

    fn push_str(&mut self, str: &str) {
        self.buffer.push_str(str);
        self.remaining_width = self.remaining_width.saturating_sub(str.len());
    }

    fn indent(&mut self) {
        self.indent += 2;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }

    fn break_line(&mut self, ctx: &FormatContext) {
        self.push('\n');
        self.remaining_width = self.config.line_width;
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
                            self.push_str(&value);
                        }
                        HeredocPart::Exprs(embedded) => {
                            self.format_embedded_exprs(embedded, ctx);
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
                            self.push_str(&value);
                        }
                        HeredocPart::Exprs(embedded) => {
                            self.format_embedded_exprs(embedded, ctx);
                        }
                    }
                }
                self.put_indent();
                self.push_str(&heredoc.id);
            }
        }
        self.push('\n');
    }

    fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.push_str(&spaces);
    }
}
