use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    Comment, LeadingTrivia, LineTrivia,
};

#[derive(Debug)]
pub(crate) struct VirtualEnd {
    pub shape: Shape,
    pub leading_trivia: LeadingTrivia,
}

impl VirtualEnd {
    pub(crate) fn new(leading_trivia: LeadingTrivia) -> Self {
        Self {
            shape: *leading_trivia.shape(),
            leading_trivia,
        }
    }

    pub(crate) fn format(
        &self,
        o: &mut Output,
        ctx: &FormatContext,
        break_first: bool,
        trim_start: bool,
    ) {
        let mut trailing_empty_lines = 0;
        let leading_lines = &self.leading_trivia.lines();
        for trivia in leading_lines.iter().rev() {
            match trivia {
                LineTrivia::EmptyLine => {
                    trailing_empty_lines += 1;
                }
                LineTrivia::Comment(_) => {
                    break;
                }
            }
        }
        if trailing_empty_lines == leading_lines.len() {
            return;
        }

        if break_first {
            o.break_line(ctx);
        }
        let target_len = leading_lines.len() - trailing_empty_lines;
        let last_idx = target_len - 1;
        for (i, trivia) in leading_lines.iter().take(target_len).enumerate() {
            match trivia {
                LineTrivia::EmptyLine => {
                    if !(trim_start && i == 0) || i == last_idx {
                        o.break_line(ctx);
                    }
                }
                LineTrivia::Comment(comment) => {
                    match &comment {
                        Comment::Oneline(comment) => {
                            o.put_indent_if_needed();
                            o.push_str(comment);
                        }
                        Comment::Block(comment) => o.push_str(comment),
                    }
                    if i < last_idx {
                        o.break_line(ctx);
                    }
                }
            }
        }
    }
}
