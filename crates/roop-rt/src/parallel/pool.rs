use crate::parallel::{Job, PoolState, Shared, run_blocks, worker_loop};
use std::sync::atomic::AtomicI64;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// Persistent worker threads. The calling thread works too, so a pool of
/// `n` workers runs `n + 1` threads.
pub struct Pool {
    shared: Arc<Shared>,
    workers: usize,
    busy: Mutex<()>,
}

impl Pool {
    pub fn new(workers: usize) -> Pool {
        let shared = Arc::new(Shared {
            state: Mutex::new(PoolState {
                generation: 0,
                job: None,
                running: 0,
            }),
            wake: Condvar::new(),
            done: Condvar::new(),
        });
        for _ in 0..workers {
            let shared = Arc::clone(&shared);
            thread::spawn(move || worker_loop(shared));
        }
        Pool {
            shared,
            workers,
            busy: Mutex::new(()),
        }
    }

    pub fn threads(&self) -> usize {
        self.workers + 1
    }

    /// Runs the job on the pool. Returns false, having done nothing, when the
    /// pool is already running a loop (a nested or concurrent call).
    pub fn run(&self, mut job: Job) -> bool {
        let Ok(_exclusive) = self.busy.try_lock() else {
            return false;
        };
        let next = AtomicI64::new(0);
        job.next = crate::parallel::SendPtr(&next as *const AtomicI64 as *mut _);
        {
            let mut state = self.shared.state.lock().unwrap();
            state.job = Some(job);
            state.running = self.workers;
            state.generation += 1;
        }
        self.shared.wake.notify_all();
        run_blocks(&job);
        let mut state = self.shared.state.lock().unwrap();
        while state.running > 0 {
            state = self.shared.done.wait(state).unwrap();
        }
        state.job = None;
        true
    }
}
