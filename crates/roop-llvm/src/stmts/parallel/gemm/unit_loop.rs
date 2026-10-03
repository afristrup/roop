use crate::const_int;
use roop_syntax::{Block, Expr, counted_loop};

/// The variable and length of a loop `from v == 0 ... until v == n - 1` that
/// steps by one.
pub fn unit_loop<'a>(entry: &'a Expr, step: &'a Block, until: &'a Expr) -> Option<(&'a str, i64)> {
    let counted = counted_loop(entry, step, until)?;
    if counted.step != 1 || const_int(counted.lo)? != 0 {
        return None;
    }
    Some((counted.var, const_int(counted.hi)?.checked_add(1)?))
}
