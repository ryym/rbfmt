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
                let (min_spaces, line_start_str_indice) = Self::inspect_body_indent(&self.parts);
                let desired_indent = " ".repeat(o.indent);
                for (i, part) in self.parts.iter().enumerate() {
                    let is_line_start = line_start_str_indice.contains(&i);
                    if is_line_start {
                        o.push_str_without_indent(&desired_indent);
                    }
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            if is_line_start {
                                o.push_str_without_indent(&value[min_spaces..]);
                            } else {
                                o.push_str_without_indent(&value);
                            }
                        }
                        HeredocPart::Statements(embedded) => {
                            embedded.format(o, ctx);
                        }
                        HeredocPart::Variable(var) => {
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

    fn inspect_body_indent(parts: &Vec<HeredocPart>) -> (usize, Vec<usize>) {
        if parts.is_empty() {
            return (0, vec![]);
        }
        let mut is_line_start = matches!(parts[0], HeredocPart::Str(_));
        let mut min_spaces = usize::MAX;
        let mut line_start_str_indice: Vec<usize> = vec![];
        for (part_idx, part) in parts.iter().enumerate() {
            match part {
                HeredocPart::Str(str) => {
                    if is_line_start {
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
                                    return (0, vec![]);
                                }
                                _ => break,
                            }
                            i += 1;
                        }
                        if spaces < min_spaces {
                            min_spaces = spaces;
                        }
                        line_start_str_indice.push(part_idx);
                    }
                    is_line_start = str.value.last().unwrap() == &b'\n';
                }
                _ => {
                    if is_line_start {
                        line_start_str_indice.push(part_idx);
                    }
                    is_line_start = false;
                }
            }
        }
        if min_spaces == usize::MAX {
            min_spaces = 0;
        }
        (min_spaces, line_start_str_indice)
    }
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
