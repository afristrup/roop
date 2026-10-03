use roop_syntax::Behaviour;

/// `body[with / name]`: replaces the free variable `name`.
pub fn substitute(body: &Behaviour, name: &str, with: &Behaviour) -> Behaviour {
    match body {
        Behaviour::End => Behaviour::End,
        Behaviour::Var(v) if v == name => with.clone(),
        Behaviour::Var(v) => Behaviour::Var(v.clone()),
        Behaviour::Rec(v, _) if v == name => body.clone(),
        Behaviour::Rec(v, inner) => {
            Behaviour::Rec(v.clone(), Box::new(substitute(inner, name, with)))
        }
        Behaviour::Choice {
            kind,
            checkpoint,
            branches,
        } => Behaviour::Choice {
            kind: *kind,
            checkpoint: *checkpoint,
            branches: branches
                .iter()
                .map(|(label, next)| (label.clone(), substitute(next, name, with)))
                .collect(),
        },
    }
}
