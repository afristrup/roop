use crate::{Binding, Gate, HybridError, Layout, Op};
use roop_syntax::{FnDef, Span};
use std::collections::{HashMap, HashSet};

/// How deep calls are inlined before recursion is refused.
pub const MAX_INLINE_DEPTH: usize = 64;

/// The state of lowering a roop program into the hybrid IR: the functions
/// it may call, the registers allocated so far, and the names in scope in
/// the function being inlined, innermost last.
pub struct Lowerer<'a> {
    pub fns: HashMap<&'a str, &'a FnDef>,
    pub irreversible: HashSet<&'a str>,
    pub layout: Layout,
    pub frame: Vec<(String, Binding)>,
    pub depth: usize,
}

impl<'a> Lowerer<'a> {
    pub fn lookup(&self, name: &str, span: Span) -> Result<Binding, HybridError> {
        self.frame
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, b)| b.clone())
            .ok_or_else(|| {
                unsupported(
                    &format!("`{name}`, which is not a scalar or a qubit,"),
                    span,
                )
            })
    }

    pub fn fresh(names: &mut Vec<String>, name: &str) -> usize {
        let mut unique = name.to_string();
        let mut n = 1;
        while names.contains(&unique) {
            unique = format!("{name}_{n}");
            n += 1;
        }
        names.push(unique);
        names.len() - 1
    }
}

pub fn unsupported(what: &str, span: Span) -> HybridError {
    HybridError::Unsupported {
        what: what.into(),
        start: span.start,
        end: span.end,
    }
}

/// Appends a gate to the kernel at the end of `out`, or starts one.
pub fn push_gate(out: &mut Vec<Op>, gate: Gate) {
    match out.last_mut() {
        Some(Op::Kernel(gates)) => gates.push(gate),
        _ => out.push(Op::Kernel(vec![gate])),
    }
}
