use crate::Dialect;

/// Address space of the kernel's launch-parameter buffer.
pub fn params_space(dialect: Dialect) -> u32 {
    if dialect == Dialect::Air { 2 } else { 1 }
}
