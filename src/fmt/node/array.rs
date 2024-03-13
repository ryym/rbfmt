use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Array {
    pub shape: Shape,
    pub opening: Option<String>,
    pub closing: Option<String>,
    pub elements: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl Array {
    pub(crate) fn new(opening: Option<String>, closing: Option<String>) -> Self {
        let opening_len = opening.as_ref().map_or(0, |s| s.len());
        let closing_len = closing.as_ref().map_or(0, |s| s.len());
        let shape = Shape::inline(opening_len + closing_len);
        Self {
            shape,
            opening,
            closing,
            elements: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn separator(&self) -> &str {
        if let Some(opening) = &self.opening {
            if opening.as_bytes()[0] == b'%' {
                return "";
            }
        }
        ","
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if !self.elements.is_empty() {
            let sep_len = self.separator().len() + 1; // space
            self.shape.insert(&Shape::inline(sep_len));
        }
        self.shape.insert(&element.shape);
        self.elements.push(element);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.insert(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.shape.fits_in_one_line(o.remaining_width) {
            if let Some(opening) = &self.opening {
                o.push_str(opening);
            }
            for (i, n) in self.elements.iter().enumerate() {
                if i > 0 {
                    o.push_str(self.separator());
                    o.push(' ');
                }
                n.format(o, ctx);
            }
            if let Some(closing) = &self.closing {
                o.push_str(closing);
            }
        } else {
            o.push_str(self.opening.as_deref().unwrap_or("["));
            o.indent();
            for (i, element) in self.elements.iter().enumerate() {
                o.break_line(ctx);
                element.leading_trivia.format(
                    o,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                o.put_indent_if_needed();
                element.format(o, ctx);
                o.push_str(self.separator());
                element.trailing_trivia.format(o);
            }
            o.write_trivia_at_virtual_end(ctx, &self.virtual_end, true, self.elements.is_empty());
            o.dedent();
            o.break_line(ctx);
            o.put_indent_if_needed();
            o.push_str(self.closing.as_deref().unwrap_or("]"));
        }
    }
}
