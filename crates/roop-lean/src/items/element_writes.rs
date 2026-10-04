use roop_syntax::{FnDef, Place, StmtKind};

/// Whether a function only updates elements of arrays. Callers use the lemmas
/// of such a function instead of its body: a run of writes into nested arrays is
/// slow to simplify, and the cost grows quickly with its length.
pub fn element_writes(def: &FnDef) -> bool {
    !def.body.stmts.is_empty()
        && def.body.stmts.iter().all(|stmt| {
            matches!(
                &stmt.kind,
                StmtKind::Update {
                    target: Place::Index(..),
                    ..
                }
            )
        })
}
