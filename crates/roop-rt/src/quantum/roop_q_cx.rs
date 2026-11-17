use crate::quantum::{device, position};

/// Flips `target` where `control` is 1. Its own inverse.
///
/// # Safety
/// Both pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q_cx(control: *const i64, target: *const i64) {
    let mut d = device();
    let (c, t) = (
        position(&d, unsafe { *control }),
        position(&d, unsafe { *target }),
    );
    let (cb, tb) = (1usize << c, 1usize << t);
    for i in (0..d.amps.len()).filter(|i| i & cb != 0 && i & tb == 0) {
        d.amps.swap(i, i | tb);
    }
}

/// # Safety
/// Both pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q_cx_inv(control: *const i64, target: *const i64) {
    unsafe { roop_q_cx(control, target) }
}
