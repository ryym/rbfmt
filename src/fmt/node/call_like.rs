use crate::fmt::shape::Shape;

use super::Arguments;

#[derive(Debug)]
pub(crate) struct CallLike {
    pub shape: Shape,
    pub name: String,
    pub arguments: Option<Arguments>,
}

impl CallLike {
    pub(crate) fn new(name: String) -> Self {
        Self {
            shape: Shape::inline(name.len()),
            name,
            arguments: None,
        }
    }

    pub(crate) fn set_arguments(&mut self, args: Arguments) {
        self.shape.append(&args.shape);
        self.arguments = Some(args);
    }
}
