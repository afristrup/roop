use roop_syntax::Type;

/// Variables in scope with their types, innermost last.
#[derive(Clone, Default)]
pub struct Env {
    pub vars: Vec<(String, Type)>,
    /// Translating an irreversible function: no inverse, no restoration checks.
    pub irreversible: bool,
}

impl Env {
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.vars
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
    }
}
