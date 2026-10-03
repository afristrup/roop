use crate::{
    Ctx, Doc, print_attrs, print_block, print_call, print_expr, print_from, print_if, print_match,
    print_place, print_try, print_type, update_text,
};
use roop_syntax::{OverwriteOp, Stmt, StmtKind};

pub fn print_stmt(ctx: &Ctx, stmt: &Stmt) -> Doc {
    let body = match &stmt.kind {
        StmtKind::Update { target, op, value } => Doc::concat(vec![
            print_place(target),
            Doc::text(format!(" {} ", update_text(*op))),
            print_expr(value),
            Doc::text(";"),
        ]),
        StmtKind::Swap(a, b) => Doc::concat(vec![
            print_place(a),
            Doc::text(" <=> "),
            print_place(b),
            Doc::text(";"),
        ]),
        StmtKind::If {
            cond,
            then_block,
            else_block,
            exit,
        } => print_if(ctx, cond, then_block, else_block, exit),
        StmtKind::Match { scrutinee, arms } => print_match(ctx, scrutinee, arms, stmt.span),
        StmtKind::Borrow { name, source, body } => Doc::concat(vec![
            Doc::text(format!("borrow {name} = ")),
            print_place(source),
            Doc::text(" "),
            print_block(ctx, body, false),
        ]),
        StmtKind::Irrev(body) => {
            Doc::concat(vec![Doc::text("irrev "), print_block(ctx, body, false)])
        }
        StmtKind::Overwrite { target, op, value } => {
            let sign = match op {
                OverwriteOp::Assign => "=",
                OverwriteOp::Rem => "%=",
            };
            Doc::concat(vec![
                print_place(target),
                Doc::text(format!(" {sign} ")),
                print_expr(value),
                Doc::text(";"),
            ])
        }
        StmtKind::Block(body) => print_block(ctx, body, false),
        StmtKind::Chan { name, ty, body } => Doc::concat(vec![
            Doc::text(format!("chan {name}: {} ", print_type(ty))),
            print_block(ctx, body, false),
        ]),
        StmtKind::Send { chan, source } => Doc::concat(vec![
            Doc::text(format!("send {chan} <- ")),
            print_place(source),
            Doc::text(";"),
        ]),
        StmtKind::Recv { chan, target } => Doc::concat(vec![
            Doc::text(format!("recv {chan} -> ")),
            print_place(target),
            Doc::text(";"),
        ]),
        StmtKind::Push { stack, source } => Doc::concat(vec![
            Doc::text("push "),
            print_place(stack),
            Doc::text(" <- "),
            print_place(source),
            Doc::text(";"),
        ]),
        StmtKind::Pop { stack, target } => Doc::concat(vec![
            Doc::text("pop "),
            print_place(stack),
            Doc::text(" -> "),
            print_place(target),
            Doc::text(";"),
        ]),
        StmtKind::Logged { history, body } => Doc::concat(vec![
            Doc::text("logged "),
            print_place(history),
            Doc::text(" "),
            print_block(ctx, body, false),
        ]),
        StmtKind::Try {
            body,
            handler,
            outcome,
        } => print_try(ctx, body, handler, outcome.as_ref()),
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => print_from(ctx, entry, body, step, until),
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => Doc::concat(vec![
            Doc::text(format!("ancilla {name}: {} = ", print_type(ty))),
            print_expr(init),
            Doc::text(" "),
            print_block(ctx, body, false),
        ]),
        StmtKind::Call {
            callee,
            generics,
            args,
        } => print_call("call", callee, generics, args),
        StmtKind::Uncall {
            callee,
            generics,
            args,
        } => print_call("uncall", callee, generics, args),
    };
    let mut parts = print_attrs(&stmt.attrs);
    parts.push(body);
    Doc::concat(parts)
}
