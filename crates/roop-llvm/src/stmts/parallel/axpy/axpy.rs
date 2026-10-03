/// A loop `y[i] += alpha * x[i]` over `i` in `0..len`, naming the places it
/// works on.
pub struct Axpy<'a> {
    pub y: &'a str,
    pub x: &'a str,
    pub alpha: &'a str,
    pub len: i64,
}
