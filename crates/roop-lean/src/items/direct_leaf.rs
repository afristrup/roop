use roop_syntax::{Block, FnDef, Place, StmtKind, UpdateOp};

/// Whether a function can get a no-ancilla contract by direct simplification.
pub fn direct_leaf(def: &FnDef) -> bool {
    direct_block(&def.body)
}

fn direct_block(block: &Block) -> bool {
    block.stmts.iter().all(|stmt| {
        matches!(
            &stmt.kind,
            StmtKind::Update {
                target: Place::Var(_),
                op: UpdateOp::Add | UpdateOp::Sub | UpdateOp::Xor,
                ..
            } | StmtKind::Swap(Place::Var(_), Place::Var(_))
        ) || matches!(
            &stmt.kind,
            StmtKind::If {
                then_block,
                else_block,
                ..
            } if direct_block(then_block) && direct_block(else_block)
        )
    })
}
