pub(crate) fn format(node: Node) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
    };
    formatter.format(node);
    if !formatter.buffer.ends_with('\n') {
        formatter.buffer.push('\n');
    }
    formatter.buffer
}

#[derive(Debug)]
pub(crate) enum Node {
    EmptyLine,
    LineComment(LineComment),
    Number(Number),
    Identifier(Identifier),
    Statements(Statements),
}

impl Node {
    fn is_trivia(&self) -> bool {
        match self {
            Self::EmptyLine | Self::LineComment(_) => true,
            _ => false,
        }
    }
}

pub(crate) trait GroupNodeEntity {
    fn append_node(&mut self, node: Node);
}

#[derive(Debug)]
pub(crate) struct LineComment {
    pub value: String,
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

impl GroupNodeEntity for Statements {
    fn append_node(&mut self, node: Node) {
        self.nodes.push(node);
    }
}

#[derive(Debug)]
struct Formatter {
    buffer: String,
}

impl Formatter {
    fn format(&mut self, node: Node) {
        match node {
            Node::EmptyLine => {
                self.buffer.push('\n');
            }
            Node::LineComment(node) => {
                self.buffer.push_str(&node.value);
            }
            Node::Number(node) => {
                self.buffer.push_str(&node.value);
            }
            Node::Identifier(node) => {
                self.buffer.push_str(&node.name);
            }
            Node::Statements(node) => {
                let mut was_trivia = false;
                for (i, n) in node.nodes.into_iter().enumerate() {
                    if i > 0 && !was_trivia {
                        self.buffer.push('\n');
                    }
                    was_trivia = n.is_trivia();
                    self.format(n);
                }
            }
        };
    }
}
