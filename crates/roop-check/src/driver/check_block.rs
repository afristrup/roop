use crate::{
    CheckError, Scope, check_ancilla, check_borrow, check_parallel, check_try, check_update,
};
use roop_syntax::{Block, StmtKind};

/// Checks a block. In reversible code every update must be invertible and every
/// ancilla restored; irreversible code (`irrev`) lifts those rules but keeps
/// the safety ones: borrows, parallel loops and concurrent tasks.
pub fn check_block(block: &Block, scope: Scope) -> Result<(), CheckError> {
    for stmt in &block.stmts {
        check_parallel(stmt)?;
        let outside = || CheckError::IrreversibleOutsideIrrev { span: stmt.span };
        match &stmt.kind {
            StmtKind::Update { target, value, .. } => {
                if !scope.irrev {
                    check_update(target, value, stmt.span)?;
                }
            }
            StmtKind::Overwrite { .. } if !scope.irrev => return Err(outside()),
            StmtKind::Overwrite { .. } | StmtKind::Swap(..) => {}
            StmtKind::Send { .. } | StmtKind::Recv { .. } => {}
            StmtKind::Call { callee, .. } => {
                if scope.irreversible_fns.contains(callee.as_str()) && !scope.irrev {
                    return Err(CheckError::CallsIrreversible {
                        callee: callee.clone(),
                        span: stmt.span,
                    });
                }
            }
            StmtKind::Uncall { callee, .. } => {
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
                then_block,
                else_block,
                ..
            } => {
                check_block(then_block, scope)?;
                check_block(else_block, scope)?;
            }
            StmtKind::Match { arms, .. } => {
                for arm in arms {
                    check_block(&arm.body, scope)?;
                }
            }
            StmtKind::Borrow { source, body, .. } => check_borrow(source, body, stmt.span, scope)?,
            StmtKind::Try { body, handler } => {
                if !scope.irrev {
                    return Err(outside());
                }
                check_try(body, handler, stmt.span, scope)?;
            }
            StmtKind::From { body, step, .. } => {
                check_block(body, scope)?;
                check_block(step, scope)?;
            }
            StmtKind::Ancilla { name, body, .. } => {
                if !scope.irrev {
                    check_ancilla(name, body, stmt.span)?;
                }
                check_block(body, scope)?;
            }
        }
    }
    Ok(())
}
