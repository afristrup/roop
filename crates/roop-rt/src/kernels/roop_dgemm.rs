/// The plain loops, for hosts without the SME kernel: `C += alpha * A * B`
/// for row-major `m x k` and `k x n` matrices.
///
/// # Safety
/// The pointers must address `m * n`, `m * k` and `k * n` doubles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_dgemm(
    c: *mut f64,
    a: *const f64,
    b: *const f64,
    alpha: f64,
    m: i64,
    n: i64,
    k: i64,
) {
    let (m, n, k) = (m as usize, n as usize, k as usize);
    let c = unsafe { std::slice::from_raw_parts_mut(c, m * n) };
    let a = unsafe { std::slice::from_raw_parts(a, m * k) };
    let b = unsafe { std::slice::from_raw_parts(b, k * n) };
    for (i, row) in c.chunks_mut(n).enumerate() {
        for (l, scaled) in a[i * k..(i + 1) * k].iter().map(|x| alpha * x).enumerate() {
            for (cell, y) in row.iter_mut().zip(&b[l * n..(l + 1) * n]) {
                *cell += scaled * y;
            }
        }
    }
}
