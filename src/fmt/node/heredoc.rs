use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::{EmbeddedStatements, EmbeddedVariable, StringLike};

#[derive(Debug, Clone, Copy)]
pub(crate) struct HeredocState {
    pub pos: Pos,
    pub opening_line_indent: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos(pub usize);

#[derive(Debug)]
pub(crate) struct HeredocOpening {
    pub pos: Pos,
    pub shape: Shape,
    pub id: String,
    pub indent_mode: HeredocIndentMode,
}

impl HeredocOpening {
    pub(crate) fn new(pos: Pos, id: String, indent_mode: HeredocIndentMode) -> Self {
        let shape = Shape::inline(id.len() + indent_mode.prefix_symbols().len());
        Self {
            pos,
            shape,
            id,
            indent_mode,
        }
    }

    pub(crate) fn shape(&self) -> &Shape {
        &self.shape
    }

    pub(crate) fn format(&self, o: &mut Output) {
        o.push_str(self.indent_mode.prefix_symbols());
        o.push_str(&self.id);
        o.heredoc_queue.push_back(HeredocState {
            pos: self.pos,
            opening_line_indent: o.indent,
        });
    }
}

#[derive(Debug)]
pub(crate) struct Heredoc {
    pub id: String,
    pub indent_mode: HeredocIndentMode,
    pub parts: Vec<HeredocPart>,
}

impl Heredoc {
    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext, state: &HeredocState) {
        let actual_indent = o.indent;
        o.indent = state.opening_line_indent;
        match self.indent_mode {
            HeredocIndentMode::None | HeredocIndentMode::EndIndented => {
                for part in &self.parts {
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            o.push_str_without_indent(&value);
                        }
                        HeredocPart::Statements(embedded) => {
                            embedded.format(o, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            var.format(o);
                        }
                    }
                }
                if matches!(self.indent_mode, HeredocIndentMode::EndIndented) {
                    o.put_indent();
                }
                o.push_str(&self.id);
            }
            HeredocIndentMode::AllIndented => {
                o.indent();
                let body_info =
                    inspect_body_indent(&self.parts).unwrap_or(SquigglyHeredocBodyInfo {
                        min_spaces: 0,
                        line_starts: vec![],
                    });
                let desired_indent = " ".repeat(o.indent);
                for (i, part) in self.parts.iter().enumerate() {
                    let line_start = body_info.line_starts.get(i).unwrap_or(&None);
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            if let Some(line_start) = line_start {
                                if let Some(empty_line) = &line_start.empty_line {
                                    if body_info.min_spaces < empty_line.prefix_spaces {
                                        o.push_str_without_indent(&desired_indent);
                                        o.push_str_without_indent(&value[body_info.min_spaces..]);
                                    } else {
                                        o.push_str_without_indent(
                                            &value[empty_line.prefix_spaces..],
                                        );
                                    }
                                } else {
                                    o.push_str_without_indent(&desired_indent);
                                    o.push_str_without_indent(&value[body_info.min_spaces..]);
                                }
                            } else {
                                o.push_str_without_indent(&value);
                            }
                        }
                        HeredocPart::Statements(embedded) => {
                            if line_start.is_some() {
                                o.push_str_without_indent(&desired_indent);
                            }
                            embedded.format(o, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            if line_start.is_some() {
                                o.push_str_without_indent(&desired_indent);
                            }
                            var.format(o);
                        }
                    }
                }
                o.dedent();
                o.put_indent();
                o.push_str(&self.id);
            }
        }
        o.indent = actual_indent;
    }
}

fn inspect_body_indent(parts: &Vec<HeredocPart>) -> Option<SquigglyHeredocBodyInfo> {
    if parts.is_empty() {
        return None;
    }
    let mut is_line_start = matches!(parts[0], HeredocPart::Str(_));
    let mut min_spaces = usize::MAX;
    let mut line_starts = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            HeredocPart::Str(str) => {
                let line_start = if is_line_start {
                    // Empty lines does not contribute to determining `min_spaces`,
                    // even if it starts with some spaces.
                    if let Some(prefix_spaces) = prefix_spaces_of_empty_line(&str.value) {
                        Some(SquigglyHeredocLineStartPartInfo {
                            empty_line: Some(SquigglyHeredocEmptyLineInfo { prefix_spaces }),
                        })
                    } else {
                        let mut i = 0;
                        let mut spaces = 0;
                        while i < str.value.len() {
                            match str.value[i] {
                                b' ' => {
                                    spaces += 1;
                                }
                                b'\t' => {
                                    // If there is a line that has a mixture of spaces and tabs
                                    // at the beginning of the line, do not adjust the indentation.
                                    // Its handling seems so complicated: https://bugs.ruby-lang.org/issues/9098.
                                    return None;
                                }
                                _ => break,
                            }
                            i += 1;
                        }
                        if spaces < min_spaces {
                            min_spaces = spaces;
                        }
                        Some(SquigglyHeredocLineStartPartInfo { empty_line: None })
                    }
                } else {
                    None
                };
                line_starts.push(line_start);
                is_line_start = str.value.last().unwrap() == &b'\n';
            }
            _ => {
                let line_start = if is_line_start {
                    Some(SquigglyHeredocLineStartPartInfo { empty_line: None })
                } else {
                    None
                };
                line_starts.push(line_start);
                is_line_start = false;
            }
        }
    }
    if min_spaces == usize::MAX {
        min_spaces = 0;
    }
    Some(SquigglyHeredocBodyInfo {
        min_spaces,
        line_starts,
    })
}

// return a number of spaces at line start only if it is an empty (space-only) line.
// return None if
//   - the value is not a line, that is, it does not end with a line break
//   - the value is a line but contains non-space characters
fn prefix_spaces_of_empty_line(value: &Vec<u8>) -> Option<usize> {
    if value.last().map_or(false, |c| c != &b'\n') {
        return None;
    }
    let mut spaces_len = 0;
    for char in value.iter().take(value.len() - 1) {
        if char != &b' ' {
            return None;
        }
        spaces_len += 1;
    }
    Some(spaces_len)
}

#[derive(Debug)]
struct SquigglyHeredocBodyInfo {
    min_spaces: usize,
    line_starts: Vec<Option<SquigglyHeredocLineStartPartInfo>>,
}

#[derive(Debug)]
struct SquigglyHeredocLineStartPartInfo {
    empty_line: Option<SquigglyHeredocEmptyLineInfo>,
}

#[derive(Debug)]
struct SquigglyHeredocEmptyLineInfo {
    prefix_spaces: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HeredocIndentMode {
    None,
    EndIndented,
    AllIndented,
}

impl HeredocIndentMode {
    pub(crate) fn parse_mode_and_id(opening: &[u8]) -> (Self, &[u8]) {
        let (indent_mode, id_start) = match opening[2] {
            b'~' => (Self::AllIndented, 3),
            b'-' => (Self::EndIndented, 3),
            _ => (Self::None, 2),
        };
        (indent_mode, &opening[id_start..])
    }

    pub(crate) fn prefix_symbols(&self) -> &'static str {
        match self {
            Self::None => "<<",
            Self::EndIndented => "<<-",
            Self::AllIndented => "<<~",
        }
    }
}

#[derive(Debug)]
pub(crate) enum HeredocPart {
    Str(StringLike),
    Statements(EmbeddedStatements),
    Variable(EmbeddedVariable),
}
