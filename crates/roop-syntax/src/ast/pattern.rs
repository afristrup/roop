#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    Int(i64),
    Bool(bool),
    Wildcard,
}
