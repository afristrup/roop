use crate::world::{Entry, refuse, world};

/// The entry for the operation being taken back.
pub fn last_entry() -> Entry {
    world()
        .journal
        .pop()
        .unwrap_or_else(|| refuse("there is no file operation to take back"))
}
