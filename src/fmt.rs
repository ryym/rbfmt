pub(crate) fn format(node: Node) -> String {
    let mut formatter = Formatter {
        buffer: String::new(),
    };
    formatter.format(node);
    formatter.buffer
}

pub(crate) enum Node {
    Identifier(Identifier),
}

pub(crate) struct Identifier {
    pub name: String,
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
        };
        self.buffer.push('\n');
    }
}
