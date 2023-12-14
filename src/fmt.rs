use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Node {
    pub id: usize,
    pub kind: Kind,
}

impl Node {
    pub(crate) fn new(id: usize, kind: Kind) -> Self {
        Self { id, kind }
    }
}

#[derive(Debug)]
pub(crate) enum Kind {
    Atom(String),
    Exprs(Vec<Node>),
    EndDecors,
    IfExpr(IfExpr),
}

impl Kind {
    fn is_end_decors(&self) -> bool {
        matches!(self, Self::EndDecors)
    }
}

#[derive(Debug)]
pub(crate) struct IfExpr {
    pub if_first: IfPart,
    pub elsifs: Vec<IfPart>,
    pub if_last: Option<Box<Node>>,
}

#[derive(Debug)]
pub(crate) struct IfPart {
    pub cond: Box<Node>,
    pub body: Box<Node>,
}

impl IfPart {
    pub(crate) fn new(cond: Node, body: Node) -> Self {
        Self {
            cond: Box::new(cond),
            body: Box::new(body),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DecorStore {
    map: HashMap<usize, DecorSet>,
}

impl DecorStore {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn consume(&mut self, node_id: usize) -> (Vec<LineDecor>, Option<Comment>) {
        if let Some(decors) = self.map.remove(&node_id) {
            (decors.leading, decors.trailing)
        } else {
            (vec![], None)
        }
    }

    pub(crate) fn append_leading_decors(&mut self, node_id: usize, mut decors: Vec<LineDecor>) {
        match self.map.get_mut(&node_id) {
            Some(d) => {
                d.leading.append(&mut decors);
            }
            None => {
                let d = DecorSet {
                    leading: decors,
                    trailing: None,
                };
                self.map.insert(node_id, d);
            }
        }
    }

    pub(crate) fn set_trailing_comment(&mut self, node_id: usize, comment: Comment) {
        match self.map.get_mut(&node_id) {
            Some(d) => {
                d.trailing = Some(comment);
            }
            None => {
                let d = DecorSet {
                    leading: vec![],
                    trailing: Some(comment),
                };
                self.map.insert(node_id, d);
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
    if !formatter.buffer.is_empty() {
        formatter.buffer.push('\n');
    }
    formatter.buffer
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
            Kind::Atom(value) => {
                self.buffer.push_str(&value);
            }
            Kind::Exprs(nodes) => {
                if nodes.is_empty() {
                    return;
                }
                for (i, n) in nodes.into_iter().enumerate() {
                    if i > 0 {
                        self.break_line();
                    }
                    let (leading_decors, trailing_comment) = self.decor_store.consume(n.id);
                    self.write_leading_decors(leading_decors, i == 0, n.kind.is_end_decors());
                    self.format(n);
                    self.write_trailing_comment(trailing_comment);
                }
            }
            Kind::EndDecors => {
                let line_len = self.indent + 1; // newline
                if line_len < self.buffer.len() {
                    self.buffer.truncate(self.buffer.len() - line_len);
                }
            }
            Kind::IfExpr(node) => {
                self.buffer.push_str("if ");
                let (_, cond_trailing) = self.decor_store.consume(node.if_first.cond.id);
                self.format(*node.if_first.cond);
                self.write_trailing_comment(cond_trailing);
                self.indent();
                self.break_line();
                self.format(*node.if_first.body);
                self.dedent();
                self.break_line();

                for elsif in node.elsifs {
                    self.buffer.push_str("elsif ");
                    // todo: write decors around elsif
                    self.format(*elsif.cond);
                    self.indent();
                    self.break_line();
                    self.format(*elsif.body);
                    self.dedent();
                    self.break_line();
                }

                if let Some(if_last) = node.if_last {
                    self.buffer.push_str("else");
                    self.indent();
                    self.break_line();
                    // todo: write decors around else
                    self.format(*if_last);
                    self.dedent();
                    self.break_line();
                }

                self.buffer.push_str("end");
            }
        }
    }

    fn write_leading_decors(&mut self, decors: Vec<LineDecor>, trim_start: bool, trim_end: bool) {
        if decors.is_empty() {
            return;
        }
        // NOTE: If decors is not empty, the result ends with a newline.
        let last_idx = decors.len() - 1;
        for (i, decor) in decors.into_iter().enumerate() {
            match decor {
                LineDecor::EmptyLine => {
                    if (!trim_start || 0 < i) && (!trim_end || i < last_idx) {
                        self.break_line();
                    }
                }
                LineDecor::Comment(comment) => {
                    self.buffer.push_str(&comment.value);
                    self.break_line();
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
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
    }
}
