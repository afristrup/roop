/// Who decides a choice: the party that waits for an action (external) or the
/// one that picks it (internal).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChoiceKind {
    Offer,
    Select,
}

impl ChoiceKind {
    pub fn dual(self) -> Self {
        match self {
            Self::Offer => Self::Select,
            Self::Select => Self::Offer,
        }
    }
}
