use crate::io::with_fd;
use crate::world::World;
use std::io::Write;

/// Shows what is pending, in the order it was written, and forgets it: from
/// here on it cannot be taken back.
pub fn commit_output(world: &mut World) {
    for chunk in world.out.drain(..) {
        let _ = unsafe { with_fd(chunk.fd, |f| f.write_all(&chunk.bytes)) };
    }
}
