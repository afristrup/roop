use crate::{Scope, infer_block};
use roop_syntax::{Item, Program};
use std::collections::HashMap;

/// Fills in the lengths of calls that leave them out, from the types of the
/// arguments: `call axpy(y, x, k)` for `fn axpy<N>(y: &mut [i64; N], ..)` is
/// `call axpy<4>(y, x, k)` when `y` is `[i64; 4]`, or `<N>` when it is the
/// caller's own `[i64; N]`. A call the arguments do not settle is left alone,
/// and monomorphization reports it.
pub fn infer_lengths(program: &Program) -> Program {
    let fns: HashMap<_, _> = program
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some((f.name.as_str(), f)),
            _ => None,
        })
        .collect();
    let structs: HashMap<_, _> = program
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Struct(s) => Some((s.name.as_str(), s)),
            _ => None,
        })
        .collect();
    let items = program
        .items
        .iter()
        .map(|item| match item {
            Item::Fn(f) if !f.external => {
                let mut scope = Scope {
                    vars: f
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    fns: &fns,
                    structs: &structs,
                };
                let mut def = f.clone();
                def.body = infer_block(&mut scope, &f.body);
                Item::Fn(def)
            }
            other => other.clone(),
        })
        .collect();
    Program { items }
}
