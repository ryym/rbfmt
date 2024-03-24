use std::collections::HashSet;

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

    fn string_content(&mut self, value: Vec<u8>) {
        self.break_line();
        self.buffer.push_str("---\n");
        self.u8_bytes(value);
        self.buffer.push_str("\n---");
    }

    fn none_value(&mut self) {
        self.break_line();
        self.put_indent();
        self.buffer.push_str("(none)");
    }

    fn string_or_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        content_loc: prism::Location,
    ) {
        let content = if is_squiggly_heredoc(&opening_loc) {
            content_loc
                .as_slice()
                .iter()
                .skip_while(|c| **c == b' ')
                .copied()
                .collect()
        } else {
            content_loc.as_slice().to_vec()
        };
        self.string_content(content);
    }

    fn interpolated_string_or_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        node_parts: prism::NodeList,
    ) {
        let parts = node_parts.iter().collect();
        let heredoc_info = if is_squiggly_heredoc(&opening_loc) {
            calc_squiggly_heredoc_indent(&parts)
        } else {
            None
        };
        if let Some((indent_to_remove, line_starts)) = heredoc_info {
            for (i, part) in parts.into_iter().enumerate() {
                self.start_field(i);
                match part {
                    prism::Node::StringNode { .. } => {
                        let node = part.as_string_node().unwrap();
                        let content = node.content_loc().as_slice();
                        let content = if line_starts.contains(&i) {
                            content
                                .iter()
                                .enumerate()
                                .skip_while(|(i, c)| *i < indent_to_remove && **c == b' ')
                                .map(|(_, c)| *c)
                                .collect()
                        } else {
                            content.to_vec()
                        };
                        self.start_node("StringNode");
                        self.string_content(content);
                        self.end_node();
                    }
                    _ => self.node(&part),
                }
                self.end_field();
            }
        } else {
            self.list_field("parts", node_parts);
        }
    }
}

fn is_squiggly_heredoc(opening_loc: &Option<prism::Location>) -> bool {
    if let Some(loc) = opening_loc {
        loc.as_slice().starts_with(b"<<~")
    } else {
        false
    }
}

fn calc_squiggly_heredoc_indent(parts: &Vec<prism::Node>) -> Option<(usize, HashSet<usize>)> {
    if parts.is_empty() {
        return None;
    }
    let mut is_line_start = true;
    let mut line_starts = HashSet::new();
    let mut min_indent = usize::MAX;
    for (i, part) in parts.iter().enumerate() {
        if let Some(str) = part.as_string_node() {
            let content = str.content_loc().as_slice();
            if is_line_start {
                line_starts.insert(i);
                let mut indent = 0;
                let mut is_empty_line = false;
                for ch in content {
                    match *ch {
                        b'\t' => return None,
                        b' ' => indent += 1,
                        b'\n' => {
                            is_empty_line = true;
                            break;
                        }
                        _ => break,
                    }
                }
                if !is_empty_line && indent < min_indent {
                    min_indent = indent;
                }
            } else {
                is_line_start = content.ends_with(b"\n");
            }
        } else {
            is_line_start = false;
        }
    }
    let indent_to_remove = if min_indent == usize::MAX {
        0
    } else {
        min_indent
    };
    Some((indent_to_remove, line_starts))
}
