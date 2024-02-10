use crate::fmt::shape::Shape;

use super::{EmbeddedStatements, EmbeddedVariable, StringLike};

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
}

#[derive(Debug)]
pub(crate) struct Heredoc {
    pub id: String,
    pub indent_mode: HeredocIndentMode,
    pub parts: Vec<HeredocPart>,
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
