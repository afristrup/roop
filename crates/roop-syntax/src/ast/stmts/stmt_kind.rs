use crate::{Block, Expr, MatchArm, OverwriteOp, Place, Type, UpdateOp};

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
    /// Irreversible code, like `unsafe`: the reversibility checks are lifted
    /// inside, and the enclosing function has no inverse.
    Irrev(Block),
    /// Destroys the old value of `target`. Only allowed in irreversible code.
    Overwrite {
        target: Place,
        op: OverwriteOp,
        value: Expr,
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
    /// Moves the value of `source` onto the stack, leaving it zero.
    Push {
        stack: Place,
        source: Place,
    },
    /// Moves the top of the stack into `target`, which must be zero.
    Pop {
        stack: Place,
        target: Place,
    },
    /// Destroying updates are allowed inside; each pushes what it destroys on
    /// `history`, so the block stays reversible.
    Logged {
        history: Place,
        body: Block,
    },
    /// Runs `body`; if it fails, undoes it and runs `handler`. With an
    /// `outcome` place the failure is recorded there (zero before, set when the
    /// handler ran), which keeps the statement reversible.
    Try {
        body: Block,
        handler: Block,
        outcome: Option<Place>,
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
