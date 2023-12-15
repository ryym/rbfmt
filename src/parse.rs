use std::collections::HashMap;

use crate::fmt;
use lib_ruby_parser::{
    nodes::{Block, Send},
    source::Comment,
    Lexer, Loc, Node, Parser, Token,
};

pub(crate) fn parse_into_fmt_node(source: Vec<u8>) -> Option<ParserResult> {
    let parser = Parser::new(source.clone(), Default::default());
    let mut result = parser.do_parse();
    // dbg!(&result.ast);
    // dbg!(&result.comments);

    // Sort the comments by their locations, because they are unordered when there is a heredoc.
    result.comments.sort_by_key(|c| c.location.begin);
    let reversed_comments = result.comments.into_iter().rev().collect();

    let decor_store = fmt::DecorStore::new();
    let token_set = TokenSet::new(result.tokens);

    let mut builder = FmtNodeBuilder {
        src: source,
        comments: reversed_comments,
        token_set,
        decor_store,
        position_gen: 0,
        last_pos: fmt::Pos(0),
        last_loc_end: 0,
    };
    let fmt_node = builder.build_fmt_node(result.ast);
    // dbg!(&fmt_node);
    // dbg!(&builder.decor_store);
    Some(ParserResult {
        node: fmt_node,
        decor_store: builder.decor_store,
    })
}

#[derive(Debug)]
pub(crate) struct ParserResult {
    pub node: fmt::Node,
    pub decor_store: fmt::DecorStore,
}

#[derive(Debug)]
struct TokenSet {
    tokens: Vec<Token>,
    begin_to_idx: HashMap<usize, usize>,
}

impl TokenSet {
    fn new(tokens: Vec<Token>) -> Self {
        let mut map = HashMap::with_capacity(tokens.len());
        for (idx, token) in tokens.iter().enumerate() {
            map.insert(token.loc.begin, idx);
        }
        Self {
            tokens,
            begin_to_idx: map,
        }
    }

    fn try_token_at(&self, begin: usize) -> Option<&Token> {
        self.begin_to_idx
            .get(&begin)
            .and_then(|i| self.tokens.get(*i))
    }

    fn token_at(&self, begin: usize) -> &Token {
        self.try_token_at(begin).expect("token must be at {begin}")
    }
}

#[derive(Debug)]
struct FmtNodeBuilder {
    src: Vec<u8>,
    comments: Vec<Comment>,
    token_set: TokenSet,
    decor_store: fmt::DecorStore,
    position_gen: usize,
    last_pos: fmt::Pos,
    last_loc_end: usize,
}

type MidDecors = (Option<fmt::Comment>, Vec<fmt::LineDecor>);

impl FmtNodeBuilder {
    fn build_fmt_node(&mut self, node: Option<Box<Node>>) -> fmt::Node {
        let fmt_node = node.map(|n| self.visit(*n));
        let exprs = self.wrap_as_exprs(fmt_node, Some(self.src.len()));
        fmt::Node::new(fmt::Pos::none(), fmt::Kind::Exprs(exprs))
    }

    fn next_pos(&mut self) -> fmt::Pos {
        self.position_gen += 1;
        fmt::Pos(self.position_gen)
    }

    fn visit(&mut self, node: Node) -> fmt::Node {
        let pos = self.next_pos();
        let node_end = node.expression().end;
        let fmt_node = match node {
            Node::Nil(node) => self.parse_atom(&node.expression_l, pos, "nil".to_string()),
            Node::True(node) => self.parse_atom(&node.expression_l, pos, "true".to_string()),
            Node::False(node) => self.parse_atom(&node.expression_l, pos, "false".to_string()),
            Node::Int(node) => self.parse_atom(&node.expression_l, pos, node.value),
            Node::Float(node) => self.parse_atom(&node.expression_l, pos, node.value),
            Node::Rational(node) => self.parse_atom(&node.expression_l, pos, node.value),
            Node::Complex(node) => self.parse_atom(&node.expression_l, pos, node.value),
            Node::Ivar(node) => self.parse_atom(&node.expression_l, pos, node.name),
            Node::Cvar(node) => self.parse_atom(&node.expression_l, pos, node.name),
            Node::Gvar(node) => self.parse_atom(&node.expression_l, pos, node.name),
            Node::Begin(node) => {
                let nodes = node.statements.into_iter().map(|n| self.visit(n)).collect();
                fmt::Node::new(pos, fmt::Kind::Exprs(fmt::Exprs(nodes)))
            }
            Node::If(node) => {
                // Consume decors above the if expression.
                self.consume_and_store_decors_until(pos, node.expression_l.begin);

                // Consume decors between "if" and the condition expression.
                // Then merge it to the decors of "if" itself.
                let decors_in_if_and_cond = self.consume_decors_until(node.cond.expression().begin);
                if let Some((if_trailing, cond_leading)) = decors_in_if_and_cond {
                    if let Some(c) = if_trailing {
                        self.decor_store
                            .append_leading_decors(pos, vec![fmt::LineDecor::Comment(c)]);
                    }
                    if !cond_leading.is_empty() {
                        self.decor_store.append_leading_decors(pos, cond_leading);
                    }
                }

                let cond = self.visit(*node.cond);

                let if_token = self.token_set.token_at(node.keyword_l.begin);
                let is_unless = if_token.token_type == Lexer::kUNLESS;
                let (if_first, if_next) = if is_unless {
                    (node.if_false, node.if_true)
                } else {
                    (node.if_true, node.if_false)
                };

                let body = if_first.map(|n| self.visit(*n));
                let ifexpr = match (node.else_l, if_next, node.end_l) {
                    // if...end
                    (None, None, Some(end_l)) => {
                        let if_first = self.wrap_as_exprs(body, Some(end_l.begin));
                        let if_part = fmt::IfPart::new(cond, if_first);
                        fmt::IfExpr::new(is_unless, if_part)
                    }
                    // if...(elsif...|else...)+end
                    (Some(else_l), if_next, Some(end_l)) => {
                        let if_first = self.wrap_as_exprs(body, Some(else_l.begin));
                        let if_part = fmt::IfPart::new(cond, if_first);
                        let mut ifexpr = fmt::IfExpr::new(is_unless, if_part);
                        self.visit_ifelse(if_next, &mut ifexpr, true);
                        ifexpr.end_pos = self.next_pos();
                        self.consume_and_store_decors_until(ifexpr.end_pos, end_l.begin);
                        ifexpr
                    }
                    _ => panic!("invalid if expression"),
                };

                fmt::Node::new(pos, fmt::Kind::IfExpr(ifexpr))
            }
            Node::Send(node) => {
                self.consume_and_store_decors_until(pos, node.expression_l.begin);
                let chain = self.visit_inner_send(node);
                fmt::Node::new(pos, fmt::Kind::MethodChain(chain))
            }
            Node::Block(node) => {
                self.consume_and_store_decors_until(pos, node.expression_l.begin);
                let chain = self.visit_inner_block(node);
                fmt::Node::new(pos, fmt::Kind::MethodChain(chain))
            }

            _ => {
                todo!("{}", format!("convert node {:?}", node));
            }
        };
        self.last_pos = pos;
        self.last_loc_end = node_end;
        fmt_node
    }

    fn parse_atom(&mut self, loc: &Loc, pos: fmt::Pos, value: String) -> fmt::Node {
        self.consume_and_store_decors_until(pos, loc.begin);
        fmt::Node::new(pos, fmt::Kind::Atom(value))
    }

    fn visit_ifelse(&mut self, node: Option<Box<Node>>, ifexpr: &mut fmt::IfExpr, has_else: bool) {
        let node = node.map(|n| *n);
        match node {
            // elsif ("if" only, "unles...elsif" is syntax error)
            Some(Node::If(node)) => {
                let elsif_pos = self.next_pos();
                self.last_pos = elsif_pos;

                let cond = self.visit(*node.cond);
                let body = node.if_true.map(|n| self.visit(*n));
                let body_end_loc = node.else_l.map(|l| l.begin);
                let body = self.wrap_as_exprs(body, body_end_loc);

                ifexpr.elsifs.push(fmt::Elsif {
                    pos: elsif_pos,
                    part: fmt::IfPart::new(cond, body),
                });
                self.visit_ifelse(node.if_false, ifexpr, node.else_l.is_some());
            }
            // else
            _ if has_else => {
                let else_pos = self.next_pos();
                self.last_pos = else_pos;
                let body = node.map(|n| self.visit(n));
                let body = self.wrap_as_exprs(body, None);
                ifexpr.if_last = Some(fmt::Else {
                    pos: else_pos,
                    body,
                });
            }
            _ => {}
        }
    }

    fn visit_inner_send(&mut self, send: Send) -> fmt::MethodChain {
        let mut chain = match send.recv {
            Some(recv) => match *recv {
                Node::Send(send) => self.visit_inner_send(send),
                Node::Block(block) => self.visit_inner_block(block),
                _ => {
                    let recv = self.visit(*recv);
                    fmt::MethodChain {
                        receiver: Some(Box::new(recv)),
                        calls: vec![],
                    }
                }
            },
            None => fmt::MethodChain {
                receiver: None,
                calls: vec![],
            },
        };

        let call_pos = self.next_pos();
        if let Some(selector_l) = send.selector_l {
            self.consume_and_store_decors_until(call_pos, selector_l.begin);
        } else {
            // foo.\n#hoge\n(2)
        }

        let args = send.args.into_iter().map(|n| self.visit(n)).collect();
        chain.calls.push(fmt::MethodCall {
            pos: call_pos,
            name: send.method_name,
            args,
            block: None,
        });

        self.last_pos = call_pos;
        self.last_loc_end = send.expression_l.end;
        chain
    }

    fn visit_inner_block(&mut self, block: Block) -> fmt::MethodChain {
        let send = match *block.call {
            Node::Send(send) => send,
            _ => panic!("unexpected block call node: {:?}", block.call),
        };
        let mut chain = self.visit_inner_send(send);

        let block_pos = self.next_pos();
        self.last_pos = block_pos;
        let body = block.body.map(|b| self.visit(*b));
        let body = self.wrap_as_exprs(body, Some(block.end_l.end));

        let last_call = chain.calls.last_mut().unwrap();
        last_call.block = Some(fmt::MethodBlock {
            pos: block_pos,
            body,
        });

        chain
    }

    // Wrap the given node as Exprs to handle decors around it.
    // If the given node is Exprs, just add the EndDecors to it if necessary.
    fn wrap_as_exprs(&mut self, orig_node: Option<fmt::Node>, end: Option<usize>) -> fmt::Exprs {
        let mut expr_nodes = match orig_node {
            None => vec![],
            Some(node) => match node.kind {
                fmt::Kind::Exprs(fmt::Exprs(nodes)) => nodes,
                _ => vec![node],
            },
        };

        if let Some(end) = end {
            if let Some(end_decors) = self.consume_decors_until(end) {
                let end_node = fmt::Node::new(self.next_pos(), fmt::Kind::EndDecors);
                self.store_decors_to(self.last_pos, end_node.pos, end_decors);
                expr_nodes.push(end_node);
            }
        }

        fmt::Exprs(expr_nodes)
    }

    fn consume_and_store_decors_until(&mut self, pos: fmt::Pos, end: usize) {
        if let Some(decors) = self.consume_decors_until(end) {
            self.store_decors_to(self.last_pos, pos, decors);
        }
    }

    fn store_decors_to(&mut self, last_pos: fmt::Pos, pos: fmt::Pos, decors: MidDecors) {
        let (trailing_comment, line_decors) = decors;
        if let Some(comment) = trailing_comment {
            self.decor_store.set_trailing_comment(last_pos, comment);
        }
        if !line_decors.is_empty() {
            self.decor_store.append_leading_decors(pos, line_decors);
        }
    }

    fn consume_decors_until(&mut self, end: usize) -> Option<MidDecors> {
        let mut line_decors = Vec::new();
        let mut trailing_comment = None;

        // Find the first comment. It may be a trailing comment of the last node.
        match self.comments.last() {
            Some(comment) if comment.location.begin <= end => {
                let (comment_begin, comment_end) = (comment.location.begin, comment.location.end);
                let fmt_comment = self.get_comment_content(comment);
                if self.is_at_line_start(comment_begin) {
                    self.consume_empty_lines_until(comment_begin, &mut line_decors);
                    line_decors.push(fmt::LineDecor::Comment(fmt_comment));
                } else {
                    trailing_comment = Some(fmt_comment);
                }
                self.last_loc_end = comment_end - 1;
                self.comments.pop();
            }
            _ => {}
        };

        // Then find the other comments. They must not be a trailing comment.
        if !line_decors.is_empty() || trailing_comment.is_some() {
            loop {
                let comment = match self.comments.last() {
                    Some(comment) if comment.location.begin <= end => comment,
                    _ => break,
                };
                let fmt_comment = self.get_comment_content(comment);
                let comment_end = comment.location.end;
                self.consume_empty_lines_until(comment.location.begin, &mut line_decors);
                line_decors.push(fmt::LineDecor::Comment(fmt_comment));
                self.last_loc_end = comment_end - 1;
                self.comments.pop();
            }
        }

        // Finally consume the remaining empty lines.
        self.consume_empty_lines_until(end, &mut line_decors);

        if line_decors.is_empty() && trailing_comment.is_none() {
            None
        } else {
            Some((trailing_comment, line_decors))
        }
    }

    fn get_comment_content(&self, comment: &Comment) -> fmt::Comment {
        let comment_bytes = &self.src[comment.location.begin..comment.location.end];
        // Ignore non-UTF8 source code for now.
        let comment_str = String::from_utf8_lossy(comment_bytes)
            .trim_end()
            .to_string();
        fmt::Comment { value: comment_str }
    }

    fn consume_empty_lines_until(&mut self, end: usize, line_decors: &mut Vec<fmt::LineDecor>) {
        let line_loc = self.last_empty_line_loc_within(self.last_loc_end, end);
        if let Some(line_loc) = line_loc {
            line_decors.push(fmt::LineDecor::EmptyLine);
            self.last_loc_end = line_loc.end;
        }
    }

    fn last_empty_line_loc_within(&self, begin: usize, end: usize) -> Option<Loc> {
        let mut line_begin: Option<usize> = None;
        let mut line_end: Option<usize> = None;
        for i in (begin..end).rev() {
            let b = self.src[i];
            if b == b'\n' {
                if line_end.is_none() {
                    line_end = Some(i + 1);
                } else {
                    line_begin = Some(i);
                    break;
                }
            } else if line_end.is_some() && b != b' ' {
                line_end = None;
            }
        }
        match (line_begin, line_end) {
            (Some(begin), Some(end)) => Some(Loc { begin, end }),
            _ => None,
        }
    }

    fn is_at_line_start(&self, begin: usize) -> bool {
        if begin == 0 {
            return true;
        }
        let mut idx = begin - 1;
        let mut has_char_between_last_newline = false;
        while let Some(b) = self.src.get(idx) {
            match b {
                b' ' => {
                    idx -= 1;
                    continue;
                }
                b'\n' => break,
                _ => {
                    has_char_between_last_newline = true;
                    break;
                }
            }
        }
        !has_char_between_last_newline
    }
}
