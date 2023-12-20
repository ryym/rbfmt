use std::{
    collections::{HashMap, VecDeque},
    mem,
};

pub(crate) fn format(node: Node, decor_store: DecorStore, heredoc_map: HeredocMap) -> String {
    let ctx = FormatContext {
        decor_store,
        heredoc_map,
    };
    let mut formatter = Formatter {
        buffer: String::new(),
        heredoc_queue: VecDeque::new(),
        indent: 0,
    };
    formatter.format(&node, &ctx);
    if formatter.buffer.is_empty() {
        formatter.buffer
    } else {
        formatter.break_line(&ctx);
        formatter.buffer.trim_start().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos(pub usize);

impl Pos {
    pub(crate) fn none() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Width {
    Flat(usize),
    NotFlat,
}

impl Width {
    pub(crate) fn append(&mut self, other: &Self) {
        let width = match (&self, other) {
            (Self::Flat(w1), Self::Flat(w2)) => Self::Flat(*w1 + w2),
            _ => Self::NotFlat,
        };
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
    pub pos: Pos,
    pub kind: Kind,
    pub width: Width,
}

#[derive(Debug)]
pub(crate) enum Kind {
    Atom(String),
    Str(Str),
    DynStr(DynStr),
    HeredocOpening,
    Exprs(Exprs),
    IfExpr(IfExpr),
    MethodChain(MethodChain),
}

#[derive(Debug)]
pub(crate) struct Str {
    pub opening: Option<String>,
    pub value: Vec<u8>,
    pub closing: Option<String>,
}

impl Str {
    pub(crate) fn len(&self) -> usize {
        let open = self.opening.as_ref().map_or(0, |s| s.len());
        let close = self.closing.as_ref().map_or(0, |s| s.len());
        self.value.len() + open + close
    }
}

#[derive(Debug)]
pub(crate) struct DynStr {
    pub opening: Option<String>,
    pub parts: Vec<DynStrPart>,
    pub closing: Option<String>,
}

#[derive(Debug)]
pub(crate) enum DynStrPart {
    Str(Str),
    DynStr(DynStr),
    Exprs(EmbeddedExprs),
}

#[derive(Debug)]
pub(crate) struct EmbeddedExprs {
    pub opening: String,
    pub exprs: Exprs,
    pub closing: String,
}

#[derive(Debug)]
pub(crate) struct Heredoc {
    pub id: String,
    pub indent_mode: HeredocIndentMode,
    pub parts: Vec<HeredocPart>,
}

#[derive(Debug)]
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
    Str(Str),
    Exprs(EmbeddedExprs),
}

#[derive(Debug)]
pub(crate) struct Exprs {
    nodes: Vec<Node>,
    width: Width,
    phantom_end_pos: Option<Pos>,
}

impl Exprs {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![],
            width: Width::Flat(0),
            phantom_end_pos: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        if self.nodes.is_empty() && !matches!(node.kind, Kind::HeredocOpening) {
            self.width = node.width;
        } else {
            self.width = Width::NotFlat;
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_end_decors_pos(&mut self, pos: Pos) {
        self.phantom_end_pos = Some(pos);
        self.width = Width::NotFlat;
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
}

#[derive(Debug)]
pub(crate) struct Conditional {
    pub pos: Pos,
    pub cond: Box<Node>,
    pub body: Exprs,
}

#[derive(Debug)]
pub(crate) struct Else {
    pub pos: Pos,
    pub body: Exprs,
}

#[derive(Debug)]
pub(crate) struct MethodCall {
    pub pos: Pos,
    pub width: Width,
    pub chain_type: ChainType,
    pub name: String,
    pub args: Vec<Node>,
    pub block: Option<MethodBlock>,
}

#[derive(Debug)]
pub(crate) enum ChainType {
    Normal,
    SafeNav,
}

impl ChainType {
    pub fn dot(&self) -> &'static str {
        match self {
            Self::Normal => ".",
            Self::SafeNav => "&.",
        }
    }
}

#[derive(Debug)]
pub(crate) struct MethodBlock {
    pub pos: Pos,
    // pub args
    pub body: Exprs,
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

    pub(crate) fn width(&self) -> Width {
        self.width
    }
}

#[derive(Debug)]
pub(crate) struct DecorStore {
    map: HashMap<Pos, DecorSet>,
    empty_decors: DecorSet,
}

impl DecorStore {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
            empty_decors: DecorSet::default(),
        }
    }

    pub(crate) fn get(&self, pos: &Pos) -> &DecorSet {
        self.map.get(pos).unwrap_or(&self.empty_decors)
    }

    pub(crate) fn append_leading_decors(&mut self, pos: Pos, mut decors: Vec<LineDecor>) {
        match self.map.get_mut(&pos) {
            Some(d) => {
                d.leading.append(&mut decors);
            }
            None => {
                let d = DecorSet {
                    leading: decors,
                    trailing: None,
                };
                self.map.insert(pos, d);
            }
        }
    }

    pub(crate) fn set_trailing_comment(&mut self, pos: Pos, comment: Comment) {
        match self.map.get_mut(&pos) {
            Some(d) => {
                d.trailing = Some(comment);
            }
            None => {
                let d = DecorSet {
                    leading: vec![],
                    trailing: Some(comment),
                };
                self.map.insert(pos, d);
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct DecorSet {
    pub leading: Vec<LineDecor>,
    pub trailing: Option<Comment>,
}

#[derive(Debug)]
pub(crate) struct Comment {
    pub value: String,
}

#[derive(Debug)]
pub(crate) enum LineDecor {
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
struct FormatContext {
    decor_store: DecorStore,
    heredoc_map: HeredocMap,
}

#[derive(Debug)]
struct Formatter {
    buffer: String,
    heredoc_queue: VecDeque<Pos>,
    indent: usize,
}

impl Formatter {
    fn format(&mut self, node: &Node, ctx: &FormatContext) {
        match &node.kind {
            Kind::Atom(value) => self.buffer.push_str(value),
            Kind::Str(str) => self.format_str(str),
            Kind::DynStr(dstr) => self.format_dyn_str(dstr, ctx),
            Kind::HeredocOpening => self.format_heredoc_opening(node.pos, ctx),
            Kind::Exprs(exprs) => {
                self.format_exprs(exprs, ctx, false);
            }
            Kind::IfExpr(expr) => self.format_if_expr(expr, ctx),
            Kind::MethodChain(chain) => self.format_method_chain(chain, ctx),
        }
    }

    fn format_str(&mut self, str: &Str) {
        // Ignore non-UTF8 source code for now.
        let value = String::from_utf8_lossy(&str.value);
        if let Some(opening) = &str.opening {
            self.buffer.push_str(opening);
        }
        self.buffer.push_str(&value);
        if let Some(closing) = &str.closing {
            self.buffer.push_str(closing);
        }
    }

    fn format_dyn_str(&mut self, dstr: &DynStr, ctx: &FormatContext) {
        if let Some(opening) = &dstr.opening {
            self.buffer.push_str(opening);
        }
        let mut divided = false;
        for part in &dstr.parts {
            if divided {
                self.buffer.push(' ');
            }
            match part {
                DynStrPart::Str(str) => {
                    divided = str.opening.is_some();
                    self.format_str(str);
                }
                DynStrPart::DynStr(dstr) => {
                    divided = true;
                    self.format_dyn_str(dstr, ctx);
                }
                DynStrPart::Exprs(embedded) => {
                    self.format_embedded_exprs(embedded, ctx);
                }
            }
        }
        if let Some(closing) = &dstr.closing {
            self.buffer.push_str(closing);
        }
    }

    fn format_embedded_exprs(&mut self, embedded: &EmbeddedExprs, ctx: &FormatContext) {
        self.buffer.push_str(&embedded.opening);

        self.indent();
        let is_block = self.format_exprs(&embedded.exprs, ctx, false);
        if is_block {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
        } else {
            self.dedent();
        }

        self.buffer.push_str(&embedded.closing);
    }

    fn format_heredoc_opening(&mut self, pos: Pos, ctx: &FormatContext) {
        let heredoc = ctx.heredoc_map.get(&pos).expect("heredoc must exist");
        self.buffer.push_str(heredoc.indent_mode.prefix_symbols());
        self.buffer.push_str(&heredoc.id);
        self.heredoc_queue.push_back(pos);
    }

    fn format_exprs(&mut self, exprs: &Exprs, ctx: &FormatContext, block_always: bool) -> bool {
        if exprs.can_be_flat() && !block_always {
            if let Some(node) = exprs.nodes.get(0) {
                self.format(node, ctx);
            }
            return false;
        }
        for (i, n) in exprs.nodes.iter().enumerate() {
            let decors = ctx.decor_store.get(&n.pos);
            self.write_leading_decors(
                &decors.leading,
                ctx,
                EmptyLineHandling::Trim {
                    start: i == 0,
                    end: false,
                },
            );
            self.break_line(ctx);
            self.put_indent();
            self.format(n, ctx);
            self.write_trailing_comment(&decors.trailing);
        }
        if let Some(end_pos) = &exprs.phantom_end_pos {
            let end_decors = ctx.decor_store.get(end_pos);
            if !end_decors.leading.is_empty() {
                self.write_leading_decors(
                    &end_decors.leading,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: exprs.nodes.is_empty(),
                        end: true,
                    },
                );
            }
        }
        true
    }

    fn format_if_expr(&mut self, expr: &IfExpr, ctx: &FormatContext) {
        if expr.is_if {
            self.buffer.push_str("if");
        } else {
            self.buffer.push_str("unless");
        }
        let if_decors = ctx.decor_store.get(&expr.if_first.pos);
        let cond_decors = ctx.decor_store.get(&expr.if_first.cond.pos);
        self.format_decors_in_keyword_gap(ctx, if_decors, cond_decors, |self_| {
            self_.format(&expr.if_first.cond, ctx);
        });
        self.format_exprs(&expr.if_first.body, ctx, true);

        for elsif in &expr.elsifs {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.buffer.push_str("elsif");
            let elsif_decors = ctx.decor_store.get(&elsif.pos);
            let cond_decors = ctx.decor_store.get(&elsif.cond.pos);
            self.format_decors_in_keyword_gap(ctx, elsif_decors, cond_decors, |self_| {
                self_.format(&elsif.cond, ctx);
            });
            self.format_exprs(&elsif.body, ctx, true);
        }

        if let Some(if_last) = &expr.if_last {
            let else_decors = ctx.decor_store.get(&if_last.pos);
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.buffer.push_str("else");
            self.write_trailing_comment(&else_decors.trailing);
            self.indent();
            self.format_exprs(&if_last.body, ctx, true);
        }

        self.break_line(ctx);
        self.dedent();
        self.put_indent();
        self.buffer.push_str("end");
    }

    // Handle comments like "if # foo\n #bar\n predicate"
    fn format_decors_in_keyword_gap(
        &mut self,
        ctx: &FormatContext,
        keyword_decors: &DecorSet,
        next_decors: &DecorSet,
        next_node: impl FnOnce(&mut Self),
    ) {
        if keyword_decors.trailing.is_none() && next_decors.leading.is_empty() {
            self.buffer.push(' ');
            next_node(self);
            self.write_trailing_comment(&next_decors.trailing);
            self.indent();
        } else {
            self.write_trailing_comment(&keyword_decors.trailing);
            self.indent();
            self.write_leading_decors(&next_decors.leading, ctx, EmptyLineHandling::trim_start());
            self.break_line(ctx);
            self.put_indent();
            next_node(self);
            self.write_trailing_comment(&next_decors.trailing);
        }
    }

    fn format_method_chain(&mut self, chain: &MethodChain, ctx: &FormatContext) {
        let mut has_receiver = false;
        if let Some(recv) = &chain.receiver {
            let recv_decor = ctx.decor_store.get(&recv.pos);
            self.format(recv, ctx);
            if recv_decor.trailing.is_some() {
                self.write_trailing_comment(&recv_decor.trailing);
                self.break_line(ctx);
                self.put_indent();
            }
            has_receiver = true;
        }

        let mut is_flat = true;
        for (i, call) in chain.calls.iter().enumerate() {
            let call_decor = ctx.decor_store.get(&call.pos);
            if !call_decor.leading.is_empty() {
                if is_flat {
                    self.indent();
                    is_flat = false;
                }
                self.write_leading_decors(&call_decor.leading, ctx, EmptyLineHandling::Skip);
                self.break_line(ctx);
                self.put_indent();
            }
            has_receiver = has_receiver || i > 0;
            if has_receiver {
                self.buffer.push_str(call.chain_type.dot());
            }
            self.buffer.push_str(&call.name);

            if !call.args.is_empty() {
                self.buffer.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.buffer.push_str(", ");
                    }
                    self.format(arg, ctx);
                }
                self.buffer.push(')');
            }
            if let Some(block) = &call.block {
                let block_decors = ctx.decor_store.get(&block.pos);
                if block.body.is_empty() && block_decors.trailing.is_none() {
                    self.buffer.push_str(" {}");
                } else {
                    self.buffer.push_str(" do");
                    self.write_trailing_comment(&block_decors.trailing);
                    self.indent();
                    self.format_exprs(&block.body, ctx, true);
                    self.dedent();
                    self.break_line(ctx);
                    self.put_indent();
                    self.buffer.push_str("end");
                }
            }

            self.write_trailing_comment(&call_decor.trailing);
        }
        if !is_flat {
            self.dedent();
        }
    }

    fn write_leading_decors(
        &mut self,
        decors: &Vec<LineDecor>,
        ctx: &FormatContext,
        emp_line_handling: EmptyLineHandling,
    ) {
        if decors.is_empty() {
            return;
        }
        let last_idx = decors.len() - 1;
        for (i, decor) in decors.iter().enumerate() {
            match decor {
                LineDecor::EmptyLine => {
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
                LineDecor::Comment(comment) => {
                    self.break_line(ctx);
                    self.put_indent();
                    self.buffer.push_str(&comment.value);
                }
            }
        }
    }

    fn write_trailing_comment(&mut self, comment: &Option<Comment>) {
        if let Some(comment) = comment {
            self.buffer.push(' ');
            self.buffer.push_str(&comment.value);
        }
    }

    fn indent(&mut self) {
        self.indent += 2;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }

    fn break_line(&mut self, ctx: &FormatContext) {
        self.buffer.push('\n');
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
                            self.buffer.push_str(&value);
                        }
                        HeredocPart::Exprs(embedded) => {
                            self.format_embedded_exprs(embedded, ctx);
                        }
                    }
                }
                if matches!(heredoc.indent_mode, HeredocIndentMode::EndIndented) {
                    self.put_indent();
                }
                self.buffer.push_str(&heredoc.id);
            }
            HeredocIndentMode::AllIndented => {
                for part in &heredoc.parts {
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            self.buffer.push_str(&value);
                        }
                        HeredocPart::Exprs(embedded) => {
                            self.format_embedded_exprs(embedded, ctx);
                        }
                    }
                }
                self.put_indent();
                self.buffer.push_str(&heredoc.id);
            }
        }
        self.buffer.push('\n');
    }

    fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
    }
}
