use crate::world::{refuse, require_zero, world};

/// Takes the bytes last kept back into `size` bytes at `ptr`, which are zero.
///
/// # Safety
/// `ptr` must be valid for `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_unkeep(ptr: *mut u8, size: i64) {
    let Some(kept) = world().kept.pop() else {
        refuse("there is nothing kept to take back");
    };
    if kept.size != size.max(0) as usize {
        refuse("the value taken back is not the size of the one kept last");
    }
    world().kept_bytes -= kept.cost();
    unsafe {
        require_zero(ptr, kept.size, "the place a kept value returns to");
        std::ptr::copy_nonoverlapping(kept.bytes.as_ptr(), ptr, kept.bytes.len());
    }
}
