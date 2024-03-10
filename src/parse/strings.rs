use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_string_or_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        content_loc: prism::Location,
        closing_loc: Option<prism::Location>,
    ) -> fmt::Node {
        let kind = if is_heredoc(opening_loc.as_ref()) {
            let opening = self.parse_heredoc(opening_loc, content_loc, closing_loc);
            fmt::Kind::HeredocOpening(opening)
        } else {
            let str = self.parse_string(opening_loc, content_loc, closing_loc);
            fmt::Kind::StringLike(str)
        };
        fmt::Node::new(kind)
    }

    pub(super) fn parse_interpolated_string_or_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        parts: prism::NodeList,
        closing_loc: Option<prism::Location>,
    ) -> fmt::Node {
        let kind = if is_heredoc(opening_loc.as_ref()) {
            let opening = self.parse_interpolated_heredoc(opening_loc, parts, closing_loc);
            fmt::Kind::HeredocOpening(opening)
        } else {
            let str = self.parse_interpolated_string(opening_loc, parts, closing_loc);
            fmt::Kind::DynStringLike(str)
        };
        fmt::Node::new(kind)
    }

    pub(super) fn parse_string(
        &self,
        opening_loc: Option<prism::Location>,
        value_loc: prism::Location,
        closing_loc: Option<prism::Location>,
    ) -> fmt::StringLike {
        let value = Self::source_lossy_at(&value_loc);
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        fmt::StringLike::new(opening, value.into(), closing)
    }

    pub(super) fn parse_interpolated_string(
        &mut self,
        opening_loc: Option<prism::Location>,
        parts: prism::NodeList,
        closing_loc: Option<prism::Location>,
    ) -> fmt::DynStringLike {
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let mut dstr = fmt::DynStringLike::new(opening, closing);
        for part in parts.iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.parse_string(
                        node.opening_loc(),
                        node.content_loc(),
                        node.closing_loc(),
                    );
                    dstr.append_part(fmt::DynStrPart::Str(str));
                    self.last_loc_end = node_end;
                }
                prism::Node::InterpolatedStringNode { .. } => {
                    let node = part.as_interpolated_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.parse_interpolated_string(
                        node.opening_loc(),
                        node.parts(),
                        node.closing_loc(),
                    );
                    dstr.append_part(fmt::DynStrPart::DynStr(str));
                    self.last_loc_end = node_end;
                }
                prism::Node::EmbeddedStatementsNode { .. } => {
                    let node = part.as_embedded_statements_node().unwrap();
                    let loc = node.location();
                    self.last_loc_end = node.opening_loc().end_offset();
                    let statements =
                        self.parse_statements_body(node.statements(), Some(loc.end_offset()));
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    let embedded_stmts = fmt::EmbeddedStatements::new(opening, statements, closing);
                    dstr.append_part(fmt::DynStrPart::Statements(embedded_stmts));
                }
                prism::Node::EmbeddedVariableNode { .. } => {
                    let node = part.as_embedded_variable_node().unwrap();
                    let operator = Self::source_lossy_at(&node.operator_loc());
                    let variable = Self::source_lossy_at(&node.variable().location());
                    let embedded_var = fmt::EmbeddedVariable::new(operator, variable);
                    dstr.append_part(fmt::DynStrPart::Variable(embedded_var));
                }
                _ => panic!("unexpected string interpolation node: {:?}", part),
            }
        }
        dstr
    }

    fn parse_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        content_loc: prism::Location,
        closing_loc: Option<prism::Location>,
    ) -> fmt::HeredocOpening {
        let open = opening_loc.as_ref().unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let opening_id = String::from_utf8_lossy(id).to_string();
        let closing_loc = closing_loc.expect("heredoc must have closing");
        let closing_id = Self::source_lossy_at(&closing_loc)
            .trim_start()
            .trim_end_matches('\n')
            .to_string();
        let str = self.parse_string(None, content_loc, None);
        let heredoc = fmt::Heredoc {
            id: closing_id,
            indent_mode,
            parts: vec![fmt::HeredocPart::Str(str)],
        };
        let pos = self.next_pos();
        self.register_heredoc(pos, heredoc, closing_loc.end_offset());
        fmt::HeredocOpening::new(pos, opening_id, indent_mode)
    }

    fn parse_interpolated_heredoc(
        &mut self,
        opening_loc: Option<prism::Location>,
        content_parts: prism::NodeList,
        closing_loc: Option<prism::Location>,
    ) -> fmt::HeredocOpening {
        let open = opening_loc.unwrap().as_slice();
        let (indent_mode, id) = fmt::HeredocIndentMode::parse_mode_and_id(open);
        let opening_id = String::from_utf8_lossy(id).to_string();

        let mut parts = vec![];
        let mut last_part_end: Option<usize> = None;
        for part in content_parts.iter() {
            match part {
                prism::Node::StringNode { .. } => {
                    let node = part.as_string_node().unwrap();
                    let node_end = node.location().end_offset();
                    let str = self.parse_string(
                        node.opening_loc(),
                        node.content_loc(),
                        node.closing_loc(),
                    );
                    parts.push(fmt::HeredocPart::Str(str));
                    self.last_loc_end = node_end;
                    last_part_end = Some(node_end);
                }
                prism::Node::EmbeddedStatementsNode { .. } => {
                    let node = part.as_embedded_statements_node().unwrap();
                    let loc = node.location();
                    parse_spaces_before_interpolation(
                        last_part_end,
                        loc.start_offset(),
                        self.src,
                        &mut parts,
                    );
                    let statements =
                        self.parse_statements_body(node.statements(), Some(loc.end_offset()));
                    let opening = Self::source_lossy_at(&node.opening_loc());
                    let closing = Self::source_lossy_at(&node.closing_loc());
                    let embedded = fmt::EmbeddedStatements::new(opening, statements, closing);
                    parts.push(fmt::HeredocPart::Statements(embedded));
                    self.last_loc_end = loc.end_offset();
                    last_part_end = Some(loc.end_offset());
                }
                prism::Node::EmbeddedVariableNode { .. } => {
                    let node = part.as_embedded_variable_node().unwrap();
                    let loc = node.location();
                    parse_spaces_before_interpolation(
                        last_part_end,
                        loc.start_offset(),
                        self.src,
                        &mut parts,
                    );
                    let operator = Self::source_lossy_at(&node.operator_loc());
                    let variable = Self::source_lossy_at(&node.variable().location());
                    let embedded_var = fmt::EmbeddedVariable::new(operator, variable);
                    parts.push(fmt::HeredocPart::Variable(embedded_var));
                    self.last_loc_end = loc.end_offset();
                    last_part_end = Some(loc.end_offset());
                }
                _ => panic!("unexpected heredoc part: {:?}", part),
            }
        }
        let closing_loc = closing_loc.expect("heredoc must have closing");
        let closing_id = Self::source_lossy_at(&closing_loc)
            .trim_start()
            .trim_end_matches('\n')
            .to_string();
        let heredoc = fmt::Heredoc {
            id: closing_id,
            indent_mode,
            parts,
        };
        let pos = self.next_pos();
        self.register_heredoc(pos, heredoc, closing_loc.end_offset());
        fmt::HeredocOpening::new(pos, opening_id, indent_mode)
    }
}

fn is_heredoc(str_opening_loc: Option<&prism::Location>) -> bool {
    if let Some(loc) = str_opening_loc {
        let bytes = loc.as_slice();
        bytes.len() > 2 && bytes[0] == b'<' && bytes[1] == b'<'
    } else {
        false
    }
}

// I don't know why but ruby-prism ignores spaces before an interpolation in some cases.
// It is confusing so we parse all spaces before interpolation.
fn parse_spaces_before_interpolation(
    last_part_end: Option<usize>,
    embedded_start: usize,
    src: &[u8],
    parts: &mut Vec<fmt::HeredocPart>,
) {
    let str = if let Some(last_part_end) = last_part_end {
        if last_part_end < embedded_start {
            let value = src[last_part_end..embedded_start].to_vec();
            Some(fmt::StringLike::new(None, value, None))
        } else {
            None
        }
    } else {
        let mut i = embedded_start - 1;
        while src[i] != b'\n' {
            i -= 1;
        }
        if i + 1 < embedded_start {
            let value = src[(i + 1)..embedded_start].to_vec();
            Some(fmt::StringLike::new(None, value, None))
        } else {
            None
        }
    };
    if let Some(str) = str {
        parts.push(fmt::HeredocPart::Str(str));
    }
}
