use std::sync::atomic::{AtomicI64, Ordering};

static ZERO_COPY_BUFFERS: AtomicI64 = AtomicI64::new(0);

pub fn count_zero_copy() {
    ZERO_COPY_BUFFERS.fetch_add(1, Ordering::Relaxed);
}

/// How many kernel buffers so far were the caller's own memory, shared with
/// the GPU without copying. Page-aligned arrays qualify.
#[unsafe(no_mangle)]
pub extern "C" fn roop_gpu_zero_copy_count() -> i64 {
    ZERO_COPY_BUFFERS.load(Ordering::Relaxed)
}
