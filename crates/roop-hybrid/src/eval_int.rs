use crate::{HybridError, Int, Store};

pub fn eval_int(int: &Int, store: &Store) -> Result<i64, HybridError> {
    let pair = |a, b| Ok::<_, HybridError>((eval_int(a, store)?, eval_int(b, store)?));
    let checked = |v: Option<i64>| v.ok_or(HybridError::Overflow);
    match int {
        Int::Lit(n) => Ok(*n),
        Int::Reg(r) => Ok(store.ints[*r]),
        Int::Add(a, b) => pair(a, b).and_then(|(a, b)| checked(a.checked_add(b))),
        Int::Sub(a, b) => pair(a, b).and_then(|(a, b)| checked(a.checked_sub(b))),
        Int::Mul(a, b) => pair(a, b).and_then(|(a, b)| checked(a.checked_mul(b))),
        Int::Div(a, b) | Int::Rem(a, b) => {
            let (a, b) = pair(a, b)?;
            if b == 0 {
                return Err(HybridError::DivisionByZero);
            }
            checked(if matches!(int, Int::Div(..)) {
                a.checked_div(b)
            } else {
                a.checked_rem(b)
            })
        }
        Int::Xor(a, b) => pair(a, b).map(|(a, b)| a ^ b),
        Int::And(a, b) => pair(a, b).map(|(a, b)| a & b),
        Int::Or(a, b) => pair(a, b).map(|(a, b)| a | b),
        Int::Pow(a, b) => {
            let (a, b) = pair(a, b)?;
            let e = u32::try_from(b).map_err(|_| match b < 0 {
                true => HybridError::NegativeExponent(b),
                false => HybridError::Overflow,
            })?;
            checked(a.checked_pow(e))
        }
    }
}
