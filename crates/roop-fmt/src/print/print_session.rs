use crate::{Doc, print_behaviour};
use roop_syntax::SessionDef;

pub fn print_session(def: &SessionDef) -> Doc {
    let public = if def.public { "pub " } else { "" };
    let roles = def
        .roles
        .iter()
        .map(|r| {
            Doc::concat(vec![
                Doc::text(format!("{}: ", r.name)),
                print_behaviour(&r.behaviour),
                Doc::text(";"),
            ])
        })
        .collect();
    Doc::concat(vec![
        Doc::text(format!("{public}session {} {{", def.name)),
        Doc::nest(Doc::concat(vec![
            Doc::HardLine,
            Doc::join(roles, Doc::HardLine),
        ])),
        Doc::HardLine,
        Doc::text("}"),
    ])
}
