use crate::{
    Access, CheckError, collect_accesses, expr_vars, place_root, places_overlap, private_key,
};
use roop_syntax::{Block, Expr, Span};
use std::collections::HashSet;

/// Iterations may run in any order, or at once, when every write is private
/// to its iteration and every access overlapping a write is the same
/// iteration's own cell. Read-only data is shared freely (the "in"
/// arguments of the RBLAS interface); only written data constrains anything.
pub fn check_parallel_body(
    var: &str,
    lo: &Expr,
    hi: &Expr,
    body: &Block,
    span: Span,
) -> Result<(), CheckError> {
    let mut accesses: Vec<Access> = Vec::new();
    collect_accesses(body, &mut Vec::new(), &mut accesses);
    let shared: Vec<&Access> = accesses.iter().filter(|a| !a.local).collect();
    let written = |name: &str| {
        shared
            .iter()
            .any(|a| a.write && place_root(&a.place) == name)
    };

    let mut bound_vars = HashSet::new();
    expr_vars(lo, &mut bound_vars);
    expr_vars(hi, &mut bound_vars);
    if let Some(moved) = bound_vars.into_iter().find(|v| written(v)) {
        return Err(CheckError::ParallelBoundModified {
            var: moved.into(),
            span,
        });
    }
    for access in shared.iter().filter(|a| a.write) {
        let root = place_root(&access.place);
        if root == var {
            return Err(CheckError::ParallelInductionWritten {
                var: var.into(),
                span,
            });
        }
        if private_key(&access.place, var).is_none() {
            return Err(CheckError::ParallelWriteNotDisjoint {
                var: root.into(),
                span,
            });
        }
    }
    for write in shared.iter().filter(|a| a.write) {
        for other in &shared {
            let same_cell = private_key(&write.place, var)
                .zip(private_key(&other.place, var))
                .is_some_and(|(a, b)| a == b);
            if places_overlap(&write.place, &other.place) && !same_cell {
                let var = place_root(&write.place).into();
                return Err(CheckError::ParallelCrossIteration { var, span });
            }
        }
    }
    Ok(())
}
