/// The names in scope, innermost last, and whether each holds a qubit.
#[derive(Default)]
pub struct QubitScope {
    names: Vec<(String, bool)>,
}

impl QubitScope {
    pub fn push(&mut self, name: &str, qubit: bool) {
        self.names.push((name.into(), qubit));
    }

    pub fn pop(&mut self) {
        self.names.pop();
    }

    pub fn is_qubit(&self, name: &str) -> bool {
        self.names
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .is_some_and(|(_, q)| *q)
    }
}
