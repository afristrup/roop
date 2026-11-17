use crate::{
    CheckError, QubitScope, holds_qubit, index_qubit, place_root, read_place_qubit, read_qubit,
};
use roop_syntax::{Block, Expr, Place, Span, StmtKind};

/// No-cloning: a qubit is never read as a value. It may be an argument of a
/// call, swapped with another qubit, kept, or be an ancilla that starts at
/// zero; anything else that mentions it is an error.
pub fn check_qubit_block(block: &Block, scope: &mut QubitScope) -> Result<(), CheckError> {
    for stmt in &block.stmts {
        let span = stmt.span;
        match &stmt.kind {
            StmtKind::Update {
                target, value: v, ..
            }
            | StmtKind::Overwrite {
                target, value: v, ..
            } => {
                place(target, scope, span)?;
                value(v, scope, span)?;
            }
            StmtKind::Swap(a, b) => {
                let (qa, qb) = (scope.is_qubit(place_root(a)), scope.is_qubit(place_root(b)));
                if qa != qb {
                    let var = if qa { place_root(a) } else { place_root(b) };
                    return Err(misuse(var.into(), span));
                }
                for p in [a, b] {
                    if let Some(var) = index_qubit(p, scope) {
                        return Err(misuse(var, span));
                    }
                }
            }
            StmtKind::If {
                cond,
                then_block,
                else_block,
                exit,
            } => {
                value(cond, scope, span)?;
                value(exit, scope, span)?;
                check_qubit_block(then_block, scope)?;
                check_qubit_block(else_block, scope)?;
            }
            StmtKind::Match { scrutinee, arms } => {
                value(scrutinee, scope, span)?;
                for arm in arms {
                    value(&arm.exit, scope, span)?;
                    check_qubit_block(&arm.body, scope)?;
                }
            }
            StmtKind::Borrow { name, source, body } => {
                place(source, scope, span)?;
                scope.push(name, false);
                check_qubit_block(body, scope)?;
                scope.pop();
            }
            StmtKind::Irrev(body) | StmtKind::Block(body) => check_qubit_block(body, scope)?,
            StmtKind::Logged { history, body } => {
                place(history, scope, span)?;
                check_qubit_block(body, scope)?;
            }
            StmtKind::Try {
                body,
                handler,
                outcome,
            } => {
                if let Some(outcome) = outcome {
                    place(outcome, scope, span)?;
                }
                check_qubit_block(body, scope)?;
                check_qubit_block(handler, scope)?;
            }
            StmtKind::Chan { name, body, .. } => {
                scope.push(name, false);
                check_qubit_block(body, scope)?;
                scope.pop();
            }
            StmtKind::Send { source: p, .. } | StmtKind::Recv { target: p, .. } => {
                place(p, scope, span)?
            }
            StmtKind::Push { stack, source: p } | StmtKind::Pop { stack, target: p } => {
                place(stack, scope, span)?;
                place(p, scope, span)?;
            }
            StmtKind::Keep(p) => {
                if let Some(var) = index_qubit(p, scope) {
                    return Err(misuse(var, span));
                }
            }
            StmtKind::From {
                entry,
                body,
                step,
                until,
            } => {
                value(entry, scope, span)?;
                value(until, scope, span)?;
                check_qubit_block(body, scope)?;
                check_qubit_block(step, scope)?;
            }
            StmtKind::Ancilla {
                name,
                ty,
                init,
                body,
            } => {
                let qubit = holds_qubit(ty);
                if qubit && *init != Expr::Int(0) {
                    return Err(CheckError::QubitAncillaStart {
                        name: name.clone(),
                        span,
                    });
                }
                if !qubit {
                    value(init, scope, span)?;
                }
                scope.push(name, qubit);
                check_qubit_block(body, scope)?;
                scope.pop();
            }
            StmtKind::Call { generics, args, .. } | StmtKind::Uncall { generics, args, .. } => {
                generics.iter().try_for_each(|g| value(g, scope, span))?;
                for arg in args {
                    match arg {
                        Expr::Place(p) if scope.is_qubit(place_root(p)) => {
                            if let Some(var) = index_qubit(p, scope) {
                                return Err(misuse(var, span));
                            }
                        }
                        _ => value(arg, scope, span)?,
                    }
                }
            }
        }
    }
    Ok(())
}

fn value(e: &Expr, scope: &QubitScope, span: Span) -> Result<(), CheckError> {
    read_qubit(e, scope).map_or(Ok(()), |var| Err(misuse(var, span)))
}

fn place(p: &Place, scope: &QubitScope, span: Span) -> Result<(), CheckError> {
    read_place_qubit(p, scope).map_or(Ok(()), |var| Err(misuse(var, span)))
}

fn misuse(var: String, span: Span) -> CheckError {
    CheckError::QubitAsValue { var, span }
}
