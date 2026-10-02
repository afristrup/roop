/// The `k`-th of `n` components of a right-nested pair held in `value`.
pub fn tuple_proj(value: &str, k: usize, n: usize) -> String {
    match (n, k) {
        (1, _) => value.to_string(),
        (_, 0) => format!("{value}.1"),
        _ => tuple_proj(&format!("{value}.2"), k - 1, n - 1),
    }
}
