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
                o.put_indent();
                o.push_str(&self.id);
            }
        }
        o.indent = actual_indent;
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
