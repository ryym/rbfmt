use super::{
    output::{FormatContext, Output},
    shape::Shape,
};

#[derive(Debug)]
pub(crate) struct LeadingTrivia {
    lines: Vec<LineTrivia>,
    shape: Shape,
}

impl LeadingTrivia {
    pub(crate) fn new() -> Self {
        Self {
            lines: vec![],
            shape: Shape::inline(0),
        }
    }

    pub(crate) fn shape(&self) -> &Shape {
        &self.shape
    }

    pub(crate) fn lines(&self) -> &Vec<LineTrivia> {
        &self.lines
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub(crate) fn append_line(&mut self, trivia: LineTrivia) {
        if matches!(trivia, LineTrivia::Comment(_)) {
            self.shape = Shape::Multilines;
        }
        self.lines.push(trivia);
    }

    pub(crate) fn merge(&mut self, other: LeadingTrivia) {
        for line in other.lines {
            self.append_line(line);
        }
    }

    pub(crate) fn format(
        &self,
        o: &mut Output,
        ctx: &FormatContext,
        emp_line_handling: EmptyLineHandling,
    ) {
        if self.is_empty() {
            return;
        }
        let last_idx = self.lines().len() - 1;
        for (i, trivia) in self.lines().iter().enumerate() {
            match trivia {
                LineTrivia::EmptyLine => {
                    let should_skip = match emp_line_handling {
                        EmptyLineHandling::Skip => true,
                        EmptyLineHandling::Trim { start, end } => {
                            (start && i == 0) || (end && i == last_idx)
                        }
                    };
                    if !should_skip {
                        o.break_line(ctx);
                    }
                }
                LineTrivia::Comment(comment) => {
                    match &comment {
                        Comment::Oneline(comment) => o.push_str(comment),
                        Comment::Block(comment) => o.push_str_without_indent(comment),
                    }
                    o.break_line(ctx);
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct TrailingTrivia {
    comment: Option<String>,
    shape: Shape,
}

impl TrailingTrivia {
    pub(crate) fn new(comment: Option<String>) -> Self {
        let shape = if comment.is_some() {
            Shape::LineEnd {
                // Do not take into account the length of trailing comment.
                len: 0,
            }
        } else {
            Shape::inline(0)
        };
        Self { comment, shape }
    }

    pub(crate) fn none() -> Self {
        Self::new(None)
    }

    pub(crate) fn comment(&self) -> &Option<String> {
        &self.comment
    }

    pub(crate) fn shape(&self) -> &Shape {
        &self.shape
    }

    pub(crate) fn is_none(&self) -> bool {
        self.comment.is_none()
    }

    pub(crate) fn format(&self, o: &mut Output) {
        if let Some(comment) = &self.comment() {
            o.push(' ');
            o.buffer.push_str(comment);
        }
    }
}

#[derive(Debug)]
pub(crate) enum Comment {
    Oneline(String),
    Block(String),
}

#[derive(Debug)]
pub(crate) enum LineTrivia {
    EmptyLine,
    Comment(Comment),
}

#[derive(Debug)]
pub(crate) enum EmptyLineHandling {
    Trim { start: bool, end: bool },
    Skip,
}

impl EmptyLineHandling {
    pub(crate) fn trim() -> Self {
        Self::Trim {
            start: true,
            end: true,
        }
    }

    pub(crate) fn none() -> Self {
        Self::Trim {
            start: false,
            end: false,
        }
    }
}
