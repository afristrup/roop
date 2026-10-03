use crate::{GenericError, Instances, eval_const};
use roop_syntax::{Expr, Type, Visitor};
use std::collections::HashMap;

/// Substitutes lengths for generic parameters and replaces calls to generic
/// functions with calls to their instances. Keeps the first error.
pub struct Rewriter<'a, 'p> {
    pub env: &'a HashMap<String, i64>,
    pub instances: &'a mut Instances<'p>,
    pub error: Option<GenericError>,
}

impl Visitor for Rewriter<'_, '_> {
    fn expr(&mut self, expr: &mut Expr) {
        if let Some((n, true)) = eval_const(expr, self.env) {
            *expr = Expr::Int(n);
        }
    }

    fn ty(&mut self, ty: &mut Type) {
        let Type::Param { elem, len, stack } = ty else {
            return;
        };
        let Some(&n) = self.env.get(len.as_str()).filter(|n| **n >= 0) else {
            self.error.get_or_insert(GenericError::Unbound(len.clone()));
            return;
        };
        let elem = elem.clone();
        *ty = if *stack {
            Type::Stack(elem, n as u64)
        } else {
            Type::Array(elem, n as u64)
        };
    }

    fn call(&mut self, callee: &mut String, generics: &mut Vec<Expr>) {
        let Some(def) = self.instances.defs.get(callee.as_str()) else {
            if !generics.is_empty() {
                self.error
                    .get_or_insert(GenericError::NotGeneric(callee.clone()));
            }
            return;
        };
        let expected = def.generics.len();
        if generics.len() != expected {
            self.error.get_or_insert(GenericError::Arity {
                callee: callee.clone(),
                expected,
                found: generics.len(),
            });
            return;
        }
        let lengths: Option<Vec<i64>> = generics
            .iter()
            .map(|g| eval_const(g, self.env).map(|(n, _)| n))
            .collect();
        match lengths {
            Some(lengths) if lengths.iter().all(|n| *n >= 0) => {
                *callee = self.instances.request(callee, lengths);
                generics.clear();
            }
            Some(_) => {
                self.error
                    .get_or_insert(GenericError::Negative(callee.clone()));
            }
            None => {
                self.error
                    .get_or_insert(GenericError::NotConstant(callee.clone()));
            }
        }
    }
}
