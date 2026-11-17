use crate::{HybridError, Signature, Stmt, check_block};

/// The signature a program allocates, starting from no memory.
pub fn check(stmts: &[Stmt]) -> Result<Signature, HybridError> {
    let mut sig = Signature::default();
    check_block(stmts, &mut sig)?;
    Ok(sig)
}
