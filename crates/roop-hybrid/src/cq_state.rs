use crate::{Density, Store};
use std::collections::BTreeMap;

/// A classical-quantum state in the sense of Feng and Ying: for each classical
/// store, the partial density operator of the qubits. Traces sum to at most 1.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CqState {
    pub parts: BTreeMap<Store, Density>,
}

impl CqState {
    pub fn insert(&mut self, store: Store, rho: Density) {
        match self.parts.get_mut(&store) {
            Some(existing) => existing.add(&rho),
            None => {
                self.parts.insert(store, rho);
            }
        }
    }

    pub fn merge(&mut self, other: CqState) {
        for (store, rho) in other.parts {
            self.insert(store, rho);
        }
    }

    pub fn trace(&self) -> f64 {
        self.parts.values().map(Density::trace).sum()
    }
}
