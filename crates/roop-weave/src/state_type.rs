/// The type of the state `q`, `p`, `aq` or `ap`: `[i64; 4]` for one sample, and
/// `[[i64; 4]; B]` for a batch, with `B` the length of the function.
pub fn state_type(width: usize, batched: bool) -> String {
    match batched {
        true => format!("[[i64; {width}]; B]"),
        false => format!("[i64; {width}]"),
    }
}
