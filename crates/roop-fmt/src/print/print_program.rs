use crate::{Ctx, Doc, Lines, print_item};
use roop_syntax::{Item, Span};

/// Imports and module declarations may sit together; everything else is set
/// apart by a blank line.
fn needs_gap(prev: &Item, next: &Item) -> bool {
    let compact = |i: &Item| matches!(i, Item::Mod(_) | Item::Use(_));
    !(compact(prev) && compact(next))
}

pub fn print_program(ctx: &Ctx, items: &[(Item, Span)]) -> Doc {
    let mut lines = Lines::new(0);
    let mut prev: Option<&Item> = None;
    for (item, span) in items {
        let force = prev.is_some_and(|p| needs_gap(p, item));
        let comments = ctx.take_before(span.start);
        for (i, c) in comments.iter().enumerate() {
            lines.comment(ctx, c, force && i == 0);
        }
        lines.gap(ctx, span.start, force && comments.is_empty());
        lines.entry(ctx, print_item(ctx, item), span.end);
        ctx.check_inside(span.end);
        prev = Some(item);
    }
    for c in ctx.take_before(usize::MAX) {
        lines.comment(ctx, &c, false);
    }
    let docs = lines.into_docs();
    if docs.is_empty() {
        return Doc::nothing();
    }
    Doc::concat(vec![Doc::join(docs, Doc::HardLine), Doc::HardLine])
}
