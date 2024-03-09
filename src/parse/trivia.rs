use std::ops::Range;

use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn take_end_trivia_as_virtual_end(
        &mut self,
        end: Option<usize>,
    ) -> Option<fmt::VirtualEnd> {
        if let Some(end) = end {
            let trivia = self.take_leading_trivia(end);
            if !trivia.is_empty() {
                return Some(fmt::VirtualEnd::new(trivia));
            }
        }
        None
    }

    pub(super) fn take_leading_trivia(&mut self, loc_start: usize) -> fmt::LeadingTrivia {
        let mut trivia = fmt::LeadingTrivia::new();

        while let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if !(self.last_loc_end..=loc_start).contains(&loc.start_offset()) {
                break;
            };
            let mut value = Self::source_lossy_at(&loc);
            let comment = if value.starts_with("=begin") {
                value = value.trim_end().to_string();
                fmt::Comment::Block(value)
            } else {
                fmt::Comment::Oneline(value)
            };
            self.take_empty_lines_until(loc.start_offset(), &mut trivia);
            trivia.append_line(fmt::LineTrivia::Comment(comment));
            self.last_loc_end = loc.end_offset() - 1;
            self.comments.next();
        }

        self.take_empty_lines_until(loc_start, &mut trivia);
        trivia
    }

    pub(super) fn take_empty_lines_until(&mut self, end: usize, trivia: &mut fmt::LeadingTrivia) {
        let range = self.last_empty_line_range_within(self.last_loc_end, end);
        if let Some(range) = range {
            trivia.append_line(fmt::LineTrivia::EmptyLine);
            self.last_loc_end = range.end;
        }
    }

    pub(super) fn take_trailing_comment(&mut self, end: usize) -> fmt::TrailingTrivia {
        if let Some(comment) = self.comments.peek() {
            let loc = comment.location();
            if (self.last_loc_end..=end).contains(&loc.start_offset())
                && !self.is_at_line_start(loc.start_offset())
            {
                self.last_loc_end = loc.end_offset() - 1;
                self.comments.next();
                let value = Self::source_lossy_at(&loc);
                return fmt::TrailingTrivia::new(Some(value));
            }
        };
        fmt::TrailingTrivia::none()
    }

    fn last_empty_line_range_within(&self, start: usize, end: usize) -> Option<Range<usize>> {
        let mut line_start: Option<usize> = None;
        let mut line_end: Option<usize> = None;
        for i in (start..end).rev() {
            let b = self.src[i];
            if b == b'\n' {
                if line_end.is_none() {
                    line_end = Some(i + 1);
                } else {
                    line_start = Some(i);
                    break;
                }
            } else if line_end.is_some() && b != b' ' {
                line_end = None;
            }
        }
        match (line_start, line_end) {
            (Some(start), Some(end)) => Some(start..end),
            _ => None,
        }
    }

    fn is_at_line_start(&self, start: usize) -> bool {
        if start == 0 {
            return true;
        }
        let mut idx = start - 1;
        let mut has_char_between_last_newline = false;
        while let Some(b) = self.src.get(idx) {
            match b {
                b' ' => {
                    idx -= 1;
                    continue;
                }
                b'\n' => break,
                _ => {
                    has_char_between_last_newline = true;
                    break;
                }
            }
        }
        !has_char_between_last_newline
    }
}
