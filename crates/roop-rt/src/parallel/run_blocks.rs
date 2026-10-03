use crate::parallel::Job;
use std::sync::atomic::{AtomicI64, Ordering};

/// Claims blocks of iterations until none are left. Blocks are handed out
/// dynamically so fast and slow cores (performance and efficiency cores
/// alike) all stay busy.
pub fn run_blocks(job: &Job) {
    let next = unsafe { &*(job.next.0 as *const AtomicI64) };
    loop {
        let start = next.fetch_add(job.block, Ordering::Relaxed);
        if start >= job.count {
            break;
        }
        for k in start..(start + job.block).min(job.count) {
            (job.body)(job.env.0, job.lo + k * job.step);
        }
    }
}
