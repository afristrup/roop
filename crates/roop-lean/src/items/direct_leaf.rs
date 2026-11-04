use roop_syntax::{FnDef, Place, StmtKind, UpdateOp};

/// Whether a function can get a no-ancilla contract by direct simplification.
pub fn direct_leaf(def: &FnDef) -> bool {
    def.body.stmts.iter().all(|stmt| {
        matches!(
            &stmt.kind,
            StmtKind::Update {
                target: Place::Var(_),
                op: UpdateOp::Add | UpdateOp::Sub | UpdateOp::Xor,
                ..
            } | StmtKind::Swap(Place::Var(_), Place::Var(_))
        )
    })
}
