use crate::{Ctx, Doc, print_block_after, print_type};
use roop_syntax::{FnDef, Type};

/// `test name { fixtures; statements }`, where the fixtures are the
/// parameters, which the parser made references.
pub fn print_test(ctx: &Ctx, f: &FnDef) -> Doc {
    let fixtures: Vec<Doc> = f
        .params
        .iter()
        .map(|p| {
            let ty = match &p.ty {
                Type::Ref { inner, .. } => inner,
                other => other,
            };
            Doc::text(format!("{}: {}", p.name, print_type(ty)))
        })
        .collect();
    let first = (!fixtures.is_empty()).then(|| {
        let sep = Doc::concat(vec![Doc::text(","), Doc::Line]);
        let list = Doc::group(Doc::nest(Doc::join(fixtures, sep)));
        Doc::concat(vec![list, Doc::text(";")])
    });
    Doc::concat(vec![
        Doc::text(format!("test {} ", f.name)),
        print_block_after(ctx, &f.body, false, first),
    ])
}
