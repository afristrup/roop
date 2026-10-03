use crate::world::{Entry, world};

/// Notes what an operation changed, in order.
pub fn journal(entry: Entry) {
    world().journal.push(entry);
}
