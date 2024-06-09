use crate::fmt;

impl<'src> super::Parser<'src> {
    pub(super) fn parse_constant_path(
        &mut self,
        parent: Option<prism::Node>,
        name_loc: prism::Location,
    ) -> fmt::Node {
        let mut const_path = match parent {
            Some(parent) => {
                let parent = self.parse(parent, None);
                match parent.kind {
                    fmt::Kind::ConstantPath(const_path) => const_path,
                    _ => fmt::ConstantPath::new(Some(parent)),
                }
            }
            None => fmt::ConstantPath::new(None),
        };
        let path_leading = self.take_leading_trivia(name_loc.start_offset());
        let path = Self::source_lossy_at(&name_loc);
        const_path.append_part(path_leading, path);
        fmt::Node::new(fmt::Kind::ConstantPath(const_path))
    }
}
