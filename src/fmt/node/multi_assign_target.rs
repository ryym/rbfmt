use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
    trivia::EmptyLineHandling,
};

use super::{Node, VirtualEnd};

#[derive(Debug)]
pub(crate) struct MultiAssignTarget {
    pub shape: Shape,
    pub lparen: Option<String>,
    pub rparen: Option<String>,
    pub targets: Vec<Node>,
    pub with_implicit_rest: bool,
    pub virtual_end: Option<VirtualEnd>,
}

impl MultiAssignTarget {
    pub(crate) fn new(lparen: Option<String>, rparen: Option<String>) -> Self {
        let parens_len = match (&lparen, &rparen) {
            (Some(lp), Some(rp)) => lp.len() + rp.len(),
            _ => 0,
        };
        Self {
            shape: Shape::inline(parens_len),
            lparen,
            rparen,
            targets: vec![],
            with_implicit_rest: false,
            virtual_end: None,
        }
    }

    pub(crate) fn append_target(&mut self, target: Node) {
        self.shape.insert(&target.shape);
        self.targets.push(target);
    }

    pub(crate) fn set_implicit_rest(&mut self, yes: bool) {
        if yes {
            self.shape.insert(&Shape::inline(",".len()));
        }
        self.with_implicit_rest = yes;
    }

    pub(crate) fn set_virtual_end(&mut self, end: Option<VirtualEnd>) {
        if let Some(end) = &end {
            self.shape.append(&end.shape);
        }
        self.virtual_end = end;
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        if self.shape.fits_in_inline(o.remaining_width) {
            if let Some(lparen) = &self.lparen {
                o.push_str(lparen);
            }
            for (i, target) in self.targets.iter().enumerate() {
                if i > 0 {
                    o.push_str(", ");
                }
                o.format(target, ctx);
            }
            if self.with_implicit_rest {
                o.push(',');
            }
            if let Some(rparen) = &self.rparen {
                o.push_str(rparen);
            }
        } else {
            o.push('(');
            o.indent();
            let last_idx = self.targets.len() - 1;
            for (i, target) in self.targets.iter().enumerate() {
                o.break_line(ctx);
                target.leading_trivia.format(
                    o,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                o.format(target, ctx);
                if i < last_idx || self.with_implicit_rest {
                    o.push(',');
                }
                target.trailing_trivia.format(o);
            }
            o.write_trivia_at_virtual_end(ctx, &self.virtual_end, true, false);
            o.dedent();
            o.break_line(ctx);
            o.push(')');
        }
    }
}
