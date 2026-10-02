#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLBuffer;
use std::collections::HashMap;
use std::sync::Mutex;

type Buffer = Retained<ProtocolObject<dyn MTLBuffer>>;

/// Metal objects are thread-safe; objc2 just does not mark them so.
struct Pooled(Buffer);
unsafe impl Send for Pooled {}

static POOL: Mutex<Option<HashMap<usize, Vec<Pooled>>>> = Mutex::new(None);

/// A previously used staging buffer of exactly `size` bytes, if any.
pub fn take_pooled(size: usize) -> Option<Buffer> {
    let mut pool = POOL.lock().ok()?;
    pool.as_mut()?.get_mut(&size)?.pop().map(|p| p.0)
}

/// Keeps a staging buffer for the next launch, which saves the allocation and
/// the page faults of touching fresh memory.
pub fn return_pooled(size: usize, buffer: Buffer) {
    if let Ok(mut pool) = POOL.lock() {
        pool.get_or_insert_with(HashMap::new)
            .entry(size)
            .or_default()
            .push(Pooled(buffer));
    }
}
