use super::{
    node::*,
    trivia::{EmptyLineHandling, LeadingTrivia, LineTrivia, TrailingTrivia},
    FormatConfig,
};
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

    pub(crate) fn execute(mut self, node: &Node, ctx: &FormatContext) -> String {
        self.format(node, ctx);
        if !self.buffer.is_empty() {
            self.break_line(ctx);
        }
        self.buffer
    }

    pub(super) fn format(&mut self, node: &Node, ctx: &FormatContext) {
        match &node.kind {
            Kind::Atom(atom) => atom.format(self),
            Kind::StringLike(str) => str.format(self),
            Kind::DynStringLike(dstr) => dstr.format(self, ctx),
            Kind::HeredocOpening(opening) => opening.format(self),
            Kind::ConstantPath(const_path) => const_path.format(self, ctx),
            Kind::Statements(statements) => statements.format(self, ctx, false),
            Kind::Parens(parens) => parens.format(self, ctx),
            Kind::If(ifexpr) => ifexpr.format(self, ctx),
            Kind::Ternary(ternary) => ternary.format(self, ctx),
            Kind::Case(case) => case.format(self, ctx),
            Kind::While(whle) => whle.format(self, ctx),
            Kind::For(expr) => expr.format(self, ctx),
            Kind::Postmodifier(modifier) => modifier.format(self, ctx),
            Kind::MethodChain(chain) => chain.format(self, ctx),
            Kind::Lambda(lambda) => lambda.format(self, ctx),
            Kind::CallLike(call) => call.format(self, ctx),
            Kind::InfixChain(chain) => chain.format(self, ctx),
            Kind::Assign(assign) => assign.format(self, ctx),
            Kind::MultiAssignTarget(multi) => multi.format(self, ctx),
            Kind::Prefix(prefix) => prefix.format(self, ctx),
            Kind::Array(array) => array.format(self, ctx),
            Kind::Hash(hash) => hash.format(self, ctx),
            Kind::Assoc(assoc) => assoc.format(self, ctx),
            Kind::Begin(begin) => begin.format(self, ctx),
            Kind::Def(def) => def.format(self, ctx),
            Kind::ClassLike(class) => class.format(self, ctx),
            Kind::SingletonClass(class) => class.format(self, ctx),
            Kind::RangeLike(range) => range.format(self, ctx),
            Kind::PrePostExec(exec) => exec.format(self, ctx),
            Kind::Alias(alias) => alias.format(self, ctx),
        }
    }

    pub(super) fn write_trivia_at_virtual_end(
        &mut self,
        ctx: &FormatContext,
        end: &Option<VirtualEnd>,
        break_first: bool,
        trim_start: bool,
    ) {
        if let Some(end) = end {
            let mut trailing_empty_lines = 0;
            let leading_lines = &end.leading_trivia.lines();
            for trivia in leading_lines.iter().rev() {
                match trivia {
                    LineTrivia::EmptyLine => {
                        trailing_empty_lines += 1;
                    }
                    LineTrivia::Comment(_) => {
                        break;
                    }
                }
            }
            if trailing_empty_lines == leading_lines.len() {
                return;
            }

            if break_first {
                self.break_line(ctx);
            }
            let target_len = leading_lines.len() - trailing_empty_lines;
            let last_idx = target_len - 1;
            for (i, trivia) in leading_lines.iter().take(target_len).enumerate() {
                match trivia {
                    LineTrivia::EmptyLine => {
                        if !(trim_start && i == 0) || i == last_idx {
                            self.break_line(ctx);
                        }
                    }
                    LineTrivia::Comment(comment) => {
                        self.push_str(&comment.value);
                        if i < last_idx {
                            self.break_line(ctx);
                        }
                    }
                }
            }
        }
    }

    pub(super) fn write_leading_trivia(
        &mut self,
        trivia: &LeadingTrivia,
        ctx: &FormatContext,
        emp_line_handling: EmptyLineHandling,
    ) {
        if trivia.is_empty() {
            return;
        }
        let last_idx = trivia.lines().len() - 1;
        for (i, trivia) in trivia.lines().iter().enumerate() {
            match trivia {
                LineTrivia::EmptyLine => {
                    let should_skip = match emp_line_handling {
                        EmptyLineHandling::Skip => true,
                        EmptyLineHandling::Trim { start, end } => {
                            (start && i == 0) || (end && i == last_idx)
                        }
                    };
                    if !should_skip {
                        self.break_line(ctx);
                    }
                }
                LineTrivia::Comment(comment) => {
                    self.push_str(&comment.value);
                    self.break_line(ctx);
                }
            }
        }
    }

    pub(super) fn write_trailing_comment(&mut self, trivia: &TrailingTrivia) {
        if let Some(comment) = &trivia.comment() {
            self.push(' ');
            self.buffer.push_str(&comment.value);
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

    pub(super) fn write_heredoc_body(&mut self, pos: &Pos, ctx: &FormatContext) {
        let heredoc = ctx.heredoc_map.get(pos).expect("heredoc must exist");
        match heredoc.indent_mode {
            HeredocIndentMode::None | HeredocIndentMode::EndIndented => {
                for part in &heredoc.parts {
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            self.push_str_without_indent(&value);
                        }
                        HeredocPart::Statements(embedded) => {
                            embedded.format(self, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            var.format(self);
                        }
                    }
                }
                if matches!(heredoc.indent_mode, HeredocIndentMode::EndIndented) {
                    self.put_indent();
                }
                self.push_str(&heredoc.id);
            }
            HeredocIndentMode::AllIndented => {
                for part in &heredoc.parts {
                    match part {
                        HeredocPart::Str(str) => {
                            // Ignore non-UTF8 source code for now.
                            let value = String::from_utf8_lossy(&str.value);
                            self.push_str_without_indent(&value);
                        }
                        HeredocPart::Statements(embedded) => {
                            embedded.format(self, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            var.format(self);
                        }
                    }
                }
                self.put_indent();
                self.push_str(&heredoc.id);
            }
        }
        self.buffer.push('\n');
    }
}
