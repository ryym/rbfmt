use crate::fmt::output::Output;

#[derive(Debug)]
pub(crate) struct Atom(pub String);

impl Atom {
    pub(crate) fn format(&self, o: &mut Output) {
        o.push_str(&self.0);
    }
}
