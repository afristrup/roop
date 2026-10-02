/// Unfolds the functions and the monad plumbing in the given hypothesis or goal.
pub fn unfold_simp(names: &[String], target: &str) -> String {
    format!(
        "  simp [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at {target}\n",
        names.join(", ")
    )
}
