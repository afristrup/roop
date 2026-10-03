use crate::IntLayout;

/// A loop nest `c[i][k] += a * b` summed over `j`, with the product divided by
/// 4096 when `scaled` (the `q12` einsums) and as it is when not (the `i64`
/// ones), over `i` in `0..rows`, `k` in `0..cols` and `j` in `0..inner`, naming
/// the places it works on.
pub struct IntMatmul<'a> {
    pub c: &'a str,
    pub a: &'a str,
    pub b: &'a str,
    pub rows: i64,
    pub inner: i64,
    pub cols: i64,
    pub layout: IntLayout,
    pub scaled: bool,
}

impl IntMatmul<'_> {
    /// The rows and columns that `c`, `a` and `b` must have.
    pub fn shapes(&self) -> [(i64, i64); 3] {
        let (m, n, k) = (self.rows, self.cols, self.inner);
        let (a, b) = match self.layout {
            IntLayout::NN => ((m, k), (k, n)),
            IntLayout::NT => ((m, k), (n, k)),
            IntLayout::TN => ((k, m), (k, n)),
        };
        [(m, n), a, b]
    }
}
