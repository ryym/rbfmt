use super::shape::Shape;

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
}

#[derive(Debug)]
pub(crate) struct TrailingTrivia {
    comment: Option<Comment>,
    shape: Shape,
}

impl TrailingTrivia {
    pub(crate) fn new(comment: Option<Comment>) -> Self {
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

    pub(crate) fn comment(&self) -> &Option<Comment> {
        &self.comment
    }

    pub(crate) fn shape(&self) -> &Shape {
        &self.shape
    }

    pub(crate) fn is_none(&self) -> bool {
        self.comment.is_none()
    }
}

#[derive(Debug)]
pub(crate) struct Comment {
    pub value: String,
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
