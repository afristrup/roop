use crate::{StmtKind, Visitor, walk_block, walk_expr, walk_pattern, walk_place, walk_type};

pub fn walk_stmt(v: &mut dyn Visitor, kind: &mut StmtKind) {
    match kind {
        StmtKind::Update { target, value, .. } | StmtKind::Overwrite { target, value, .. } => {
            walk_place(v, target);
            walk_expr(v, value);
        }
        StmtKind::Swap(a, b) => {
            walk_place(v, a);
            walk_place(v, b);
        }
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => {
            walk_expr(v, cond);
            walk_block(v, then_block);
            walk_block(v, else_block);
            walk_expr(v, exit);
        }
        StmtKind::Match { scrutinee, arms } => {
            walk_expr(v, scrutinee);
            for arm in arms {
                walk_pattern(v, &mut arm.pattern);
                walk_block(v, &mut arm.body);
                walk_expr(v, &mut arm.exit);
            }
        }
        StmtKind::Borrow { source, body, .. } => {
            walk_place(v, source);
            walk_block(v, body);
        }
        StmtKind::Irrev(body) | StmtKind::Block(body) => walk_block(v, body),
        StmtKind::Chan { ty, body, .. } => {
            walk_type(v, ty);
            walk_block(v, body);
        }
        StmtKind::Send { source: place, .. } | StmtKind::Recv { target: place, .. } => {
            walk_place(v, place)
        }
        StmtKind::Push { stack, source } => {
            walk_place(v, stack);
            walk_place(v, source);
        }
        StmtKind::Pop { stack, target } => {
            walk_place(v, stack);
            walk_place(v, target);
        }
        StmtKind::Logged { history, body } => {
            walk_place(v, history);
            walk_block(v, body);
        }
        StmtKind::Try {
            body,
            handler,
            outcome,
        } => {
            walk_block(v, body);
            walk_block(v, handler);
            if let Some(place) = outcome {
                walk_place(v, place);
            }
        }
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => {
            walk_expr(v, entry);
            walk_block(v, body);
            walk_block(v, step);
            walk_expr(v, until);
        }
        StmtKind::Ancilla { ty, init, body, .. } => {
            walk_type(v, ty);
            walk_expr(v, init);
            walk_block(v, body);
        }
        StmtKind::Call {
            callee,
            generics,
            args,
        }
        | StmtKind::Uncall {
            callee,
            generics,
            args,
        } => {
            v.call(callee, generics);
            for g in generics {
                walk_expr(v, g);
            }
            for arg in args {
                walk_expr(v, arg);
            }
        }
    }
}
