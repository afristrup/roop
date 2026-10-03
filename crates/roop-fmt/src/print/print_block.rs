use crate::{Ctx, Doc, Lines, is_simple, print_entries};
use roop_syntax::Block;

/// `{ ... }` with one statement to a line. When `collapse` is set, a block of
/// a single simple statement is laid out with a line to break at, which the
/// caller puts in a group: it stays on the line of its header if it fits.
pub fn print_block(ctx: &Ctx, block: &Block, collapse: bool) -> Doc {
    print_block_after(ctx, block, collapse, None)
}

/// The same, with a first line of its own before the statements.
pub fn print_block_after(ctx: &Ctx, block: &Block, collapse: bool, first: Option<Doc>) -> Doc {
    let before = ctx.taken();
    let open = block.span.start + 1;
    let mut lines = Lines::new(open);
    let header = ctx.take_trailing(open);
    if let Some(c) = &header {
        lines = Lines::new(c.span.end);
    }
    print_entries(ctx, &mut lines, &block.stmts);
    for c in ctx.take_before(block.span.end) {
        lines.comment(ctx, &c, false);
    }
    let commented = ctx.taken() != before;
    let had_first = first.is_some();
    if let Some(doc) = first {
        lines.first(doc);
    }
    let docs = lines.into_docs();
    if docs.is_empty() {
        return Doc::text("{}");
    }
    let single = block.stmts.len() == 1 && is_simple(&block.stmts[0]);
    if collapse && single && !commented && !had_first {
        return Doc::concat(vec![
            Doc::text("{"),
            Doc::nest(Doc::concat(vec![Doc::Line, Doc::concat(docs)])),
            Doc::Line,
            Doc::text("}"),
        ]);
    }
    let header = header.map_or_else(Doc::nothing, |c| Doc::Suffix(format!(" {}", c.text)));
    Doc::concat(vec![
        Doc::text("{"),
        header,
        Doc::nest(Doc::concat(vec![
            Doc::HardLine,
            Doc::join(docs, Doc::HardLine),
        ])),
        Doc::HardLine,
        Doc::text("}"),
    ])
}
