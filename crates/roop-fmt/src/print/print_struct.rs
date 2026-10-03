use crate::{Ctx, Doc, print_build_fn, print_list, print_type};
use roop_syntax::StructDef;

pub fn print_struct(ctx: &Ctx, def: &StructDef) -> Doc {
    let public = if def.public { "pub " } else { "" };
    let head = Doc::text(format!("{public}struct {} ", def.name));
    let fields: Vec<Doc> = def
        .fields
        .iter()
        .map(|f| Doc::text(format!("{}: {}", f.name, print_type(&f.ty))))
        .collect();
    if def.build.is_none() && def.unbuild.is_none() {
        return Doc::concat(vec![head, print_list("{", fields, "}", true)]);
    }
    let mut lines: Vec<Doc> = fields
        .into_iter()
        .map(|f| Doc::concat(vec![f, Doc::text(",")]))
        .collect();
    let builders = [("build", &def.build), ("unbuild", &def.unbuild)];
    for (keyword, f) in builders {
        if let Some(f) = f {
            lines.push(print_build_fn(ctx, keyword, f));
        }
    }
    Doc::concat(vec![
        head,
        Doc::text("{"),
        Doc::nest(Doc::concat(vec![
            Doc::HardLine,
            Doc::join(lines, Doc::HardLine),
        ])),
        Doc::HardLine,
        Doc::text("}"),
    ])
}
