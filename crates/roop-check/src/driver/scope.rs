use crate::{Facts, Mutability};
use std::collections::HashSet;

/// What the code being checked may do: whether it is irreversible code (a
/// function declared `irrev`, or inside an `irrev` block), and which
/// functions in the program are irreversible.
#[derive(Clone, Copy)]
pub struct Scope<'a> {
    pub irrev: bool,
    /// Inside a `logged` block, where destroying updates push what they destroy.
    pub logged: bool,
    pub irreversible_fns: &'a HashSet<&'a str>,
    /// Functions a failed `try` could not roll back.
    pub non_atomic_fns: &'a HashSet<&'a str>,
    /// Functions that change the world outside the program.
    pub world_fns: &'a HashSet<&'a str>,
    /// Which parameters of each function it may write.
    pub mutability: &'a Mutability<'a>,
    /// What the enclosing `if` conditions guarantee.
    pub facts: Option<&'a Facts<'a>>,
}

impl<'a> Scope<'a> {
    pub fn inside_irrev(self) -> Scope<'a> {
        Scope {
            irrev: true,
            ..self
        }
    }

    pub fn inside_logged(self) -> Scope<'a> {
        Scope {
            logged: true,
            ..self
        }
    }
}
