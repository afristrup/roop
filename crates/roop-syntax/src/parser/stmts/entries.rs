use crate::{Entry, Err, Stmt, TokenInput, flat_ancilla};
use chumsky::prelude::*;

/// The statements and declarations of a block, in order.
pub fn entries<'a, I: TokenInput<'a>>(
    stmt: impl Parser<'a, I, Stmt, Err<'a>> + Clone,
) -> impl Parser<'a, I, Vec<Entry>, Err<'a>> + Clone {
    flat_ancilla()
        .or(stmt.map(Entry::Stmt))
        .repeated()
        .collect()
}
