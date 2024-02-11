use std::mem;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Shape {
    Inline { len: usize },
    LineEnd { len: usize },
    Multilines,
}

impl Shape {
    pub(crate) fn inline(len: usize) -> Self {
        Self::Inline { len }
    }

    pub(crate) fn is_inline(&self) -> bool {
        matches!(self, Self::Inline { .. })
    }

    pub(crate) fn fits_in_inline(&self, width: usize) -> bool {
        match self {
            Self::Inline { len } => *len <= width,
            _ => false,
        }
    }

    pub(crate) fn fits_in_one_line(&self, width: usize) -> bool {
        match self {
            Self::Inline { len } | Self::LineEnd { len } => *len <= width,
            Self::Multilines => false,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Self::Inline { len } | Self::LineEnd { len } => *len == 0,
            Self::Multilines => false,
        }
    }

    pub(crate) fn argument_style(&self) -> ArgumentStyle {
        match self {
            Self::Inline { len } => ArgumentStyle::Horizontal {
                min_first_line_len: *len,
            },
            _ => ArgumentStyle::Vertical,
        }
    }

    pub(crate) fn append(&mut self, other: &Self) {
        let shape = self.add(other);
        let _ = mem::replace(self, shape);
    }

    pub(crate) fn add(self, other: &Self) -> Self {
        match self {
            Self::Inline { len: len1 } => match other {
                Self::Inline { len: len2 } => Self::Inline { len: len1 + len2 },
                Self::LineEnd { len: len2 } => Self::LineEnd { len: len1 + len2 },
                Self::Multilines => Self::Multilines,
            },
            Self::LineEnd { .. } | Self::Multilines => Self::Multilines,
        }
    }

    pub(crate) fn insert(&mut self, other: &Self) {
        let shape = match self {
            Self::Inline { len: len1 } => match other {
                Self::Inline { len: len2 } => Self::Inline { len: *len1 + *len2 },
                Self::LineEnd { len: len2 } => {
                    if *len1 == 0 {
                        Self::LineEnd { len: *len2 }
                    } else {
                        Self::Multilines
                    }
                }
                Self::Multilines => Self::Multilines,
            },
            Self::LineEnd { .. } | Self::Multilines => Self::Multilines,
        };
        let _ = mem::replace(self, shape);
    }
}

#[derive(Debug)]
pub(crate) enum ArgumentStyle {
    Vertical,
    Horizontal { min_first_line_len: usize },
}

impl ArgumentStyle {
    pub(crate) fn add(&self, other: Self) -> Self {
        match (self, other) {
            (
                Self::Horizontal {
                    min_first_line_len: len1,
                },
                Self::Horizontal {
                    min_first_line_len: len2,
                },
            ) => Self::Horizontal {
                min_first_line_len: len1 + len2,
            },
            _ => Self::Vertical,
        }
    }
}
