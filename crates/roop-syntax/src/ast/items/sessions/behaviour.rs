use crate::ChoiceKind;

/// A session behaviour with checkpoints (Barbanera, Dezani-Ciancaglini and
/// de'Liguoro): what one party of a conversation does.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Behaviour {
    /// Success: nothing more to do.
    End,
    /// A choice among actions, each followed by a behaviour. A checkpoint
    /// marks the point either party can roll the conversation back to.
    Choice {
        kind: ChoiceKind,
        checkpoint: bool,
        branches: Vec<(String, Behaviour)>,
    },
    Rec(String, Box<Behaviour>),
    Var(String),
}
