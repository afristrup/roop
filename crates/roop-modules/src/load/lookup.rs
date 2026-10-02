use std::collections::HashMap;

/// Resolves a name through `scope`. A struct's generated constructors,
/// `S_build` and `S_unbuild`, resolve through the struct. Builtin names are
/// not in scope and stay as they are.
pub fn lookup(scope: &HashMap<String, String>, name: &str) -> Option<String> {
    if let Some(global) = scope.get(name) {
        return Some(global.clone());
    }
    ["_build", "_unbuild"].into_iter().find_map(|suffix| {
        let base = name.strip_suffix(suffix)?;
        scope.get(base).map(|global| format!("{global}{suffix}"))
    })
}
