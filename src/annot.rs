// Made to be zipped with others in available mod
pub(crate) fn parse_annots(s: &str) -> impl Iterator<Item = impl Iterator<Item = &str>> {
    s.split("\0\0")
        .map(|l| l.split("\0").map(|c| if c == "\x18" { "" } else { c }))
}
