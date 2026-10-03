use crate::{Expr, Stmt, Type};

/// One thing in a block as written: a statement, or `ancilla x: T = e;`, which
/// holds from there to the end of the block.
pub enum Entry {
    Stmt(Stmt),
    Declare {
        name: String,
        ty: Type,
        init: Expr,
        /// Where the declaration starts, and where it ends, after its `;`.
        start: usize,
        after: usize,
    },
}
