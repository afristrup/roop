use crate::{OnName, visit_block, visit_expr, visit_pattern, visit_place, visit_type};
use roop_syntax::StmtKind;

pub fn visit_stmt(kind: &mut StmtKind, on: OnName) {
    match kind {
        StmtKind::Update { target, value, .. } | StmtKind::Overwrite { target, value, .. } => {
            visit_place(target, on);
            visit_expr(value, on);
        }
        StmtKind::Swap(a, b) => {
            visit_place(a, on);
            visit_place(b, on);
        }
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => {
            visit_expr(cond, on);
            visit_block(then_block, on);
            visit_block(else_block, on);
            visit_expr(exit, on);
        }
        StmtKind::Match { scrutinee, arms } => {
            visit_expr(scrutinee, on);
            for arm in arms {
                visit_pattern(&mut arm.pattern, on);
                visit_block(&mut arm.body, on);
                visit_expr(&mut arm.exit, on);
            }
        }
        StmtKind::Borrow { source, body, .. } => {
            visit_place(source, on);
            visit_block(body, on);
        }
        StmtKind::Irrev(body) | StmtKind::Block(body) => visit_block(body, on),
        StmtKind::Chan { ty, body, .. } => {
            visit_type(ty, on);
            visit_block(body, on);
        }
        StmtKind::Send { source: place, .. } | StmtKind::Recv { target: place, .. } => {
            visit_place(place, on)
        }
        StmtKind::Push { stack, source } => {
            visit_place(stack, on);
            visit_place(source, on);
        }
        StmtKind::Pop { stack, target } => {
            visit_place(stack, on);
            visit_place(target, on);
        }
        StmtKind::Logged { history, body } => {
            visit_place(history, on);
            visit_block(body, on);
        }
        StmtKind::Try {
            body,
            handler,
            outcome,
        } => {
            visit_block(body, on);
            visit_block(handler, on);
            if let Some(place) = outcome {
                visit_place(place, on);
            }
        }
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => {
            visit_expr(entry, on);
            visit_block(body, on);
            visit_block(step, on);
            visit_expr(until, on);
        }
        StmtKind::Ancilla { ty, init, body, .. } => {
            visit_type(ty, on);
            visit_expr(init, on);
            visit_block(body, on);
        }
        StmtKind::Call { callee, args } | StmtKind::Uncall { callee, args } => {
            on(callee);
            for arg in args {
                visit_expr(arg, on);
            }
        }
    }
}
