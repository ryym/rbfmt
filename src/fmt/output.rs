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

    pub(super) fn format_embedded_statements(
        &mut self,
        embedded: &EmbeddedStatements,
        ctx: &FormatContext,
    ) {
        self.push_str(&embedded.opening);

        if embedded.shape.is_inline() {
            let remaining = self.remaining_width;
            self.remaining_width = usize::MAX;
            embedded.statements.format(self, ctx, false);
            self.remaining_width = remaining;
        } else {
            self.indent();
            self.break_line(ctx);
            embedded.statements.format(self, ctx, true);
            self.break_line(ctx);
            self.dedent();
        }

        self.push_str(&embedded.closing);
    }

    pub(super) fn format_embedded_variable(&mut self, var: &EmbeddedVariable) {
        self.push_str(&var.operator);
        self.push_str(&var.variable);
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

    pub(super) fn format_block(&mut self, block: &Block, ctx: &FormatContext) {
        if block.shape.fits_in_one_line(self.remaining_width) {
            self.push(' ');
            self.push_str(&block.opening);
            if let Some(params) = &block.parameters {
                self.push(' ');
                self.format_block_parameters(params, ctx);
            }
            if !block.body.shape.is_empty() {
                self.push(' ');
                self.format_block_body(&block.body, ctx, false);
                self.push(' ');
            }
            if &block.closing == "end" {
                self.push(' ');
            }
            self.push_str(&block.closing);
        } else {
            self.push(' ');
            self.push_str(&block.opening);
            self.write_trailing_comment(&block.opening_trailing);
            if let Some(params) = &block.parameters {
                if block.opening_trailing.is_none() {
                    self.push(' ');
                    self.format_block_parameters(params, ctx);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.format_block_parameters(params, ctx);
                    self.dedent();
                }
            }
            if !block.body.shape.is_empty() {
                self.format_block_body(&block.body, ctx, true);
            }
            self.break_line(ctx);
            self.push_str(&block.closing);
        }
    }

    pub(super) fn format_block_body(
        &mut self,
        body: &BlockBody,
        ctx: &FormatContext,
        block_always: bool,
    ) {
        if body.shape.fits_in_inline(self.remaining_width) && !block_always {
            body.statements.format(self, ctx, block_always);
            return;
        }

        if !body.statements.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            body.statements.format(self, ctx, true);
            self.dedent();
        }
        for rescue in &body.rescues {
            self.break_line(ctx);
            self.format_rescue(rescue, ctx);
        }
        if let Some(rescue_else) = &body.rescue_else {
            self.break_line(ctx);
            self.push_str("else");
            self.write_trailing_comment(&rescue_else.keyword_trailing);
            if !rescue_else.body.shape().is_empty() {
                self.indent();
                self.break_line(ctx);
                rescue_else.body.format(self, ctx, true);
                self.dedent();
            }
        }
        if let Some(ensure) = &body.ensure {
            self.break_line(ctx);
            self.push_str("ensure");
            self.write_trailing_comment(&ensure.keyword_trailing);
            if !ensure.body.shape().is_empty() {
                self.indent();
                self.break_line(ctx);
                ensure.body.format(self, ctx, true);
                self.dedent();
            }
        }
    }

    pub(super) fn format_rescue(&mut self, rescue: &Rescue, ctx: &FormatContext) {
        self.push_str("rescue");
        if !rescue.exceptions.is_empty() {
            if rescue
                .exceptions_shape
                .fits_in_one_line(self.remaining_width)
            {
                self.push(' ');
                for (i, exception) in rescue.exceptions.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(exception, ctx);
                    self.write_trailing_comment(&exception.trailing_trivia);
                }
            } else {
                self.push(' ');
                self.format(&rescue.exceptions[0], ctx);
                if rescue.exceptions.len() > 1 {
                    self.push(',');
                }
                self.write_trailing_comment(&rescue.exceptions[0].trailing_trivia);
                if rescue.exceptions.len() > 1 {
                    self.indent();
                    let last_idx = rescue.exceptions.len() - 1;
                    for (i, exception) in rescue.exceptions.iter().enumerate().skip(1) {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &exception.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: false,
                                end: false,
                            },
                        );
                        self.format(exception, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&exception.trailing_trivia);
                    }
                    self.dedent();
                }
            }
        }
        if let Some(reference) = &rescue.reference {
            self.push_str(" =>");
            if reference.shape.fits_in_one_line(self.remaining_width) || reference.is_diagonal() {
                self.push(' ');
                self.format(reference, ctx);
                self.write_trailing_comment(&reference.trailing_trivia);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &reference.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: false,
                    },
                );
                self.format(reference, ctx);
                self.write_trailing_comment(&reference.trailing_trivia);
                self.dedent();
            }
        }
        self.write_trailing_comment(&rescue.head_trailing);
        if !rescue.statements.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            rescue.statements.format(self, ctx, true);
            self.dedent();
        }
    }

    pub(super) fn format_method_parameters(
        &mut self,
        params: &Option<MethodParameters>,
        ctx: &FormatContext,
    ) {
        if let Some(params) = params {
            if params.shape.fits_in_one_line(self.remaining_width) {
                let opening = params.opening.as_deref().unwrap_or(" ");
                self.push_str(opening);
                for (i, n) in params.params.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
                if let Some(closing) = &params.closing {
                    self.push_str(closing);
                }
            } else {
                self.push('(');
                self.indent();
                if !params.params.is_empty() {
                    let last_idx = params.params.len() - 1;
                    for (i, n) in params.params.iter().enumerate() {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &n.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: i == 0,
                                end: false,
                            },
                        );
                        self.format(n, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&n.trailing_trivia);
                    }
                }
                self.write_trivia_at_virtual_end(
                    ctx,
                    &params.virtual_end,
                    true,
                    params.params.is_empty(),
                );
                self.dedent();
                self.break_line(ctx);
                self.push(')');
            }
        }
    }

    pub(super) fn format_block_parameters(
        &mut self,
        params: &BlockParameters,
        ctx: &FormatContext,
    ) {
        if params.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&params.opening);
            for (i, n) in params.params.iter().enumerate() {
                if n.shape.is_empty() {
                    self.push(',');
                } else {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
            }
            if !params.locals.is_empty() {
                self.push_str("; ");
                for (i, n) in params.locals.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
            }
            self.push_str(&params.closing);
            self.write_trailing_comment(&params.closing_trailing);
        } else {
            self.push_str(&params.opening);
            self.indent();
            if !params.params.is_empty() {
                let last_idx = params.params.len() - 1;
                for (i, n) in params.params.iter().enumerate() {
                    if n.shape.is_empty() {
                        self.write_trailing_comment(&n.trailing_trivia);
                        continue;
                    }
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &n.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    self.format(n, ctx);
                    if i < last_idx {
                        self.push(',');
                    }
                    self.write_trailing_comment(&n.trailing_trivia);
                }
            }
            if !params.locals.is_empty() {
                self.break_line(ctx);
                self.push(';');
                let last_idx = params.locals.len() - 1;
                for (i, n) in params.locals.iter().enumerate() {
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &n.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: false,
                            end: false,
                        },
                    );
                    self.format(n, ctx);
                    if i < last_idx {
                        self.push(',');
                    }
                    self.write_trailing_comment(&n.trailing_trivia);
                }
            }
            self.write_trivia_at_virtual_end(
                ctx,
                &params.virtual_end,
                true,
                params.params.is_empty(),
            );
            self.dedent();
            self.break_line(ctx);
            self.push_str(&params.closing);
            self.write_trailing_comment(&params.closing_trailing);
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
                            self.format_embedded_statements(embedded, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            self.format_embedded_variable(var);
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
                            self.format_embedded_statements(embedded, ctx);
                        }
                        HeredocPart::Variable(var) => {
                            self.format_embedded_variable(var);
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
