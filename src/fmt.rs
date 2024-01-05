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
pub(crate) enum Width {
    Flat(usize),
    NotFlat,
}

impl Width {
    pub(crate) fn fits_in(&self, width: usize) -> bool {
        match self {
            Self::Flat(w) => *w <= width,
            Self::NotFlat => false,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Self::Flat(w) => *w == 0,
            Self::NotFlat => false,
        }
    }

    pub(crate) fn add(self, other: &Self) -> Self {
        match (&self, other) {
            (Self::Flat(w1), Self::Flat(w2)) => Self::Flat(*w1 + w2),
            _ => Self::NotFlat,
        }
    }

    pub(crate) fn append(&mut self, other: &Self) {
        let width = self.add(other);
        let _ = mem::replace(self, width);
    }

    pub(crate) fn append_value(&mut self, value: usize) {
        if let Self::Flat(width) = &self {
            let _ = mem::replace(self, Self::Flat(width + value));
        }
    }
}

#[derive(Debug)]
pub(crate) struct Node {
    pub trivia: Trivia,
    pub kind: Kind,
    pub width: Width,
}

impl Node {
    pub(crate) fn new(trivia: Trivia, kind: Kind) -> Self {
        let width = trivia.width.add(&kind.width());
        Self {
            trivia,
            kind,
            width,
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
}

impl Kind {
    pub(crate) fn width(&self) -> Width {
        match self {
            Self::Atom(s) => Width::Flat(s.len()),
            Self::StringLike(s) => s.width,
            Self::DynStringLike(s) => s.width,
            Self::HeredocOpening(opening) => *opening.width(),
            Self::Exprs(exprs) => exprs.width,
            Self::IfExpr(_) => IfExpr::width(),
            Self::Postmodifier(pmod) => pmod.width,
            Self::MethodChain(chain) => chain.width,
            Self::Assign(assign) => assign.width,
            Self::MultiAssignTarget(multi) => multi.width,
            Self::Splat(splat) => splat.width,
            Self::Array(array) => array.width,
            Self::Hash(hash) => hash.width,
            Self::KeywordHash(khash) => khash.width,
            Self::Assoc(assoc) => assoc.width,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringLike {
    pub width: Width,
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
            width: Width::Flat(len),
            opening,
            value,
            closing,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DynStringLike {
    pub width: Width,
    pub opening: Option<String>,
    pub parts: Vec<DynStrPart>,
    pub closing: Option<String>,
}

impl DynStringLike {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |s| s.len());
        let closing_len = closing.as_ref().map_or(0, |s| s.len());
        Self {
            width: Width::Flat(opening_len + closing_len),
            opening,
            parts: vec![],
            closing,
        }
    }

    pub(crate) fn append_part(&mut self, part: DynStrPart) {
        self.width.append(part.width());
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
    pub(crate) fn width(&self) -> &Width {
        match self {
            Self::Str(s) => &s.width,
            Self::DynStr(s) => &s.width,
            Self::Exprs(e) => &e.width,
        }
    }
}

#[derive(Debug)]
pub(crate) struct EmbeddedExprs {
    pub width: Width,
    pub opening: String,
    pub exprs: Exprs,
    pub closing: String,
}

impl EmbeddedExprs {
    pub(crate) fn new(opening: String, exprs: Exprs, closing: String) -> Self {
        let width = Width::Flat(opening.len() + closing.len()).add(&exprs.width);
        Self {
            width,
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
    pub width: Width,
    pub trivia: Trivia,
}

#[derive(Debug)]
pub(crate) struct Exprs {
    width: Width,
    nodes: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
}

impl Exprs {
    pub(crate) fn new() -> Self {
        Self {
            width: Width::Flat(0),
            nodes: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        if self.nodes.is_empty() && !matches!(node.kind, Kind::HeredocOpening(_)) {
            self.width = node.width;
        } else {
            self.width = Width::NotFlat;
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.width.append(&end.width);
        }
        self.virtual_end = end;
    }

    pub(crate) fn width(&self) -> Width {
        self.width
    }

    pub(crate) fn can_be_flat(&self) -> bool {
        matches!(self.width, Width::Flat(_))
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self.width {
            Width::Flat(w) => w == 0,
            Width::NotFlat => false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct HeredocOpening {
    pos: Pos,
    width: Width,
    id: String,
    indent_mode: HeredocIndentMode,
}

impl HeredocOpening {
    pub(crate) fn new(pos: Pos, id: String, indent_mode: HeredocIndentMode) -> Self {
        let width = Width::Flat(id.len() + indent_mode.prefix_symbols().len());
        Self {
            pos,
            width,
            id,
            indent_mode,
        }
    }

    pub(crate) fn width(&self) -> &Width {
        &self.width
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

    pub(crate) fn width() -> Width {
        Width::NotFlat
    }
}

#[derive(Debug)]
pub(crate) struct Postmodifier {
    pub width: Width,
    pub keyword: String,
    pub conditional: Conditional,
}

impl Postmodifier {
    pub(crate) fn new(keyword: String, conditional: Conditional) -> Self {
        let kwd_width = Width::Flat(keyword.len() + 2); // keyword and spaces around it.
        let width = conditional.width.add(&kwd_width);
        Self {
            width,
            keyword,
            conditional,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Conditional {
    pub width: Width,
    pub trivia: Trivia,
    pub cond: Box<Node>,
    pub body: Exprs,
}

impl Conditional {
    pub(crate) fn new(trivia: Trivia, cond: Node, body: Exprs) -> Self {
        let width = trivia.width.add(&cond.width).add(&body.width);
        Self {
            width,
            trivia,
            cond: Box::new(cond),
            body,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Else {
    pub trivia: Trivia,
    pub body: Exprs,
}

#[derive(Debug)]
pub(crate) struct Arguments {
    nodes: Vec<Node>,
    width: Width,
    virtual_end: Option<VirtualEnd>,
}

impl Arguments {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![],
            width: Width::Flat(0),
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        self.width.append(&node.width);
        if !self.nodes.is_empty() {
            self.width.append_value(", ".len());
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.width.append(&end.width);
        }
        self.virtual_end = end;
    }
}

#[derive(Debug)]
pub(crate) struct MethodCall {
    pub width: Width,
    pub trivia: Trivia,
    pub call_op: Option<String>,
    pub name: String,
    pub args: Option<Arguments>,
    pub block: Option<MethodBlock>,
}

impl MethodCall {
    pub(crate) fn new(call_op: Option<String>, name: String) -> Self {
        let width = Width::Flat(name.len() + call_op.as_ref().map_or(0, |s| s.len()));
        Self {
            width,
            trivia: Trivia::new(),
            call_op,
            name,
            args: None,
            block: None,
        }
    }

    pub(crate) fn set_trivia(&mut self, trivia: Trivia) {
        self.width.append(&trivia.width);
        self.trivia = trivia;
    }

    pub(crate) fn set_args(&mut self, args: Arguments) {
        // For now surround the arguments by parentheses always.
        self.width.append_value("()".len());
        self.width.append(&args.width);
        self.args = Some(args);
    }

    pub(crate) fn set_block(&mut self, block: MethodBlock) {
        self.width.append(&block.width);
        self.block = Some(block);
    }
}

#[derive(Debug)]
pub(crate) struct MethodBlock {
    pub width: Width,
    pub trivia: Trivia,
    // pub args
    pub body: Exprs,
    pub was_flat: bool,
}

#[derive(Debug)]
pub(crate) struct MethodChain {
    width: Width,
    receiver: Option<Box<Node>>,
    calls: Vec<MethodCall>,
}

impl MethodChain {
    pub(crate) fn new(receiver: Option<Node>) -> Self {
        Self {
            width: receiver.as_ref().map_or(Width::Flat(0), |r| r.width),
            receiver: receiver.map(Box::new),
            calls: vec![],
        }
    }

    pub(crate) fn append_call(&mut self, call: MethodCall) {
        self.width.append(&call.width);
        self.calls.push(call);
    }
}

#[derive(Debug)]
pub(crate) struct Assign {
    width: Width,
    target: Box<Node>,
    operator: String,
    value: Box<Node>,
}

impl Assign {
    pub(crate) fn new(target: Node, operator: String, value: Node) -> Self {
        let width = target
            .width
            .add(&value.width)
            .add(&Width::Flat(operator.len()));
        Self {
            width,
            target: Box::new(target),
            operator,
            value: Box::new(value),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MultiAssignTarget {
    width: Width,
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
            width: Width::Flat(parens_len),
            lparen,
            rparen,
            targets: vec![],
            with_implicit_rest: false,
            virtual_end: None,
        }
    }

    pub(crate) fn append_target(&mut self, target: Node) {
        self.width.append(&target.width);
        self.targets.push(target);
    }

    pub(crate) fn set_implicit_rest(&mut self, yes: bool) {
        if yes {
            self.width.append_value(",".len());
        }
        self.with_implicit_rest = yes;
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.width.append(&end.width);
        }
        self.virtual_end = end;
    }
}

#[derive(Debug)]
pub(crate) struct Array {
    width: Width,
    opening: String,
    closing: String,
    elements: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
}

impl Array {
    pub(crate) fn new(opening: String, closing: String) -> Self {
        let width = Width::Flat(opening.len() + closing.len());
        Self {
            width,
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
            self.width.append_value(self.separator().len() + 1);
        }
        self.width.append(&element.width);
        self.elements.push(element);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.width.append(&end.width);
        }
        self.virtual_end = end;
    }
}

#[derive(Debug)]
pub(crate) struct Splat {
    width: Width,
    operator: String,
    expression: Box<Node>,
}

impl Splat {
    pub(crate) fn new(operator: String, expression: Node) -> Self {
        let width = Width::Flat(operator.len()).add(&expression.width);
        Self {
            width,
            operator,
            expression: Box::new(expression),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Hash {
    width: Width,
    opening: String,
    closing: String,
    elements: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
}

impl Hash {
    pub(crate) fn new(opening: String, closing: String) -> Self {
        let width = Width::Flat(opening.len() + closing.len());
        Self {
            width,
            opening,
            closing,
            elements: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if !self.elements.is_empty() {
            self.width.append_value(", ".len());
        }
        self.width.append(&element.width);
        self.elements.push(element);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.width.append(&end.width);
        }
        self.virtual_end = end;
    }
}

#[derive(Debug)]
pub(crate) struct KeywordHash {
    width: Width,
    elements: Vec<Node>,
}

impl KeywordHash {
    pub(crate) fn new() -> Self {
        let width = Width::Flat(0);
        Self {
            width,
            elements: vec![],
        }
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if !self.elements.is_empty() {
            self.width.append_value(", ".len());
        }
        self.width.append(&element.width);
        self.elements.push(element);
    }
}

#[derive(Debug)]
pub(crate) struct Assoc {
    width: Width,
    key: Box<Node>,
    value: Box<Node>,
    operator: Option<String>,
}

impl Assoc {
    pub(crate) fn new(key: Node, operator: Option<String>, value: Node) -> Self {
        let mut width = key.width.add(&value.width);
        width.append_value(1); // space
        if let Some(op) = &operator {
            width.append_value(op.len());
            width.append_value(1); // space
        }
        Self {
            width,
            key: Box::new(key),
            value: Box::new(value),
            operator,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Trivia {
    pub leading: Vec<LineTrivia>,
    pub trailing: Option<Comment>,
    pub width: Width,
}

impl Trivia {
    pub(crate) fn new() -> Self {
        Self {
            leading: vec![],
            trailing: None,
            width: Width::Flat(0),
        }
    }

    pub(crate) fn append_leading(&mut self, trivia: LineTrivia) {
        if matches!(trivia, LineTrivia::Comment(_)) {
            self.width = Width::NotFlat;
        }
        self.leading.push(trivia);
    }

    pub(crate) fn set_trailing(&mut self, comment: Option<Comment>) {
        if comment.is_some() {
            self.width = Width::NotFlat;
        }
        self.trailing = comment;
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

        if embedded.exprs.width.fits_in(self.remaining_width) {
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
        if exprs.can_be_flat() && !block_always {
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
                &n.trivia.leading,
                ctx,
                EmptyLineHandling::Trim {
                    start: i == 0,
                    end: false,
                },
            );
            self.put_indent();
            self.format(n, ctx);
            self.write_trailing_comment(&n.trivia.trailing);
        }
        self.write_trivia_at_virtual_end(
            ctx,
            &exprs.virtual_end,
            !exprs.nodes.is_empty(),
            exprs.nodes.is_empty(),
        );
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
            for trivia in end.trivia.leading.iter().rev() {
                match trivia {
                    LineTrivia::EmptyLine => {
                        trailing_empty_lines += 1;
                    }
                    LineTrivia::Comment(_) => {
                        break;
                    }
                }
            }
            if trailing_empty_lines == end.trivia.leading.len() {
                return;
            }

            if break_first {
                self.break_line(ctx);
            }
            let target_len = end.trivia.leading.len() - trailing_empty_lines;
            let last_idx = target_len - 1;
            for (i, trivia) in end.trivia.leading.iter().take(target_len).enumerate() {
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

        let if_trivia = &expr.if_first.trivia;
        self.format_node_after_keyword(ctx, if_trivia, &expr.if_first.cond);
        if !expr.if_first.body.is_empty() {
            self.break_line(ctx);
            self.format_exprs(&expr.if_first.body, ctx, true);
        }

        for elsif in &expr.elsifs {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.push_str("elsif");
            let elsif_trivia = &elsif.trivia;
            self.format_node_after_keyword(ctx, elsif_trivia, &elsif.cond);
            if !elsif.body.is_empty() {
                self.break_line(ctx);
                self.format_exprs(&elsif.body, ctx, true);
            }
        }

        if let Some(if_last) = &expr.if_last {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.push_str("else");
            self.write_trailing_comment(&if_last.trivia.trailing);
            self.indent();
            if !if_last.body.is_empty() {
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

        let if_trivia = &modifier.conditional.trivia;
        self.format_node_after_keyword(ctx, if_trivia, &modifier.conditional.cond);
        self.dedent();
    }

    // Handle comments like "if # foo\n #bar\n predicate"
    fn format_node_after_keyword(
        &mut self,
        ctx: &FormatContext,
        keyword_trivia: &Trivia,
        node: &Node,
    ) {
        if keyword_trivia.trailing.is_none() && node.trivia.leading.is_empty() {
            self.push(' ');
            self.format(node, ctx);
            self.write_trailing_comment(&node.trivia.trailing);
            self.indent();
        } else {
            self.write_trailing_comment(&keyword_trivia.trailing);
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(&node.trivia.leading, ctx, EmptyLineHandling::trim_start());
            self.put_indent();
            self.format(node, ctx);
            self.write_trailing_comment(&node.trivia.trailing);
        }
    }

    fn format_method_chain(&mut self, chain: &MethodChain, ctx: &FormatContext) {
        if let Some(recv) = &chain.receiver {
            self.format(recv, ctx);
            if recv.trivia.trailing.is_some() {
                self.write_trailing_comment(&recv.trivia.trailing);
            }
        }

        if chain.width.fits_in(self.remaining_width) {
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
                    if block.body.is_empty() {
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
                    self.write_leading_trivia(&call.trivia.leading, ctx, EmptyLineHandling::Skip);
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
                    if args.width.fits_in(self.remaining_width.saturating_sub(1)) {
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
                                &arg.trivia.leading,
                                ctx,
                                EmptyLineHandling::Trim {
                                    start: i == 0,
                                    end: false,
                                },
                            );
                            self.put_indent();
                            self.format(arg, ctx);
                            self.push(',');
                            self.write_trailing_comment(&arg.trivia.trailing);
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
                    if block.trivia.trailing.is_some()
                        || !block.body.width.fits_in(self.remaining_width)
                        || !block.was_flat
                    {
                        self.push_str(" do");
                        self.write_trailing_comment(&block.trivia.trailing);
                        self.indent();
                        if !block.body.is_empty() {
                            self.break_line(ctx);
                            self.format_exprs(&block.body, ctx, true);
                        }
                        self.dedent();
                        self.break_line(ctx);
                        self.put_indent();
                        self.push_str("end");
                    } else if block.body.is_empty() {
                        self.push_str(" {}");
                    } else {
                        self.push_str(" { ");
                        self.format_exprs(&block.body, ctx, false);
                        self.push_str(" }");
                    }
                }
                self.write_trailing_comment(&call.trivia.trailing);
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
        if value.width.fits_in(self.remaining_width) {
            self.push(' ');
            self.format(value, ctx);
        } else {
            self.break_line(ctx);
            self.indent();
            self.write_leading_trivia(
                &value.trivia.leading,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.put_indent();
            self.format(value, ctx);
            self.write_trailing_comment(&value.trivia.trailing);
            self.dedent();
        }
    }

    fn format_multi_assign_target(&mut self, multi: &MultiAssignTarget, ctx: &FormatContext) {
        if multi.width.fits_in(self.remaining_width) {
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
                    &target.trivia.leading,
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
                self.write_trailing_comment(&target.trivia.trailing);
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
        if array.width.fits_in(self.remaining_width) {
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
                    &element.trivia.leading,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.put_indent();
                self.format(element, ctx);
                self.push_str(array.separator());
                self.write_trailing_comment(&element.trivia.trailing);
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
        if hash.width.fits_in(self.remaining_width) {
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
                    &element.trivia.leading,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.put_indent();
                self.format(element, ctx);
                self.push(',');
                self.write_trailing_comment(&element.trivia.trailing);
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
        if khash.width.fits_in(self.remaining_width) {
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
                        &element.trivia.leading,
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
                self.write_trailing_comment(&element.trivia.trailing);
            }
        }
    }

    fn format_assoc(&mut self, assoc: &Assoc, ctx: &FormatContext) {
        self.format(&assoc.key, ctx);
        if assoc.value.width.fits_in(self.remaining_width) {
            if let Some(op) = &assoc.operator {
                self.push(' ');
                self.push_str(op);
            }
            if !assoc.value.width.is_empty() {
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
                &assoc.value.trivia.leading,
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

    fn write_leading_trivia(
        &mut self,
        trivia: &Vec<LineTrivia>,
        ctx: &FormatContext,
        emp_line_handling: EmptyLineHandling,
    ) {
        if trivia.is_empty() {
            return;
        }
        let last_idx = trivia.len() - 1;
        for (i, trivia) in trivia.iter().enumerate() {
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

    fn write_trailing_comment(&mut self, comment: &Option<Comment>) {
        if let Some(comment) = comment {
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
