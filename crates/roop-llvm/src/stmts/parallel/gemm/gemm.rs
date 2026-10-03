/// A loop nest `c[i][j] += alpha * a[i][l] * b[l][j]` over `i` in `0..rows`,
/// `l` in `0..inner` and `j` in `0..cols`, naming the places it works on.
pub struct Gemm<'a> {
    pub c: &'a str,
    pub a: &'a str,
    pub b: &'a str,
    pub alpha: &'a str,
    pub rows: i64,
    pub inner: i64,
    pub cols: i64,
}
