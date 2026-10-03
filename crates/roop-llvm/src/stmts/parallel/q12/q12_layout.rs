/// Which operand of a `q12` matrix product is read transposed: `NN` is `a[i][j]
/// b[j][k]`, `NT` is `a[i][j] b[k][j]` and `TN` is `a[j][i] b[j][k]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Q12Layout {
    NN,
    NT,
    TN,
}

impl Q12Layout {
    /// The number the runtime's kernel takes for it.
    pub fn code(self) -> i64 {
        match self {
            Self::NN => 0,
            Self::NT => 1,
            Self::TN => 2,
        }
    }
}
