use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_block_body(
        &mut self,
        body: Option<prism::Node>,
        trailing_end: usize,
    ) -> fmt::BlockBody {
        match body {
            Some(body) => match body {
                prism::Node::StatementsNode { .. } => {
                    let stmts = body.as_statements_node().unwrap();
                    let statements = self.parse_statements_body(Some(stmts), Some(trailing_end));
                    fmt::BlockBody::new(statements)
                }
                prism::Node::BeginNode { .. } => {
                    let node = body.as_begin_node().unwrap();
                    self.parse_begin_body(node)
                }
                _ => panic!("unexpected def body: {:?}", body),
            },
            None => {
                let statements = self.wrap_as_statements(None, trailing_end);
                fmt::BlockBody::new(statements)
            }
        }
    }
}
