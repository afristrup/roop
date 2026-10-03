use crate::{Dir, Slot};

/// A statement that is compiled as zeroing an ancilla, in the direction given:
/// the one that would take off what the other made.
#[derive(Clone, Debug)]
pub struct Clear {
    /// The address of the statement, to tell it apart; it is never read through.
    pub stmt: usize,
    pub dir: Dir,
    pub slot: Slot,
}
