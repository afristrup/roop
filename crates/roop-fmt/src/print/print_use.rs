use crate::{Doc, print_list};
use roop_syntax::{UseDecl, UseShape};

pub fn print_use(decl: &UseDecl) -> Doc {
    let public = if decl.public { "pub " } else { "" };
    let path = decl.path.join("::");
    let alias = decl
        .alias
        .as_ref()
        .map_or_else(String::new, |a| format!(" as {a}"));
    let tail = match &decl.shape {
        UseShape::Single => Doc::nothing(),
        UseShape::Glob => Doc::text("::*"),
        UseShape::Group(names) => {
            let items = names
                .iter()
                .map(|(name, alias)| match alias {
                    Some(a) => Doc::text(format!("{name} as {a}")),
                    None => Doc::text(name),
                })
                .collect();
            Doc::concat(vec![Doc::text("::"), print_list("{", items, "}", false)])
        }
    };
    Doc::concat(vec![
        Doc::text(format!("{public}use {path}")),
        tail,
        Doc::text(format!("{alias};")),
    ])
}
