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
    pub(crate) fn is_flat(&self) -> bool {
        matches!(self, Self::Flat(_))
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
    pub pos: Pos,
    pub kind: Kind,
    pub width: Width,
}

#[derive(Debug)]
pub(crate) enum Kind {
    Atom(String),
    Str(Str),
    DynStr(DynStr),
    HeredocOpening(HeredocOpening),
    Exprs(Exprs),
    IfExpr(IfExpr),
    Postmodifier(Postmodifier),
    MethodChain(MethodChain),
}

#[derive(Debug)]
pub(crate) struct Str {
    pub width: Width,
    pub opening: Option<String>,
    pub value: Vec<u8>,
    pub closing: Option<String>,
}

impl Str {
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
pub(crate) struct DynStr {
    pub width: Width,
    pub opening: Option<String>,
    pub parts: Vec<DynStrPart>,
    pub closing: Option<String>,
}

impl DynStr {
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
    Str(Str),
    DynStr(DynStr),
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
    Str(Str),
    Exprs(EmbeddedExprs),
}

#[derive(Debug)]
pub(crate) struct VirtualEnd {
    pub width: Width,
    pub decors: Decors,
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
    width: Width,
    id: String,
    indent_mode: HeredocIndentMode,
}

impl HeredocOpening {
    pub(crate) fn new(id: String, indent_mode: HeredocIndentMode) -> Self {
        let width = Width::Flat(id.len() + indent_mode.prefix_symbols().len());
        Self {
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
    pub pos: Pos,
    pub width: Width,
    pub cond: Box<Node>,
    pub body: Exprs,
}

impl Conditional {
    pub(crate) fn new(pos: Pos, cond: Node, body: Exprs) -> Self {
        let width = cond.width.add(&body.width);
        Self {
            pos,
            width,
            cond: Box::new(cond),
            body,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Else {
    pub pos: Pos,
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
    pub decors: Decors,
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
            decors: Decors::new(),
            call_op,
            name,
            args: None,
            block: None,
        }
    }

    pub(crate) fn set_decors(&mut self, decors: Decors) {
        self.width.append(&decors.width);
        self.decors = decors;
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
    pub decors: Decors,
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

    pub(crate) fn body_width(&self) -> Width {
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
pub(crate) struct Decors {
    pub leading: Vec<LineDecor>,
    pub trailing: Option<Comment>,
    pub width: Width,
}

impl Decors {
    pub(crate) fn new() -> Self {
        Self {
            leading: vec![],
            trailing: None,
            width: Width::Flat(0),
        }
    }

    pub(crate) fn append_leading(&mut self, decor: LineDecor) {
        if matches!(decor, LineDecor::Comment(_)) {
            self.width = Width::NotFlat;
        }
        self.leading.push(decor);
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
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
}

impl Formatter {
    fn format(&mut self, node: &Node, ctx: &FormatContext) {
        match &node.kind {
            Kind::Atom(value) => self.push_str(value),
            Kind::Str(str) => self.format_str(str),
            Kind::DynStr(dstr) => self.format_dyn_str(dstr, ctx),
            Kind::HeredocOpening(opening) => self.format_heredoc_opening(node.pos, opening),
            Kind::Exprs(exprs) => self.format_exprs(exprs, ctx, false),
            Kind::IfExpr(expr) => self.format_if_expr(expr, ctx),
            Kind::Postmodifier(modifier) => self.format_postmodifier(modifier, ctx),
            Kind::MethodChain(chain) => self.format_method_chain(chain, ctx),
        }
    }

    fn format_str(&mut self, str: &Str) {
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

    fn format_dyn_str(&mut self, dstr: &DynStr, ctx: &FormatContext) {
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
            self.push_str(closing);
        }
    }

    fn format_embedded_exprs(&mut self, embedded: &EmbeddedExprs, ctx: &FormatContext) {
        self.push_str(&embedded.opening);

        if embedded.exprs.width.is_flat() {
            self.format_exprs(&embedded.exprs, ctx, false);
        } else {
            self.break_line(ctx);
            self.indent();
            self.format_exprs(&embedded.exprs, ctx, false);
            self.break_line(ctx);
            self.dedent();
        }

        self.push_str(&embedded.closing);
    }

    fn format_heredoc_opening(&mut self, pos: Pos, opening: &HeredocOpening) {
        self.push_str(opening.indent_mode.prefix_symbols());
        self.push_str(&opening.id);
        self.heredoc_queue.push_back(pos);
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
            let decors = ctx.decor_store.get(&n.pos);
            self.write_leading_decors(
                &decors.leading,
                ctx,
                EmptyLineHandling::Trim {
                    start: i == 0,
                    end: false,
                },
            );
            self.put_indent();
            self.format(n, ctx);
            self.write_trailing_comment(&decors.trailing);
        }
        self.write_decors_at_virtual_end(
            ctx,
            &exprs.virtual_end,
            !exprs.nodes.is_empty(),
            exprs.nodes.is_empty(),
        );
    }

    fn write_decors_at_virtual_end(
        &mut self,
        ctx: &FormatContext,
        end: &Option<VirtualEnd>,
        break_first: bool,
        trim_start: bool,
    ) {
        if let Some(end) = end {
            let mut trailing_empty_lines = 0;
            for decor in end.decors.leading.iter().rev() {
                match decor {
                    LineDecor::EmptyLine => {
                        trailing_empty_lines += 1;
                    }
                    LineDecor::Comment(_) => {
                        break;
                    }
                }
            }
            if trailing_empty_lines == end.decors.leading.len() {
                return;
            }

            if break_first {
                self.break_line(ctx);
            }
            let target_len = end.decors.leading.len() - trailing_empty_lines;
            let last_idx = target_len - 1;
            for (i, decor) in end.decors.leading.iter().take(target_len).enumerate() {
                match decor {
                    LineDecor::EmptyLine => {
                        if !(trim_start && i == 0) || i == last_idx {
                            self.break_line(ctx);
                        }
                    }
                    LineDecor::Comment(comment) => {
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

        let if_decors = ctx.decor_store.get(&expr.if_first.pos);
        let cond_decors = ctx.decor_store.get(&expr.if_first.cond.pos);
        self.format_decors_in_keyword_gap(ctx, if_decors, cond_decors, |self_| {
            self_.put_indent();
            self_.format(&expr.if_first.cond, ctx);
        });
        if !expr.if_first.body.is_empty() {
            self.break_line(ctx);
            self.format_exprs(&expr.if_first.body, ctx, true);
        }

        for elsif in &expr.elsifs {
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.push_str("elsif");
            let elsif_decors = ctx.decor_store.get(&elsif.pos);
            let cond_decors = ctx.decor_store.get(&elsif.cond.pos);
            self.format_decors_in_keyword_gap(ctx, elsif_decors, cond_decors, |self_| {
                self_.put_indent();
                self_.format(&elsif.cond, ctx);
            });
            if !elsif.body.is_empty() {
                self.break_line(ctx);
                self.format_exprs(&elsif.body, ctx, true);
            }
        }

        if let Some(if_last) = &expr.if_last {
            let else_decors = ctx.decor_store.get(&if_last.pos);
            self.break_line(ctx);
            self.dedent();
            self.put_indent();
            self.push_str("else");
            self.write_trailing_comment(&else_decors.trailing);
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

        let if_decors = ctx.decor_store.get(&modifier.conditional.pos);
        let cond_decors = ctx.decor_store.get(&modifier.conditional.cond.pos);
        self.format_decors_in_keyword_gap(ctx, if_decors, cond_decors, |self_| {
            self_.put_indent();
            self_.format(&modifier.conditional.cond, ctx);
        });
        self.dedent();
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
            self.push(' ');
            next_node(self);
            self.write_trailing_comment(&next_decors.trailing);
            self.indent();
        } else {
            self.write_trailing_comment(&keyword_decors.trailing);
            self.indent();
            self.break_line(ctx);
            self.write_leading_decors(&next_decors.leading, ctx, EmptyLineHandling::trim_start());
            next_node(self);
            self.write_trailing_comment(&next_decors.trailing);
        }
    }

    fn format_method_chain(&mut self, chain: &MethodChain, ctx: &FormatContext) {
        if let Some(recv) = &chain.receiver {
            let recv_decor = ctx.decor_store.get(&recv.pos);
            self.format(recv, ctx);
            if recv_decor.trailing.is_some() {
                self.write_trailing_comment(&recv_decor.trailing);
            }
        }

        if chain.width.is_flat() {
            for call in chain.calls.iter() {
                if let Some(call_op) = &call.call_op {
                    self.push_str(call_op);
                }
                self.push_str(&call.name);

                if let Some(args) = &call.args {
                    self.push('(');
                    for (i, arg) in args.nodes.iter().enumerate() {
                        if i > 0 {
                            self.push_str(", ");
                        }
                        self.format(arg, ctx);
                    }
                    self.push(')');
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
                    self.write_leading_decors(&call.decors.leading, ctx, EmptyLineHandling::Skip);
                    self.put_indent();
                    self.push_str(call_op);
                }
                self.push_str(&call.name);

                if let Some(args) = &call.args {
                    self.push('(');
                    if args.width.is_flat() {
                        for (i, arg) in args.nodes.iter().enumerate() {
                            if i > 0 {
                                self.push_str(", ");
                            }
                            self.format(arg, ctx);
                        }
                    } else {
                        self.indent();
                        for (i, arg) in args.nodes.iter().enumerate() {
                            let decors = ctx.decor_store.get(&arg.pos);
                            self.break_line(ctx);
                            self.write_leading_decors(
                                &decors.leading,
                                ctx,
                                EmptyLineHandling::Trim {
                                    start: i == 0,
                                    end: false,
                                },
                            );
                            self.put_indent();
                            self.format(arg, ctx);
                            self.push(',');
                            self.write_trailing_comment(&decors.trailing);
                        }
                        self.write_decors_at_virtual_end(
                            ctx,
                            &args.virtual_end,
                            true,
                            args.nodes.is_empty(),
                        );
                        self.dedent();
                        self.break_line(ctx);
                        self.put_indent();
                    }
                    self.push(')');
                }

                if let Some(block) = &call.block {
                    if block.decors.trailing.is_some()
                        || !block.body.width.is_flat()
                        || !block.was_flat
                    {
                        self.push_str(" do");
                        self.write_trailing_comment(&block.decors.trailing);
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
                self.write_trailing_comment(&call.decors.trailing);
            }
            if indented {
                self.dedent();
            }
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
    }

    fn push_str(&mut self, str: &str) {
        self.buffer.push_str(str);
    }

    fn indent(&mut self) {
        self.indent += 2;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }

    fn break_line(&mut self, ctx: &FormatContext) {
        self.push('\n');
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
