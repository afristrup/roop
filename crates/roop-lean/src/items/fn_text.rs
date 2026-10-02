use crate::Lifted;

/// A translated function definition and what its theorems need to know.
pub struct FnText {
    pub text: String,
    /// The loop pieces the function's text refers to.
    pub lifted: String,
    pub pieces: Vec<Lifted>,
    pub ancillas: usize,
}
