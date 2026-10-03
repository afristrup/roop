/// A number as weave stores it: the integer nearest to x * 4096.
pub fn quantize(x: f64) -> i64 {
    (x * 4096.0).round() as i64
}
