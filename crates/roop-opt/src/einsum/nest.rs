use crate::{counter, length_name, loop_over};
use roop_syntax::{Attr, Stmt, StmtKind};

/// The loops around `inner`, outermost label first. The first loop is parallel
/// when `parallel_first`: each of its iterations writes its own part of the
/// output.
pub fn nest(labels: &[char], parallel_first: bool, inner: Vec<Stmt>) -> Vec<Stmt> {
    let Some((label, rest)) = labels.split_first() else {
        return inner;
    };
    let body = nest(rest, false, inner);
    let mut looped = loop_over(&counter(*label), &Err(length_name(*label)), body);
    if parallel_first && let StmtKind::Ancilla { body, .. } = &mut looped.kind {
        body.stmts[0].attrs = vec![Attr::Parallel { target: None }];
    }
    vec![looped]
}
