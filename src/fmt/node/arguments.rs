use crate::fmt::{
    output::{DraftResult, FormatContext, Output},
    shape::{ArgumentStyle, Shape},
    trivia::EmptyLineHandling,
};

use super::{Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Arguments {
    pub opening: Option<String>,
    pub closing: Option<String>,
    pub shape: Shape,
    pub nodes: Vec<Node>,
    pub last_comma_allowed: bool,
    pub virtual_end: Option<VirtualEnd>,
}

impl Arguments {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |o| o.len());
        let closing_len = closing.as_ref().map_or(0, |o| o.len());
        Self {
            opening,
            closing,
            shape: Shape::inline(opening_len + closing_len),
            nodes: vec![],
            last_comma_allowed: true,
            virtual_end: None,
        }
    }

    pub(crate) fn append_node(&mut self, node: Node) {
        self.shape.insert(&node.shape);
        if !self.nodes.is_empty() {
            self.shape.insert(&Shape::inline(", ".len()));
        }
        self.nodes.push(node);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.insert(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.virtual_end.is_none()
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        // Format horizontally if all these are met:
        //   - no intermediate comments
        //   - all nodes' ArgumentStyle is horizontal
        //   - only the last argument can span in multilines
        let draft_result = o.draft(|d| {
            if self.virtual_end.is_some() {
                return DraftResult::Rollback;
            }
            d.push_str(self.opening.as_ref().map_or(" ", |s| s));
            for (i, arg) in self.nodes.iter().enumerate() {
                if i > 0 {
                    d.push_str(", ");
                }
                if matches!(arg.shape, Shape::LineEnd { .. }) {
                    return DraftResult::Rollback;
                }
                match arg.argument_style() {
                    ArgumentStyle::Vertical => match arg.shape {
                        Shape::Inline { len } if len <= d.remaining_width => {
                            arg.format(d, ctx);
                        }
                        _ => return DraftResult::Rollback,
                    },
                    ArgumentStyle::Horizontal { min_first_line_len } => {
                        if d.remaining_width < min_first_line_len {
                            return DraftResult::Rollback;
                        }
                        let prev_line_count = d.line_count;
                        arg.format(d, ctx);
                        if prev_line_count < d.line_count && i < self.nodes.len() - 1 {
                            return DraftResult::Rollback;
                        }
                    }
                }
            }
            if let Some(closing) = &self.closing {
                if d.remaining_width < closing.len() {
                    return DraftResult::Rollback;
                }
                d.push_str(closing);
            }
            DraftResult::Commit
        });

        if matches!(draft_result, DraftResult::Commit) {
            return;
        }

        if let Some(opening) = &self.opening {
            o.push_str(opening);
            o.indent();
            if !self.nodes.is_empty() {
                let last_idx = self.nodes.len() - 1;
                for (i, arg) in self.nodes.iter().enumerate() {
                    o.break_line(ctx);
                    arg.leading_trivia.format(
                        o,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    o.put_indent_if_needed();
                    arg.format(o, ctx);
                    if i < last_idx {
                        o.push(',');
                    }
                    arg.trailing_trivia.format(o);
                }
            }
            o.write_trivia_at_virtual_end(ctx, &self.virtual_end, true, self.nodes.is_empty());
            o.dedent();
            o.break_line(ctx);
            if let Some(closing) = &self.closing {
                o.put_indent_if_needed();
                o.push_str(closing);
            }
        } else if !self.nodes.is_empty() {
            o.push(' ');
            self.nodes[0].format(o, ctx);
            if self.nodes.len() > 1 {
                o.push(',');
            }
            self.nodes[0].trailing_trivia.format(o);
            match self.nodes.len() {
                1 => {}
                2 if self.nodes[0].trailing_trivia.is_none()
                    && self.nodes[1].shape.fits_in_one_line(o.remaining_width) =>
                {
                    o.push(' ');
                    self.nodes[1].format(o, ctx);
                }
                _ => {
                    o.indent();
                    let last_idx = self.nodes.len() - 1;
                    for (i, arg) in self.nodes.iter().enumerate().skip(1) {
                        o.break_line(ctx);
                        arg.leading_trivia.format(
                            o,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: i == 0,
                                end: false,
                            },
                        );
                        o.put_indent_if_needed();
                        arg.format(o, ctx);
                        if i < last_idx {
                            o.push(',');
                        }
                        arg.trailing_trivia.format(o);
                    }
                    o.dedent();
                }
            }
        }
    }
}
