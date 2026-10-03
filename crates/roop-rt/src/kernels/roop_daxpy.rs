/// The plain loop, for hosts without the SME kernel: `y += alpha * x`.
///
/// # Safety
/// `y` and `x` must address `n` doubles each.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_daxpy(y: *mut f64, x: *const f64, alpha: f64, n: i64) {
    let y = unsafe { std::slice::from_raw_parts_mut(y, n as usize) };
    let x = unsafe { std::slice::from_raw_parts(x, n as usize) };
    for (cell, value) in y.iter_mut().zip(x) {
        *cell += alpha * value;
    }
}
