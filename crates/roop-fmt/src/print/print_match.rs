use crate::{Ctx, Doc, Lines, print_block, print_expr, print_pattern};
use roop_syntax::{Expr, MatchArm, Span};

pub fn print_match(ctx: &Ctx, scrutinee: &Expr, arms: &[MatchArm], span: Span) -> Doc {
    let mut lines = Lines::new(span.start);
    for arm in arms {
        for c in ctx.take_before(arm.body.span.start) {
            lines.comment(ctx, &c, false);
        }
        let doc = Doc::group(Doc::concat(vec![
            Doc::text(format!("{} => ", print_pattern(&arm.pattern))),
            print_block(ctx, &arm.body, true),
            Doc::text(" assert "),
            print_expr(&arm.exit),
            Doc::text(";"),
        ]));
        lines.entry(ctx, doc, arm.body.span.end);
    }
    for c in ctx.take_before(span.end) {
        lines.comment(ctx, &c, false);
    }
    Doc::concat(vec![
        Doc::text("match "),
        print_expr(scrutinee),
        Doc::text(" {"),
        Doc::nest(Doc::concat(vec![
            Doc::HardLine,
            Doc::join(lines.into_docs(), Doc::HardLine),
        ])),
        Doc::HardLine,
        Doc::text("}"),
    ])
}
