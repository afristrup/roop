use crate::{Ctx, Doc, print_block, print_params};
use roop_syntax::BuildFn;

pub fn print_build_fn(ctx: &Ctx, keyword: &str, f: &BuildFn) -> Doc {
    Doc::concat(vec![
        Doc::text(keyword),
        print_params(&f.params),
        Doc::text(" "),
        print_block(ctx, &f.body, false),
    ])
}
