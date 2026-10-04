/// Unfolds the functions and the monad plumbing in the given hypothesis or goal.
/// Array accesses at literal indices are settled by their own lemmas first; the
/// rest, at an index that is a variable, are unfolded after.
pub fn unfold_simp(names: &[String], target: &str) -> String {
    format!(
        "  simp [{}, Roop.check, Roop.agetN_ok, Roop.asetN_ok, bind, Except.bind, pure, Except.pure] at {target}\n  try simp [Roop.aget, Roop.aset, Roop.agetN, Roop.asetN] at {target}\n",
        names.join(", ")
    )
}
