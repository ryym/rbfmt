pub(crate) fn format(node: Node) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
    };
    formatter.format(node);
    formatter.buffer.push('\n');
    formatter.buffer
}

#[derive(Debug)]
pub(crate) enum Node {
    EmptyLine,
    Number(Number),
    Identifier(Identifier),
    Statements(Statements),
}

pub(crate) trait GroupNodeEntity {
    fn append_node(&mut self, node: Node);
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
                // Do nothing here because the parent group node breaks a line.
            }
            Node::Number(node) => {
                self.buffer.push_str(&node.value);
            }
            Node::Identifier(node) => {
                self.buffer.push_str(&node.name);
            }
            Node::Statements(node) => {
                for (i, n) in node.nodes.into_iter().enumerate() {
                    if i > 0 {
                        self.buffer.push('\n');
                    }
                    self.format(n);
                }
            }
        };
    }
}
