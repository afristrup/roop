/// Which operand of an integer matrix product is read transposed: `NN` is
/// `a[i][j] b[j][k]`, `NT` is `a[i][j] b[k][j]` and `TN` is `a[j][i] b[j][k]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntLayout {
    NN,
    NT,
    TN,
}

impl IntLayout {
    /// The number the runtime's kernels take for it.
    pub fn code(self) -> i64 {
        match self {
            Self::NN => 0,
            Self::NT => 1,
            Self::TN => 2,
        }
    }
}
