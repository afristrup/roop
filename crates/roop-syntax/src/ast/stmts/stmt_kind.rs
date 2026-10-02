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
    /// A bare block, mainly the body of a `#[concurrent]` task.
    Block(Block),
    /// A channel for messages of type `ty`, alive for `body`.
    Chan {
        name: String,
        ty: Type,
        body: Block,
    },
    /// Moves the value of `source` into the channel, leaving it zero.
    Send {
        chan: String,
        source: Place,
    },
    /// Moves the next message into `target`, which must be zero.
    Recv {
        chan: String,
        target: Place,
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
