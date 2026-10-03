/// The plain loops, for hosts without the Q12 kernel: `c += sign * sum_j
/// trunc(a * b / 4096)`. `layout` is 0 for `a[i][j] b[j][k]`, 1 for `a[i][j]
/// b[k][j]` and 2 for `a[j][i] b[j][k]`.
///
/// # Safety
/// The pointers must address `m * n`, `m * kk` and `kk * n` integers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q12_matmul(
    c: *mut i64,
    a: *const i64,
    b: *const i64,
    sign: i64,
    m: i64,
    n: i64,
    kk: i64,
    layout: i64,
) {
    let (m, n, kk) = (m as usize, n as usize, kk as usize);
    let c = unsafe { std::slice::from_raw_parts_mut(c, m * n) };
    let a = unsafe { std::slice::from_raw_parts(a, m * kk) };
    let b = unsafe { std::slice::from_raw_parts(b, kk * n) };
    for i in 0..m {
        for k in 0..n {
            let sum = (0..kk).fold(0i64, |sum, j| {
                let x = if layout == 2 {
                    a[j * m + i]
                } else {
                    a[i * kk + j]
                };
                let y = if layout == 1 {
                    b[k * kk + j]
                } else {
                    b[j * n + k]
                };
                sum.wrapping_add(x.wrapping_mul(y) / 4096)
            });
            c[i * n + k] = c[i * n + k].wrapping_add(sign.wrapping_mul(sum));
        }
    }
}
