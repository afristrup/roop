use crate::{Violation, holds};
use roop_syntax::Behaviour;
use std::collections::HashSet;

/// Checkpoint compliance of a client with a server (the paper's `rho -| server`):
/// whatever either does, including rolling back to its last checkpoint, the
/// conversation never gets stuck before the client has succeeded. Decided by
/// the axiomatic system of the paper, with every configuration visited once.
pub fn compliant(client: &Behaviour, server: &Behaviour) -> Result<(), Violation> {
    let mut seen = HashSet::new();
    holds(&mut seen, &mut Vec::new(), None, client, None, server)
}
