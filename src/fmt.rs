mod node;
mod shape;
mod trivia;

pub(crate) use node::*;
pub(crate) use trivia::{Comment, LeadingTrivia, LineTrivia, TrailingTrivia};

use self::{
    shape::{ArgumentStyle, Shape},
    trivia::EmptyLineHandling,
};
use std::{collections::VecDeque, mem};

pub(crate) fn format(node: Node, heredoc_map: HeredocMap) -> String {
    let config = FormatConfig {
        line_width: 100,
        indent_size: 2,
    };
    let ctx = FormatContext { heredoc_map };
    let mut formatter = Formatter {
        remaining_width: config.line_width,
        line_count: 0,
        config,
        buffer: String::new(),
        indent: 0,
        heredoc_queue: VecDeque::new(),
        drafts: vec![],
    };
    formatter.format(&node, &ctx);
    if !formatter.buffer.is_empty() {
        formatter.break_line(&ctx);
    }
    formatter.buffer
}

#[derive(Debug)]
struct FormatConfig {
    line_width: usize,
    indent_size: usize,
}

#[derive(Debug)]
struct FormatContext {
    heredoc_map: HeredocMap,
}

#[derive(Debug)]
struct FormatDraft {
    index: usize,
    snapshot: FormatStateSnapshot,
}

#[derive(Debug)]
struct FormatStateSnapshot {
    buffer_len: usize,
    remaining_width: usize,
    line_count: usize,
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
}

#[derive(Debug)]
enum DraftResult {
    Commit,
    Rollback,
}

#[derive(Debug)]
struct Formatter {
    config: FormatConfig,
    remaining_width: usize,
    line_count: usize,
    buffer: String,
    indent: usize,
    heredoc_queue: VecDeque<Pos>,
    drafts: Vec<FormatDraft>,
}

impl Formatter {
    fn draft(&mut self, mut f: impl FnMut(&mut Self) -> DraftResult) -> DraftResult {
        let index = self.drafts.len();
        let draft = FormatDraft {
            index,
            snapshot: FormatStateSnapshot {
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

    fn format(&mut self, node: &Node, ctx: &FormatContext) {
        match &node.kind {
            Kind::Atom(atom) => self.format_atom(atom),
            Kind::StringLike(str) => self.format_string_like(str),
            Kind::DynStringLike(dstr) => self.format_dyn_string_like(dstr, ctx),
            Kind::HeredocOpening(opening) => self.format_heredoc_opening(opening),
            Kind::ConstantPath(const_path) => self.format_constant_path(const_path, ctx),
            Kind::Statements(statements) => self.format_statements(statements, ctx, false),
            Kind::Parens(parens) => self.format_parens(parens, ctx),
            Kind::If(ifexpr) => self.format_if(ifexpr, ctx),
            Kind::Ternary(ternary) => self.format_ternary(ternary, ctx),
            Kind::Case(case) => self.format_case(case, ctx),
            Kind::While(whle) => self.format_while(whle, ctx),
            Kind::For(expr) => self.format_for(expr, ctx),
            Kind::Postmodifier(modifier) => self.format_postmodifier(modifier, ctx),
            Kind::MethodChain(chain) => self.format_method_chain(chain, ctx),
            Kind::Lambda(lambda) => self.format_lambda(lambda, ctx),
            Kind::CallLike(call) => self.format_call_like(call, ctx),
            Kind::InfixChain(chain) => self.format_infix_chain(chain, ctx),
            Kind::Assign(assign) => self.format_assign(assign, ctx),
            Kind::MultiAssignTarget(multi) => self.format_multi_assign_target(multi, ctx),
            Kind::Prefix(prefix) => self.format_prefix(prefix, ctx),
            Kind::Array(array) => self.format_array(array, ctx),
            Kind::Hash(hash) => self.format_hash(hash, ctx),
            Kind::Assoc(assoc) => self.format_assoc(assoc, ctx),
            Kind::Begin(begin) => self.format_begin(begin, ctx),
            Kind::Def(def) => self.format_def(def, ctx),
            Kind::ClassLike(class) => self.format_class_like(class, ctx),
            Kind::SingletonClass(class) => self.format_singleton_class(class, ctx),
            Kind::RangeLike(range) => self.format_range_like(range, ctx),
            Kind::PrePostExec(exec) => self.format_pre_post_exec(exec, ctx),
            Kind::Alias(alias) => self.format_alias(alias, ctx),
        }
    }

    fn format_atom(&mut self, atom: &Atom) {
        self.push_str(&atom.0);
    }

    fn format_string_like(&mut self, str: &StringLike) {
        // Ignore non-UTF8 source code for now.
        let value = String::from_utf8_lossy(&str.value);
        if let Some(opening) = &str.opening {
            self.push_str(opening);
        }
        self.push_str(&value);
        if let Some(closing) = &str.closing {
            self.push_str(closing);
        }
    }

    fn format_dyn_string_like(&mut self, dstr: &DynStringLike, ctx: &FormatContext) {
        if let Some(opening) = &dstr.opening {
            self.push_str(opening);
        }
        let mut divided = false;
        for part in &dstr.parts {
            if divided {
                self.push(' ');
            }
            match part {
                DynStrPart::Str(str) => {
                    divided = str.opening.is_some();
                    self.format_string_like(str);
                }
                DynStrPart::DynStr(dstr) => {
                    divided = true;
                    self.format_dyn_string_like(dstr, ctx);
                }
                DynStrPart::Statements(embedded) => {
                    self.format_embedded_statements(embedded, ctx);
                }
                DynStrPart::Variable(var) => {
                    self.format_embedded_variable(var);
                }
            }
        }
        if let Some(closing) = &dstr.closing {
            self.push_str(closing);
        }
    }

    fn format_embedded_statements(&mut self, embedded: &EmbeddedStatements, ctx: &FormatContext) {
        self.push_str(&embedded.opening);

        if embedded.shape.is_inline() {
            let remaining = self.remaining_width;
            self.remaining_width = usize::MAX;
            self.format_statements(&embedded.statements, ctx, false);
            self.remaining_width = remaining;
        } else {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&embedded.statements, ctx, true);
            self.break_line(ctx);
            self.dedent();
        }

        self.push_str(&embedded.closing);
    }

    fn format_embedded_variable(&mut self, var: &EmbeddedVariable) {
        self.push_str(&var.operator);
        self.push_str(&var.variable);
    }

    fn format_heredoc_opening(&mut self, opening: &HeredocOpening) {
        self.push_str(opening.indent_mode.prefix_symbols());
        self.push_str(&opening.id);
        self.heredoc_queue.push_back(opening.pos);
    }

    fn format_constant_path(&mut self, const_path: &ConstantPath, ctx: &FormatContext) {
        if let Some(root) = &const_path.root {
            self.format(root, ctx);
        }
        self.push_str("::");
        let last_idx = const_path.parts.len() - 1;
        for (i, (leading, path)) in const_path.parts.iter().enumerate() {
            if leading.is_empty() {
                self.push_str(path);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    leading,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.push_str(path);
                self.dedent();
            }
            if i < last_idx {
                self.push_str("::");
            }
        }
    }

    fn format_statements(
        &mut self,
        statements: &Statements,
        ctx: &FormatContext,
        block_always: bool,
    ) {
        if statements.shape.is_inline() && !block_always {
            if let Some(node) = statements.nodes.get(0) {
                self.format(node, ctx);
            }
            return;
        }
        for (i, n) in statements.nodes.iter().enumerate() {
            if i > 0 {
                self.break_line(ctx);
            }
            self.write_leading_trivia(
                &n.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: i == 0,
                    end: false,
                },
            );
            self.format(n, ctx);
            self.write_trailing_comment(&n.trailing_trivia);
        }
        self.write_trivia_at_virtual_end(
            ctx,
            &statements.virtual_end,
            !statements.nodes.is_empty(),
            statements.nodes.is_empty(),
        );
    }

    fn format_parens(&mut self, parens: &Parens, ctx: &FormatContext) {
        if parens.body.shape().is_empty() {
            self.push_str("()");
        } else {
            self.push('(');
            if parens.body.shape.fits_in_inline(self.remaining_width) {
                self.format_statements(&parens.body, ctx, false);
            } else {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&parens.body, ctx, true);
                self.dedent();
                self.break_line(ctx);
            }
            self.push(')');
        }
    }

    fn write_trivia_at_virtual_end(
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

    fn format_if(&mut self, expr: &If, ctx: &FormatContext) {
        if expr.is_if {
            self.push_str("if");
        } else {
            self.push_str("unless");
        }

        self.format_conditional(&expr.if_first, ctx);
        if !expr.if_first.body.shape.is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&expr.if_first.body, ctx, true);
            self.dedent();
        }

        for elsif in &expr.elsifs {
            self.break_line(ctx);
            self.push_str("elsif");
            self.format_conditional(elsif, ctx);
            if !elsif.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&elsif.body, ctx, true);
                self.dedent();
            }
        }

        if let Some(if_last) = &expr.if_last {
            self.break_line(ctx);
            self.push_str("else");
            self.write_trailing_comment(&if_last.keyword_trailing);
            if !if_last.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&if_last.body, ctx, true);
                self.dedent();
            }
        }

        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_case(&mut self, case: &Case, ctx: &FormatContext) {
        self.push_str("case");
        match &case.predicate {
            Some(pred) => {
                if pred.shape.fits_in_one_line(self.remaining_width) || pred.is_diagonal() {
                    self.push(' ');
                    self.format(pred, ctx);
                    self.write_trailing_comment(&pred.trailing_trivia);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &pred.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: true,
                            end: true,
                        },
                    );
                    self.format(pred, ctx);
                    self.write_trailing_comment(&pred.trailing_trivia);
                    self.dedent();
                }
            }
            None => {
                self.write_trailing_comment(&case.case_trailing);
            }
        }
        if case.first_branch_leading.is_empty() {
            self.break_line(ctx);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &case.first_branch_leading,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.dedent();
        }
        for (i, branch) in case.branches.iter().enumerate() {
            if i > 0 {
                self.break_line(ctx);
            }
            self.format_case_when(branch, ctx);
        }
        if let Some(otherwise) = &case.otherwise {
            self.break_line(ctx);
            self.push_str("else");
            self.write_trailing_comment(&otherwise.keyword_trailing);
            if !otherwise.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&otherwise.body, ctx, true);
                self.dedent();
            }
        }
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_case_when(&mut self, when: &CaseWhen, ctx: &FormatContext) {
        self.push_str("when");
        if when.shape.fits_in_one_line(self.remaining_width) {
            self.push(' ');
            for (i, cond) in when.conditions.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(cond, ctx);
                self.write_trailing_comment(&cond.trailing_trivia);
            }
            if !when.body.shape.is_empty() {
                self.push_str(" then ");
                self.format_statements(&when.body, ctx, false);
            }
        } else {
            if when.conditions_shape.fits_in_one_line(self.remaining_width) {
                for (i, cond) in when.conditions.iter().enumerate() {
                    if i == 0 {
                        self.push(' ');
                    } else {
                        self.push_str(", ");
                    }
                    self.format(cond, ctx);
                    self.write_trailing_comment(&cond.trailing_trivia);
                }
            } else {
                if when.conditions[0].is_diagonal() {
                    self.push(' ');
                    self.format(&when.conditions[0], ctx);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &when.conditions[0].leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: true,
                            end: true,
                        },
                    );
                    self.format(&when.conditions[0], ctx);
                    self.dedent();
                }
                if when.conditions.len() > 1 {
                    self.push(',');
                }
                self.write_trailing_comment(&when.conditions[0].trailing_trivia);
                if when.conditions.len() > 1 {
                    self.indent();
                    let last_idx = when.conditions.len() - 1;
                    for (i, cond) in when.conditions.iter().enumerate().skip(1) {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &cond.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: false,
                                end: false,
                            },
                        );
                        self.format(cond, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&cond.trailing_trivia);
                    }
                    self.dedent();
                }
            }
            if !when.body.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&when.body, ctx, true);
                self.dedent();
            }
        }
    }

    fn format_ternary(&mut self, tern: &Ternary, ctx: &FormatContext) {
        // Format `predicate`.
        self.format(&tern.predicate, ctx);
        self.push_str(" ?");

        // Format `then`.
        if tern.predicate_trailing.is_none()
            && tern.then.shape.fits_in_one_line(self.remaining_width)
        {
            self.push(' ');
            self.format(&tern.then, ctx);
            self.write_trailing_comment(&tern.then.trailing_trivia);
        } else {
            self.write_trailing_comment(&tern.predicate_trailing);
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &tern.then.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&tern.then, ctx);
            self.write_trailing_comment(&tern.then.trailing_trivia);
            self.dedent();
        }

        // Format `otherwise`.
        if tern.predicate_trailing.is_none()
            && tern.then.shape.is_inline()
            && tern.otherwise.shape.fits_in_one_line(self.remaining_width)
        {
            self.push_str(" : ");
            self.format(&tern.otherwise, ctx);
            self.write_trailing_comment(&tern.otherwise.trailing_trivia);
        } else {
            self.break_line(ctx);
            self.push(':');
            if tern.otherwise.shape.fits_in_one_line(self.remaining_width)
                || tern.otherwise.is_diagonal()
            {
                self.push(' ');
                self.format(&tern.otherwise, ctx);
                self.write_trailing_comment(&tern.otherwise.trailing_trivia);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &tern.otherwise.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.format(&tern.otherwise, ctx);
                self.write_trailing_comment(&tern.otherwise.trailing_trivia);
                self.dedent();
            }
        }
    }

    fn format_while(&mut self, expr: &While, ctx: &FormatContext) {
        if expr.is_while {
            self.push_str("while");
        } else {
            self.push_str("until");
        }
        self.format_conditional(&expr.content, ctx);
        if !expr.content.body.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&expr.content.body, ctx, true);
            self.dedent();
        }
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_for(&mut self, expr: &For, ctx: &FormatContext) {
        self.push_str("for");
        if expr.index.shape.fits_in_inline(self.remaining_width) || expr.index.is_diagonal() {
            self.push(' ');
            self.format(&expr.index, ctx);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &expr.index.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&expr.index, ctx);
            self.dedent();
        }
        self.push_str(" in");
        let collection = &expr.collection;
        if collection.shape.fits_in_inline(self.remaining_width) || collection.is_diagonal() {
            self.push(' ');
            self.format(collection, ctx);
            self.write_trailing_comment(&collection.trailing_trivia);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &collection.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(collection, ctx);
            self.write_trailing_comment(&collection.trailing_trivia);
            self.dedent();
        }
        if !expr.body.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&expr.body, ctx, true);
            self.dedent();
        }
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_postmodifier(&mut self, modifier: &Postmodifier, ctx: &FormatContext) {
        self.format_statements(&modifier.conditional.body, ctx, false);
        self.push(' ');
        self.push_str(&modifier.keyword);
        let cond = &modifier.conditional;
        if cond.predicate.is_diagonal() {
            self.push(' ');
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
        } else {
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &cond.predicate.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
            self.dedent();
        }
    }

    fn format_conditional(&mut self, cond: &Conditional, ctx: &FormatContext) {
        if cond.predicate.is_diagonal() {
            self.push(' ');
            self.indent();
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
            self.dedent();
        } else {
            self.indent();
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &cond.predicate.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&cond.predicate, ctx);
            self.write_trailing_comment(&cond.predicate.trailing_trivia);
            self.dedent();
            self.dedent();
        }
    }

    fn format_method_chain(&mut self, chain: &MethodChain, ctx: &FormatContext) {
        match &chain.head {
            MethodChainHead::Receiver(receiver) => {
                self.format(&receiver.node, ctx);
                self.write_trailing_comment(&receiver.node.trailing_trivia);
                for idx_call in &receiver.index_calls {
                    self.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        self.format_block(block, ctx);
                    }
                }
            }
            MethodChainHead::FirstCall(call) => {
                self.push_str(&call.name);
                if let Some(args) = &call.arguments {
                    self.format_arguments(args, ctx);
                }
                if let Some(block) = &call.block {
                    self.format_block(block, ctx);
                }
                for idx_call in &call.index_calls {
                    self.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        self.format_block(block, ctx);
                    }
                }
                self.write_trailing_comment(&call.trailing_trivia);
            }
        }
        if chain.calls.is_empty() {
            return;
        }

        // Format horizontally if all these are met:
        //   - no intermediate comments
        //   - one or zero blocks
        //   - only one multilines arguments or block
        //   - no more arguments after multilines call
        // Problems:
        //   - The format can change to vertical by a subtle modification
        //   - Sometimes the vertical format is more beautiful
        let draft_result = self.draft(|d| {
            if chain.head.has_trailing_trivia() {
                return DraftResult::Rollback;
            }
            let mut call_expanded = false;
            let mut non_empty_block_exists = false;
            let last_idx = chain.calls.len() - 1;
            for (i, call) in chain.calls.iter().enumerate() {
                if i < last_idx && !call.trailing_trivia.is_none() {
                    return DraftResult::Rollback;
                }
                match call.min_first_line_len() {
                    Some(len) if len <= d.remaining_width => {}
                    _ => return DraftResult::Rollback,
                };
                let prev_line_count = d.line_count;
                if let Some(call_op) = &call.operator {
                    d.push_str(call_op);
                }
                d.push_str(&call.name);
                if let Some(args) = &call.arguments {
                    if !args.is_empty() && call_expanded {
                        return DraftResult::Rollback;
                    }
                    d.format_arguments(args, ctx);
                }
                if let Some(block) = &call.block {
                    if !block.is_empty() {
                        if call_expanded {
                            return DraftResult::Rollback;
                        }
                        if !non_empty_block_exists {
                            non_empty_block_exists = true
                        } else {
                            return DraftResult::Rollback;
                        }
                    }
                    d.format_block(block, ctx);
                }
                for idx_call in &call.index_calls {
                    // XXX: Handle single arg index as non-breakable
                    if !idx_call.arguments.is_empty() && call_expanded {
                        return DraftResult::Rollback;
                    }
                    d.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        d.format_block(block, ctx);
                    }
                }
                if prev_line_count < d.line_count {
                    call_expanded = true
                }
            }
            DraftResult::Commit
        });

        if !matches!(draft_result, DraftResult::Commit) {
            self.indent();
            for call in chain.calls.iter() {
                if let Some(call_op) = &call.operator {
                    self.break_line(ctx);
                    self.write_leading_trivia(&call.leading_trivia, ctx, EmptyLineHandling::Skip);
                    self.push_str(call_op);
                }
                self.push_str(&call.name);
                if let Some(args) = &call.arguments {
                    self.format_arguments(args, ctx);
                }
                if let Some(block) = &call.block {
                    self.format_block(block, ctx);
                }
                for idx_call in &call.index_calls {
                    self.format_arguments(&idx_call.arguments, ctx);
                    if let Some(block) = &idx_call.block {
                        self.format_block(block, ctx);
                    }
                }
                self.write_trailing_comment(&call.trailing_trivia);
            }
            self.dedent();
        }
    }

    fn format_call_like(&mut self, call: &CallLike, ctx: &FormatContext) {
        self.push_str(&call.name);
        if let Some(args) = &call.arguments {
            self.format_arguments(args, ctx);
        }
    }

    fn format_arguments(&mut self, args: &Arguments, ctx: &FormatContext) {
        // Format horizontally if all these are met:
        //   - no intermediate comments
        //   - all nodes' ArgumentStyle is horizontal
        //   - only the last argument can span in multilines
        let draft_result = self.draft(|d| {
            if args.virtual_end.is_some() {
                return DraftResult::Rollback;
            }
            d.push_str(args.opening.as_ref().map_or(" ", |s| s));
            for (i, arg) in args.nodes.iter().enumerate() {
                if i > 0 {
                    d.push_str(", ");
                }
                if matches!(arg.shape, Shape::LineEnd { .. }) {
                    return DraftResult::Rollback;
                }
                match arg.argument_style() {
                    ArgumentStyle::Vertical => match arg.shape {
                        Shape::Inline { len } if len <= d.remaining_width => {
                            d.format(arg, ctx);
                        }
                        _ => return DraftResult::Rollback,
                    },
                    ArgumentStyle::Horizontal { min_first_line_len } => {
                        if d.remaining_width < min_first_line_len {
                            return DraftResult::Rollback;
                        }
                        let prev_line_count = d.line_count;
                        d.format(arg, ctx);
                        if prev_line_count < d.line_count && i < args.nodes.len() - 1 {
                            return DraftResult::Rollback;
                        }
                    }
                }
            }
            if let Some(closing) = &args.closing {
                if d.remaining_width < closing.len() {
                    return DraftResult::Rollback;
                }
                d.push_str(closing);
            }
            DraftResult::Commit
        });

        if matches!(draft_result, DraftResult::Commit) {
            return;
        }

        if let Some(opening) = &args.opening {
            self.push_str(opening);
            self.indent();
            if !args.nodes.is_empty() {
                let last_idx = args.nodes.len() - 1;
                for (i, arg) in args.nodes.iter().enumerate() {
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &arg.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: i == 0,
                            end: false,
                        },
                    );
                    self.format(arg, ctx);
                    if i < last_idx || args.last_comma_allowed {
                        self.push(',');
                    }
                    self.write_trailing_comment(&arg.trailing_trivia);
                }
            }
            self.write_trivia_at_virtual_end(ctx, &args.virtual_end, true, args.nodes.is_empty());
            self.dedent();
            self.break_line(ctx);
            if let Some(closing) = &args.closing {
                self.push_str(closing);
            }
        } else if !args.nodes.is_empty() {
            self.push(' ');
            self.format(&args.nodes[0], ctx);
            if args.nodes.len() > 1 {
                self.push(',');
            }
            self.write_trailing_comment(&args.nodes[0].trailing_trivia);
            match args.nodes.len() {
                1 => {}
                2 if args.nodes[0].trailing_trivia.is_none()
                    && args.nodes[1].shape.fits_in_one_line(self.remaining_width) =>
                {
                    self.push(' ');
                    self.format(&args.nodes[1], ctx);
                }
                _ => {
                    self.indent();
                    let last_idx = args.nodes.len() - 1;
                    for (i, arg) in args.nodes.iter().enumerate().skip(1) {
                        self.break_line(ctx);
                        self.write_leading_trivia(
                            &arg.leading_trivia,
                            ctx,
                            EmptyLineHandling::Trim {
                                start: i == 0,
                                end: false,
                            },
                        );
                        self.format(arg, ctx);
                        if i < last_idx {
                            self.push(',');
                        }
                        self.write_trailing_comment(&arg.trailing_trivia);
                    }
                    self.dedent();
                }
            }
        }
    }

    fn format_block(&mut self, block: &Block, ctx: &FormatContext) {
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

    fn format_lambda(&mut self, lambda: &Lambda, ctx: &FormatContext) {
        self.push_str("->");
        if let Some(params) = &lambda.parameters {
            self.format_block_parameters(params, ctx);
        }
        self.format_block(&lambda.block, ctx);
    }

    fn format_infix_chain(&mut self, chain: &InfixChain, ctx: &FormatContext) {
        self.format(&chain.left, ctx);
        if chain.rights_shape.fits_in_one_line(self.remaining_width) {
            for right in &chain.rights {
                self.push(' ');
                self.push_str(&right.operator);
                self.push(' ');
                self.format(&right.value, ctx);
            }
        } else {
            for right in &chain.rights {
                self.push(' ');
                self.push_str(&right.operator);
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &right.value.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: false,
                        end: false,
                    },
                );
                self.format(&right.value, ctx);
                self.dedent();
            }
        }
    }

    fn format_assign(&mut self, assign: &Assign, ctx: &FormatContext) {
        self.format(&assign.target, ctx);
        self.push(' ');
        self.push_str(&assign.operator);
        self.format_assign_right(&assign.value, ctx);
    }

    fn format_assign_right(&mut self, value: &Node, ctx: &FormatContext) {
        if value.shape.fits_in_one_line(self.remaining_width) || value.is_diagonal() {
            self.push(' ');
            self.format(value, ctx);
        } else {
            self.break_line(ctx);
            self.indent();
            self.write_leading_trivia(
                &value.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(value, ctx);
            self.dedent();
        }
    }

    fn format_multi_assign_target(&mut self, multi: &MultiAssignTarget, ctx: &FormatContext) {
        if multi.shape.fits_in_inline(self.remaining_width) {
            if let Some(lparen) = &multi.lparen {
                self.push_str(lparen);
            }
            for (i, target) in multi.targets.iter().enumerate() {
                if i > 0 {
                    self.push_str(", ");
                }
                self.format(target, ctx);
            }
            if multi.with_implicit_rest {
                self.push(',');
            }
            if let Some(rparen) = &multi.rparen {
                self.push_str(rparen);
            }
        } else {
            self.push('(');
            self.indent();
            let last_idx = multi.targets.len() - 1;
            for (i, target) in multi.targets.iter().enumerate() {
                self.break_line(ctx);
                self.write_leading_trivia(
                    &target.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.format(target, ctx);
                if i < last_idx || multi.with_implicit_rest {
                    self.push(',');
                }
                self.write_trailing_comment(&target.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(ctx, &multi.virtual_end, true, false);
            self.dedent();
            self.break_line(ctx);
            self.push(')');
        }
    }

    fn format_prefix(&mut self, prefix: &Prefix, ctx: &FormatContext) {
        self.push_str(&prefix.operator);
        if let Some(expr) = &prefix.expression {
            self.format(expr, ctx);
        }
    }

    fn format_array(&mut self, array: &Array, ctx: &FormatContext) {
        if array.shape.fits_in_one_line(self.remaining_width) {
            if let Some(opening) = &array.opening {
                self.push_str(opening);
            }
            for (i, n) in array.elements.iter().enumerate() {
                if i > 0 {
                    self.push_str(array.separator());
                    self.push(' ');
                }
                self.format(n, ctx);
            }
            if let Some(closing) = &array.closing {
                self.push_str(closing);
            }
        } else {
            self.push_str(array.opening.as_deref().unwrap_or("["));
            self.indent();
            for (i, element) in array.elements.iter().enumerate() {
                self.break_line(ctx);
                self.write_leading_trivia(
                    &element.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.format(element, ctx);
                self.push_str(array.separator());
                self.write_trailing_comment(&element.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(
                ctx,
                &array.virtual_end,
                true,
                array.elements.is_empty(),
            );
            self.dedent();
            self.break_line(ctx);
            self.push_str(array.closing.as_deref().unwrap_or("]"));
        }
    }

    fn format_hash(&mut self, hash: &Hash, ctx: &FormatContext) {
        if hash.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&hash.opening);
            if !hash.elements.is_empty() {
                self.push(' ');
                for (i, n) in hash.elements.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format(n, ctx);
                }
                self.push(' ');
            }
            self.push_str(&hash.closing);
        } else {
            self.push_str(&hash.opening);
            self.indent();
            for (i, element) in hash.elements.iter().enumerate() {
                self.break_line(ctx);
                self.write_leading_trivia(
                    &element.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: i == 0,
                        end: false,
                    },
                );
                self.format(element, ctx);
                self.push(',');
                self.write_trailing_comment(&element.trailing_trivia);
            }
            self.write_trivia_at_virtual_end(
                ctx,
                &hash.virtual_end,
                true,
                hash.elements.is_empty(),
            );
            self.dedent();
            self.break_line(ctx);
            self.push_str(&hash.closing);
        }
    }

    fn format_assoc(&mut self, assoc: &Assoc, ctx: &FormatContext) {
        self.format(&assoc.key, ctx);
        if assoc.value.shape.fits_in_inline(self.remaining_width) || assoc.value.is_diagonal() {
            if let Some(op) = &assoc.operator {
                self.push(' ');
                self.push_str(op);
            }
            if !assoc.value.shape.is_empty() {
                self.push(' ');
                self.format(&assoc.value, ctx);
            }
        } else {
            if let Some(op) = &assoc.operator {
                self.push(' ');
                self.push_str(op);
            }
            self.break_line(ctx);
            self.indent();
            self.write_leading_trivia(
                &assoc.value.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: true,
                    end: true,
                },
            );
            self.format(&assoc.value, ctx);
            self.dedent();
        }
    }

    fn format_begin(&mut self, begin: &Begin, ctx: &FormatContext) {
        self.push_str("begin");
        self.write_trailing_comment(&begin.keyword_trailing);
        self.format_block_body(&begin.body, ctx, true);
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_def(&mut self, def: &Def, ctx: &FormatContext) {
        self.push_str("def");
        if let Some(receiver) = &def.receiver {
            if receiver.shape.fits_in_one_line(self.remaining_width) || receiver.is_diagonal() {
                self.push(' ');
                self.format(receiver, ctx);
            } else {
                self.indent();
                self.break_line(ctx);
                // no leading trivia here.
                self.format(receiver, ctx);
            }
            self.push('.');
            if receiver.trailing_trivia.is_none() {
                self.push_str(&def.name);
                self.format_method_parameters(&def.parameters, ctx);
            } else {
                self.write_trailing_comment(&receiver.trailing_trivia);
                self.indent();
                self.break_line(ctx);
                self.push_str(&def.name);
                self.format_method_parameters(&def.parameters, ctx);
                self.dedent();
            }
        } else {
            self.push(' ');
            self.push_str(&def.name);
            self.format_method_parameters(&def.parameters, ctx);
        }
        match &def.body {
            // def foo = body
            DefBody::Short { body } => {
                self.push_str(" =");
                if body.shape.fits_in_one_line(self.remaining_width) || body.is_diagonal() {
                    self.push(' ');
                    self.format(body, ctx);
                    self.write_trailing_comment(&body.trailing_trivia);
                } else {
                    self.indent();
                    self.break_line(ctx);
                    self.write_leading_trivia(
                        &body.leading_trivia,
                        ctx,
                        EmptyLineHandling::Trim {
                            start: true,
                            end: true,
                        },
                    );
                    self.format(body, ctx);
                    self.write_trailing_comment(&body.trailing_trivia);
                    self.dedent();
                }
            }
            // def foo\n body\n end
            DefBody::Block {
                head_trailing,
                body,
            } => {
                self.write_trailing_comment(head_trailing);
                self.format_block_body(body, ctx, true);
                self.break_line(ctx);
                self.push_str("end");
            }
        }
    }

    fn format_block_body(&mut self, body: &BlockBody, ctx: &FormatContext, block_always: bool) {
        if body.shape.fits_in_inline(self.remaining_width) && !block_always {
            self.format_statements(&body.statements, ctx, block_always);
            return;
        }

        if !body.statements.shape().is_empty() {
            self.indent();
            self.break_line(ctx);
            self.format_statements(&body.statements, ctx, true);
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
                self.format_statements(&rescue_else.body, ctx, true);
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
                self.format_statements(&ensure.body, ctx, true);
                self.dedent();
            }
        }
    }

    fn format_rescue(&mut self, rescue: &Rescue, ctx: &FormatContext) {
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
            self.format_statements(&rescue.statements, ctx, true);
            self.dedent();
        }
    }

    fn format_method_parameters(&mut self, params: &Option<MethodParameters>, ctx: &FormatContext) {
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

    fn format_block_parameters(&mut self, params: &BlockParameters, ctx: &FormatContext) {
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

    fn format_class_like(&mut self, class: &ClassLike, ctx: &FormatContext) {
        self.push_str(&class.keyword);
        self.push(' ');
        self.push_str(&class.name);
        if let Some(superclass) = &class.superclass {
            self.push_str(" <");
            if superclass.shape.fits_in_one_line(self.remaining_width) || superclass.is_diagonal() {
                self.push(' ');
                self.format(superclass, ctx);
                self.write_trailing_comment(&superclass.trailing_trivia);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &superclass.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.format(superclass, ctx);
                self.write_trailing_comment(&superclass.trailing_trivia);
                self.dedent();
            }
        } else {
            self.write_trailing_comment(&class.head_trailing);
        }
        self.format_block_body(&class.body, ctx, true);
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_singleton_class(&mut self, class: &SingletonClass, ctx: &FormatContext) {
        self.push_str("class <<");
        if class
            .expression
            .shape
            .fits_in_one_line(self.remaining_width)
            || class.expression.is_diagonal()
        {
            self.push(' ');
            self.format(&class.expression, ctx);
            self.write_trailing_comment(&class.expression.trailing_trivia);
        } else {
            self.indent();
            self.indent();
            self.break_line(ctx);
            self.write_leading_trivia(
                &class.expression.leading_trivia,
                ctx,
                EmptyLineHandling::Trim {
                    start: false,
                    end: false,
                },
            );
            self.format(&class.expression, ctx);
            self.write_trailing_comment(&class.expression.trailing_trivia);
            self.dedent();
            self.dedent();
        }
        self.format_block_body(&class.body, ctx, true);
        self.break_line(ctx);
        self.push_str("end");
    }

    fn format_range_like(&mut self, range: &RangeLike, ctx: &FormatContext) {
        if let Some(left) = &range.left {
            self.format(left, ctx);
        }
        self.push_str(&range.operator);
        if let Some(right) = &range.right {
            if right.shape.fits_in_one_line(self.remaining_width) || right.is_diagonal() {
                let need_space = match &right.kind {
                    Kind::RangeLike(range) => range.left.is_none(),
                    _ => false,
                };
                if need_space {
                    self.push(' ');
                }
                self.format(right, ctx);
            } else {
                self.indent();
                self.break_line(ctx);
                self.write_leading_trivia(
                    &right.leading_trivia,
                    ctx,
                    EmptyLineHandling::Trim {
                        start: true,
                        end: true,
                    },
                );
                self.format(right, ctx);
                self.dedent();
            }
        }
    }

    fn format_pre_post_exec(&mut self, exec: &PrePostExec, ctx: &FormatContext) {
        if exec.shape.fits_in_one_line(self.remaining_width) {
            self.push_str(&exec.keyword);
            self.push_str(" {");
            if !exec.statements.shape.is_empty() {
                self.push(' ');
                self.format_statements(&exec.statements, ctx, false);
                self.push(' ');
            }
            self.push('}');
        } else {
            self.push_str(&exec.keyword);
            self.push_str(" {");
            if !exec.statements.shape.is_empty() {
                self.indent();
                self.break_line(ctx);
                self.format_statements(&exec.statements, ctx, true);
                self.dedent();
            }
            self.break_line(ctx);
            self.push('}');
        }
    }

    fn format_alias(&mut self, alias: &Alias, ctx: &FormatContext) {
        self.push_str("alias ");
        self.format(&alias.new_name, ctx);
        self.push(' ');
        self.format(&alias.old_name, ctx);
    }

    fn write_leading_trivia(
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

    fn write_trailing_comment(&mut self, trivia: &TrailingTrivia) {
        if let Some(comment) = &trivia.comment() {
            self.push(' ');
            self.buffer.push_str(&comment.value);
        }
    }

    fn push(&mut self, c: char) {
        if self.remaining_width == self.config.line_width {
            self.put_indent();
        }
        self.buffer.push(c);
        self.remaining_width = self.remaining_width.saturating_sub(1);
    }

    fn push_str(&mut self, str: &str) {
        if self.remaining_width == self.config.line_width {
            self.put_indent();
        }
        self.push_str_without_indent(str);
    }

    fn push_str_without_indent(&mut self, str: &str) {
        self.buffer.push_str(str);
        self.remaining_width = self.remaining_width.saturating_sub(str.len());
    }

    fn put_indent(&mut self) {
        let spaces = " ".repeat(self.indent);
        self.buffer.push_str(&spaces);
        self.remaining_width = self.remaining_width.saturating_sub(spaces.len());
    }

    fn indent(&mut self) {
        self.indent += self.config.indent_size;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(self.config.indent_size);
    }

    fn break_line(&mut self, ctx: &FormatContext) {
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
