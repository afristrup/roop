use crate::world::{Input, World, commit_output, errno_of};
use std::io::BufRead;

/// The next thing the program reads: what was put back, else a line of the real
/// standard input. Reading the real input is where the program learns something
/// the world has not told it before, so what is pending is shown first, as it
/// is for anyone waiting to answer it.
pub fn next_input(world: &mut World) -> Input {
    if let Some(input) = world.ahead.pop_front() {
        return input;
    }
    commit_output(world);
    let mut line = Vec::new();
    match std::io::stdin().lock().read_until(b'\n', &mut line) {
        Ok(0) => Input::Eof,
        Ok(_) => {
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            Input::Line(line)
        }
        Err(e) => Input::Failed(errno_of(&e)),
    }
}
