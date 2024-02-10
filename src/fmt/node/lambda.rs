use crate::fmt::shape::Shape;

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
}
