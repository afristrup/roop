use crate::parallel::SendPtr;
use std::ffi::c_void;
use std::sync::atomic::{AtomicI64, Ordering};
use std::thread;

type Body = extern "C" fn(*mut c_void, i64);

/// Runs `body(env, lo + k * step)` for `k` in `0..count`. Blocks are handed
/// out dynamically so fast and slow cores (performance and efficiency cores
/// alike) all stay busy.
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
    let threads = thread::available_parallelism().map_or(1, |n| n.get() as i64);
    let workers = threads.min(count);
    if workers <= 1 {
        (0..count).for_each(|k| body(env, lo + k * step));
        return;
    }
    let block = (count / (workers * 4)).max(1);
    let next = AtomicI64::new(0);
    let env = SendPtr(env);
    let work = || {
        let env = env;
        loop {
            let start = next.fetch_add(block, Ordering::Relaxed);
            if start >= count {
                break;
            }
            for k in start..(start + block).min(count) {
                body(env.0, lo + k * step);
            }
        }
    };
    thread::scope(|scope| {
        for _ in 1..workers {
            scope.spawn(work);
        }
        work();
    });
}
