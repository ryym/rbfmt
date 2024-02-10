mod node;
mod output;
mod shape;
mod trivia;

pub(crate) use node::*;
pub(crate) use trivia::{Comment, LeadingTrivia, LineTrivia, TrailingTrivia};

use self::output::{FormatContext, Formatter};

pub(crate) fn format(node: Node, heredoc_map: HeredocMap) -> String {
    let config = FormatConfig {
        line_width: 100,
        indent_size: 2,
    };
    let ctx = FormatContext { heredoc_map };
    let formatter = Formatter::new(config);
    formatter.execute(&node, &ctx)
}

#[derive(Debug)]
struct FormatConfig {
    line_width: usize,
    indent_size: usize,
}
