use roop_syntax::FnDef;

/// Whether a function is long enough that its callers are better off with its
/// lemmas than with its body: a run of writes into nested arrays is slow to
/// simplify, and the cost grows quickly with the length of the run.
pub fn long_body(def: &FnDef) -> bool {
    def.body.stmts.len() > 0
}
