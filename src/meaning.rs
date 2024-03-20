mod autogen;

pub fn extract(node: &prism::Node) -> String {
    let mut meaning = Meaning {
        buffer: String::new(),
        indent: 0,
    };
    meaning.node(node);
    meaning.buffer.trim().to_string()
}

struct Meaning {
    buffer: String,
    indent: usize,
}

impl Meaning {
    fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
    }

    fn break_line(&mut self) {
        self.buffer.push('\n');
    }

    fn start_node(&mut self, name: &str) {
        self.break_line();
        self.put_indent();
        self.buffer.push('[');
        self.buffer.push_str(name);
        self.buffer.push(']');
        self.indent = self.indent.saturating_add(2);
    }

    fn end_node(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }

    fn atom_node(&mut self, name: &str, node: &prism::Node) {
        self.break_line();
        self.put_indent();
        self.buffer.push('[');
        self.buffer.push_str(name);
        self.buffer.push_str("] ");
        self.u8_bytes(node.location().as_slice().to_vec());
    }

    fn numbered_parameters_node(&mut self, name: &str) {
        self.break_line();
        self.put_indent();
        self.buffer.push('[');
        self.buffer.push_str(name);
        self.buffer.push(']');
    }

    fn start_field(&mut self, name: impl ToString) {
        self.break_line();
        self.put_indent();
        self.buffer.push_str(&name.to_string());
        self.buffer.push(':');
        self.indent = self.indent.saturating_add(2);
    }

    fn end_field(&mut self) {
        self.indent = self.indent.saturating_sub(2);
    }

    fn node_field(&mut self, name: impl ToString, node: prism::Node) {
        self.start_field(name);
        self.node(&node);
        self.end_field();
    }

    fn opt_field(&mut self, name: &str, node: Option<prism::Node>) {
        self.start_field(name);
        if let Some(node) = node {
            self.node(&node);
        } else {
            self.none_value();
        }
        self.end_field();
    }

    fn list_field(&mut self, name: &str, nodes: prism::NodeList) {
        self.start_field(name);
        self.node_list(nodes);
        self.end_field();
    }

    fn opt_loc_field(&mut self, name: &str, loc: Option<prism::Location>) {
        self.start_field(name);
        if let Some(loc) = loc {
            self.break_line();
            self.put_indent();
            self.u8_bytes(loc.as_slice().to_vec());
        } else {
            self.none_value();
        }
        self.end_field();
    }

    fn call_operator_loc_field(&mut self, loc: Option<prism::Location>) {
        self.start_field("call_operator_loc");
        if let Some(loc) = loc {
            let mut bytes = loc.as_slice();
            if matches!(bytes, [b':', b':']) {
                bytes = &[b'.'];
            }
            self.break_line();
            self.put_indent();
            self.u8_bytes(bytes.to_vec());
        } else {
            self.none_value();
        }
        self.end_field();
    }

    fn message_loc_field(&mut self, loc: Option<prism::Location>) {
        self.start_field("message_loc");
        let bytes = if let Some(loc) = loc {
            let slice = loc.as_slice();
            let len = slice.len();
            if len > 2 && slice[0] == b'[' && slice[len - 1] == b']' {
                // Prism parses index accesses in a call like 'foo[1]' as a message '[1]' as is,
                // so its message_loc can contain spaces, line breaks, comments, etc.
                // We want to ignore such details so treat index accesses as '[]'.
                "[]".as_bytes()
            } else {
                slice
            }
        } else {
            "call".as_bytes()
        };
        self.break_line();
        self.put_indent();
        self.u8_bytes(bytes.to_vec());
        self.end_field();
    }

    fn node_list(&mut self, list: prism::NodeList) {
        for (i, child) in list.iter().enumerate() {
            self.node_field(i, child);
        }
    }

    fn u8_bytes(&mut self, bytes: Vec<u8>) {
        match String::from_utf8(bytes) {
            Ok(value) => self.buffer.push_str(&value),
            Err(err) => {
                self.buffer.push_str(&format!("(non-utf8) {:?}", err));
            }
        }
    }

    fn string_content(&mut self, loc: prism::Location) {
        let value = loc.as_slice().to_vec();
        if value.iter().any(|b| *b == b'\n') {
            self.break_line();
            self.buffer.push_str("---\n");
            self.u8_bytes(value);
            self.buffer.push_str("\n---");
        } else {
            self.buffer.push(' ');
            self.u8_bytes(value);
        }
    }

    fn none_value(&mut self) {
        self.break_line();
        self.put_indent();
        self.buffer.push_str("(none)");
    }
}
