use crate::{HybridError, Int, Store};

pub fn eval_int(int: &Int, store: &Store) -> Result<i64, HybridError> {
    let pair = |a, b| Ok::<_, HybridError>((eval_int(a, store)?, eval_int(b, store)?));
    match int {
        Int::Lit(n) => Ok(*n),
        Int::Reg(r) => Ok(store.ints[*r]),
        Int::Add(a, b) => {
            let (a, b) = pair(a, b)?;
            a.checked_add(b).ok_or(HybridError::Overflow)
        }
        Int::Mul(a, b) => {
            let (a, b) = pair(a, b)?;
            a.checked_mul(b).ok_or(HybridError::Overflow)
        }
        Int::Pow(a, b) => {
            let (a, b) = pair(a, b)?;
            let e = u32::try_from(b).map_err(|_| match b < 0 {
                true => HybridError::NegativeExponent(b),
                false => HybridError::Overflow,
            })?;
            a.checked_pow(e).ok_or(HybridError::Overflow)
        }
    }
}
