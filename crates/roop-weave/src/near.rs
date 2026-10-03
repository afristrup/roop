/// `a - e` for a number a and an integer e, as roop writes it.
fn difference(a: &str, e: i64) -> String {
    match e < 0 {
        true => format!("({a} + {})", -e),
        false => format!("({a} - {e})"),
    }
}

/// An `expect` that the number `a` is within `tolerance` of the integer `e`.
pub fn near(a: &str, e: i64, tolerance: i64) -> String {
    let d = difference(a, e);
    format!("    expect {d} * {d} <= {};\n", tolerance * tolerance)
}
