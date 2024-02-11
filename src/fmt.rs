mod node;
mod output;
mod shape;
mod trivia;

pub(crate) use node::*;
pub(crate) use output::HeredocMap;
pub(crate) use trivia::{Comment, LeadingTrivia, LineTrivia, TrailingTrivia};

use self::output::{FormatContext, Output};

pub(crate) fn format(node: Node, heredoc_map: HeredocMap) -> String {
    let config = FormatConfig {
        line_width: 100,
        indent_size: 2,
    };
    let ctx = FormatContext { heredoc_map };
    let mut output = Output::new(config);
    node.format(&mut output, &ctx);
    if !output.buffer.is_empty() {
        output.break_line(&ctx);
    }
    output.buffer
}

#[derive(Debug)]
pub(crate) struct FormatConfig {
    line_width: usize,
    indent_size: usize,
}
