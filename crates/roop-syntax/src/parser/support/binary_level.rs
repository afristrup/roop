use crate::{BinOp, Expr};
use crate::{Err, TokenInput};
use chumsky::prelude::*;

pub fn binary_level<'a, I: TokenInput<'a>>(
    next: impl Parser<'a, I, Expr, Err<'a>> + Clone,
    ops: impl Parser<'a, I, BinOp, Err<'a>> + Clone,
) -> impl Parser<'a, I, Expr, Err<'a>> + Clone {
    next.clone().foldl(ops.then(next).repeated(), |l, (op, r)| {
        Expr::Binary(Box::new(l), op, Box::new(r))
    })
}
