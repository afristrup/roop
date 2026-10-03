use roop_syntax::{Expr, Place};

/// The matrix `m` when `place` is `m[row][col]`.
pub fn cell<'a>(place: &'a Place, row: &str, col: &str) -> Option<&'a str> {
    let Place::Index(outer, col_index) = place else {
        return None;
    };
    let Place::Index(matrix, row_index) = &**outer else {
        return None;
    };
    let Place::Var(name) = &**matrix else {
        return None;
    };
    (is_var(col_index, col) && is_var(row_index, row)).then_some(name.as_str())
}

fn is_var(expr: &Expr, name: &str) -> bool {
    matches!(expr, Expr::Place(Place::Var(v)) if v == name)
}
