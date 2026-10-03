use crate::parallel::{Shared, run_blocks};
use std::sync::Arc;

/// Sleeps until a new job is published, helps run it, then reports in.
pub fn worker_loop(shared: Arc<Shared>) {
    let mut seen = 0;
    loop {
        let job = {
            let mut state = shared.state.lock().unwrap();
            while state.generation == seen {
                state = shared.wake.wait(state).unwrap();
            }
            seen = state.generation;
            state.job.expect("a published generation carries a job")
        };
        run_blocks(&job);
        let mut state = shared.state.lock().unwrap();
        state.running -= 1;
        if state.running == 0 {
            shared.done.notify_one();
        }
    }
}
