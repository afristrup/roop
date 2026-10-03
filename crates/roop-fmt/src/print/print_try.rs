use crate::{Ctx, Doc, print_block, print_place};
use roop_syntax::{Block, Place};

pub fn print_try(ctx: &Ctx, body: &Block, handler: &Block, outcome: Option<&Place>) -> Doc {
    let mut parts = vec![
        Doc::text("try "),
        print_block(ctx, body, true),
        Doc::text(" catch_rollback "),
        print_block(ctx, handler, true),
    ];
    if let Some(place) = outcome {
        parts.extend([Doc::text(" -> "), print_place(place), Doc::text(";")]);
    }
    Doc::group(Doc::concat(parts))
}
