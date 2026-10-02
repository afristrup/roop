use std::collections::HashSet;

/// What the code being checked may do: whether it is irreversible code (a
/// function declared `irrev`, or inside an `irrev` block), and which
/// functions in the program are irreversible.
#[derive(Clone, Copy)]
pub struct Scope<'a> {
    pub irrev: bool,
    pub irreversible_fns: &'a HashSet<&'a str>,
}

impl<'a> Scope<'a> {
    pub fn inside_irrev(self) -> Scope<'a> {
        Scope {
            irrev: true,
            ..self
        }
    }
}
