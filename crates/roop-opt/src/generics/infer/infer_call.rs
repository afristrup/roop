use crate::{Scope, arg_type, unify};
use roop_syntax::Expr;
use std::collections::HashMap;

/// The lengths for a call that gave none, from the types of its arguments. None
/// when the callee is not generic, the call names its lengths, or the arguments
/// do not determine them all.
pub fn infer_call(
    scope: &Scope,
    callee: &str,
    generics: &[Expr],
    args: &[Expr],
) -> Option<Vec<Expr>> {
    if !generics.is_empty() {
        return None;
    }
    let def = scope
        .fns
        .get(callee)
        .filter(|d| !d.generics.is_empty() && !d.external)?;
    let mut bound = HashMap::new();
    for (param, arg) in def.params.iter().zip(args) {
        if let Some(ty) = arg_type(scope, arg)
            && !unify(&param.ty, &ty, &mut bound)
        {
            return None;
        }
    }
    def.generics.iter().map(|g| bound.get(g).cloned()).collect()
}
