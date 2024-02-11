use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct Hash {
    pub shape: Shape,
    pub opening: String,
    pub closing: String,
    pub elements: Vec<Node>,
    pub virtual_end: Option<VirtualEnd>,
}

impl Hash {
    pub(crate) fn new(opening: String, closing: String, should_be_inline: bool) -> Self {
        let shape = if should_be_inline {
            Shape::inline(opening.len() + closing.len())
        } else {
            Shape::Multilines
        };
        Self {
            shape,
            opening,
            closing,
            elements: vec![],
            virtual_end: None,
        }
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if self.elements.is_empty() {
            self.shape.insert(&Shape::inline("  ".len()));
        } else {
            self.shape.insert(&Shape::inline(", ".len()));
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
            o.push_str(&self.opening);
            if !self.elements.is_empty() {
                o.push(' ');
                for (i, n) in self.elements.iter().enumerate() {
                    if i > 0 {
                        o.push_str(", ");
                    }
                    n.format(o, ctx);
                }
                o.push(' ');
            }
            o.push_str(&self.closing);
        } else {
            o.push_str(&self.opening);
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
                element.format(o, ctx);
                o.push(',');
                element.trailing_trivia.format(o);
            }
            o.write_trivia_at_virtual_end(ctx, &self.virtual_end, true, self.elements.is_empty());
            o.dedent();
            o.break_line(ctx);
            o.push_str(&self.closing);
        }
    }
}
