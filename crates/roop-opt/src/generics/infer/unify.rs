use roop_syntax::{Expr, Place, Type};
use std::collections::HashMap;

/// Matches a parameter's type against an argument's and records the length
/// each generic parameter must have. False when they cannot agree.
pub fn unify(param: &Type, arg: &Type, bound: &mut HashMap<String, Expr>) -> bool {
    match (param, arg) {
        (Type::Ref { inner, .. }, _) => unify(inner, arg, bound),
        (Type::Named(a), Type::Named(b)) => a == b,
        (Type::Array(pe, pn), Type::Array(ae, an)) => pn == an && unify(pe, ae, bound),
        (Type::Stack(pe, pn), Type::Stack(ae, an)) => pn == an && unify(pe, ae, bound),
        (Type::Param { elem, len, stack }, Type::Array(ae, an)) if !stack => {
            bind(bound, len, Expr::Int(*an as i64)) && unify(elem, ae, bound)
        }
        (Type::Param { elem, len, stack }, Type::Stack(ae, an)) if *stack => {
            bind(bound, len, Expr::Int(*an as i64)) && unify(elem, ae, bound)
        }
        (
            Type::Param { elem, len, stack },
            Type::Param {
                elem: ae,
                len: al,
                stack: astack,
            },
        ) if stack == astack => {
            bind(bound, len, Expr::Place(Place::Var(al.clone()))) && unify(elem, ae, bound)
        }
        _ => false,
    }
}

fn bind(bound: &mut HashMap<String, Expr>, len: &str, value: Expr) -> bool {
    match bound.get(len) {
        Some(existing) => *existing == value,
        None => {
            bound.insert(len.to_string(), value);
            true
        }
    }
}
