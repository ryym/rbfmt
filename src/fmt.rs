pub(crate) fn format(node: Node) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
    };
    formatter.format(node);
    formatter.buffer.push('\n');
    formatter.buffer
}

pub(crate) enum Node {
    Identifier(Identifier),
    Statements(Statements),
}

pub(crate) struct Identifier {
    pub name: String,
}

pub(crate) struct Statements {
    pub nodes: Vec<Node>,
}

struct Formatter {
    buffer: String,
}

impl Formatter {
    fn format(&mut self, node: Node) {
        match node {
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
