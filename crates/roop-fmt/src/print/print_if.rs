use crate::{Ctx, Doc, print_block, print_expr, print_optional_block};
use roop_syntax::{Block, Expr};

pub fn print_if(
    ctx: &Ctx,
    cond: &Expr,
    then_block: &Block,
    else_block: &Block,
    exit: &Expr,
) -> Doc {
    let bare = |b: &Block| b.stmts.is_empty() && !ctx.has_comment_in(b.span);
    if bare(then_block) && bare(else_block) && *exit == Expr::Bool(true) {
        return Doc::concat(vec![Doc::text("expect "), print_expr(cond), Doc::text(";")]);
    }
    let mut parts = vec![
        Doc::text("if "),
        print_expr(cond),
        Doc::text(" "),
        print_block(ctx, then_block, true),
    ];
    if let Some(block) = print_optional_block(ctx, else_block, true) {
        parts.extend([Doc::text(" else "), block]);
    }
    parts.extend([Doc::text(" fi "), print_expr(exit), Doc::text(";")]);
    Doc::group(Doc::concat(parts))
}
