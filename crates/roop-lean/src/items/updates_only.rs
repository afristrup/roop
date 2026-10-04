use roop_syntax::{FnDef, StmtKind};

/// Whether a function is only updates of its places. Callers use the lemmas of
/// such a function instead of its body: a run of writes into nested arrays is
/// slow to simplify, and so is a chain of scalar updates repeated per call.
pub fn updates_only(def: &FnDef) -> bool {
    !def.body.stmts.is_empty()
        && def
            .body
            .stmts
            .iter()
            .all(|stmt| matches!(&stmt.kind, StmtKind::Update { .. }))
}
