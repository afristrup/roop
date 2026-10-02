use roop_syntax::Target;

/// Loops may fuse when their targets agree or one leaves it to the compiler.
pub fn merge_target(a: Option<Target>, b: Option<Target>) -> Option<Option<Target>> {
    match (a, b) {
        (x, None) | (None, x) => Some(x),
        (Some(x), Some(y)) if x == y => Some(Some(x)),
        _ => None,
    }
}
