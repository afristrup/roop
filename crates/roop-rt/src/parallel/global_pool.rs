use crate::parallel::Pool;
use std::sync::OnceLock;
use std::thread;

/// The process-wide pool, sized to the machine on first use.
pub fn global_pool() -> &'static Pool {
    static POOL: OnceLock<Pool> = OnceLock::new();
    POOL.get_or_init(|| {
        let threads = thread::available_parallelism().map_or(1, |n| n.get());
        Pool::new(threads - 1)
    })
}
