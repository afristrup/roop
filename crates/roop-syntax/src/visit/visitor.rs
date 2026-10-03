use crate::{Expr, Type};

/// Hooks for walking and rewriting a program in place. Every hook has a
/// do-nothing default, and the `walk_*` functions supply the traversal.
pub trait Visitor {
    /// A name that refers to a function, struct or enum.
    fn name(&mut self, _name: &mut String) {}

    /// An expression, before its parts are walked.
    fn expr(&mut self, _expr: &mut Expr) {}

    /// A type, before its parts are walked.
    fn ty(&mut self, _ty: &mut Type) {}

    /// A call or uncall, before its arguments are walked.
    fn call(&mut self, callee: &mut String, _generics: &mut Vec<Expr>) {
        self.name(callee);
    }
}
