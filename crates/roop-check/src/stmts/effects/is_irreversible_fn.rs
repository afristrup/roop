use crate::contains_irrev;
use roop_syntax::FnDef;

/// A function has no inverse when it is declared `irrev` or contains
/// irreversible code. Calling one needs irreversible code of your own.
pub fn is_irreversible_fn(def: &FnDef) -> bool {
    def.irreversible || contains_irrev(&def.body)
}
