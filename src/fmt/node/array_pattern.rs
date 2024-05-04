use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{Kind, Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct ArrayPattern {
    constant: Option<Box<Node>>,
    array_shape: Shape,
    opening: Option<String>,
    closing: Option<String>,
    elements: Vec<Node>,
    virtual_end: Option<VirtualEnd>,
    pub last_comma_allowed: bool,
}

impl ArrayPattern {
    pub(crate) fn new(constant: Option<Node>) -> Self {
        Self {
            array_shape: Shape::inline(0),
            constant: constant.map(Box::new),
            opening: None,
            closing: None,
            elements: vec![],
            virtual_end: None,
            last_comma_allowed: true,
        }
    }

    pub(crate) fn shape(&self) -> Shape {
        let constant_shape = self.constant.as_ref().map_or(Shape::inline(0), |c| c.shape);
        constant_shape.add(&self.array_shape)
    }

    pub(crate) fn append_element(&mut self, element: Node) {
        if !self.elements.is_empty() {
            self.array_shape.insert(&Shape::inline(", ".len()));
        }
        self.array_shape.insert(&element.shape);
        self.elements.push(element);
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.array_shape.insert(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn finish_with(&mut self, mut opening: Option<String>, mut closing: Option<String>) {
        if closing.is_none() && self.should_close_pattern_to_ensure_valid_syntax() {
            opening = Some("[".to_string());
            closing = Some("]".to_string());
        }
        if let (Some(opening), Some(closing)) = (&opening, &closing) {
            let mut array_shape = Shape::inline(opening.len() + closing.len());
            array_shape.insert(&self.array_shape);
            self.array_shape = array_shape;
        }
        self.opening = opening;
        self.closing = closing;
    }

    fn should_close_pattern_to_ensure_valid_syntax(&self) -> bool {
        if let Some(last) = self.elements.last() {
            match &last.kind {
                //  `a in b,` -> `a in [b,]`
                Kind::Atom(atom) => atom.is_implicit_value(),
                //  `a in b,*` -> `a in [b,*]`
                Kind::Prefix(prefix) => prefix.is_anonymous_splat(),
                _ => false,
            }
        } else {
            false
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if let Some(constant) = &self.constant {
            constant.format(o, ctx);
        }
        if self.array_shape.fits_in_one_line(o.remaining_width) {
            if let Some(opening) = &self.opening {
                o.push_str(opening);
            }
            for (i, n) in self.elements.iter().enumerate() {
                if i > 0 {
                    o.push_str(", ");
                }
                n.format(o, ctx);
            }
            if let Some(closing) = &self.closing {
                o.push_str(closing);
            }
        } else {
            o.push_str(self.opening.as_deref().unwrap_or("["));
            o.indent();
            let last_idx = self.elements.len() - 1;
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
                if i < last_idx {
                    o.push(',');
                }
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
