use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct AltPatternChain {
    shape: Shape,
    left: Box<Node>,
    rights_shape: Shape,
    rights: Vec<Node>,
}

impl AltPatternChain {
    pub(crate) fn new(left: Node) -> Self {
        Self {
            shape: left.shape,
            left: Box::new(left),
            rights_shape: Shape::inline(0),
            rights: vec![],
        }
    }

    pub(crate) fn shape(&self) -> Shape {
        self.shape
    }

    pub(crate) fn append_right(&mut self, right: Node) {
        self.shape.append(&right.shape);
        self.rights_shape.append(&right.shape);
        self.rights.push(right);
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        self.left.format(o, ctx);
        if self.rights_shape.fits_in_one_line(o.remaining_width) {
            for right in &self.rights {
                o.push_str(" | ");
                right.format(o, ctx);
            }
        } else {
            for right in &self.rights {
                o.push_str(" |");
                o.indent();
                o.break_line(ctx);
                right
                    .leading_trivia
                    .format(o, ctx, EmptyLineHandling::none());
                o.put_indent_if_needed();
                right.format(o, ctx);
                o.dedent();
            }
        }
    }
}
