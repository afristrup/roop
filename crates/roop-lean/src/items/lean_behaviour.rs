use roop_syntax::{Behaviour, ChoiceKind};

/// A behaviour as a Lean term of `Roop.Session.B`.
pub fn lean_behaviour(behaviour: &Behaviour) -> String {
    match behaviour {
        Behaviour::End => "Roop.Session.B.done".into(),
        Behaviour::Var(name) => format!("(Roop.Session.B.var \"{name}\")"),
        Behaviour::Rec(name, body) => {
            format!("(Roop.Session.B.mu \"{name}\" {})", lean_behaviour(body))
        }
        Behaviour::Choice {
            kind,
            checkpoint,
            branches,
        } => {
            let branches: Vec<String> = branches
                .iter()
                .map(|(label, next)| format!("(\"{label}\", {})", lean_behaviour(next)))
                .collect();
            format!(
                "(Roop.Session.B.choice {} {checkpoint} [{}])",
                *kind == ChoiceKind::Select,
                branches.join(", ")
            )
        }
    }
}
