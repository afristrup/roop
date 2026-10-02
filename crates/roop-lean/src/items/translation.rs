/// The result of translating a program.
#[derive(Debug, Default)]
pub struct Translation {
    /// A complete Lean file: the prelude, the declarations, the functions and
    /// their theorems.
    pub lean: String,
    /// Reversible functions, each with `f_inv_f` and `f_f_inv` theorems.
    pub reversible: Vec<String>,
    /// Irreversible functions: translated forward only, with no theorems.
    pub forward_only: Vec<String>,
    /// Theorems left as `sorry` because proving them needs an induction (loops).
    pub open: Vec<String>,
    /// Functions that could not be translated, with the reason.
    pub skipped: Vec<(String, String)>,
}
