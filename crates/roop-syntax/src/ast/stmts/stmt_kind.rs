use crate::{Block, Expr, MatchArm, Place, Type, UpdateOp};

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Update {
        target: Place,
        op: UpdateOp,
        value: Expr,
    },
    Swap(Place, Place),
    If {
        cond: Expr,
        then_block: Block,
        else_block: Block,
        exit: Expr,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
    },
    Borrow {
        name: String,
        source: Place,
        body: Block,
    },
    Try {
        body: Block,
        handler: Block,
    },
    From {
        entry: Expr,
        body: Block,
        step: Block,
        until: Expr,
    },
    Ancilla {
        name: String,
        ty: Type,
        init: Expr,
        body: Block,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    Uncall {
        callee: String,
        args: Vec<Expr>,
    },
}
