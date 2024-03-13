use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
    LeadingTrivia,
};

use super::Node;

#[derive(Debug)]
pub(crate) struct ConstantPath {
    pub shape: Shape,
    pub root: Option<Box<Node>>,
    pub parts: Vec<(LeadingTrivia, String)>,
}

impl ConstantPath {
    pub(crate) fn new(root: Option<Node>) -> Self {
        let shape = root.as_ref().map_or(Shape::inline(0), |r| r.shape);
        Self {
            shape,
            root: root.map(Box::new),
            parts: vec![],
        }
    }

    pub(crate) fn append_part(&mut self, leading: LeadingTrivia, path: String) {
        self.shape.append(leading.shape());
        self.shape.append(&Shape::inline("::".len() + path.len()));
        self.parts.push((leading, path));
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if let Some(root) = &self.root {
            root.format(o, ctx);
        }
        o.push_str("::");
        let last_idx = self.parts.len() - 1;
        for (i, (leading, path)) in self.parts.iter().enumerate() {
            if leading.is_empty() {
                o.push_str(path);
            } else {
                o.indent();
                o.break_line(ctx);
                leading.format(o, ctx, EmptyLineHandling::trim());
                o.put_indent_if_needed();
                o.push_str(path);
                o.dedent();
            }
            if i < last_idx {
                o.push_str("::");
            }
        }
    }
}
