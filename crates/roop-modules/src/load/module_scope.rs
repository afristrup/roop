use std::collections::HashMap;

/// What a module can mention, and what it pulls in from elsewhere.
pub struct ModuleScope {
    /// Local name to global name: its own definitions and its imports.
    pub names: HashMap<String, String>,
    /// The modules it imports from.
    pub deps: Vec<usize>,
    /// The global names of what it imports.
    pub imports: Vec<String>,
}
