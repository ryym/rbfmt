pub(super) fn source_lossy_at(loc: &prism::Location) -> String {
    String::from_utf8_lossy(loc.as_slice()).to_string()
}
