use crate::fmt::{shape::Shape, LeadingTrivia, TrailingTrivia};

use super::{Arguments, Block, Node};

#[derive(Debug)]
pub(crate) struct MethodChain {
    pub shape: Shape,
    pub head: MethodChainHead,
    pub calls: Vec<CallUnit>,
    pub calls_shape: Shape,
}

impl MethodChain {
    pub(crate) fn with_receiver(receiver: Node) -> Self {
        Self {
            shape: receiver.shape,
            head: MethodChainHead::Receiver(Receiver::new(receiver)),
            calls: vec![],
            calls_shape: Shape::inline(0),
        }
    }

    pub(crate) fn without_receiver(call: MessageCall) -> Self {
        Self {
            shape: call.shape,
            head: MethodChainHead::FirstCall(CallUnit::from_message(call)),
            calls: vec![],
            calls_shape: Shape::inline(0),
        }
    }

    pub(crate) fn append_message_call(
        &mut self,
        last_call_trailing: TrailingTrivia,
        msg_call: MessageCall,
    ) {
        self.shape.append(last_call_trailing.shape());
        self.shape.append(&msg_call.shape);
        self.calls_shape.append(last_call_trailing.shape());
        self.calls_shape.append(&msg_call.shape);

        if !last_call_trailing.is_none() {
            let last_call = self
                .calls
                .last_mut()
                .or(match &mut self.head {
                    MethodChainHead::FirstCall(call) => Some(call),
                    _ => None,
                })
                .expect("call must exist when last trailing exist");
            last_call.shape.append(last_call_trailing.shape());
            last_call.trailing_trivia = last_call_trailing;
        }

        let call = CallUnit::from_message(msg_call);
        self.calls.push(call);
    }

    pub(crate) fn append_index_call(&mut self, idx_call: IndexCall) {
        self.shape.append(&idx_call.shape);
        self.calls_shape.append(&idx_call.shape);

        if let Some(prev) = self.calls.last_mut() {
            prev.append_index_call(idx_call);
        } else {
            self.head.append_index_call(idx_call);
        }
    }
}

#[derive(Debug)]
pub(crate) enum MethodChainHead {
    Receiver(Receiver),
    FirstCall(CallUnit),
}

impl MethodChainHead {
    pub(crate) fn has_trailing_trivia(&self) -> bool {
        match self {
            Self::Receiver(receiver) => !receiver.node.trailing_trivia.is_none(),
            Self::FirstCall(call) => !call.trailing_trivia.is_none(),
        }
    }

    fn append_index_call(&mut self, idx_call: IndexCall) {
        match self {
            Self::Receiver(receiver) => receiver.append_index_call(idx_call),
            Self::FirstCall(call) => call.append_index_call(idx_call),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Receiver {
    pub shape: Shape,
    pub node: Box<Node>,
    pub index_calls: Vec<IndexCall>,
}

impl Receiver {
    fn new(node: Node) -> Self {
        Self {
            shape: node.shape,
            node: Box::new(node),
            index_calls: vec![],
        }
    }

    fn append_index_call(&mut self, idx_call: IndexCall) {
        self.shape.append(&idx_call.shape);
        self.index_calls.push(idx_call);
    }
}

#[derive(Debug)]
pub(crate) struct CallUnit {
    pub shape: Shape,
    pub leading_trivia: LeadingTrivia,
    pub trailing_trivia: TrailingTrivia,
    pub operator: Option<String>,
    pub name: String,
    pub arguments: Option<Arguments>,
    pub block: Option<Block>,
    pub index_calls: Vec<IndexCall>,
}

impl CallUnit {
    fn from_message(call: MessageCall) -> Self {
        Self {
            shape: call.shape,
            leading_trivia: call.leading_trivia,
            trailing_trivia: TrailingTrivia::none(),
            operator: call.operator,
            name: call.name,
            arguments: call.arguments,
            block: call.block,
            index_calls: vec![],
        }
    }

    fn append_index_call(&mut self, idx_call: IndexCall) {
        self.shape.append(&idx_call.shape);
        self.index_calls.push(idx_call);
    }

    pub(crate) fn min_first_line_len(&self) -> Option<usize> {
        if self.leading_trivia.is_empty() {
            let mut len = self.operator.as_ref().map_or(0, |op| op.len());
            len += self.name.len();
            if let Some(args) = &self.arguments {
                len += args.opening.as_ref().map_or(0, |op| op.len());
            } else if let Some(block) = &self.block {
                len += block.min_first_line_len();
            } else if let Some(index) = self.index_calls.first() {
                len += index.min_first_line_len();
            };
            Some(len)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct MessageCall {
    shape: Shape,
    leading_trivia: LeadingTrivia,
    operator: Option<String>,
    name: String,
    arguments: Option<Arguments>,
    block: Option<Block>,
}

impl MessageCall {
    pub(crate) fn new(
        leading_trivia: LeadingTrivia,
        operator: Option<String>,
        name: String,
        arguments: Option<Arguments>,
        block: Option<Block>,
    ) -> Self {
        let operator_len = operator.as_ref().map_or(0, |s| s.len());
        let msg_shape = Shape::inline(name.len() + operator_len);
        let mut shape = leading_trivia.shape().add(&msg_shape);
        if let Some(args) = &arguments {
            shape.append(&args.shape);
        }
        if let Some(block) = &block {
            shape.append(&block.shape);
        }
        Self {
            shape,
            leading_trivia,
            operator,
            name,
            arguments,
            block,
        }
    }
}

#[derive(Debug)]
pub(crate) struct IndexCall {
    pub shape: Shape,
    pub arguments: Arguments,
    pub block: Option<Block>,
}

impl IndexCall {
    pub(crate) fn new(arguments: Arguments, block: Option<Block>) -> Self {
        let mut shape = arguments.shape;
        if let Some(block) = &block {
            shape.append(&block.shape);
        }
        Self {
            shape,
            arguments,
            block,
        }
    }

    pub(crate) fn min_first_line_len(&self) -> usize {
        self.arguments.opening.as_ref().map_or(0, |op| op.len())
    }
}
