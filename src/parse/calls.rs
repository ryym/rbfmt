use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_call(&mut self, node: prism::CallNode) -> fmt::Node {
        match detect_method_type(&node) {
            MethodType::Normal => self.parse_call_root(&node),
            MethodType::Not => self.parse_not(node),
            MethodType::Unary => self.parse_prefix_call(node),
            MethodType::Binary => self.parse_infix_call(node),
            MethodType::Assign => self.parse_write_call(node),
            MethodType::IndexAssign => self.parse_index_write_call(node),
        }
    }

    pub(super) fn parse_call_root<C: CallRoot>(&mut self, call: &C) -> fmt::Node {
        let current_chain = call.receiver().map(|receiver| {
            let receiver_trailing_end = call
                .message_loc()
                .or_else(|| call.opening_loc())
                .or_else(|| call.arguments().map(|a| a.location()))
                .or_else(|| call.block().map(|a| a.location()))
                .map(|l| l.start_offset());
            let node = self.visit(receiver, receiver_trailing_end);
            match node.kind {
                fmt::Kind::MethodChain(chain) => (chain, node.trailing_trivia),
                _ => (
                    fmt::MethodChain::with_receiver(node),
                    fmt::TrailingTrivia::none(),
                ),
            }
        });

        let call_leading = if let Some(msg_loc) = call.message_loc() {
            self.take_leading_trivia(msg_loc.start_offset())
        } else {
            fmt::LeadingTrivia::new()
        };

        let arguments = call.arguments();
        let block = call.block();
        let opening_loc = call.opening_loc();
        let closing_loc = call.closing_loc();
        let (args, block) = match block {
            Some(node) => match node {
                // method call with block literal (e.g. "foo { a }", "foo(a) { b }")
                prism::Node::BlockNode { .. } => {
                    let args = self.visit_arguments(arguments, None, opening_loc, closing_loc);
                    let block = node.as_block_node().unwrap();
                    let block = self.visit_block(block);
                    (args, Some(block))
                }
                // method call with a block argument (e.g. "foo(&a)", "foo(a, &b)")
                prism::Node::BlockArgumentNode { .. } => {
                    let block_arg = node.as_block_argument_node().unwrap();
                    let args =
                        self.visit_arguments(arguments, Some(block_arg), opening_loc, closing_loc);
                    (args, None)
                }
                _ => panic!("unexpected block node of call: {:?}", node),
            },
            // method call without block (e.g. "foo", "foo(a)")
            None => {
                let args = self.visit_arguments(arguments, None, opening_loc, closing_loc);
                (args, None)
            }
        };

        let name = String::from_utf8_lossy(call.name()).to_string();
        let chain = match current_chain {
            Some((mut chain, last_call_trailing)) => {
                if name == "[]" {
                    chain.append_index_call(fmt::IndexCall::new(
                        args.expect("index call must have arguments"),
                        block,
                    ));
                } else {
                    let call_operator = call.call_operator_loc().map(|l| Self::source_lossy_at(&l));
                    chain.append_message_call(
                        last_call_trailing,
                        fmt::MessageCall::new(call_leading, call_operator, name, args, block),
                    );
                }
                chain
            }
            None => fmt::MethodChain::without_receiver(fmt::MessageCall::new(
                call_leading,
                None,
                name,
                args,
                block,
            )),
        };

        self.last_loc_end = call.location().end_offset();
        fmt::Node::new(fmt::Kind::MethodChain(chain))
    }

    fn parse_not(&mut self, node: prism::CallNode) -> fmt::Node {
        // not(v) is parsed like below:
        //   receiver: v
        //   method name: "!"
        //   args: empty but has parentheses

        let receiver = node.receiver().expect("not must have receiver (argument)");

        let opening_loc = node.opening_loc();
        let closing_loc = node.closing_loc();
        let opening = opening_loc.as_ref().map(Self::source_lossy_at);
        let closing = closing_loc.as_ref().map(Self::source_lossy_at);
        let mut args = fmt::Arguments::new(opening, closing);

        let closing_start = closing_loc.map(|l| l.start_offset());
        let receiver = self.visit(receiver, closing_start);
        args.append_node(receiver);
        let virtual_end = self.take_end_trivia_as_virtual_end(closing_start);
        args.set_virtual_end(virtual_end);
        args.last_comma_allowed = false;

        let chain = fmt::MethodChain::without_receiver(fmt::MessageCall::new(
            fmt::LeadingTrivia::new(),
            None,
            "not".to_string(),
            Some(args),
            None,
        ));
        fmt::Node::new(fmt::Kind::MethodChain(chain))
    }

    fn parse_prefix_call(&mut self, call: prism::CallNode) -> fmt::Node {
        let msg_loc = call
            .message_loc()
            .expect("prefix operation must have message");
        let operator = Self::source_lossy_at(&msg_loc);
        let receiver = call
            .receiver()
            .expect("prefix operation must have receiver");
        let receiver = self.visit(receiver, None);
        let prefix = fmt::Prefix::new(operator, Some(receiver));
        fmt::Node::new(fmt::Kind::Prefix(prefix))
    }

    fn parse_infix_call(&mut self, call: prism::CallNode) -> fmt::Node {
        let msg_loc = call
            .message_loc()
            .expect("infix operation must have message");
        let receiver = call.receiver().expect("infix operation must have receiver");
        let right = call
            .arguments()
            .and_then(|args| args.arguments().iter().next())
            .expect("infix operation must have argument");
        self.parse_infix_operation(receiver, msg_loc, right)
    }

    pub(super) fn parse_infix_operation(
        &mut self,
        left: prism::Node,
        operator_loc: prism::Location,
        right: prism::Node,
    ) -> fmt::Node {
        let left = self.visit(left, Some(operator_loc.start_offset()));
        let operator = Self::source_lossy_at(&operator_loc);
        let precedence = fmt::InfixPrecedence::from_operator(&operator);
        let mut chain = match left.kind {
            fmt::Kind::InfixChain(chain) if chain.precedence() == &precedence => chain,
            _ => fmt::InfixChain::new(left, precedence),
        };
        let right = self.visit(right, None);
        chain.append_right(operator, right);
        fmt::Node::new(fmt::Kind::InfixChain(chain))
    }

    fn parse_write_call(&mut self, call: prism::CallNode) -> fmt::Node {
        let msg_loc = call.message_loc().expect("call write must have message");
        let receiver = call.receiver().expect("call write must have receiver");
        let receiver = self.visit(receiver, Some(msg_loc.start_offset()));

        let call_leading = self.take_leading_trivia(msg_loc.start_offset());
        let call_operator = call.call_operator_loc().as_ref().map(Self::source_lossy_at);
        let mut name = String::from_utf8_lossy(call.name().as_slice()).to_string();
        name.truncate(name.len() - 1); // Remove '='

        let arg = call
            .arguments()
            .and_then(|args| args.arguments().iter().next())
            .expect("call write must have argument");

        let mut chain = fmt::MethodChain::with_receiver(receiver);
        chain.append_message_call(
            fmt::TrailingTrivia::none(),
            fmt::MessageCall::new(call_leading, call_operator, name, None, None),
        );

        let left = fmt::Node::new(fmt::Kind::MethodChain(chain));
        let right = self.visit(arg, None);
        let operator = "=".to_string();
        let assign = fmt::Assign::new(left, operator, right);
        fmt::Node::new(fmt::Kind::Assign(assign))
    }

    fn parse_index_write_call(&mut self, call: prism::CallNode) -> fmt::Node {
        let (opening_loc, closing_loc) = match (call.opening_loc(), call.closing_loc()) {
            (Some(op), Some(cl)) => (op, cl),
            _ => panic!("index write must have opening and closing"),
        };

        let receiver = call.receiver().expect("index write must have receiver");
        let receiver = self.visit(receiver, Some(opening_loc.start_offset()));

        let args = call.arguments().expect("index write must have arguments");
        let mut args_iter = args.arguments().iter();
        let (arg1, arg2) = match (args_iter.next(), args_iter.next(), args_iter.next()) {
            (Some(arg1), Some(arg2), None) => (arg1, arg2),
            _ => panic!("index write must have exactly two arguments"),
        };

        let mut left_args = fmt::Arguments::new(Some("[".to_string()), Some("]".to_string()));
        let closing_start = closing_loc.start_offset();
        left_args.append_node(self.visit(arg1, Some(closing_start)));
        let left_args_end = self.take_end_trivia_as_virtual_end(Some(closing_start));
        left_args.set_virtual_end(left_args_end);

        let mut chain = fmt::MethodChain::with_receiver(receiver);
        chain.append_index_call(fmt::IndexCall::new(left_args, None));

        let left = fmt::Node::new(fmt::Kind::MethodChain(chain));
        let right = self.visit(arg2, None);
        let operator = "=".to_string();
        let assign = fmt::Assign::new(left, operator, right);
        fmt::Node::new(fmt::Kind::Assign(assign))
    }
}

fn detect_method_type(call: &prism::CallNode) -> MethodType {
    let method_name = call.name().as_slice();
    if method_name == b"!" && call.message_loc().map_or(false, |m| m.as_slice() == b"not") {
        return MethodType::Not;
    }
    if call.receiver().is_some()
        && call.call_operator_loc().is_none()
        && call.opening_loc().is_none()
    {
        return if call.arguments().is_some() {
            MethodType::Binary
        } else {
            MethodType::Unary
        };
    }
    if method_name[method_name.len() - 1] == b'='
        && method_name != b"=="
        && method_name != b"==="
        && method_name != b"<="
        && method_name != b">="
        && method_name != b"!="
    {
        return if method_name == b"[]=" {
            MethodType::IndexAssign
        } else {
            MethodType::Assign
        };
    }
    MethodType::Normal
}

#[derive(Debug)]
enum MethodType {
    Normal,      // foo(a)
    Not,         // not
    Unary,       // -a
    Binary,      // a - b
    Assign,      // a = b
    IndexAssign, // a[b] = c
}

pub(super) trait CallRoot {
    fn location(&self) -> prism::Location;
    fn receiver(&self) -> Option<prism::Node>;
    fn message_loc(&self) -> Option<prism::Location>;
    fn call_operator_loc(&self) -> Option<prism::Location>;
    fn name(&self) -> &[u8];
    fn arguments(&self) -> Option<prism::ArgumentsNode>;
    fn opening_loc(&self) -> Option<prism::Location>;
    fn closing_loc(&self) -> Option<prism::Location>;
    fn block(&self) -> Option<prism::Node>;
}

impl<'src> CallRoot for prism::CallNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        self.opening_loc()
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        self.closing_loc()
    }
    fn block(&self) -> Option<prism::Node> {
        self.block()
    }
}

impl<'src> CallRoot for prism::CallAndWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.read_name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::CallOrWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.read_name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::CallOperatorWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        self.message_loc()
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        self.read_name().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::CallTargetNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        Some(self.receiver())
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.message_loc())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        Some(self.call_operator_loc())
    }
    fn name(&self) -> &[u8] {
        // We cannot use `name()` because it automatically has `=` suffix.
        self.message_loc().as_slice()
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::IndexAndWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::IndexOrWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::IndexOperatorWriteNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        self.receiver()
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        self.call_operator_loc()
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::IndexTargetNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        Some(self.receiver())
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        None
    }
    fn name(&self) -> &[u8] {
        b"[]"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        Some(self.opening_loc())
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        Some(self.closing_loc())
    }
    fn block(&self) -> Option<prism::Node> {
        None
    }
}

impl<'src> CallRoot for prism::ForwardingSuperNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        None
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        None
    }
    fn name(&self) -> &[u8] {
        b"super"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        None
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        None
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        None
    }
    fn block(&self) -> Option<prism::Node> {
        self.block().map(|b| b.as_node())
    }
}

impl<'src> CallRoot for prism::SuperNode<'src> {
    fn location(&self) -> prism::Location {
        self.location()
    }
    fn receiver(&self) -> Option<prism::Node> {
        None
    }
    fn message_loc(&self) -> Option<prism::Location> {
        Some(self.location())
    }
    fn call_operator_loc(&self) -> Option<prism::Location> {
        None
    }
    fn name(&self) -> &[u8] {
        b"super"
    }
    fn arguments(&self) -> Option<prism::ArgumentsNode> {
        self.arguments()
    }
    fn opening_loc(&self) -> Option<prism::Location> {
        self.lparen_loc()
    }
    fn closing_loc(&self) -> Option<prism::Location> {
        self.rparen_loc()
    }
    fn block(&self) -> Option<prism::Node> {
        self.block()
    }
}
