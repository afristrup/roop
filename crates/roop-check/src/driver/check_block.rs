use crate::{
    CheckError, Facts, Scope, check_ancilla, check_borrow, check_call_aliasing, check_no_world,
    check_parallel, check_try, check_uncall_after_keep, check_update,
};
use roop_syntax::{Block, StmtKind};

/// Checks a block. In reversible code every update must be invertible and every
/// ancilla restored; irreversible code (`irrev`) lifts those rules but keeps
/// the safety ones: borrows, parallel loops and concurrent tasks.
pub fn check_block(block: &Block, scope: Scope) -> Result<(), CheckError> {
    check_uncall_after_keep(block, scope.keeping_fns)?;
    for stmt in &block.stmts {
        check_no_world(stmt, scope.world_fns)?;
        check_parallel(stmt)?;
        let outside = || CheckError::IrreversibleOutsideIrrev { span: stmt.span };
        match &stmt.kind {
            StmtKind::Update { target, value, .. } => {
                if !scope.irrev {
                    check_update(target, value, stmt.span, scope.facts)?;
                }
            }
            StmtKind::Overwrite { .. } if !scope.irrev && !scope.logged => return Err(outside()),
            StmtKind::Overwrite { target, value, .. } => {
                if !scope.irrev {
                    check_update(target, value, stmt.span, scope.facts)?;
                }
            }
            StmtKind::Swap(..) => {}
            StmtKind::Send { .. } | StmtKind::Recv { .. } => {}
            StmtKind::Push { .. } | StmtKind::Pop { .. } | StmtKind::Keep(_) => {}
            StmtKind::Logged { body, .. } => check_block(body, scope.inside_logged())?,
            StmtKind::Call { callee, args, .. } => {
                check_call_aliasing(callee, args, stmt.span, scope.facts, scope.mutability)?;
                if scope.irreversible_fns.contains(callee.as_str()) && !scope.irrev {
                    return Err(CheckError::CallsIrreversible {
                        callee: callee.clone(),
                        span: stmt.span,
                    });
                }
            }
            StmtKind::Uncall { callee, args, .. } => {
                check_call_aliasing(callee, args, stmt.span, scope.facts, scope.mutability)?;
                if scope.irreversible_fns.contains(callee.as_str()) {
                    return Err(CheckError::UncallIrreversible {
                        callee: callee.clone(),
                        span: stmt.span,
                    });
                }
            }
            StmtKind::Block(block) | StmtKind::Chan { body: block, .. } => {
                check_block(block, scope)?
            }
            StmtKind::Irrev(block) => check_block(block, scope.inside_irrev())?,
            StmtKind::If {
                cond,
                then_block,
                else_block,
                ..
            } => {
                let fact = Facts {
                    cond,
                    parent: scope.facts,
                };
                check_block(
                    then_block,
                    Scope {
                        facts: Some(&fact),
                        ..scope
                    },
                )?;
                check_block(else_block, scope)?;
            }
            StmtKind::Match { arms, .. } => {
                for arm in arms {
                    check_block(&arm.body, scope)?;
                }
            }
            StmtKind::Borrow { source, body, .. } => check_borrow(source, body, stmt.span, scope)?,
            StmtKind::Try {
                body,
                handler,
                outcome,
            } => {
                if outcome.is_none() && !scope.irrev {
                    return Err(outside());
                }
                check_try(body, handler, outcome.as_ref(), stmt.span, scope)?;
            }
            StmtKind::From { body, step, .. } => {
                check_block(body, scope)?;
                check_block(step, scope)?;
            }
            StmtKind::Ancilla {
                name, init, body, ..
            } => {
                if !scope.irrev {
                    check_ancilla(name, init, body, stmt.span, scope.mutability)?;
                }
                check_block(body, scope)?;
            }
        }
    }
    Ok(())
}
