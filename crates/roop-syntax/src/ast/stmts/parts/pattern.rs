#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    Int(i64),
    Bool(bool),
    Variant(String, String),
    Wildcard,
}
