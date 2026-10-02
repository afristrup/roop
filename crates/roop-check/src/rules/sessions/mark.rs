use roop_syntax::Behaviour;

/// What a party returns to on a rollback after performing `current`: that
/// choice itself when it is a checkpoint, otherwise what it returned to before.
pub fn mark(past: &Option<Behaviour>, current: &Behaviour) -> Option<Behaviour> {
    match current {
        Behaviour::Choice {
            checkpoint: true, ..
        } => Some(current.clone()),
        _ => past.clone(),
    }
}
