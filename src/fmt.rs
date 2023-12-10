pub(crate) fn format(node: Node) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
        indent: 0,
    };
    formatter.format(node);
    formatter.buffer.push('\n');
    formatter.buffer.trim_start().to_string()
}

#[derive(Debug)]
pub(crate) struct Trivia {
    pub last_trailing_comment: Option<Comment>,
    pub leading_trivia: Vec<TriviaNode>,
}

impl Trivia {
    pub(crate) fn new() -> Self {
        Self {
            last_trailing_comment: None,
            leading_trivia: vec![],
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.last_trailing_comment.is_none() && self.leading_trivia.is_empty()
    }
}

#[derive(Debug)]
pub(crate) enum TriviaNode {
    EmptyLine,
    LineComment(Comment),
}

#[derive(Debug)]
pub(crate) struct Comment {
    pub value: String,
}

#[derive(Debug)]
pub(crate) enum Node {
    Nil(Option<Trivia>),
    Boolean(Option<Trivia>, Boolean),
    Number(Option<Trivia>, Number),
    Identifier(Option<Trivia>, Identifier),
    Statements(Statements),
    IfExpr(Option<Trivia>, IfExpr),
    None(Trivia),
}

impl Node {
    fn is_none(&self) -> bool {
        matches!(self, Self::None(_))
    }

    fn trivia(&self) -> Option<&Trivia> {
        match self {
            Self::Nil(t)
            | Self::Boolean(t, _)
            | Self::Number(t, _)
            | Self::Identifier(t, _)
            | Self::IfExpr(t, _) => t.as_ref(),
            Self::None(t) => Some(t),
            Self::Statements(_) => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Boolean {
    pub is_true: bool,
}

#[derive(Debug)]
pub(crate) struct Number {
    pub value: String,
}

#[derive(Debug)]
pub(crate) struct Identifier {
    pub name: String,
}

#[derive(Debug)]
pub(crate) struct Statements {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub(crate) struct IfExpr {
    pub cond: Box<Node>,
    pub body: Statements,
}

#[derive(Debug)]
enum Indent {
    Keep,
    Incr,
    Decr,
}

#[derive(Debug)]
struct Formatter {
    buffer: String,
    indent: usize,
}

impl Formatter {
    fn format(&mut self, node: Node) {
        match node {
            Node::Nil(_) => {
                self.buffer.push_str("nil");
            }
            Node::Boolean(_, node) => {
                let value = if node.is_true { "true" } else { "false" };
                self.buffer.push_str(value);
            }
            Node::Number(_, node) => {
                self.buffer.push_str(&node.value);
            }
            Node::Identifier(_, node) => {
                self.buffer.push_str(&node.name);
            }
            Node::Statements(node) => {
                self.format_statements(node);
            }
            Node::IfExpr(_, node) => {
                self.buffer.push_str("if ");
                self.format(*node.cond);
                self.break_line(Indent::Incr);
                self.format_statements(node.body);
                self.break_line(Indent::Decr);
                self.buffer.push_str("end");
            }
            Node::None(_) => {}
        }
    }

    fn format_statements(&mut self, node: Statements) {
        for (i, n) in node.nodes.into_iter().enumerate() {
            match n.trivia() {
                Some(t) => {
                    self.write_trivia(t);
                    if !n.is_none() {
                        self.break_line(Indent::Keep);
                    }
                }
                None => {
                    if i > 0 {
                        self.break_line(Indent::Keep);
                    }
                }
            }
            self.format(n);
        }
    }

    fn write_trivia(&mut self, trivia: &Trivia) {
        if let Some(comment) = &trivia.last_trailing_comment {
            self.buffer.push(' ');
            self.buffer.push_str(&comment.value);
        }
        for node in &trivia.leading_trivia {
            match node {
                TriviaNode::EmptyLine => self.break_line(Indent::Keep),
                TriviaNode::LineComment(comment) => {
                    self.break_line(Indent::Keep);
                    self.buffer.push_str(&comment.value);
                }
            }
        }
    }

    fn break_line(&mut self, indent: Indent) {
        self.buffer.push('\n');
        match indent {
            Indent::Keep => {}
            Indent::Incr => self.indent(),
            Indent::Decr => self.dedent(),
        };
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
    }

    fn indent(&mut self) {
        self.indent = self.indent.saturating_add(2);
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }
}
