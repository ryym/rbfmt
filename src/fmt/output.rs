use super::{node::*, FormatConfig};
use std::{
    collections::{HashMap, VecDeque},
    mem,
};

pub(crate) type HeredocMap = HashMap<Pos, Heredoc>;

#[derive(Debug)]
pub(crate) struct FormatContext {
    pub heredoc_map: HeredocMap,
}

#[derive(Debug)]
struct Draft {
    index: usize,
    snapshot: OutputSnapshot,
}

#[derive(Debug)]
struct OutputSnapshot {
    buffer_len: usize,
    remaining_width: usize,
    line_count: usize,
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
}

#[derive(Debug)]
pub(crate) enum DraftResult {
    Commit,
    Rollback,
}

#[derive(Debug)]
pub(crate) struct Output {
    pub config: FormatConfig,
    pub remaining_width: usize,
    pub line_count: usize,
    pub buffer: String,
    pub indent: usize,
    pub heredoc_queue: VecDeque<Pos>,
    drafts: Vec<Draft>,
}

impl Output {
    pub(crate) fn new(config: FormatConfig) -> Self {
        Self {
            remaining_width: config.line_width,
            line_count: 0,
            config,
            buffer: String::new(),
            indent: 0,
            heredoc_queue: VecDeque::new(),
            drafts: vec![],
        }
    }

    pub(super) fn draft(&mut self, mut f: impl FnMut(&mut Self) -> DraftResult) -> DraftResult {
        let index = self.drafts.len();
        let draft = Draft {
            index,
            snapshot: OutputSnapshot {
                buffer_len: self.buffer.len(),
                remaining_width: self.remaining_width,
                line_count: self.line_count,
                indent: self.indent,
                heredoc_queue: self.heredoc_queue.clone(),
            },
        };
        self.drafts.push(draft);
        let result = f(self);
        let draft = self.drafts.pop();
        match draft {
            Some(draft) if draft.index == index => match result {
                DraftResult::Commit => {}
                DraftResult::Rollback => {
                    self.buffer.truncate(draft.snapshot.buffer_len);
                    self.remaining_width = draft.snapshot.remaining_width;
                    self.line_count = draft.snapshot.line_count;
                    self.indent = draft.snapshot.indent;
                    self.heredoc_queue = draft.snapshot.heredoc_queue;
                }
            },
            _ => panic!("invalid draft state: {:?} finished in {}", draft, index),
        };
        result
    }

    pub(super) fn write_trivia_at_virtual_end(
        &mut self,
        ctx: &FormatContext,
        end: &Option<VirtualEnd>,
        break_first: bool,
        trim_start: bool,
    ) {
        if let Some(end) = end {
            end.format(self, ctx, break_first, trim_start);
        }
    }

    pub(super) fn push(&mut self, c: char) {
        if self.remaining_width == self.config.line_width {
            self.put_indent();
        }
        self.buffer.push(c);
        self.remaining_width = self.remaining_width.saturating_sub(1);
    }

    pub(super) fn push_str(&mut self, str: &str) {
        if self.remaining_width == self.config.line_width {
            self.put_indent();
        }
        self.push_str_without_indent(str);
    }

    pub(super) fn push_str_without_indent(&mut self, str: &str) {
        self.buffer.push_str(str);
        self.remaining_width = self.remaining_width.saturating_sub(str.len());
    }

    pub(super) fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
        self.remaining_width = self.remaining_width.saturating_sub(spaces.len());
    }

    pub(super) fn indent(&mut self) {
        self.indent += self.config.indent_size;
    }

    pub(super) fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(self.config.indent_size);
    }

    pub(super) fn break_line(&mut self, ctx: &FormatContext) {
        self.buffer.push('\n');
        self.remaining_width = self.config.line_width;
        self.line_count += 1;
        let mut queue = mem::take(&mut self.heredoc_queue);
        while let Some(pos) = queue.pop_front() {
            self.write_heredoc_body(&pos, ctx);
        }
    }

    fn write_heredoc_body(&mut self, pos: &Pos, ctx: &FormatContext) {
        let heredoc = ctx.heredoc_map.get(pos).expect("heredoc must exist");
        heredoc.format(self, ctx);
        self.buffer.push('\n');
    }
}
