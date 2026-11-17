use crate::{Cond, HybridError, Store, eval_int};

pub fn eval_cond(cond: &Cond, store: &Store) -> Result<bool, HybridError> {
    Ok(match cond {
        Cond::Lit(b) => *b,
        Cond::Bit(b) => store.bits[*b],
        Cond::Le(a, b) => eval_int(a, store)? <= eval_int(b, store)?,
        Cond::Eq(a, b) => eval_int(a, store)? == eval_int(b, store)?,
        Cond::And(a, b) => eval_cond(a, store)? && eval_cond(b, store)?,
        Cond::Or(a, b) => eval_cond(a, store)? || eval_cond(b, store)?,
        Cond::Not(a) => !eval_cond(a, store)?,
        Cond::Xor(a, b) => eval_cond(a, store)? != eval_cond(b, store)?,
    })
}
