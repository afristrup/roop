use crate::Kind;

/// The names a program has allocated, in the order it declared them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Signature {
    pub entries: Vec<(String, Kind)>,
}

impl Signature {
    pub fn get(&self, name: &str) -> Option<Kind> {
        self.entries
            .iter()
            .find(|(n, _)| n == name)
            .map(|&(_, k)| k)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(n, _)| n.as_str())
    }
}
