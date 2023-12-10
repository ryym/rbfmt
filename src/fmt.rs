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
                // If a condition part has its own trivia, move it to top of the if expression.
                // e.g. "if #foo\n cond\n..." -> "#foo\nif cond\n..."
                if let Some(trivia) = node.cond.trivia() {
                    if let Some(comment) = &trivia.last_trailing_comment {
                        self.buffer.push_str(&comment.value);
                        self.write_leading_trivia(&trivia.leading_trivia);
                    } else {
                        if self.buffer.ends_with('\n') {
                            self.buffer.truncate(self.buffer.len() - 1);
                        }
                        self.write_trivia(trivia);
                    }
                    self.break_line();
                }
                self.buffer.push_str("if ");
                self.format(*node.cond);
                self.indent();
                self.format_statements(node.body);
                self.dedent();
                self.break_line();
                self.buffer.push_str("end");
            }
            Node::None(_) => {}
        }
    }

    fn format_statements(&mut self, node: Statements) {
        for n in node.nodes {
            match n.trivia() {
                Some(t) => {
                    self.write_trivia(t);
                    if !n.is_none() {
                        self.break_line();
                    }
                }
                None => {
                    self.break_line();
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
        self.write_leading_trivia(&trivia.leading_trivia);
    }

    fn write_leading_trivia(&mut self, trivia: &Vec<TriviaNode>) {
        for node in trivia {
            match node {
                TriviaNode::EmptyLine => self.break_line(),
                TriviaNode::LineComment(comment) => {
                    self.break_line();
                    self.buffer.push_str(&comment.value);
                }
            }
        }
    }

    fn break_line(&mut self) {
        self.buffer.push('\n');
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
