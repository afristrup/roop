use crate::Behaviour;

/// One party of a session and what it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Role {
    pub name: String,
    pub behaviour: Behaviour,
}
