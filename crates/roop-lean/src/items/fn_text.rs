use crate::LoopInfo;

/// A translated function definition and what its theorems need to know.
pub struct FnText {
    pub text: String,
    /// The loop pieces the function's text refers to.
    pub lifted: String,
    pub loops: Vec<LoopInfo>,
    pub ancillas: usize,
}
