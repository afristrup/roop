use roop_syntax::{FnDef, StructDef, Type};
use std::collections::HashMap;

/// What the length inference knows while it walks a function: the types of the
/// variables in scope, and the program's functions and structs.
pub struct Scope<'p> {
    pub vars: Vec<(String, Type)>,
    pub fns: &'p HashMap<&'p str, &'p FnDef>,
    pub structs: &'p HashMap<&'p str, &'p StructDef>,
}

impl Scope<'_> {
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.vars
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
    }
}
