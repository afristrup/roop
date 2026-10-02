use roop_syntax::Type;

/// Variables in scope with their types, innermost last.
#[derive(Clone, Default)]
pub struct Env {
    pub vars: Vec<(String, Type)>,
    /// Translating an irreversible function: no inverse, no restoration checks.
    pub irreversible: bool,
    /// The function being translated; names the definitions lifted out of it.
    pub function: String,
}

impl Env {
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.vars
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
    }

    /// Each visible variable once, with its innermost type.
    pub fn visible(&self) -> Vec<(String, Type)> {
        let mut seen: Vec<&String> = Vec::new();
        for (name, _) in &self.vars {
            if !seen.contains(&name) {
                seen.push(name);
            }
        }
        seen.into_iter()
            .filter_map(|name| Some((name.clone(), self.lookup(name)?.clone())))
            .collect()
    }
}
