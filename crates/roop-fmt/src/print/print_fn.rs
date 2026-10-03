use crate::{Ctx, Doc, escape, print_block, print_generics, print_params, print_test, print_type};
use roop_syntax::FnDef;

pub fn print_fn(ctx: &Ctx, f: &FnDef) -> Doc {
    if f.test {
        return print_test(ctx, f);
    }
    if let Some(einsum) = &f.einsum {
        let public = if f.public { "pub " } else { "" };
        return Doc::text(format!(
            "{public}einsum fn {}: {} = \"{}\";",
            f.name,
            print_type(&einsum.elem),
            escape(einsum.spec.as_bytes())
        ));
    }
    if let Some(target) = &f.bennett {
        let public = if f.public { "pub " } else { "" };
        return Doc::text(format!("{public}bennett fn {} = {target};", f.name));
    }
    let public = if f.public { "pub " } else { "" };
    if f.external {
        return Doc::concat(vec![
            Doc::text(format!(
                "{public}extern {}fn {}",
                if f.world { "world " } else { "" },
                f.name
            )),
            print_generics(f.generics.iter().map(Doc::text).collect()),
            print_params(&f.params),
            Doc::text(";"),
        ]);
    }
    let irrev = if f.irreversible { "irrev " } else { "" };
    Doc::concat(vec![
        Doc::text(format!("{public}{irrev}fn {}", f.name)),
        print_generics(f.generics.iter().map(Doc::text).collect()),
        print_params(&f.params),
        Doc::text(" "),
        print_block(ctx, &f.body, false),
    ])
}
