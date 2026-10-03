use crate::Q12Layout;

/// A loop nest `c[i][k] += a * b / 4096` summed over `j`, over `i` in `0..rows`,
/// `k` in `0..cols` and `j` in `0..inner`, naming the places it works on.
pub struct Q12Matmul<'a> {
    pub c: &'a str,
    pub a: &'a str,
    pub b: &'a str,
    pub rows: i64,
    pub inner: i64,
    pub cols: i64,
    pub layout: Q12Layout,
}

impl Q12Matmul<'_> {
    /// The rows and columns that `c`, `a` and `b` must have.
    pub fn shapes(&self) -> [(i64, i64); 3] {
        let (m, n, k) = (self.rows, self.cols, self.inner);
        let (a, b) = match self.layout {
            Q12Layout::NN => ((m, k), (k, n)),
            Q12Layout::NT => ((m, k), (n, k)),
            Q12Layout::TN => ((k, m), (k, n)),
        };
        [(m, n), a, b]
    }
}
