/// The irreversible ways to change a place: they destroy its old value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverwriteOp {
    Assign,
    Rem,
}
