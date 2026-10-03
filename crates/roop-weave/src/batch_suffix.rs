/// The ending of the names of the functions that work on a batch.
pub fn batch_suffix(batched: bool) -> &'static str {
    match batched {
        true => "_batch",
        false => "",
    }
}
