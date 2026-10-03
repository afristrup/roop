use crate::parallel::{Body, Job, SendPtr, global_pool};
use std::ffi::c_void;

/// Runs `body(env, lo + k * step)` for `k` in `0..count` on the thread pool.
/// Falls back to the calling thread for tiny loops and when the pool is busy.
///
/// # Safety
/// `body` must be safe to call concurrently with `env` for distinct values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_parallel_for(
    lo: i64,
    count: i64,
    step: i64,
    body: Body,
    env: *mut c_void,
) {
    let pool = global_pool();
    let threads = pool.threads() as i64;
    if count > 1 && threads > 1 {
        let block = (count / (threads * 4)).max(1);
        let job = Job {
            lo,
            step,
            count,
            block,
            body,
            env: SendPtr(env),
            next: SendPtr(env),
        };
        if pool.run(job) {
            return;
        }
    }
    (0..count).for_each(|k| body(env, lo + k * step));
}
