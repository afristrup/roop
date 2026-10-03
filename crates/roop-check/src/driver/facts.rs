use roop_syntax::Expr;

/// The conditions known to hold at a point in the code: those of the `if`
/// branches around it. A linked list on the stack, innermost first.
#[derive(Clone, Copy)]
pub struct Facts<'a> {
    pub cond: &'a Expr,
    pub parent: Option<&'a Facts<'a>>,
}
