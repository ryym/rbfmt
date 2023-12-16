use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos(pub usize);

impl Pos {
    pub(crate) fn none() -> Self {
        Self(0)
    }
}

#[derive(Debug)]
pub(crate) struct Node {
    pub pos: Pos,
    pub kind: Kind,
}

impl Node {
    pub(crate) fn new(pos: Pos, kind: Kind) -> Self {
        Self { pos, kind }
    }
}

#[derive(Debug)]
pub(crate) enum Kind {
    Atom(String),
    Exprs(Exprs),
    EndDecors,
    IfExpr(IfExpr),
    MethodChain(MethodChain),
}

impl Kind {
    fn is_end_decors(&self) -> bool {
        matches!(self, Self::EndDecors)
    }
}

#[derive(Debug)]
pub(crate) struct Exprs(pub Vec<Node>);

#[derive(Debug)]
pub(crate) struct IfExpr {
    pub is_unless: bool,
    pub if_first: IfPart,
    pub elsifs: Vec<Elsif>,
    pub if_last: Option<Else>,
    pub end_pos: Pos,
}

impl IfExpr {
    pub(crate) fn new(is_unless: bool, if_first: IfPart) -> Self {
        Self {
            is_unless,
            if_first,
            elsifs: vec![],
            if_last: None,
            end_pos: Pos::none(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Elsif {
    pub pos: Pos,
    pub part: IfPart,
}

#[derive(Debug)]
pub(crate) struct Else {
    pub pos: Pos,
    pub body: Exprs,
}

#[derive(Debug)]
pub(crate) struct IfPart {
    pub cond: Box<Node>,
    pub body: Exprs,
}

impl IfPart {
    pub(crate) fn new(cond: Node, body: Exprs) -> Self {
        Self {
            cond: Box::new(cond),
            body,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MethodCall {
    pub pos: Pos,
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
    pub receiver: Option<Box<Node>>,
    pub calls: Vec<MethodCall>,
}

#[derive(Debug)]
pub(crate) struct DecorStore {
    map: HashMap<Pos, DecorSet>,
}

impl DecorStore {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn consume(&mut self, pos: Pos) -> DecorSet {
        self.map.remove(&pos).unwrap_or_default()
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

pub(crate) fn format(node: Node, decor_store: DecorStore) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
        decor_store,
        indent: 0,
    };
    formatter.format(node);
    if formatter.buffer.is_empty() {
        formatter.buffer
    } else {
        formatter.buffer.push('\n');
        formatter.buffer.trim_start().to_string()
    }
}

#[derive(Debug)]
struct Formatter {
    buffer: String,
    decor_store: DecorStore,
    indent: usize,
}

impl Formatter {
    fn format(&mut self, node: Node) {
        match node.kind {
            Kind::Atom(value) => self.buffer.push_str(&value),
            Kind::Exprs(exprs) => self.format_exprs(exprs),
            Kind::EndDecors => unreachable!("end decors unexpectedly rendered"),
            Kind::IfExpr(expr) => self.format_if_expr(expr),
            Kind::MethodChain(chain) => self.format_method_chain(chain),
        }
    }

    fn format_exprs(&mut self, exprs: Exprs) {
        let Exprs(nodes) = exprs;
        if nodes.is_empty() {
            return;
        }
        for (i, n) in nodes.into_iter().enumerate() {
            let decors = self.decor_store.consume(n.pos);
            self.write_leading_decors(decors.leading, i == 0, n.kind.is_end_decors());
            if !matches!(n.kind, Kind::EndDecors) {
                self.break_line();
                self.put_indent();
                self.format(n);
            }
            self.write_trailing_comment(decors.trailing);
        }
    }

    fn format_if_expr(&mut self, expr: IfExpr) {
        if expr.is_unless {
            self.buffer.push_str("unless ");
        } else {
            self.buffer.push_str("if ");
        }
        let cond_decors = self.decor_store.consume(expr.if_first.cond.pos);
        self.format(*expr.if_first.cond);
        self.write_trailing_comment(cond_decors.trailing);
        self.indent();
        self.format_exprs(expr.if_first.body);

        for elsif in expr.elsifs {
            let elsif_decors = self.decor_store.consume(elsif.pos);
            self.break_line();
            self.dedent();
            self.put_indent();
            self.buffer.push_str("elsif");
            if elsif_decors.trailing.is_some() {
                self.write_trailing_comment(elsif_decors.trailing);
            } else {
                self.buffer.push(' ');
            }
            let cond_decors = self.decor_store.consume(elsif.part.cond.pos);
            if cond_decors.leading.is_empty() {
                self.format(*elsif.part.cond);
                self.write_trailing_comment(cond_decors.trailing);
                self.indent();
            } else {
                self.indent();
                self.write_leading_decors(cond_decors.leading, true, false);
                self.break_line();
                self.put_indent();
                self.format(*elsif.part.cond);
                self.write_trailing_comment(cond_decors.trailing);
            }
            self.format_exprs(elsif.part.body);
        }

        if let Some(if_last) = expr.if_last {
            let else_decors = self.decor_store.consume(if_last.pos);
            self.break_line();
            self.dedent();
            self.put_indent();
            self.buffer.push_str("else");
            self.write_trailing_comment(else_decors.trailing);
            self.indent();
            self.format_exprs(if_last.body);
        }

        let end_decors = self.decor_store.consume(expr.end_pos);
        self.write_leading_decors(end_decors.leading, false, true);
        self.break_line();
        self.dedent();
        self.put_indent();
        self.buffer.push_str("end");
        self.write_trailing_comment(end_decors.trailing);
    }

    fn format_method_chain(&mut self, chain: MethodChain) {
        let mut has_receiver = false;
        if let Some(recv) = chain.receiver {
            let recv_decor = self.decor_store.consume(recv.pos);
            self.format(*recv);
            if recv_decor.trailing.is_some() {
                self.write_trailing_comment(recv_decor.trailing);
                self.break_line();
                self.put_indent();
            }
            has_receiver = true;
        }

        let mut is_flat = true;
        for (i, call) in chain.calls.into_iter().enumerate() {
            let call_decor = self.decor_store.consume(call.pos);
            let call_leading = call_decor
                .leading
                .into_iter()
                .filter(|d| matches!(d, LineDecor::Comment(_)))
                .collect::<Vec<_>>();
            if !call_leading.is_empty() {
                if is_flat {
                    self.indent();
                    is_flat = false;
                }
                self.write_leading_decors(call_leading, true, true);
                self.break_line();
                self.put_indent();
            }
            has_receiver = has_receiver || i > 0;
            if has_receiver {
                self.buffer.push_str(call.chain_type.dot());
            }
            self.buffer.push_str(&call.name);

            if !call.args.is_empty() {
                self.buffer.push('(');
                for (i, arg) in call.args.into_iter().enumerate() {
                    if i > 0 {
                        self.buffer.push_str(", ");
                    }
                    self.format(arg);
                }
                self.buffer.push(')');
            }
            if let Some(block) = call.block {
                if block.body.0.is_empty() {
                    self.buffer.push_str(" {}");
                } else {
                    let block_decors = self.decor_store.consume(block.pos);
                    self.buffer.push_str(" do");
                    self.write_trailing_comment(block_decors.trailing);
                    self.indent();
                    self.format_exprs(block.body);
                    self.dedent();
                    self.break_line();
                    self.put_indent();
                    self.buffer.push_str("end");
                }
            }

            self.write_trailing_comment(call_decor.trailing);
        }
        if !is_flat {
            self.dedent();
        }
    }

    fn write_leading_decors(&mut self, decors: Vec<LineDecor>, trim_start: bool, trim_end: bool) {
        if decors.is_empty() {
            return;
        }
        let last_idx = decors.len() - 1;
        for (i, decor) in decors.into_iter().enumerate() {
            match decor {
                LineDecor::EmptyLine => {
                    if (!trim_start || 0 < i) && (!trim_end || i < last_idx) {
                        self.break_line();
                    }
                }
                LineDecor::Comment(comment) => {
                    self.break_line();
                    self.put_indent();
                    self.buffer.push_str(&comment.value);
                }
            }
        }
    }

    fn write_trailing_comment(&mut self, comment: Option<Comment>) {
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

    fn break_line(&mut self) {
        self.buffer.push('\n');
    }

    fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
    }
}
