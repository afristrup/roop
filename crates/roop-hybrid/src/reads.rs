use crate::{Cond, Int, reads_int};

/// Whether `cond` reads bit `bit`, or int `reg` when `bit` is `None`.
pub fn reads(cond: &Cond, bit: Option<usize>, reg: Option<usize>) -> bool {
    let int = |i: &Int| reads_int(i, reg);
    match cond {
        Cond::Lit(_) => false,
        Cond::Bit(b) => Some(*b) == bit,
        Cond::Le(a, b) | Cond::Eq(a, b) => int(a) || int(b),
        Cond::And(a, b) | Cond::Or(a, b) => reads(a, bit, reg) || reads(b, bit, reg),
        Cond::Not(a) => reads(a, bit, reg),
    }
}
