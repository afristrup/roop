use crate::{Ctx, Doc, print_block, print_expr, print_optional_block};
use roop_syntax::{Block, Expr};

/// The body breaks with the header, and the step stays on the line of the
/// closing brace when it fits: `} loop { i += 1; } until i == N - 1;`.
pub fn print_from(ctx: &Ctx, entry: &Expr, body: &Block, step: &Block, until: &Expr) -> Doc {
    let head = Doc::group(Doc::concat(vec![
        Doc::text("from "),
        print_expr(entry),
        Doc::text(" "),
        print_block(ctx, body, true),
    ]));
    let mut parts = vec![head];
    if let Some(block) = print_optional_block(ctx, step, true) {
        parts.extend([Doc::text(" loop "), Doc::group(block)]);
    }
    parts.extend([Doc::text(" until "), print_expr(until), Doc::text(";")]);
    Doc::concat(parts)
}
