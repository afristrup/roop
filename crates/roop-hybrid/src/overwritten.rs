use crate::{Op, reads, reads_int};

/// Whether the variable `ops[0]` assigns is assigned again in `ops[1..]`
/// before anything could read it. Control flow and the end of the block count
/// as reads.
pub fn overwritten(ops: &[Op]) -> bool {
    let (bit, reg) = match ops[0] {
        Op::SetBit { bit, .. } => (Some(bit), None),
        Op::SetInt { reg, .. } => (None, Some(reg)),
        _ => return false,
    };
    for op in &ops[1..] {
        match op {
            Op::Kernel(_) => {}
            Op::Measure { bit: b, .. } if Some(*b) == bit => return true,
            Op::Measure { .. } => {}
            Op::SetBit { bit: b, value } => {
                if reads(value, bit, reg) {
                    return false;
                }
                if Some(*b) == bit {
                    return true;
                }
            }
            Op::SetInt { reg: r, value } => {
                if reads_int(value, reg) {
                    return false;
                }
                if Some(*r) == reg {
                    return true;
                }
            }
            Op::If { .. } | Op::While { .. } => return false,
        }
    }
    false
}
