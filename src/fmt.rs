pub(crate) fn format(node: Node) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
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
    None(Trivia),
}

impl Node {
    fn is_none(&self) -> bool {
        matches!(self, Self::None(_))
    }

    fn trivia(&self) -> Option<&Trivia> {
        match self {
            Self::Nil(t) | Self::Boolean(t, _) | Self::Number(t, _) | Self::Identifier(t, _) => {
                t.as_ref()
            }
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
struct Formatter {
    buffer: String,
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
                for (i, n) in node.nodes.into_iter().enumerate() {
                    match n.trivia() {
                        Some(t) => {
                            self.write_trivia(t);
                            if !n.is_none() {
                                self.buffer.push('\n');
                            }
                        }
                        None => {
                            if i > 0 {
                                self.buffer.push('\n');
                            }
                        }
                    }
                    self.format(n);
                }
            }
            Node::None(_) => {}
        }
    }

    fn write_trivia(&mut self, trivia: &Trivia) {
        if let Some(comment) = &trivia.last_trailing_comment {
            self.buffer.push(' ');
            self.buffer.push_str(&comment.value);
        }
        for node in &trivia.leading_trivia {
            match node {
                TriviaNode::EmptyLine => self.buffer.push('\n'),
                TriviaNode::LineComment(comment) => {
                    self.buffer.push('\n');
                    self.buffer.push_str(&comment.value);
                }
            }
        }
    }
}
