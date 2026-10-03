use crate::world::{commit_output, world};

/// Makes what the program has written so far real. Output is pending until
/// then, so it can be taken back; after this it cannot.
#[unsafe(no_mangle)]
pub extern "C" fn roop_commit() {
    commit_output(&mut world());
}
