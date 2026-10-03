use crate::auto_prefix;
use crate::{Ctx, Doc, Lines, print_expr, print_stmt, print_type};
use roop_syntax::{Attr, Block, Stmt, StmtKind};

/// Adds the statements to the lines of their block. An ancilla that is the
/// last statement of its block is written as `ancilla x: T = e;`, with the
/// rest of the block under it at the same indentation, instead of in braces of
/// its own.
pub fn print_entries(ctx: &Ctx, lines: &mut Lines, stmts: &[Stmt]) {
    for (i, stmt) in stmts.iter().enumerate() {
        for c in ctx.take_before(stmt.span.start) {
            lines.comment(ctx, &c, false);
        }
        lines.gap(ctx, stmt.span.start, false);
        let last = i + 1 == stmts.len();
        if let (
            true,
            true,
            StmtKind::Ancilla {
                name,
                ty,
                init,
                body,
            },
        ) = (
            last,
            stmt.attrs.iter().all(|a| *a == Attr::Auto),
            &stmt.kind,
        ) {
            let header = Doc::concat(vec![
                Doc::text(format!(
                    "{}ancilla {name}: {} = ",
                    auto_prefix(stmt),
                    print_type(ty)
                )),
                print_expr(init),
                Doc::text(";"),
            ]);
            lines.entry(ctx, header, declaration_end(ctx, body));
            print_entries(ctx, lines, &body.stmts);
            return;
        }
        lines.entry(ctx, print_stmt(ctx, stmt), stmt.span.end);
        ctx.check_inside(stmt.span.end);
    }
}

/// Where the declaration line ends: after its `;`, or after the `{` that opened
/// the body, as it was written.
fn declaration_end(ctx: &Ctx, body: &Block) -> usize {
    match ctx.byte_at(body.span.start) {
        Some(b'{') => body.span.start + 1,
        _ => body.span.start,
    }
}
