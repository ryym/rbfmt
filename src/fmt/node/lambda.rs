use crate::fmt::{
    output::{FormatContext, Output},
    shape::Shape,
};

use super::{Block, BlockParameters};

#[derive(Debug)]
pub(crate) struct Lambda {
    pub shape: Shape,
    pub parameters: Option<BlockParameters>,
    pub block: Block,
}

impl Lambda {
    pub(crate) fn new(params: Option<BlockParameters>, block: Block) -> Self {
        let mut shape = Shape::inline("->".len());
        if let Some(params) = &params {
            shape.append(&params.shape);
        }
        shape.append(&Shape::inline(1));
        shape.append(&block.shape);
        Self {
            shape,
            parameters: params,
            block,
        }
    }

    pub(crate) fn format(&self, o: &mut Output, ctx: &FormatContext) {
        o.push_str("->");
        if let Some(params) = &self.parameters {
            params.format(o, ctx);
        }
        self.block.format(o, ctx);
    }
}
