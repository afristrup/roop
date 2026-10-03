use crate::{esc_thm, lean_behaviour};
use roop_syntax::SessionDef;

/// Lean checks what the compiler's checker decided: the client is checkpoint
/// compliant with the server (the dual of the client when only one role is
/// given), and every declared role with its own dual, which the paper proves
/// for all behaviours.
pub fn lean_session(def: &SessionDef) -> String {
    let (client, server) = match def.roles.as_slice() {
        [only] => {
            let client = lean_behaviour(&only.behaviour);
            (client.clone(), format!("(Roop.Session.dual {client})"))
        }
        [client, server, ..] => (
            lean_behaviour(&client.behaviour),
            lean_behaviour(&server.behaviour),
        ),
        [] => return String::new(),
    };
    let mut text = format!(
        "theorem {} : Roop.Session.compliant {client} {server} = true := by decide\n",
        esc_thm(&format!("{}_compliant", def.name))
    );
    for role in &def.roles {
        let behaviour = lean_behaviour(&role.behaviour);
        text.push_str(&format!(
            "theorem {} : Roop.Session.compliant {behaviour} (Roop.Session.dual {behaviour}) = true := by decide\n",
            esc_thm(&format!("{}_{}_dual", def.name, role.name))
        ));
    }
    text
}
