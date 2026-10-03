use roop_syntax::Behaviour;

/// The behaviour that meets `behaviour` halfway: every offer becomes a
/// selection and the other way round.
pub fn dual(behaviour: &Behaviour) -> Behaviour {
    match behaviour {
        Behaviour::End => Behaviour::End,
        Behaviour::Var(name) => Behaviour::Var(name.clone()),
        Behaviour::Rec(name, body) => Behaviour::Rec(name.clone(), Box::new(dual(body))),
        Behaviour::Choice {
            kind,
            checkpoint,
            branches,
        } => Behaviour::Choice {
            kind: kind.dual(),
            checkpoint: *checkpoint,
            branches: branches
                .iter()
                .map(|(label, next)| (label.clone(), dual(next)))
                .collect(),
        },
    }
}
