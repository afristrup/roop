/// The classical part of a cq-state: every bit and int register.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Store {
    pub bits: Vec<bool>,
    pub ints: Vec<i64>,
}
