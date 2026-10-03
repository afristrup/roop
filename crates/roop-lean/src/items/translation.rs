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
    /// Reversible functions that use floating point. Rounding makes `x + k - k`
    /// differ from `x`, so no roundtrip theorem is stated for them.
    pub inexact: Vec<String>,
    /// Functions with a `try`: undone only on states they produced, so they
    /// get the roundtrip theorem in one direction.
    pub one_way: Vec<String>,
    /// Functions whose `#[parallel]` loops are proved order-independent.
    pub parallel: Vec<String>,
    /// Sessions whose checkpoint compliance Lean proved.
    pub sessions: Vec<String>,
    /// Functions that could not be translated, with the reason.
    pub skipped: Vec<(String, String)>,
}
