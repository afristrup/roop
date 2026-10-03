use crate::world::world;

/// Lets go of the world's history for good: the values kept, the input read,
/// the clock readings and the changes to files. What came before can no longer
/// be run backward, and asking to is a runtime error.
#[unsafe(no_mangle)]
pub extern "C" fn roop_forget() {
    let mut world = world();
    world.kept.clear();
    world.kept_bytes = 0;
    world.consumed.clear();
    world.clock_consumed.clear();
    world.journal.clear();
}
